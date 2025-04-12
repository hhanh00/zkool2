import 'package:flutter/material.dart';
import 'package:zkool/src/rust/api/account.dart';

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

  @override
  Widget build(BuildContext context) {
    final addresses = this.addresses;

    return Scaffold(
        appBar: AppBar(
          title: Text("Receive Funds"),
          actions: [
            IconButton(tooltip: "Next Set of Addresses",
                onPressed: onGenerateAddress, icon: Icon(Icons.skip_next)),
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
                          trailing: IconButton(
                            icon: Icon(Icons.qr_code),
                            onPressed: () {},
                          ),
                        ),
                      if (addresses.oaddr != null)
                        ListTile(
                          title: Text("Orchard only Address"),
                          subtitle: SelectableText(addresses.oaddr!),
                          trailing: IconButton(
                            icon: Icon(Icons.qr_code),
                            onPressed: () {},
                          ),
                        ),
                      if (addresses.saddr != null)
                        ListTile(
                          title: Text("Sapling Address"),
                          subtitle: SelectableText(addresses.saddr!),
                          trailing: IconButton(
                            icon: Icon(Icons.qr_code),
                            onPressed: () {},
                          ),
                        ),
                      if (addresses.taddr != null)
                        ListTile(
                          title: Text("Transparent Address"),
                          subtitle: SelectableText(addresses.taddr!),
                          trailing: IconButton(
                            icon: Icon(Icons.qr_code),
                            onPressed: () {},
                          ),
                        ),
                    ]))));
  }

  void onGenerateAddress() async {
    await generateNextDindex();
    addresses = await getAddresses();

    setState(() {});
  }
}
