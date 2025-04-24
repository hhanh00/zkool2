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
import 'package:qr_flutter/qr_flutter.dart';
import 'package:showcaseview/showcaseview.dart';
import 'package:zkool/pages/tx.dart';
import 'package:zkool/router.dart';
import 'package:zkool/src/rust/account.dart';
import 'package:zkool/src/rust/api/account.dart';
import 'package:zkool/src/rust/api/sync.dart';
import 'package:zkool/store.dart';
import 'package:zkool/utils.dart';
import 'package:file_picker/file_picker.dart';
import 'package:zkool/widgets/pool_select.dart';

final logID = GlobalKey();
final sync1ID = GlobalKey();
final rewindID = GlobalKey();
final receiveID = GlobalKey();
final sendID = GlobalKey();
final tBalID = GlobalKey();
final sBalID = GlobalKey();
final oBalID = GlobalKey();
final balID = GlobalKey();
final txdID = GlobalKey();

class AccountViewPage extends StatefulWidget {
  final Account account;
  const AccountViewPage(this.account, {super.key});

  @override
  State<AccountViewPage> createState() => AccountViewPageState();
}

class AccountViewPageState extends State<AccountViewPage> {
  StreamSubscription<SyncProgress>? progressSubscription;
  PoolBalance? poolBalance;

  @override
  void initState() {
    super.initState();
    setAccount(id: widget.account.id);
    AppStoreBase.instance.accountName = widget.account.name;
    Future(refresh);
  }

  void tutorial() async {
    tutorialHelper(context, "tutAccount0", [
      tBalID,
      sBalID,
      oBalID,
      balID,
      logID,
      sync1ID,
      rewindID,
      receiveID,
      sendID
    ]);
    if (AppStoreBase.instance.transactions.isNotEmpty)
      tutorialHelper(context, "tutAccount1", [
        txdID,
      ]);
  }

  @override
  Widget build(BuildContext context) {
    final t = Theme.of(context).textTheme;
    final b = poolBalance;

    Future(tutorial);

    return DefaultTabController(
        length: 3,
        child: Scaffold(
            appBar: AppBar(
              title: Text(widget.account.name),
              actions: [
                Showcase(
                    key: logID,
                    description: "Open the App Log",
                    child: IconButton(
                        tooltip: "View Log",
                        onPressed: () => onOpenLog(context),
                        icon: Icon(Icons.description))),
                Showcase(
                    key: sync1ID,
                    description: "Synchronize only this account",
                    child: IconButton(
                        tooltip: "Sync this account",
                        onPressed: onSync,
                        icon: Icon(Icons.sync))),
                Showcase(
                    key: rewindID,
                    description: "Rewind back a few blocks",
                    child: IconButton(
                        tooltip: "Rewind to previous checkpoint",
                        onPressed: onRewind,
                        icon: Icon(Icons.fast_rewind))),
                Showcase(
                    key: receiveID,
                    description: "Show the account receiving addresses",
                    child: IconButton(
                        tooltip: "Receive Funds",
                        onPressed: onReceive,
                        icon: Icon(Icons.download))),
                Showcase(
                    key: sendID,
                    description: "Send funds to one or many addresses",
                    child: IconButton(
                        tooltip: "Send Funds",
                        onPressed: onSend,
                        icon: Icon(Icons.send))),
              ],
              bottom: TabBar(
                tabs: [
                  Tab(text: 'Transactions'),
                  Tab(text: 'Memos'),
                  Tab(text: 'Notes'),
                ],
              ),
            ),
            body: Padding(
              padding: const EdgeInsets.symmetric(horizontal: 16),
              child: Observer(builder: (context) {
                // make sure there is a dependency on transactions
                AppStoreBase.instance.transactions.length;
                AppStoreBase.instance.memos.length;
                AppStoreBase.instance.notes.length;

                return TabBarView(children: [
                  SingleChildScrollView(
                      child: Column(children: [
                    Text("Height"),
                    Gap(8),
                    Observer(
                        builder: (context) => Text(
                            AppStoreBase.instance.heights[widget.account.id]
                                .toString(),
                            style: t.bodyLarge)),
                    Gap(16),
                    Text("Balance"),
                    Gap(8),
                    if (b != null)
                      Row(children: [
                        Showcase(
                            key: tBalID,
                            description: "Balance in the Transparent Pool",
                            child: SelectableText(
                                "T: ${zatToString(b.field0[0])}")),
                        const Gap(8),
                        Showcase(
                            key: sBalID,
                            description: "Balance in the Sapling Pool",
                            child: SelectableText(
                                "S: ${zatToString(b.field0[1])}")),
                        const Gap(8),
                        Showcase(
                            key: oBalID,
                            description: "Balance in the Orchard Pool",
                            child: SelectableText(
                                "O: ${zatToString(b.field0[2])}")),
                      ]),
                    Gap(8),
                    if (b != null)
                      Showcase(
                          key: balID,
                          description: "Balance across all pools",
                          child: SelectableText(
                              "\u2211: ${zatToString(b.field0[0] + b.field0[1] + b.field0[2])}")),
                    Gap(16),
                    ...showTxHistory(AppStoreBase.instance.transactions),
                  ])),
                  SingleChildScrollView(
                      child: Column(
                    children: [
                      ...showMemos(AppStoreBase.instance.memos),
                    ],
                  )),
                  SingleChildScrollView(
                      child: Column(children: [
                    ...showNotes(AppStoreBase.instance.notes),
                  ])),
                ]);
              }),
            )));
  }

