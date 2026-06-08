import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge.dart';
import 'package:gap/gap.dart';
import 'package:go_router/go_router.dart';
import 'package:showcaseview/showcaseview.dart';
import 'package:zkool/main.dart';
import 'package:zkool/pages/sweep.dart';
import 'package:zkool/src/rust/api/account.dart';
import 'package:zkool/src/rust/api/network.dart';
import 'package:zkool/store.dart';
import 'package:zkool/utils.dart';
import 'package:zkool/widgets/error_display.dart';
import 'package:zkool/widgets/pool_select.dart';

final viewID = GlobalKey();
final sweepID = GlobalKey();
final derivePrevID = GlobalKey();
final deriveID = GlobalKey();
final qrID = GlobalKey();

class ReceivePage extends ConsumerStatefulWidget {
  const ReceivePage({super.key});

  @override
  ConsumerState<ReceivePage> createState() => ReceivePageState();
}

class ReceivePageState extends ConsumerState<ReceivePage> {
  late final c = coinContext.coin;
  Account? account;
  Addresses? addresses;
  int uaPools = 0;
  int availablePools = 0;
  // BIP-44/ZIP-32 coin type, derived from the active network. Zcash mainnet
  // uses 133'; test/regtest networks use 1' (SLIP-44 testnet convention).
  int coinType = 133;

  @override
  void initState() {
    super.initState();

    Future(() async {
      final selectedAccount = ref.read(selectedAccountProvider).requireValue!;
      final a = await ref.read(accountProvider(selectedAccount.id).future);
      final pools = a.pool; // All pools including transparent
      final defaultPools = pools & 6; // Default to shielded pools only
      final addrs = await getAddresses(uaPools: defaultPools, c: c);
      final net = await getNetworkName(c: c);
      setState(() {
        account = a.account;
        addresses = addrs;
        availablePools = pools;
        uaPools = defaultPools;
        coinType = net == "main" || net == "mainnet" ? 133 : 1;
      });
    });
  }

  // Transparent addresses use the BIP-44 path
  //   m/44'/{coin_type}'/{account}'/{scope}/{diversifier_index}
  // (scope 0 = external receive). See rust/src/account.rs derive_transparent_address.
  String transparentPath() =>
      "m/44'/$coinType'/${account!.aindex}'/0/${addresses!.diversifierIndex}";

  // Shielded (Sapling/Orchard) keys use the ZIP-32 path
  //   m/32'/{coin_type}'/{account}'
  // with the receiver selected by the diversifier index (not a path level).
  // See rust/src/account.rs (usk.sapling()/usk.orchard()).
  String shieldedPath() => "m/32'/$coinType'/${account!.aindex}'";

  // A short, copyable description of how an address was derived.
  String derivationLabel({required bool transparent}) => transparent
      ? "Path: ${transparentPath()}"
      : "Path: ${shieldedPath()}  (diversifier ${addresses!.diversifierIndex})";

  Widget derivationInfo({required bool transparent}) => Padding(
        padding: const EdgeInsets.only(top: 2),
        child: Text(
          derivationLabel(transparent: transparent),
          style: Theme.of(context).textTheme.bodySmall?.copyWith(
                fontFamily: "monospace",
                color: Theme.of(context).hintColor,
              ),
        ),
      );

  void tutorial() async {
    tutorialHelper(context, "tutReceive0", [viewID, sweepID, derivePrevID, deriveID, qrID]);
  }

