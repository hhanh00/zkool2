import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';
import 'package:showcaseview/showcaseview.dart';
import 'package:zkool/src/rust/api/account.dart';
import 'package:zkool/src/rust/api/network.dart';
import 'package:zkool/utils.dart';

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

  @override
  void initState() {
    super.initState();

    Future(() async {
      addresses = await getAddresses();
      setState(() {});
    });
  }

  void tutorial() async {
    tutorialHelper(context, "tutReceive0", [sweepID, deriveID, qrID]);
  }

  @override
  Widget build(BuildContext context) {
    Future(tutorial);
    final addresses = this.addresses;

    return Scaffold(
        appBar: AppBar(
          title: Text("Receive Funds"),
          actions: [
            Showcase(key: sweepID, description: "Find other transparent addresses. If you restored from a wallet that has address rotation (such as Ledger, Exodus, etc), Tap, then Reset and Sync", child:
            IconButton(
                tooltip: "Sweep",
                onPressed: onSweep,
                icon: Icon(Icons.search))),
            Showcase(key: deriveID, description: "Generate a new set of addresses (transparent/sapling and orchard). Previous addresses can still receive funds", child:
            IconButton(
                tooltip: "Next Set of Addresses",
                onPressed: onGenerateAddress,
                icon: Icon(Icons.skip_next))),
          ],
        ),
        body: addresses == null
            ? SizedBox.shrink()
            : SingleChildScrollView(
                child: Padding(
                    padding: EdgeInsets.symmetric(horizontal: 8),
                    child: Column(children: [
                      if (addresses.ua != null)
                        ListTile(
                          title: Text("Unified Address"),
                          subtitle: SelectableText(addresses.ua!),
                          trailing: Showcase(key: qrID, description: "Show address as a QR Code", child:
                          IconButton(
                            icon: Icon(Icons.qr_code),
                            onPressed: () => onShowQR("Unified Address", addresses.ua!),
                          )),
                        ),
                      if (addresses.oaddr != null)
                        ListTile(
                          title: Text("Orchard only Address"),
                          subtitle: SelectableText(addresses.oaddr!),
                          trailing: IconButton(
                            icon: Icon(Icons.qr_code),
                            onPressed: () => onShowQR("Orchard", addresses.oaddr!),
                          ),
                        ),
                      if (addresses.saddr != null)
                        ListTile(
                          title: Text("Sapling Address"),
                          subtitle: SelectableText(addresses.saddr!),
                          trailing: IconButton(
                            icon: Icon(Icons.qr_code),
                            onPressed: () => onShowQR("Sapling", addresses.saddr!),
                          ),
                        ),
                      if (addresses.taddr != null)
                        ListTile(
                          title: Text("Transparent Address"),
                          subtitle: SelectableText(addresses.taddr!),
                          trailing: IconButton(
                            icon: Icon(Icons.qr_code),
                            onPressed: () => onShowQR("Transparent", addresses.taddr!),
                          ),
                        ),
                    ]))));
  }

  void onGenerateAddress() async {
    await generateNextDindex();
    addresses = await getAddresses();

    setState(() {});
  }

  void onShowQR(String title, String text) {
    GoRouter.of(context).push("/qr", extra: {"title": title, "text": text});
  }

  void onSweep() async {
    showSnackbar("Starting sweep");
    final endHeight = await getCurrentHeight();
    await transparentSweep(endHeight: endHeight, gapLimit: 40);
    showSnackbar("Sweep complete");
  }
}
