import 'dart:async';

import 'package:convert/convert.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_form_builder/flutter_form_builder.dart';
import 'package:flutter_mobx/flutter_mobx.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated.dart';
import 'package:gap/gap.dart';
import 'package:go_router/go_router.dart';
import 'package:image_picker/image_picker.dart';
import 'package:zkool/router.dart';
import 'package:zkool/src/rust/api/account.dart';
import 'package:zkool/src/rust/api/sync.dart';
import 'package:zkool/store.dart';
import 'package:zkool/utils.dart';
import 'package:file_picker/file_picker.dart';

class AccountViewPage extends StatefulWidget {
  final Account account;
  const AccountViewPage(this.account, {super.key});

  @override
  State<AccountViewPage> createState() => AccountViewPageState();
}

class AccountViewPageState extends State<AccountViewPage> with RouteAware {
  StreamSubscription<SyncProgress>? progressSubscription;
  late int height = widget.account.height;
  PoolBalance? poolBalance;

  @override
  void initState() {
    super.initState();
    setAccount(id: widget.account.id);
    AppStoreBase.instance.accountName = widget.account.name;
    Future(refresh);
  }

  @override
  void didPushNext() {
    unsubscribeFromSync();
    super.didPushNext();
  }

  @override
  void didPopNext() {
    subscribeToSync();
    super.didPopNext();
  }

  @override
  void didPush() {
    subscribeToSync();
    super.didPush();
  }

  @override
  void didPop() {
    unsubscribeFromSync();
    super.didPop();
  }

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    final route = ModalRoute.of(context);
    if (route is PageRoute) {
      routeObserver.subscribe(this, route);
    }
  }

  @override
  void dispose() {
    routeObserver.unsubscribe(this);
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final t = Theme.of(context).textTheme;
    final h = height;
    final b = poolBalance;

    return DefaultTabController(
        length: 2,
        child: Scaffold(
            appBar: AppBar(
              title: Text(widget.account.name),
              actions: [
                IconButton(
                    tooltip: "Open Log",
                    onPressed: () => onOpenLog(context),
                    icon: Icon(Icons.description)),
                IconButton(
                    tooltip: "Sync this account",
                    onPressed: onSync,
                    icon: Icon(Icons.sync)),
                IconButton(
                    tooltip: "Rewind to previous checkpoint",
                    onPressed: onRewind,
                    icon: Icon(Icons.fast_rewind)),
                IconButton(
                    tooltip: "Receive Funds",
                    onPressed: onReceive,
                    icon: Icon(Icons.download)),
                IconButton(
                    tooltip: "Send Funds",
                    onPressed: onSend,
                    icon: Icon(Icons.send)),
              ],
              bottom: TabBar(
                tabs: [
                  Tab(text: 'Transactions'),
                  Tab(text: 'Memos'),
                ],
              ),
            ),
            body: Padding(
              padding: const EdgeInsets.symmetric(horizontal: 16),
              child: Observer(builder: (context) {
                // make sure there is a dependency on transactions
                AppStoreBase.instance.transactions.length;

                return TabBarView(children: [
                  SingleChildScrollView(
                      child: Column(children: [
                    Text("Height"),
                    Gap(8),
                    Text(h.toString(), style: t.bodyLarge),
                    Gap(16),
                    Text("Balance"),
                    Gap(8),
                    if (b != null)
                      Row(children: [
                        Text("T: ${zatToString(b.field0[0])}"),
                        const Gap(8),
                        Text("S: ${zatToString(b.field0[1])}"),
                        const Gap(8),
                        Text("O: ${zatToString(b.field0[2])}"),
                      ]),
                    Gap(8),
                    if (b != null)
                      Text(
                          "\u2211: ${zatToString(b.field0[0] + b.field0[1] + b.field0[2])}"),
                    Gap(16),
                    ...showTxHistory(AppStoreBase.instance.transactions),
                  ])),
                  SingleChildScrollView(
                      child: Column(
                    children: [
                      ...showMemos(AppStoreBase.instance.memos),
                    ],
                  ))
                ]);
              }),
            )));
  }

  void onSync() async {
    try {
      final progress =
          await AppStoreBase.instance.startSynchronize([widget.account.id]);
      if (progress == null) return;
      await progressSubscription?.cancel();
      progressSubscription = null;
      subscribeToSync();
    } on AnyhowException catch (e) {
      if (mounted) await showException(context, e.message);
    }
  }

  void onRewind() async {
    final dbHeight = await getDbHeight();
    await rewindSync(height: dbHeight - 60);
    final h = await getDbHeight();
    setState(() => height = h);
  }

  void onReceive() async {
    await GoRouter.of(context).push("/receive");
  }

  void onSend() async {
    await GoRouter.of(context).push("/send");
  }

  void subscribeToSync() {
    progressSubscription =
        AppStoreBase.instance.syncs[widget.account.id]?.listen(
      (event) {
        setState(() {
          height = event.height;
        });
      },
      onDone: refresh,
    );
  }

  void unsubscribeFromSync() {
    progressSubscription?.cancel();
    progressSubscription = null;
  }

  void refresh() async {
    final b = await balance();
    final h = await getDbHeight();
    await AppStoreBase.instance.loadAccounts();
    await AppStoreBase.instance.loadTxHistory();
    await AppStoreBase.instance.loadMemos();
    if (!mounted) return;
    setState(() {
      poolBalance = b;
      height = h;
    });
  }
}

