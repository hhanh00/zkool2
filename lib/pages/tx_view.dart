import 'package:flutter/material.dart';
import 'package:zkool/pages/tx.dart';
import 'package:zkool/src/rust/account.dart';
import 'package:zkool/src/rust/api/account.dart';
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
    final amountReceived =
        txd.notes.map((n) => n.value).fold(BigInt.zero, (a, b) => a + b);

    return [
      ListTile(
          title: Text("Transaction ID"),
          subtitle: Text(txIdToString(txd.txid))),
      ListTile(
          title: Text("Block Height"), subtitle: Text(txd.height.toString())),
      ListTile(
          title: Text("Timestamp"), subtitle: Text(exactTimeToString(txd.time))),
      ListTile(
          title: Text("Amount Spent"),
          subtitle: Text(zatToString(amountSpent))),
      ListTile(
          title: Text("Amount Received"),
          subtitle: Text(zatToString(amountReceived))),
      ListTile(
          title: Text("Amount Transacted"),
          subtitle: Text(zatToString(amountReceived - amountSpent))),
      Divider(),
      if (txd.spends.isNotEmpty) Text("Spent Notes", style: t.titleSmall),
      ...txd.spends.expand((n) => [
            ListTile(title: Text("Pool"), subtitle: Text(poolToString(n.pool))),
            ListTile(
                title: Text("Value"), subtitle: Text(zatToString(n.value))),
            Divider(),
          ]),
      if (txd.notes.isNotEmpty) Text("Received Notes", style: t.titleSmall),
      ...txd.notes.expand((n) => [
            ListTile(title: Text("Pool"), subtitle: Text(poolToString(n.pool))),
            ListTile(
                title: Text("Value"), subtitle: Text(zatToString(n.value))),
            Divider()
          ]),
      if (txd.memos.isNotEmpty) Text("Memos", style: t.titleSmall),
      ...txd.memos.expand((m) => [
            ListTile(title: Text("Pool"), subtitle: Text(poolToString(m.pool))),
            ListTile(
                title: Text("Memo"),
                subtitle: Text(m.memo ?? "<Binary Content>")),
            Divider()
          ]),
    ];
  }
}