  void onSync() async {
    try {
      await AppStoreBase.instance.startSynchronize([widget.account.id],
          onComplete: () {
        Future(refresh);
      });
    } on AnyhowException catch (e) {
      if (mounted) await showException(context, e.message);
    }
  }

  void onRewind() async {
    final confirmed = await confirmDialog(
      context,
      title: "Rewind",
      message:
          "Are you sure you want to rewind this account? This will rollback the account to a previous height. You will not lose any funds, but you will need to resync the account",
    );
    if (!confirmed) return;
    final dbHeight = await getDbHeight();
    await rewindSync(height: dbHeight - 60);
    final h = await getDbHeight();
    AppStoreBase.instance.heights[widget.account.id] = h;
  }

  void onReceive() async {
    await GoRouter.of(context).push("/receive");
  }

  void onSend() async {
    await GoRouter.of(context).push("/send");
  }

  void refresh() async {
    final b = await balance();
    await AppStoreBase.instance.loadAccounts();
    await AppStoreBase.instance.loadTxHistory();
    await AppStoreBase.instance.loadMemos();
    await AppStoreBase.instance.loadNotes();
    if (!mounted) return;
    setState(() {
      poolBalance = b;
    });
  }
}

final nameID2 = GlobalKey();
final iconID2 = GlobalKey();
final birthID2 = GlobalKey();
final enableID = GlobalKey();
final hideID2 = GlobalKey();
final viewID = GlobalKey();
final exportID = GlobalKey();
final resetID = GlobalKey();

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

  void tutorial() async {
    tutorialHelper(context, "tutEdit0", [
      nameID2,
      iconID2,
      birthID2,
      enableID,
      hideID2,
      viewID,
      exportID,
      resetID
    ]);
  }

  @override
  Widget build(BuildContext context) {
    Future(tutorial);

    final account = accounts.length == 1 ? accounts.first : null;

    return Scaffold(
        appBar: AppBar(title: Text('Account Edit'), actions: [
          if (account != null)
            Showcase(
                key: viewID,
                description: "Show Viewing Keys",
                child: IconButton(
                    tooltip: "Show Viewing Keys",
                    onPressed: () => GoRouter.of(context)
                        .push("/viewing_keys", extra: account.id),
                    icon: Icon(Icons.visibility))),
          if (account != null)
            Showcase(
                key: exportID,
                description: "Export an encrypted file of this account",
                child: IconButton(
                    tooltip: "Export Account",
                    onPressed: onExport,
                    icon: Icon(Icons.reset_tv))),
          Showcase(
              key: resetID,
              description: "Clear and reset account to birth height",
              child: IconButton(
                  tooltip: "Clear Sync Data",
                  onPressed: onReset,
                  icon: Icon(Icons.delete_sweep)))
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
                          child: Showcase(
                              key: nameID2,
                              description: "Edit Name of the account",
                              child: FormBuilderTextField(
                                name: 'name',
                                decoration: InputDecoration(labelText: 'Name'),
                                initialValue: account?.name ?? "(Multiple)",
                                readOnly: account == null,
                                onChanged:
                                    (account != null) ? onEditName : null,
                              ))),
                      if (account != null)
                        Showcase(
                            key: iconID2,
                            description: "Edit Account Icon",
                            child: account.avatar(onTap: (_) => onEditIcon()))
                    ],
                  ),
                  Showcase(
                      key: birthID2,
                      description: "Edit Height at the creation of the account",
                      child: FormBuilderTextField(
                        name: 'birth',
                        decoration: InputDecoration(labelText: 'Birth Height'),
                        initialValue: account?.birth.toString() ?? "",
                        keyboardType: TextInputType.number,
                        readOnly: account == null,
                        inputFormatters: [
                          FilteringTextInputFormatter.digitsOnly
                        ],
                        onChanged: (account != null) ? onEditBirth : null,
                      )),
                  Showcase(
                      key: enableID,
                      description:
                          "Enable or disable. Only enabled accounts participate in the global sync",
                      child: FormBuilderCheckbox(
                        name: "enabled",
                        title: Text("Enabled"),
                        initialValue: accounts
                                .every((a) => a.enabled == accounts[0].enabled)
                            ? accounts[0].enabled
                            : null,
                        tristate: account == null,
                        onChanged: onEditEnabled,
                      )),
                  Showcase(
                      key: hideID2,
                      description: "Hide this account from the account list",
                      child: FormBuilderCheckbox(
                        name: "hidden",
                        title: Text("Hidden"),
                        initialValue: accounts
                                .every((a) => a.hidden == accounts[0].hidden)
                            ? accounts[0].hidden
                            : null,
                        tristate: account == null,
                        onChanged: onEditHidden,
                      ))
                ],
              )),
        ));
  }

  void onEditName(String? name) async {
    if (name != null) {
      accounts[0] = accounts[0].copyWith(name: name);
      await updateAccount(
          update: AccountUpdate(
              coin: accounts[0].coin, id: accounts[0].id, name: name));
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
          update: AccountUpdate(
              coin: accounts[0].coin, id: accounts[0].id, icon: bytes));
      await AppStoreBase.instance.loadAccounts();
      setState(() {});
    }
  }

  void onEditBirth(String? birth) async {
    if (birth != null && birth.isNotEmpty) {
      accounts[0] = accounts[0].copyWith(birth: int.parse(birth));
      await updateAccount(
          update: AccountUpdate(
              coin: accounts[0].coin,
              id: accounts[0].id,
              birth: int.parse(birth)));
      await AppStoreBase.instance.loadAccounts();
      setState(() {});
    }
  }

  void onEditEnabled(bool? v) async {
    if (v == null) return;
    for (var i = 0; i < accounts.length; i++) {
      accounts[i] = accounts[i].copyWith(enabled: v);
      await updateAccount(
          update: AccountUpdate(
              coin: accounts[i].coin, id: accounts[i].id, enabled: v));
    }
    await AppStoreBase.instance.loadAccounts();
    setState(() {});
  }

  void onEditHidden(bool? v) async {
    if (v == null) return;
    for (var i = 0; i < accounts.length; i++) {
      accounts[i] = accounts[i].copyWith(hidden: v);
      await updateAccount(
          update: AccountUpdate(
              coin: accounts[i].coin, id: accounts[i].id, hidden: v));
    }
    await AppStoreBase.instance.loadAccounts();
    setState(() {});
  }

  void onExport() async {
    final account = accounts.first;
    final password = await inputPassword(context,
        title: "Export Account", message: "File Password");
    if (password != null) {
      final res = await exportAccount(id: account.id, passphrase: password);
      await FilePicker.platform.saveFile(
        dialogTitle: 'Please select an output file for the encrypted account:',
        fileName: '${account.name}.bin',
        bytes: res,
      );
    }
  }

  void onReset() async {
    for (var account in accounts) await resetSync(id: account.id);
    await AppStoreBase.instance.loadAccounts();
  }
}