class AccountEditPage extends StatefulWidget {
  final List<Account> accounts;
  const AccountEditPage(this.accounts, {super.key});

  @override
  State<AccountEditPage> createState() => AccountEditPageState();
}

class AccountEditPageState extends State<AccountEditPage> {
  late List<Account> accounts = widget.accounts;
  final formKey = GlobalKey<FormBuilderState>();

  @override
  void didUpdateWidget(covariant AccountEditPage oldWidget) {
    accounts = widget.accounts;
    super.didUpdateWidget(oldWidget);
  }

  @override
  Widget build(BuildContext context) {
    final account = accounts.length == 1 ? accounts.first : null;

    return Scaffold(
        appBar: AppBar(title: Text('Account Edit'), actions: [
          if (account != null) IconButton(
              tooltip: "Export Account",
              onPressed: onExport,
              icon: Icon(Icons.reset_tv)),
          IconButton(
              tooltip: "Clear Sync Data",
              onPressed: onReset,
              icon: Icon(Icons.delete_sweep))
        ]),
        body: Padding(
          padding: const EdgeInsets.symmetric(horizontal: 16),
          child: FormBuilder(
            key: formKey,
              child: Column(
            children: [
              Row(
                children: [
                  Expanded(
                      child: FormBuilderTextField(
                    name: 'name',
                    decoration: InputDecoration(labelText: 'Name'),
                    initialValue: account?.name ?? "(Multiple)",
                    readOnly: account == null,
                    onChanged: (account != null) ? onEditName : null,
                  )),
                  if (account != null) account.avatar(onTap: (_) => onEditIcon())
                ],
              ),
              FormBuilderTextField(
                name: 'birth',
                decoration: InputDecoration(labelText: 'Birth Height'),
                initialValue: account?.birth.toString() ?? "",
                keyboardType: TextInputType.number,
                readOnly: account == null,
                inputFormatters: [FilteringTextInputFormatter.digitsOnly],
                onChanged: (account != null) ? onEditBirth : null,
              ),
              FormBuilderCheckbox(
                name: "enabled",
                title: Text("Enabled"),
                initialValue:
                  accounts.every((a) => a.enabled == accounts[0].enabled) ? accounts[0].enabled : null,
                tristate: account == null,
                onChanged: onEditEnabled,
              ),
              FormBuilderCheckbox(
                name: "hidden",
                title: Text("Hidden"),
                initialValue:
                  accounts.every((a) => a.hidden == accounts[0].hidden) ? accounts[0].hidden : null,
                tristate: account == null,
                onChanged: onEditHidden,
              )
            ],
          )),
        ));
  }

  void onEditName(String? name) async {
    if (name != null) {
      accounts[0] = accounts[0].copyWith(name: name);
      await updateAccount(
          update:
              AccountUpdate(coin: accounts[0].coin, id: accounts[0].id, name: name));
      await AppStoreBase.instance.loadAccounts();
      setState(() {});
    }
  }

