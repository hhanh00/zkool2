import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:intl/intl.dart';
import 'package:url_launcher/url_launcher.dart';
import 'package:zkool/main.dart';
import 'package:zkool/pages/tx.dart';
import 'package:zkool/src/rust/api/account.dart';
import 'package:zkool/src/rust/api/transaction.dart';
import 'package:zkool/store.dart';
import 'package:zkool/utils.dart';

class TxViewPage extends ConsumerStatefulWidget {
  final int idTx;
  const TxViewPage(this.idTx, {super.key});

  @override
  ConsumerState<TxViewPage> createState() => TxViewPageState();
}

class TxViewPageState extends ConsumerState<TxViewPage> {
  AccountData? account;
  int? idx;
  List<Category>? categoryList;
  late final c = ref.read(coinContextProvider);

  @override
  void initState() {
    super.initState();
    Future(() async {
      final selectedAccount = ref.read(selectedAccountProvider).requireValue!;
      final account = await ref.read(accountProvider(selectedAccount.id).future);
      int? idx = account.transactions.indexWhere((tx) => tx.id == widget.idTx);
      if (idx < 0) throw Error();
      final categoryList = await ref.read(getCategoriesProvider.future);
      setState(() {
        this.idx = idx;
        this.account = account;
        this.categoryList = categoryList;
      });
    });
  }

  @override
  Widget build(BuildContext context) {
    if (account == null || idx == null) return blank(context);

    final pinlock = ref.watch(lifecycleProvider);
    if (pinlock.value ?? false) return PinLock();

    final tx = account!.transactions[idx!];
    final TxAccount txDetails;
    try {
      final txDetailsAV = ref.watch(getTxDetailsProvider(tx.id));
      ensureAV(context, txDetailsAV);
      txDetails = txDetailsAV.requireValue;
    } on Widget catch (w) {
      return w;
    }

    return Scaffold(
        appBar: AppBar(
          title: Text("Transaction"),
          actions: [
            if (idx != null) IconButton(onPressed: idx! > 0 ? onPrev : null, icon: Icon(Icons.chevron_left)),
            if (idx != null) IconButton(onPressed: idx! < account!.transactions.length - 1 ? onNext : null, icon: Icon(Icons.chevron_right)),
          ],
        ),
        body: SingleChildScrollView(
            child: Column(
          children: show(txDetails),
        )));
  }

  Future<void> onPrev() async {
    await gotoToTx(idx! - 1);
  }

  Future<void> onNext() async {
    await gotoToTx(idx! + 1);
  }

  Future<void> gotoToTx(int newIdx) async {
    setState(() => idx = newIdx);
  }

  List<Widget> show(TxAccount txd) {
    final t = Theme.of(context).textTheme;
    final amountSpent = txd.spends.map((n) => n.value).fold(BigInt.zero, (a, b) => a + b);
    final amountReceived = txd.notes.map((n) => n.value).fold(BigInt.zero, (a, b) => a + b);
    final categories = [DropdownMenuEntry(value: null, label: "Unknown"), ...categoryList!.map((c) => DropdownMenuEntry(value: c.id, label: c.name))];

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
                initialValue: doubleToString(txd.price!, decimals: 3),
                onChanged: (v) => onPriceChanged(txd.id, v),
                keyboardType: TextInputType.numberWithOptions(decimal: true),
              )
            : Text("N/A"),
      ),
      ListTile(
        title: Text("Category"),
        subtitle: DropdownMenu(initialSelection: txd.category, onSelected: (v) => onChangeTxCategory(txd.id, v), dropdownMenuEntries: categories),
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
    final settings = ref.read(appSettingsProvider).requireValue;
    final blockExplorer = settings.blockExplorer;
    final url = blockExplorer.replaceAll("{net}", settings.net).replaceAll("{txid}", txIdToString(txid));
    await launchUrl(Uri.parse(url));
  }

  void onPriceChanged(int id, String? v) async {
    final price = v?.let(((v) => v.isNotEmpty ? NumberFormat().parse(v).toDouble() : null));
    await setTxPrice(id: id, price: price, c: c);
  }

  void onChangeTxCategory(int id, int? category) async {
    await setTxCategory(id: id, category: category, c: c);
  }
}
