/**
 * sync.ts
 *
 * Fetches the latest Zcash light wallet servers from hosh.zec.rocks and writes a
 * JSON-formatted list to disk. Each entry carries the full server URL plus all the
 * info shown on the uptime table (status, height, uptime, versions, ping, etc.)
 * as a structured object.
 *
 * Usage:
 *   npm run sync                       # fetch + save to ./servers.json
 *   tsx sync.ts --out=my-servers.json  # custom output file
 *   tsx sync.ts --online-only          # keep only servers currently online
 *   tsx sync.ts --no-tor               # drop .onion servers
 *
 * Servers are sorted online-first, then by 10%-wide 30-day-uptime band
 * (90-100%, 80-89%, ...) highest first, then by ascending USA ping (the
 * `pingMs` field) fastest first. A `groups` view additionally breaks the
 * sorted list out by uptime band, mirroring the hosh.zec.rocks layout.
 *
 * Data source: https://hosh.zec.rocks/api/v0/zec.json
 */

import { writeFile } from "node:fs/promises";
import { resolve } from "node:path";

const SOURCE_URL = "https://hosh.zec.rocks/api/v0/zec.json";

/** Raw shape of a server entry as returned by the hosh API. */
interface RawServer {
  hostname: string;
  port: number;
  protocol: string;
  ping: number;
  online: boolean;
  community: boolean;
  height: number;
  uptime_30d: number;
  // Present mostly on online servers:
  lightwallet_server_version?: string;
  node_version?: string;
  donation_address?: string;
}

interface RawResponse {
  servers: RawServer[];
}

/** Normalized server record we persist. */
interface Server {
  /** Full URL, e.g. "https://zec.rocks:443". */
  url: string;
  hostname: string;
  port: number;
  protocol: string;
  online: boolean;
  community: boolean;
  /** true when the hostname is a Tor (.onion) address. */
  tor: boolean;
  height: number;
  /** 30-day uptime as a fraction in [0, 1]. */
  uptime30d: number;
  /** 30-day uptime formatted as a percentage string, e.g. "98.72%". */
  uptime30dPercent: string;
  /** USA ping in milliseconds. */
  pingMs: number;
  lightwalletServerVersion: string | null;
  nodeVersion: string | null;
  donationAddress: string | null;
}

/** A group of servers sharing the same 10%-wide uptime band. */
interface UptimeGroup {
  /** Lower bound of the band as a percent, e.g. 90 for the 90-99% band. */
  uptimeBucket: number;
  /** Human label for the band, e.g. "90-99%". */
  label: string;
  count: number;
  /** Servers in this band, in canonical order (online first, then fastest USA ping). */
  servers: Server[];
}

interface Output {
  source: string;
  fetchedAt: string;
  count: number;
  /** Flat list: online first, then uptime band (desc), then USA ping (asc). */
  servers: Server[];
  /** Same servers grouped into 10%-wide uptime bands, mirroring hosh.zec.rocks. */
  groups: UptimeGroup[];
}

interface Options {
  outFile: string;
  onlineOnly: boolean;
  includeTor: boolean;
}

function parseArgs(argv: string[]): Options {
  const opts: Options = {
    outFile: "servers.json",
    onlineOnly: false,
    includeTor: true,
  };
  for (const arg of argv) {
    if (arg.startsWith("--out=")) opts.outFile = arg.slice("--out=".length);
    else if (arg === "--online-only") opts.onlineOnly = true;
    else if (arg === "--no-tor") opts.includeTor = false;
  }
  return opts;
}

/** Build the full URL for a server based on its protocol and port. */
function buildUrl(s: RawServer): string {
  // The hosh API reports protocol "grpc". Light wallet servers speak gRPC,
  // which uses TLS and is reached over https regardless of port.
  // We always expose an https:// URL since that is how these endpoints are reached.
  const scheme = "https";
  // Omit the port when it is the https default (443).
  const isDefaultPort = s.port === 443;
  return isDefaultPort
    ? `${scheme}://${s.hostname}`
    : `${scheme}://${s.hostname}:${s.port}`;
}

function normalize(s: RawServer): Server {
  return {
    url: buildUrl(s),
    hostname: s.hostname,
    port: s.port,
    protocol: s.protocol,
    online: s.online,
    community: s.community,
    tor: s.hostname.toLowerCase().endsWith(".onion"),
    height: s.height,
    uptime30d: s.uptime_30d,
    uptime30dPercent: `${(s.uptime_30d * 100).toFixed(2)}%`,
    pingMs: s.ping,
    lightwalletServerVersion: s.lightwallet_server_version ?? null,
    nodeVersion: s.node_version ?? null,
    donationAddress: s.donation_address ?? null,
  };
}