extension AccountExtension on Account {
  Widget avatar({bool? selected, void Function(bool?)? onTap}) {
    final t = Theme.of(navigatorKey.currentContext!).colorScheme;
    final i = initials(name);
    final s = selected ?? false;
    return GestureDetector(
        onTap: () => onTap?.call(!s),
        child: CircleAvatar(
          backgroundColor: s ? Colors.blue.shade700 : t.primaryContainer,
          child: s
              ? Icon(Icons.check, color: Colors.white)
              : icon != null
                  ? ClipOval(child: Image.memory(icon!))
                  : Text(i, style: TextStyle(color: t.onPrimaryContainer)),
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

        final tile = ListTile(
          onTap: () => gotoTransaction(context, tx.id),
          leading: Text("${tx.height}"),
          title: SelectableText(txId),
          subtitle: Text(timeToString(tx.time)),
          trailing: Text(zatToString(BigInt.from(tx.value))),
        );

        return (index == 0)
            ? Showcase(
                key: txdID,
                description: "Tap on a transaction or memo to view details",
                child: tile,
              )
            : tile;
      },
    ),
  ];
}

void gotoTransaction(BuildContext context, int idTx) async {
  await GoRouter.of(context).push("/tx_view", extra: idTx);
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
          onTap: () => gotoTransaction(context, memo.idTx),
          leading: Text("${memo.height}"),
          title: SelectableText(memo.memo ?? hex.encode(memo.memoBytes)),
          subtitle: Text(timeToString(memo.time)),
          trailing: Text(memo.idNote != null ? "In" : "Out"),
        );
      },
    ),
  ];
}

