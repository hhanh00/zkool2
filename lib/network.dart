import 'package:flutter/material.dart';

/// The set of Zcash networks the wallet supports. The integer [coin] value
/// matches the Rust `Coin.coin` field and the DB `coin` prop
/// (0=mainnet, 1=testnet, 2=regtest) — see rust/src/api/coin.rs `network()`.
enum ZNetwork {
  mainnet,
  testnet,
  regtest,
}

/// Static per-network metadata: identifiers, display label, default servers and
/// the default block-explorer template. Account data is NOT shared between
/// networks — each network uses its own SQLite database file (see [dbSuffix]).
class NetworkInfo {
  /// Rust coin id (0/1/2). Authoritative network discriminator.
  final int coin;

  /// Short machine name, matches Rust `Coin::get_name` ("mainnet"/"testnet"/"regnet").
  final String name;

  /// Human-facing label shown in the network selector.
  final String label;

  /// Suffix appended to the base database name to derive this network's DB file.
  /// Empty for mainnet so existing mainnet databases keep their current name.
  final String dbSuffix;

  /// Default lightwallet (LWD) server URL used when a fresh network DB is created.
  final String defaultLwd;

  /// Additional well-known servers offered in the Settings dropdown.
  final List<String> altLwds;

  /// Default block-explorer URL template, with a {txid} placeholder. Empty when
  /// the network has no explorer (e.g. regtest).
  final String defaultExplorer;

  /// Whether the network defaults to a light node (lightwalletd) vs a full node.
  final bool defaultIsLightNode;

  const NetworkInfo({
    required this.coin,
    required this.name,
    required this.label,
    required this.dbSuffix,
    required this.defaultLwd,
    required this.altLwds,
    required this.defaultExplorer,
    required this.defaultIsLightNode,
  });

  /// All servers to surface in the Settings dropdown for this network
  /// (default first, then alternatives).
  List<String> get servers => [defaultLwd, ...altLwds];
}

/// Single source of truth for the supported networks.
const Map<ZNetwork, NetworkInfo> kNetworks = {
  ZNetwork.mainnet: NetworkInfo(
    coin: 0,
    name: "mainnet",
    label: "Zcash",
    dbSuffix: "",
    defaultLwd: "https://zec.rocks",
    altLwds: [],
    defaultExplorer: "https://cipherscan.app/tx/{txid}",
    defaultIsLightNode: true,
  ),
  ZNetwork.testnet: NetworkInfo(
    coin: 1,
    name: "testnet",
    label: "Zcash Testnet",
    dbSuffix: "-testnet",
    defaultLwd: "https://testnet.zec.rocks",
    altLwds: ["https://zcash.mysideoftheweb.com:19067"],
    defaultExplorer: "https://testnet.cipherscan.app/tx/{txid}",
    defaultIsLightNode: true,
  ),
  ZNetwork.regtest: NetworkInfo(
    coin: 2,
    name: "regnet",
    label: "Zcash Regtest",
    dbSuffix: "-regtest",
    // Regtest runs against a local Zebra full node (lightwalletd support for
    // regtest is unknown), so default to Zebra's RPC and a full-node server.
    defaultLwd: "http://127.0.0.1:18232",
    altLwds: [],
    defaultExplorer: "",
    defaultIsLightNode: false,
  ),
};

NetworkInfo networkInfo(ZNetwork net) => kNetworks[net]!;

/// Convenience accessors on [ZNetwork] so callers can use `net.coin` / `net.info`
/// directly instead of `networkInfo(net).coin`.
extension ZNetworkX on ZNetwork {
  NetworkInfo get info => networkInfo(this);

  /// Rust coin id (0=mainnet, 1=testnet, 2=regtest).
  int get coin => networkInfo(this).coin;
}

/// Resolve a [ZNetwork] from a Rust coin id (0/1/2). Defaults to mainnet.
ZNetwork networkForCoin(int coin) {
  for (final e in kNetworks.entries) {
    if (e.value.coin == coin) return e.key;
  }
  return ZNetwork.mainnet;
}

/// Resolve a [ZNetwork] from the machine network name ("mainnet"/"testnet"/"regnet").
ZNetwork networkForName(String name) {
  for (final e in kNetworks.entries) {
    if (e.value.name == name) return e.key;
  }
  return ZNetwork.mainnet;
}

/// Strip any known network suffix from [dbName] to recover the family "base"
/// name. e.g. "zkool-testnet" -> "zkool", "zkool" -> "zkool".
String baseDbName(String dbName) {
  for (final info in kNetworks.values) {
    if (info.dbSuffix.isNotEmpty && dbName.endsWith(info.dbSuffix)) {
      return dbName.substring(0, dbName.length - info.dbSuffix.length);
    }
  }
  return dbName;
}

/// Compute the database name for [net] within the [base] family.
/// Mainnet uses the base name unchanged; others append their suffix.
String dbNameForNetwork(String base, ZNetwork net) {
  final b = baseDbName(base);
  return "$b${networkInfo(net).dbSuffix}";
}

/// Zcash logo (SVG) used to represent a network in the selector. It is drawn with
/// `currentColor` for the disc, so tinting it via a flutter_svg colorFilter makes
/// it adapt to any theme.
const String kNetworkIconAsset = "assets/zcash.svg";

/// App / page title for [net]: plain "zkool" on mainnet, "zkool (testnet)" /
/// "zkool (regtest)" elsewhere so the active network is always visible.
String networkTitle(String appName, ZNetwork net) {
  switch (net) {
    case ZNetwork.mainnet:
      return appName;
    case ZNetwork.testnet:
      return "$appName (testnet)";
    case ZNetwork.regtest:
      return "$appName (regtest)";
  }
}

/// A subtle accent color per network so testnet/regtest are visually distinct.
Color networkAccent(ZNetwork net) {
  switch (net) {
    case ZNetwork.mainnet:
      return const Color(0xFFF4B728); // Zcash gold
    case ZNetwork.testnet:
      return const Color(0xFF1E88E5); // blue
    case ZNetwork.regtest:
      return const Color(0xFF8E24AA); // purple
  }
}
