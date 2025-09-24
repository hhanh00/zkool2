import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:intl/intl.dart';
import 'package:url_launcher/url_launcher.dart';
import 'package:zkool/pages/tx.dart';
import 'package:zkool/src/rust/account.dart';
import 'package:zkool/src/rust/api/account.dart';
import 'package:zkool/src/rust/api/transaction.dart';
import 'package:zkool/store.dart';
import 'package:zkool/utils.dart';

class TxViewPage extends StatefulWidget {
  final int idTx;
  const TxViewPage(this.idTx, {super.key});

  @override
  State<TxViewPage> createState() => TxViewPageState();
}

class TxViewPageState extends State<TxViewPage> {
  TxAccount? txDetails;
  late int? idx;
  late int idTx = widget.idTx;

  @override
  void initState() {
    super.initState();
    idx = appStore.transactions.indexWhere((tx) => tx.id == idTx);
    if (idx! < 0) idx = null;
    Future(refresh);
  }

  Future<void> refresh() async {
    final txd = await getTxDetails(idTx: idTx);
    txDetails = txd;
    setState(() {});
  }

  @override
  Widget build(BuildContext context) {
    final txd = txDetails;

    return Scaffold(
      appBar: AppBar(
        title: Text("Transaction"),
        actions: [
          if (idx != null) IconButton(onPressed: idx! > 0 ? onPrev : null, icon: Icon(Icons.chevron_left)),
          if (idx != null) IconButton(onPressed: idx! < appStore.transactions.length - 1 ? onNext : null, icon: Icon(Icons.chevron_right)),
        ],
      ),
      body: SingleChildScrollView(
        child: Column(
          children: [if (txd != null) ...show(txd)],
        ),
      ),
    );
  }

  Future<void> onPrev() async {
    await gotoToTx(idx! - 1);
  }

  Future<void> onNext() async {
    await gotoToTx(idx! + 1);
  }

  Future<void> gotoToTx(int newIdx) async {
    idx = newIdx;
    idTx = appStore.transactions[newIdx].id;
    await refresh();
  }

  List<Widget> show(TxAccount txd) {
    final t = Theme.of(context).textTheme;
    final amountSpent = txd.spends.map((n) => n.value).fold(BigInt.zero, (a, b) => a + b);
    final amountReceived = txd.notes.map((n) => n.value).fold(BigInt.zero, (a, b) => a + b);
    final categories = [DropdownMenuEntry(value: null, label: "Unknown"), ...appStore.categories.map((c) => DropdownMenuEntry(value: c.id, label: c.name))];

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
        subtitle: txd.price != null
            ? TextFormField(
                initialValue: txd.price!.toStringAsFixed(3),
                onChanged: (v) => onPriceChanged(txd.id, v),
                keyboardType: TextInputType.numberWithOptions(decimal: true),
              )
            : Text("N/A"),
      ),
      ListTile(
          title: Text("Category"),
          subtitle: DropdownMenu(initialSelection: txd.category, onSelected: (v) => onChangeTxCategory(txd.id, v), dropdownMenuEntries: categories)),
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

  void onPriceChanged(int id, String? v) async {
    final price = v?.let(((v) => v.isNotEmpty ? NumberFormat().parse(v).toDouble() : null));
    await setTxPrice(id: id, price: price);
  }

  void onChangeTxCategory(int id, int? category) async {
    await setTxCategory(id: id, category: category);
  }
}