  void onEditIcon() async {
    final picker = ImagePicker();
    final icon = await picker.pickImage(source: ImageSource.gallery);
    if (icon != null) {
      final bytes = await icon.readAsBytes();
      accounts[0] = accounts[0].copyWith(icon: bytes);
      await updateAccount(
          update:
              AccountUpdate(coin: accounts[0].coin, id: accounts[0].id, icon: bytes));
      await AppStoreBase.instance.loadAccounts();
      setState(() {});
    }
  }

  void onEditBirth(String? birth) async {
    if (birth != null && birth.isNotEmpty) {
      accounts[0] = accounts[0].copyWith(birth: int.parse(birth));
      await updateAccount(
          update: AccountUpdate(
              coin: accounts[0].coin, id: accounts[0].id, birth: int.parse(birth)));
      await AppStoreBase.instance.loadAccounts();
      setState(() {});
    }
  }

  void onEditEnabled(bool? v) async {
    if (v == null) return;
    for (var i = 0; i < accounts.length; i++) {
      accounts[i] = accounts[i].copyWith(enabled: v);
      await updateAccount(
          update: AccountUpdate(coin: accounts[i].coin, id: accounts[i].id, enabled: v));
    }
    await AppStoreBase.instance.loadAccounts();
    setState(() {});
  }

  void onEditHidden(bool? v) async {
    if (v == null) return;
    for (var i = 0; i < accounts.length; i++) {
      accounts[i] = accounts[i].copyWith(hidden: v);
      await updateAccount(
          update: AccountUpdate(coin: accounts[i].coin, id: accounts[i].id, hidden: v));
    }
    await AppStoreBase.instance.loadAccounts();
    setState(() {});
  }

  void onExport() async {
    final account = accounts.first;
    final password = await inputPassword(context,
        title: "Export Account", message: "File Password");
    if (password != null) {
      final res = await exportAccount(passphrase: password);
      await FilePicker.platform.saveFile(
        dialogTitle: 'Please select an output file for the encrypted account:',
        fileName: '${account.name}.bin',
        bytes: res,
      );
    }
  }

  void onReset() async {
    for (var account in accounts)
      await resetSync(id: account .id);
    await AppStoreBase.instance.loadAccounts();
  }
}

extension AccountExtension on Account {
  Widget avatar({bool? selected, void Function(bool?)? onTap}) {
    final i = initials(name);
    final s = selected ?? false;
    return GestureDetector(
        onTap: () => onTap?.call(!s),
        child: CircleAvatar(
          backgroundColor: s ? Colors.blue.shade700 : Colors.grey.shade300,
          child: s
              ? Icon(Icons.check, color: Colors.white)
              : icon != null
                  ? ClipOval(child: Image.memory(icon!))
                  : Text(i),
        ));
  }
}

List<Widget> showTxHistory(List<Tx> transactions) {
  return [
    const Text("Transaction History"),
    const Gap(8),
    ListView.builder(
      shrinkWrap: true,
      physics: const NeverScrollableScrollPhysics(),
      itemCount: transactions.length,
      itemBuilder: (context, index) {
        final tx = transactions[index];
        // encode tx.txid to hex string
        String txId = txIdToString(tx.txid);

        return ListTile(
          leading: Text("${tx.height}"),
          title: SelectableText(txId),
          subtitle: Text(timeToString(tx.time)),
          trailing: Text(zatToString(BigInt.from(tx.value))),
        );
      },
    ),
  ];
}

List<Widget> showMemos(List<Memo> memos) {
  return [
    ListView.builder(
      shrinkWrap: true,
      physics: const NeverScrollableScrollPhysics(),
      itemCount: memos.length,
      itemBuilder: (context, index) {
        final memo = memos[index];

        return ListTile(
          leading: Text("${memo.height}"),
          title: SelectableText(memo.memo ?? hex.encode(memo.memoBytes)),
          subtitle: Text(timeToString(memo.time)),
          trailing: Text(memo.idNote != null ? "In" : "Out"),
        );
      },
    ),
  ];
}

void onOpenLog(BuildContext context) async {
  await GoRouter.of(context).push("/log");
}
