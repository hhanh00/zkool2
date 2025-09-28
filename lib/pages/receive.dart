import 'package:flutter/material.dart';
import 'package:gap/gap.dart';
import 'package:go_router/go_router.dart';
import 'package:showcaseview/showcaseview.dart';
import 'package:zkool/src/rust/api/account.dart';
import 'package:zkool/store.dart';
import 'package:zkool/utils.dart';
import 'package:zkool/widgets/pool_select.dart';

final viewID = GlobalKey();
final sweepID = GlobalKey();
final deriveID = GlobalKey();
final qrID = GlobalKey();

class ReceivePage extends StatefulWidget {
  const ReceivePage({super.key});

  @override
  State<ReceivePage> createState() => ReceivePageState();
}

class ReceivePageState extends State<ReceivePage> {
  Addresses? addresses;
  int uaPools = appStore.pools & 6; // Exclude transparent pool

  @override
  void initState() {
    super.initState();

    Future(() async {
      addresses = await getAddresses(uaPools: uaPools);
      setState(() {});
    });
  }

  void tutorial() async {
    tutorialHelper(context, "tutReceive0", [viewID, sweepID, deriveID, qrID]);
  }

  @override
  Widget build(BuildContext context) {
    Future(tutorial);
    final addresses = this.addresses;

    return Scaffold(
        appBar: AppBar(
          title: Text("Receive Funds"),
          actions: [
            Showcase(key: viewID, description: "View Transparent Addresses", child:
            IconButton(
                tooltip: "Transparent Addresses",
                onPressed: onViewTransparentAddresses,
                icon: Icon(Icons.visibility),),),
            Showcase(key: sweepID, description: "Find other transparent addresses. If you restored from a wallet that has address rotation (such as Ledger, Exodus, etc), Tap, then Reset and Sync", child:
            IconButton(
                tooltip: "Sweep",
                onPressed: onSweep,
                icon: Icon(Icons.search),),),
            Showcase(key: deriveID, description: "Generate a new set of addresses (transparent/sapling and orchard). Previous addresses can still receive funds", child:
            IconButton(
                tooltip: "Next Set of Addresses",
                onPressed: onGenerateAddress,
                icon: Icon(Icons.skip_next),),),
          ],
        ),
        body: addresses == null
            ? SizedBox.shrink()
            : SingleChildScrollView(
                child: Padding(
                    padding: EdgeInsets.symmetric(horizontal: 8),
                    child: Column(children: [
                      if (addresses.saddr != null ||
                          addresses.oaddr != null)
                        PoolSelect(enabled: appStore.pools,
                          initialValue: uaPools,
                          onChanged: onChangedUAPools,),
                      if (addresses.ua != null)
                        ...[
                        Gap(8),
                        ListTile(
                          title: Text("Unified Address"),
                          subtitle: CopyableText(addresses.ua!),
                          trailing: Showcase(key: qrID, description: "Show address as a QR Code", child:
                          IconButton(
                            icon: Icon(Icons.qr_code),
                            onPressed: () => onShowQR("Unified Address", addresses.ua!),
                          ),),
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
                          trailing: IconButton(
                            icon: Icon(Icons.qr_code),
                            onPressed: () => onShowQR("Sapling", addresses.saddr!),
                          ),
                        ),
                      if (addresses.taddr != null)
                        ListTile(
                          title: Text("Transparent Address"),
                          subtitle: CopyableText(addresses.taddr!),
                          trailing: IconButton(
                            icon: Icon(Icons.qr_code),
                            onPressed: () => onShowQR("Transparent", addresses.taddr!),
                          ),
                        ),
                    ],),),),);
  }

  void onChangedUAPools(int pools) async {
    uaPools = pools;
    addresses = await getAddresses(uaPools: uaPools);
    setState(() {});
  }

  void onGenerateAddress() async {
    final confirmed = await confirmDialog(context, title: "New Addresses", message: "Do you want to generate a new set of addresses? Previous addresses can still receive funds");
    if (!confirmed) return;
    await generateNextDindex();
    addresses = await getAddresses(uaPools: uaPools);
    setState(() {});
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
    await GoRouter.of(context).push("/sweep");
  }
}

class TransparentAddressesPage extends StatelessWidget {
  final List<TAddressTxCount> txCounts;

  const TransparentAddressesPage({super.key, required this.txCounts});

  @override
  Widget build(BuildContext context) {
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
            trailing: Text(zatToString(txCount.amount)),);
        },
      ),
    );
  }
}
