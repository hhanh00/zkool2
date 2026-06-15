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
import 'package:zkool/widgets/plugin_memo_view.dart';

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
  late final c = coinContext.coin;

  // Memo inline editing state
  bool _editingMemo = false;
  late final TextEditingController _memoController = TextEditingController();
  late final FocusNode _memoFocusNode = FocusNode();

  @override
  void initState() {
    super.initState();
    _memoFocusNode.addListener(_onMemoFocusChange);
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
  void dispose() {
    _memoFocusNode.removeListener(_onMemoFocusChange);
    _memoController.dispose();
    _memoFocusNode.dispose();
    super.dispose();
  }

  void _onMemoFocusChange() {
    if (!_memoFocusNode.hasFocus && _editingMemo) {
      _commitMemoEditing();
    }
  }

  void _startMemoEditing(TxAccount txd) {
    final firstTextMemo = txd.memos
        .map((m) => m.memo)
        .firstWhere((m) => m != null && m.isNotEmpty, orElse: () => null);
    _memoController.text = txd.userMemo ?? firstTextMemo ?? '';
    _memoController.selection = TextSelection.fromPosition(
      TextPosition(offset: _memoController.text.length),
    );
    _editingMemo = true;
    _memoFocusNode.requestFocus();
    setState(() {});
  }

  Future<void> _commitMemoEditing() async {
    _editingMemo = false;
    if (!mounted) return;
    final txd = ref.read(getTxDetailsProvider(widget.idTx)).value;
    if (txd == null) return;

    final newText = _memoController.text.trim();
    // Compute the effective memo before editing
    final firstTextMemo = txd.memos
        .map((m) => m.memo)
        .firstWhere((m) => m != null && m.isNotEmpty, orElse: () => null);
    final oldText = txd.userMemo ?? firstTextMemo ?? '';

    setState(() {});

    if (newText == oldText) return;

    if (newText.isEmpty) {
      await setUserMemo(idTx: txd.id, memo: null, c: c);
    } else {
      await setUserMemo(idTx: txd.id, memo: newText, c: c);
    }
    ref.invalidate(getTxDetailsProvider(widget.idTx));
    if (account != null) {
      ref.invalidate(accountProvider(account!.account.id));
    }
  }

  @override
  Widget build(BuildContext context) {
    if (account == null || idx == null) return blank(context);

    final pinlock = ref.watch(lifecycleProvider);
    if (pinlock.value ?? false) return PinLock();

    final tx = account!.transactions[idx!];
    final txDetailsAV = ref.watch(getTxDetailsProvider(tx.id));

    return Scaffold(
      appBar: AppBar(
        title: Text("Transaction"),
        actions: [
          if (idx != null) IconButton(onPressed: idx! > 0 ? onPrev : null, icon: Icon(Icons.chevron_left)),
          if (idx != null) IconButton(onPressed: idx! < account!.transactions.length - 1 ? onNext : null, icon: Icon(Icons.chevron_right)),
        ],
      ),
      body: txDetailsAV.when(
        loading: () => blank(context),
        error: (error, stack) => showError(error),
        data: (txDetails) => SingleChildScrollView(
            child: Padding(
                padding: EdgeInsets.fromLTRB(8, 0, 8, 8),
                child: Column(
                  children: show(txDetails),
                ))),
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
      ListTile(
        title: Text("Memo"),
        subtitle: _editingMemo
            ? TextField(
                controller: _memoController,
                focusNode: _memoFocusNode,
                maxLines: null,
                minLines: 2,
                textInputAction: TextInputAction.newline,
                onEditingComplete: _commitMemoEditing,
                decoration: const InputDecoration(
                  isDense: true,
                  border: OutlineInputBorder(),
                ),
              )
            : GestureDetector(
                onLongPress: () => _startMemoEditing(txd),
                child: Text(
                  txd.userMemo ?? _firstTextMemo(txd) ?? "—",
                  style: txd.userMemo != null && txd.userMemo!.isNotEmpty
                      ? TextStyle(
                          color: Colors.orange.shade700,
                          fontWeight: FontWeight.w500,
                        )
                      : null,
                ),
              ),
      ),
      Divider(),
      if (txd.spends.isNotEmpty) Text("Spent Notes", style: t.titleSmall),
      ...txd.spends.expand(
        (n) => [
          ListTile(title: Text("Pool"), subtitle: CopyableText(poolToString(n.pool))),
          ListTile(title: Text("Asset"), subtitle: CopyableText(n.assetDisplay)),
          ListTile(
            title: Text("Value"),
            subtitle: n.idAsset != null ? Text(n.value.toString()) : zatToText(n.value, selectable: true),
          ),
          Divider(),
        ],
      ),
      if (txd.notes.isNotEmpty) Text("Received Notes", style: t.titleSmall),
      ...txd.notes.expand(
        (n) => [
          ListTile(title: Text("Pool"), subtitle: CopyableText(poolToString(n.pool))),
          ListTile(title: Text("Asset"), subtitle: CopyableText(n.assetDisplay)),
          ListTile(
            title: Text("Value"),
            subtitle: n.idAsset != null ? Text(n.value.toString()) : zatToText(n.value, selectable: true),
          ),
          Divider(),
        ],
      ),
      if (txd.outputs.isNotEmpty) Text("Outputs", style: t.titleSmall),
      ...txd.outputs.expand(
        (n) => [
          ListTile(title: Text("Pool"), subtitle: CopyableText(poolToString(n.pool))),
          ListTile(
            title: Text("Address"),
            subtitle: n.contactName != null
                ? Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(n.contactName!, style: TextStyle(fontWeight: FontWeight.bold)),
                      CopyableText(n.address),
                    ],
                  )
                : CopyableText(n.address),
          ),
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
            subtitle: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                CopyableText(m.memo ?? "<Binary Content>"),
                PluginMemoView(m.memoBytes),
              ],
            ),
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

  String? _firstTextMemo(TxAccount txd) {
    for (final m in txd.memos) {
      if (m.memo != null && m.memo!.isNotEmpty) return m.memo;
    }
    return null;
  }
}
