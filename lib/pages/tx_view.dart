import 'dart:typed_data';

import 'package:flutter/material.dart';
import 'package:intl/intl.dart';
import 'package:url_launcher/url_launcher.dart';
import 'package:zkool/pages/tx.dart';
import 'package:zkool/src/rust/account.dart';
import 'package:zkool/src/rust/api/account.dart';
import 'package:zkool/store.dart';
import 'package:zkool/utils.dart';

class TxView extends StatefulWidget {
  final int idTx;
  const TxView(this.idTx, {super.key});

  @override
  State<TxView> createState() => TxViewState();
}

class TxViewState extends State<TxView> {
  TxAccount? txDetails;

  @override
  void initState() {
    super.initState();
    Future(() async {
      final txd = await getTxDetails(idTx: widget.idTx);
      setState(() => txDetails = txd);
    });
  }

  @override
  Widget build(BuildContext context) {
    final txd = txDetails;

    return Scaffold(
      appBar: AppBar(
        title: Text("Transaction"),
      ),
      body: SingleChildScrollView(
        child: Column(
          children: [if (txd != null) ...show(txd)],
        ),
      ),
    );
  }

  List<Widget> show(TxAccount txd) {
    final t = Theme.of(context).textTheme;
    final amountSpent = txd.spends.map((n) => n.value).fold(BigInt.zero, (a, b) => a + b);
    final amountReceived = txd.notes.map((n) => n.value).fold(BigInt.zero, (a, b) => a + b);

    return [
      ListTile(
        title: Text("Transaction ID"),
        subtitle: CopyableText(txIdToString(txd.txid)),
        trailing: IconButton(onPressed: () => openBlockExplorer(txd.txid), icon: Icon(Icons.open_in_browser)),
      ),
      ListTile(
        title: Text("Block Height"),
        subtitle: CopyableText(txd.height.toString()),
      ),
      ListTile(
        title: Text("Timestamp"),
        subtitle: CopyableText(exactTimeToString(txd.time)),
      ),
      ListTile(
        title: Text("Amount Spent"),
        subtitle: zatToText(amountSpent, selectable: true),
      ),
      ListTile(
        title: Text("Amount Received"),
        subtitle: zatToText(amountReceived, selectable: true),
      ),
      ListTile(
        title: Text("Amount Transacted"),
        subtitle: zatToText(amountReceived - amountSpent, selectable: true),
      ),
      ListTile(
        title: Text("Price"),
        subtitle: Text(txd.price != null ? NumberFormat.compactCurrency(decimalDigits: 2).format(txd.price) : "N/A"),
      ),
      ListTile(
        title: Text("Category"),
        subtitle: CopyableText(txd.category ?? "Unknown"),
      ),
      Divider(),
      if (txd.spends.isNotEmpty) Text("Spent Notes", style: t.titleSmall),
      ...txd.spends.expand(
        (n) => [
          ListTile(title: Text("Pool"), subtitle: CopyableText(poolToString(n.pool))),
          ListTile(
            title: Text("Value"),
            subtitle: zatToText(n.value, selectable: true),
          ),
          Divider(),
        ],
      ),
      if (txd.notes.isNotEmpty) Text("Received Notes", style: t.titleSmall),
      ...txd.notes.expand(
        (n) => [
          ListTile(title: Text("Pool"), subtitle: CopyableText(poolToString(n.pool))),
          ListTile(
            title: Text("Value"),
            subtitle: zatToText(n.value, selectable: true),
          ),
          Divider(),
        ],
      ),
      if (txd.outputs.isNotEmpty) Text("Outputs", style: t.titleSmall),
      ...txd.outputs.expand(
        (n) => [
          ListTile(title: Text("Pool"), subtitle: CopyableText(poolToString(n.pool))),
          ListTile(title: Text("Address"), subtitle: CopyableText(n.address)),
          ListTile(
            title: Text("Value"),
            subtitle: zatToText(n.value, selectable: true),
          ),
          Divider(),
        ],
      ),
      if (txd.memos.isNotEmpty) Text("Memos", style: t.titleSmall),
      ...txd.memos.expand(
        (m) => [
          ListTile(title: Text("Pool"), subtitle: CopyableText(poolToString(m.pool))),
          ListTile(
            title: Text("Memo"),
            subtitle: CopyableText(m.memo ?? "<Binary Content>"),
          ),
          Divider(),
        ],
      ),
    ];
  }

  void openBlockExplorer(Uint8List txid) async {
    final blockExplorer = appStore.blockExplorer;
    final url = blockExplorer.replaceAll("{net}", appStore.net).replaceAll("{txid}", txIdToString(txid));
    await launchUrl(Uri.parse(url));
  }
}
