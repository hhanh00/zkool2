import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge.dart';
import 'package:gap/gap.dart';
import 'package:go_router/go_router.dart';
import 'package:showcaseview/showcaseview.dart';
import 'package:zkool/main.dart';
import 'package:zkool/pages/sweep.dart';
import 'package:zkool/src/rust/api/account.dart';
import 'package:zkool/store.dart';
import 'package:zkool/utils.dart';
import 'package:zkool/widgets/pool_select.dart';

final viewID = GlobalKey();
final sweepID = GlobalKey();
final deriveID = GlobalKey();
final qrID = GlobalKey();

class ReceivePage extends ConsumerStatefulWidget {
  const ReceivePage({super.key});

  @override
  ConsumerState<ReceivePage> createState() => ReceivePageState();
}

class ReceivePageState extends ConsumerState<ReceivePage> {
  Account? account;
  Addresses? addresses;
  int uaPools = 0;

  @override
  void initState() {
    super.initState();

    Future(() async {
      final selectedAccount = ref.read(selectedAccountProvider).requireValue!;
      final a = await ref.read(accountProvider(selectedAccount.id).future);
      final pools = a.pool & 6; // Exclude transparent pool
      final addrs = await getAddresses(uaPools: pools);
      setState(() {
        account = a.account;
        addresses = addrs;
        uaPools = pools;
      });
    });
  }

  void tutorial() async {
    tutorialHelper(context, "tutReceive0", [viewID, sweepID, deriveID, qrID]);
  }

  @override
  Widget build(BuildContext context) {
    if (this.account == null) return SizedBox.expand();
    final pinlock = ref.watch(lifecycleProvider);
    if (pinlock.value == true) return PinLock();

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
                  enabled: uaPools,
                  initialValue: uaPools,
                  onChanged: onChangedUAPools,
                ),
              if (addresses.ua != null) ...[
                Gap(8),
                ListTile(
                  title: Text("Unified Address"),
                  subtitle: CopyableText(addresses.ua!),
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
                  subtitle: CopyableText(addresses.oaddr!),
                  trailing: IconButton(
                    icon: Icon(Icons.qr_code),
                    onPressed: () => onShowQR("Orchard", addresses.oaddr!),
                  ),
                ),
              if (addresses.saddr != null)
                ListTile(
                  title: Text("Sapling Address"),
                  subtitle: CopyableText(addresses.saddr!),
                  leading: account.hw != 0 ? IconButton(onPressed: onCheckSapling, icon: Icon(Icons.check)) : null,
                  trailing: IconButton(
                    icon: Icon(Icons.qr_code),
                    onPressed: () => onShowQR("Sapling", addresses.saddr!),
                  ),
                ),
              if (addresses.taddr != null)
                ListTile(
                  title: Text("Transparent Address"),
                  subtitle: CopyableText(addresses.taddr!),
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
      await showLedgerSaplingAddress();
    } on AnyhowException catch (e) {
      await showException(context, e.message);
    }
  }

  void onCheckTransparent() async {
    showSnackbar("Check address on the device");
    try {
      await showLedgerTransparentAddress();
    } on AnyhowException catch (e) {
      await showException(context, e.message);
    }
  }

  void onChangedUAPools(int pools) async {
    uaPools = pools;
    addresses = await getAddresses(uaPools: uaPools);
    setState(() {});
  }

  void onGenerateAddress() async {
    try {
      final confirmed = await confirmDialog(context,
          title: "New Addresses", message: "Do you want to generate a new set of addresses? Previous addresses can still receive funds");
      if (!confirmed) return;
      if (!mounted) return;
      final dialog = await showMessage(context, "Please wait for the address generation\nCheck your Ledger", dismissable: false);
      await generateNextDindex(); // This takes a while on the Ledger
      addresses = await getAddresses(uaPools: uaPools);
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
    final txCounts = await fetchTransparentAddressTxCount();
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
    if (pinlock.value == true) return PinLock();

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
