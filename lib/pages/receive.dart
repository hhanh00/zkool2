import 'package:flutter/material.dart';

class ReceivePage extends StatefulWidget {
  const ReceivePage({super.key});

  @override
  State<ReceivePage> createState() => ReceivePageState();
}

class ReceivePageState extends State<ReceivePage> {
  @override
  Widget build(BuildContext context) {
    return Scaffold(
        appBar: AppBar(
          title: Text("Receive Funds"),
        ),
        body: SingleChildScrollView(
            child: Padding(
                padding: EdgeInsets.symmetric(horizontal: 8),
                child: Column(children: [
                  ListTile(
                    title: Text("Unified Address"),
                    subtitle: Text("t1a2b3c4d5e6f7g8h9i0j"),
                    trailing: IconButton(
                      icon: Icon(Icons.qr_code),
                      onPressed: () {},
                    ),
                  ),
                  ListTile(
                    title: Text("Orchard only Address"),
                    subtitle: Text("t1a2b3c4d5e6f7g8h9i0j"),
                    trailing: IconButton(
                      icon: Icon(Icons.qr_code),
                      onPressed: () {},
                    ),
                  ),
                  ListTile(
                    title: Text("Sapling Address"),
                    subtitle: Text("t1a2b3c4d5e6f7g8h9i0j"),
                    trailing: IconButton(
                      icon: Icon(Icons.qr_code),
                      onPressed: () {},
                    ),
                  ),
                  ListTile(
                    title: Text("Transparent Address"),
                    subtitle: Text("t1a2b3c4d5e6f7g8h9i0j"),
                    trailing: IconButton(
                      icon: Icon(Icons.qr_code),
                      onPressed: () {},
                    ),
                  ),
                  ElevatedButton(
                      onPressed: () {}, child: Text("Generate New Addresses"))
                ]))));
  }
}