async function fetchServers(): Promise<RawServer[]> {
  const res = await fetch(SOURCE_URL, {
    headers: { Accept: "application/json" },
  });
  if (!res.ok) {
    throw new Error(`Failed to fetch ${SOURCE_URL}: ${res.status} ${res.statusText}`);
  }
  const data = (await res.json()) as RawResponse;
  if (!data || !Array.isArray(data.servers)) {
    throw new Error("Unexpected response shape: missing `servers` array.");
  }
  return data.servers;
}

/** The 10%-wide uptime band a server falls into, as a lower-bound percent (0,10,...,100). */
function uptimeBucketOf(s: Server): number {
  const pct = s.uptime30d * 100;
  // 100% rounds down to the 90 band so it sits with the other top performers,
  // matching how the hosh page lumps the highest uptimes together.
  return Math.min(90, Math.floor(pct / 10) * 10);
}

/** USA-ping comparator: fastest first. Offline servers (ping 0) sink to the bottom. */
function byUsaPing(a: Server, b: Server): number {
  // Treat a 0ms ping (offline / unmeasured) as worst so it never ranks first.
  const pa = a.pingMs > 0 ? a.pingMs : Number.POSITIVE_INFINITY;
  const pb = b.pingMs > 0 ? b.pingMs : Number.POSITIVE_INFINITY;
  return pa - pb;
}

/**
 * Canonical sort order: online servers first, then by uptime band (highest
 * first), then by ascending USA ping (fastest first).
 */
function compareServers(a: Server, b: Server): number {
  if (a.online !== b.online) return a.online ? -1 : 1; // online before offline
  const bucketDiff = uptimeBucketOf(b) - uptimeBucketOf(a); // higher band first
  if (bucketDiff !== 0) return bucketDiff;
  return byUsaPing(a, b); // then fastest USA ping first
}

/**
 * Break a sorted server list into 10%-wide uptime bands for the `groups` view.
 * The input order is preserved within each band, so each band keeps the
 * online-first / fastest-ping ordering established by compareServers.
 */
function groupByUptime(sorted: Server[]): UptimeGroup[] {
  const byBucket = new Map<number, Server[]>();
  for (const s of sorted) {
    const bucket = uptimeBucketOf(s);
    const list = byBucket.get(bucket) ?? [];
    list.push(s);
    byBucket.set(bucket, list);
  }
  return [...byBucket.keys()]
    .sort((a, b) => b - a) // highest-uptime band first
    .map((bucket) => {
      const list = byBucket.get(bucket)!;
      return {
        uptimeBucket: bucket,
        label: bucket >= 90 ? "90-100%" : `${bucket}-${bucket + 9}%`,
        count: list.length,
        servers: list,
      };
    });
}

async function main(): Promise<void> {
  const opts = parseArgs(process.argv.slice(2));

  console.log(`Fetching Zcash light wallet servers from ${SOURCE_URL} ...`);
  const raw = await fetchServers();

  let servers = raw.map(normalize);
  if (opts.onlineOnly) servers = servers.filter((s) => s.online);
  if (!opts.includeTor) servers = servers.filter((s) => !s.tor);

  // Sort: online first, then uptime band (highest first), then USA ping (fastest first).
  const sorted = [...servers].sort(compareServers);

  // Group the already-sorted list into 10%-wide uptime bands for the `groups` view.
  const groups = groupByUptime(sorted);

  const output: Output = {
    source: SOURCE_URL,
    fetchedAt: new Date().toISOString(),
    count: sorted.length,
    servers: sorted,
    groups,
  };

  const outPath = resolve(process.cwd(), opts.outFile);
  await writeFile(outPath, JSON.stringify(output, null, 2) + "\n", "utf8");

  const onlineCount = sorted.filter((s) => s.online).length;
  console.log(`Saved ${sorted.length} servers (${onlineCount} online) -> ${outPath}`);
  for (const g of groups) {
    console.log(`  ${g.label.padEnd(8)} ${g.count} servers`);
  }
}

main().catch((err) => {
  console.error("sync failed:", err instanceof Error ? err.message : err);
  process.exit(1);
});