  @override
  Widget build(BuildContext context) {
    if (this.account == null) return blank(context);
    final pinlock = ref.watch(lifecycleProvider);
    if (pinlock.value ?? false) return PinLock();

    Future(tutorial);

    final account = this.account!;
    final addresses = this.addresses!;

    return Scaffold(
      appBar: AppBar(
        title: Text("Receive Funds"),
        actions: [
          Showcase(
            key: viewID,
            description: "View Transparent Addresses",
            child: IconButton(
              tooltip: "Transparent Addresses",
              onPressed: onViewTransparentAddresses,
              icon: Icon(Icons.visibility),
            ),
          ),
          Showcase(
            key: sweepID,
            description:
                "Find other transparent addresses. If you restored from a wallet that has address rotation (such as Ledger, Exodus, etc), Tap, then Reset and Sync",
            child: IconButton(
              tooltip: "Sweep",
              onPressed: onSweep,
              icon: Icon(Icons.search),
            ),
          ),
          Showcase(
            key: derivePrevID,
            description: "Go back to the previous set of addresses (transparent/sapling and orchard)",
            child: IconButton(
              tooltip: "Previous Set of Addresses",
              onPressed: onPrevAddress,
              icon: Icon(Icons.skip_previous),
            ),
          ),
          Showcase(
            key: deriveID,
            description: "Generate a new set of addresses (transparent/sapling and orchard). Previous addresses can still receive funds",
            child: IconButton(
              tooltip: "Next Set of Addresses",
              onPressed: onGenerateAddress,
              icon: Icon(Icons.skip_next),
            ),
          ),
        ],
      ),
      body: SingleChildScrollView(
        child: Padding(
          padding: EdgeInsets.symmetric(horizontal: 8),
          child: Column(
            children: [
              if (addresses.saddr != null || addresses.oaddr != null)
                PoolSelect(
                  enabled: availablePools,
                  initialValue: uaPools,
                  onChanged: onChangedUAPools,
                ),
              if (addresses.ua != null) ...[
                Gap(8),
                ListTile(
                  title: Text("Unified Address"),
                  subtitle: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      CopyableText(addresses.ua!),
                      derivationInfo(transparent: false),
                    ],
                  ),
                  trailing: Showcase(
                    key: qrID,
                    description: "Show address as a QR Code",
                    child: IconButton(
                      icon: Icon(Icons.qr_code),
                      onPressed: () => onShowQR("Unified Address", addresses.ua!),
                    ),
                  ),
                ),
              ],
              if (addresses.oaddr != null)
                ListTile(
                  title: Text("Orchard only Address"),
                  subtitle: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      CopyableText(addresses.oaddr!),
                      derivationInfo(transparent: false),
                    ],
                  ),
                  trailing: IconButton(
                    icon: Icon(Icons.qr_code),
                    onPressed: () => onShowQR("Orchard", addresses.oaddr!),
                  ),
                ),
              if (addresses.saddr != null)
                ListTile(
                  title: Text("Sapling Address"),
                  subtitle: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      CopyableText(addresses.saddr!),
                      derivationInfo(transparent: false),
                    ],
                  ),
                  leading: account.hw != 0 ? IconButton(onPressed: onCheckSapling, icon: Icon(Icons.check)) : null,
                  trailing: IconButton(
                    icon: Icon(Icons.qr_code),
                    onPressed: () => onShowQR("Sapling", addresses.saddr!),
                  ),
                ),
              if (addresses.taddr != null)
                ListTile(
                  title: Text("Transparent Address"),
                  subtitle: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      CopyableText(addresses.taddr!),
                      derivationInfo(transparent: true),
                    ],
                  ),
                  leading: account.hw != 0 ? IconButton(onPressed: onCheckTransparent, icon: Icon(Icons.check)) : null,
                  trailing: IconButton(
                    icon: Icon(Icons.qr_code),
                    onPressed: () => onShowQR("Transparent", addresses.taddr!),
                  ),
                ),
            ],
          ),
        ),
      ),
    );
  }

  void onCheckSapling() async {
    showSnackbar("Check address on the device");
    try {
      await showLedgerSaplingAddress(c: c);
    } on AnyhowException catch (e) {
      await showException(context, e.message);
    }
  }

  void onCheckTransparent() async {
    showSnackbar("Check address on the device");
    try {
      await showLedgerTransparentAddress(c: c);
    } on AnyhowException catch (e) {
      await showException(context, e.message);
    }
  }

  void onChangedUAPools(int pools) async {
    uaPools = pools;
    addresses = await getAddresses(uaPools: uaPools, c: c);
    setState(() {});
  }

  void onGenerateAddress() async {
    try {
      final confirmed = await confirmDialog(context,
          title: "New Addresses", message: "Do you want to generate a new set of addresses? Previous addresses can still receive funds");
      if (!confirmed) return;
      if (!mounted) return;
      final dialog = await showMessage(context, "Please wait for the address generation\nCheck your Ledger", dismissable: false);
      await generateNextDindex(c: c); // This takes a while on the Ledger
      addresses = await getAddresses(uaPools: uaPools, c: c);
      dialog.dismiss();
      setState(() {});
    } on AnyhowException catch (e) {
      await showException(context, e.message);
    }
  }

  void onPrevAddress() async {
    try {
      // Already at the first address set; there is nothing earlier.
      if ((addresses?.diversifierIndex ?? 0) == 0) {
        showSnackbar("Already at the first set of addresses");
        return;
      }
      final dialog = await showMessage(context, "Please wait\nCheck your Ledger", dismissable: false);
      await generatePrevDindex(c: c); // This takes a while on the Ledger
      addresses = await getAddresses(uaPools: uaPools, c: c);
      dialog.dismiss();
      setState(() {});
    } on AnyhowException catch (e) {
      await showException(context, e.message);
    }
  }

  void onShowQR(String title, String text) {
    GoRouter.of(context).push("/qr", extra: {"title": title, "text": text});
  }

  void onViewTransparentAddresses() async {
    final txCounts = await fetchTransparentAddressTxCount(c: c);
    if (!mounted) return;
    await GoRouter.of(context).push("/transparent_addresses", extra: txCounts);
  }

  void onSweep() async {
    await showTransparentScan(ref, context);
  }
}

class TransparentAddressesPage extends ConsumerWidget {
  final List<TAddressTxCount> txCounts;

  const TransparentAddressesPage({super.key, required this.txCounts});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final pinlock = ref.watch(lifecycleProvider);
    if (pinlock.value ?? false) return PinLock();

    return Scaffold(
      appBar: AppBar(title: Text("Transparent Addresses")),
      body: ListView.builder(
        itemCount: txCounts.length,
        itemBuilder: (context, index) {
          final txCount = txCounts[index];
          final scope = txCount.scope == 0 ? "External" : "Change";
          return ListTile(
            title: CopyableText(txCount.address),
            subtitle: Text("Scope: $scope, Index: ${txCount.dindex}, Tx Count: ${txCount.txCount}, Last Used: ${timeToString(txCount.time)}"),
            trailing: Text(zatToString(txCount.amount)),
          );
        },
      ),
    );
  }
}
