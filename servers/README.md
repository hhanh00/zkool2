# Zcash Light Wallet Server Sync

A tiny TypeScript project that fetches the latest Zcash light wallet servers from
[hosh.zec.rocks](https://hosh.zec.rocks/zec) and saves them as a JSON list, with the
full server URL plus all the info shown on the uptime table attached to each entry.

## Setup

```bash
npm install
```

## Usage

```bash
npm run sync                       # fetch + write ./servers.json
tsx sync.ts --out=my-servers.json  # custom output path
tsx sync.ts --online-only          # only servers currently online
tsx sync.ts --no-tor               # exclude .onion servers
```

Compile/run with plain Node instead of tsx:

```bash
npm run build && npm start
```

## Data source

The script reads the machine-readable JSON API behind the page:

```
https://hosh.zec.rocks/api/v0/zec.json
```

## Ordering

Servers are sorted by three keys, in order:

1. **Online first** — all online servers come before offline ones.
2. **Uptime band** — 10%-wide 30-day uptime bands (`90-100%`, `80-89%`, … `0-9%`), highest band first.
3. **USA ping** — `pingMs` ascending (fastest first).

The flat `servers` array follows this order. The `groups` array additionally
breaks the same sorted list out by uptime band, mirroring the
[hosh.zec.rocks](https://hosh.zec.rocks/zec) layout. The `pingMs` field is the
USA ping reported by the source.

## Output shape

```jsonc
{
  "source": "https://hosh.zec.rocks/api/v0/zec.json",
  "fetchedAt": "2026-06-03T00:00:00.000Z",
  "count": 140,
  // Flat list in final sort order (band desc, online first, USA ping asc).
  "servers": [
    {
      "url": "https://zec.rocks:443",
      "hostname": "zec.rocks",
      "port": 443,
      "protocol": "grpc",
      "online": true,
      "community": false,
      "tor": false,
      "height": 3364423,
      "uptime30d": 0.9872,
      "uptime30dPercent": "98.72%",
      "pingMs": 17.44,
      "lightwalletServerVersion": "v0.4.19",
      "nodeVersion": "Zebra:5.0.0",
      "donationAddress": null
    }
  ],
  // Same servers, grouped into uptime bands.
  "groups": [
    {
      "uptimeBucket": 90,
      "label": "90-100%",
      "count": 43,
      "servers": [ /* ...band members in sort order... */ ]
    }
  ]
}
```