List<Widget> showNotes(List<TxNote> notes) {
  final t = Theme.of(navigatorKey.currentContext!);
  return [
    ListView.builder(
      shrinkWrap: true,
      physics: const NeverScrollableScrollPhysics(),
      itemCount: notes.length,
      itemBuilder: (context, index) {
        final note = notes[index];

        return ListTile(
          key: ValueKey(note.id),
          onTap: () => toggleLock(context, note.id, !note.locked),
          leading: Text("${note.height}"),
          title: Text(poolToString(note.pool)),
          trailing: Text(zatToString(note.value)),
          textColor: note.locked ? t.disabledColor : null,
        );
      },
    ),
  ];
}

void toggleLock(BuildContext context, int id, bool locked) async {
  await lockNote(id: id, locked: locked);
  await AppStoreBase.instance.loadNotes();
}

void onOpenLog(BuildContext context) async {
  await GoRouter.of(context).push("/log");
}

class ViewingKeysPage extends StatefulWidget {
  final int account;
  const ViewingKeysPage(this.account, {super.key});

  @override
  State<ViewingKeysPage> createState() => ViewingKeysPageState();
}

class ViewingKeysPageState extends State<ViewingKeysPage> {
  int pools = 7;
  String? uvk;
  String? fingerprint;

  @override
  void initState() {
    super.initState();
    Future(() async {
      fingerprint = await getAccountFingerprint(account: widget.account);
      setState(() {});
    });
    Future(() => onPoolChanged(pools));
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
        appBar: AppBar(title: Text('Viewing Keys')),
        body: SingleChildScrollView(
            child: Padding(
                padding: EdgeInsets.symmetric(horizontal: 16),
                child: Column(children: [
                  Center(child: PoolSelect(onChanged: onPoolChanged)),
                  Gap(32),
                  if (uvk != null) SelectableText(uvk!),
                  Gap(32),
                  if (uvk != null) QrImageView(data: uvk!, size: 200),
                  Gap(8),
                  if (fingerprint != null) SelectableText(fingerprint!),
                  Gap(32),
                  Text("If the account does not include a pool, its receiver will be absent"),
                ]))));
  }

  onPoolChanged(int? v) async {
    if (v == null) return;
    try {
      final uuvk = await getAccountUfvk(account: widget.account, pools: v);
      setState(() {
        pools = v;
        uvk = uuvk;
      });
    } on AnyhowException catch (e) {
      if (!mounted) return;
      await showException(context, e.message);
      setState(() {
        uvk = null;
      });
    }
  }
}
