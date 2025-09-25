import 'dart:async';
import 'dart:convert';
import 'dart:io';

import 'package:bubble/bubble.dart';
import 'package:convert/convert.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_form_builder/flutter_form_builder.dart';
import 'package:flutter_mobx/flutter_mobx.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated.dart';
import 'package:gap/gap.dart';
import 'package:go_router/go_router.dart';
import 'package:qr_flutter/qr_flutter.dart';
import 'package:searchable_listview/searchable_listview.dart';
import 'package:showcaseview/showcaseview.dart';
import 'package:zkool/main.dart';
import 'package:zkool/pages/tx.dart';
import 'package:zkool/router.dart';
import 'package:zkool/src/rust/account.dart';
import 'package:zkool/src/rust/api/account.dart';
import 'package:zkool/src/rust/api/sync.dart';
import 'package:zkool/src/rust/api/transaction.dart';
import 'package:zkool/store.dart';
import 'package:zkool/utils.dart';
import 'package:zkool/widgets/pool_select.dart';

class AccountViewPage extends StatefulWidget {
  const AccountViewPage({super.key});

  @override
  State<AccountViewPage> createState() => AccountViewPageState();
}

class AccountViewPageState extends State<AccountViewPage> {
  final logID = GlobalKey(debugLabel: "logID");
  final sync1ID = GlobalKey(debugLabel: "sync1ID");
  final receiveID = GlobalKey(debugLabel: "receiveID");
  final sendID = GlobalKey(debugLabel: "sendID");
  final balID = GlobalKey(debugLabel: "balID");

  final List<String> tabNames = ["Transactions", "Memos", "Notes"];

  StreamSubscription<SyncProgress>? progressSubscription;

  @override
  void initState() {
    super.initState();
    Future(appStore.refresh);
  }

  void tutorial() async {
    tutorialHelper(context, "tutAccount0", [balID, logID, sync1ID, receiveID, sendID]);
  }

  @override
  Widget build(BuildContext context) {
    final t = Theme.of(context).textTheme;

    Future(tutorial);

    assert(account != null);

    return DefaultTabController(
      length: 3,
      child: Scaffold(
        appBar: AppBar(
          title: Text(account!.name),
          actions: [
            Showcase(
              key: sync1ID,
              description: "Synchronize only this account",
              child: IconButton(tooltip: "Sync this account", onPressed: onSync, icon: Icon(Icons.sync)),
            ),
            Showcase(
              key: receiveID,
              description: "Show the account receiving addresses",
              child: IconButton(tooltip: "Receive Funds", onPressed: onReceive, icon: Icon(Icons.download)),
            ),
            Showcase(
              key: sendID,
              description: "Send funds to one or many addresses",
              child: IconButton(tooltip: "Send Funds", onPressed: onSend, icon: Icon(Icons.send)),
            ),
            PopupMenuButton<String>(
              onSelected: (String result) {
                switch (result) {
                  case "update_fx":
                    onUpdateAllTxPrices();
                  case "charts":
                    GoRouter.of(context).push("/chart");
                  default:
                    onExport(int.parse(result));
                }
              },
              itemBuilder: (BuildContext context) => <PopupMenuEntry<String>>[
                const PopupMenuItem<String>(
                  value: "update_fx",
                  child: Text("Fetch Tx Prices"),
                ),
                PopupMenuItem<String>(
                  value: tabIndex(context).toString(),
                  child: Text("Export ${tabNames[tabIndex(context)]}"),
                ),
                if (!Platform.isLinux)
                  const PopupMenuItem<String>(
                    value: "charts",
                    child: Text("Charts"),
                  ),
              ],
            ),
          ],
          bottom: TabBar(
            tabs: tabNames.map((n) => Tab(text: n)).toList(),
          ),
        ),
        body: Padding(
          padding: const EdgeInsets.symmetric(horizontal: 16),
          child: Observer(
            builder: (context) {
              appStore.seqno;
              final b = appStore.poolBalance;

              return TabBarView(
                children: [
                  CustomScrollView(
                    slivers: [
                      PinnedHeaderSliver(
                        child: Container(
                          color: Theme.of(context).colorScheme.surface,
                          child: Column(
                            children: [
                              Text("Height"),
                              Gap(8),
                              Observer(
                                builder: (context) {
                                  final height = appStore.heights[account!.id]!;
                                  return height.buildHero(context);
                                },
                              ),
                              Gap(16),
                              Text("Balance"),
                              Gap(8),
                              BalanceWidget(b, showcase: true),
                              Gap(8),
                              Observer(
                                builder: (context) {
                                  final unconfirmedAmount = appStore.mempoolAccounts[account!.id];
                                  return unconfirmedAmount != null
                                      ? zatToText(
                                          BigInt.from(unconfirmedAmount),
                                          prefix: "Unconfirmed: ",
                                          colored: true,
                                          selectable: true,
                                          style: t.bodyLarge,
                                        )
                                      : SizedBox.shrink();
                                },
                              ),
                              Gap(8),
                              Showcase(
                                key: balID,
                                description: "Balance across all pools",
                                child: zatToText(b.field0[0] + b.field0[1] + b.field0[2], selectable: true, style: t.titleLarge!),
                              ),
                              Gap(8),
                            ],
                          ),
                        ),
                      ),
                      ...showTxHistory(appStore.transactions),
                    ],
                  ),
                  showMemos(context, appStore.memos),
                  showNotes(appStore.notes),
                ],
              );
            },
          ),
        ),
      ),
    );
  }

  void onSync() async {
    try {
      await appStore.startSynchronize([account!.id], int.parse(appStore.actionsPerSync));
    } on AnyhowException catch (e) {
      if (mounted) await showException(context, e.message);
    }
  }

  void onReceive() async {
    await GoRouter.of(context).push("/receive");
  }

  void onSend() async {
    await GoRouter.of(context).push("/send");
  }

  void onUpdateAllTxPrices() async {
    final confirmed =
        await confirmDialog(context, title: "Fetch Tx Market Price", message: "Do you want to retrieve historical ZEC prices for your past transactions?");
    if (confirmed) await fillMissingTxPrices();
  }

  void onExport(int index) async {
    final data = await getExportedData(type: index);
    final filename = await appWatcher.saveFile(data: utf8.encode(data));
    if (!mounted) return;
    if (filename != null) await showMessage(context, "$filename Saved");
  }

  Account? get account => appStore.selectedAccount;
  // the context must be from inside the DefaultTabController, which
  // means it cannot be this widget's context
  int tabIndex(BuildContext context) => DefaultTabController.of(context).index;
}

class AccountEditPage extends StatefulWidget {
  final List<Account> accounts;
  const AccountEditPage(this.accounts, {super.key});

  @override
  State<AccountEditPage> createState() => AccountEditPageState();
}

class AccountEditPageState extends State<AccountEditPage> {
  final nameID2 = GlobalKey(debugLabel: "nameID2");
  final iconID2 = GlobalKey(debugLabel: "iconID2");
  final birthID2 = GlobalKey(debugLabel: "birthID2");
  final enableID = GlobalKey(debugLabel: "enableID");
  final hideID2 = GlobalKey(debugLabel: "hideID2");
  final viewID = GlobalKey(debugLabel: "viewID");
  final exportID = GlobalKey(debugLabel: "exportID");
  final rewindID = GlobalKey(debugLabel: "rewindID");
  final resetID = GlobalKey(debugLabel: "resetID");
  final folderID = GlobalKey(debugLabel: "folderID");

  late List<Account> accounts = widget.accounts;
  final formKey = GlobalKey<FormBuilderState>(debugLabel: "formKey");

  @override
  void didUpdateWidget(covariant AccountEditPage oldWidget) {
    accounts = accounts;
    super.didUpdateWidget(oldWidget);
  }

  void tutorial() async {
    tutorialHelper(context, "tutEdit0", [nameID2, iconID2, birthID2, enableID, hideID2, folderID, viewID, exportID, rewindID, resetID]);
  }

  @override
  Widget build(BuildContext context) {
    Future(tutorial);

    final account = accounts.length == 1 ? accounts.first : null;
    final folder = accounts.first.folder;
    final folderOptions = [DropdownMenuItem(value: 0, child: Text("No Folder"))] +
        appStore.folders
            .map(
              (f) => DropdownMenuItem(value: f.id, child: Text(f.name)),
            )
            .toList();

    return Scaffold(
      appBar: AppBar(
        title: Text('Account Edit'),
        actions: [
          if (account != null)
            Showcase(
              key: viewID,
              description: "Show Viewing Keys",
              child: IconButton(
                tooltip: "Show Viewing Keys",
                onPressed: () => GoRouter.of(context).push("/viewing_keys", extra: account.id),
                icon: Icon(Icons.visibility),
              ),
            ),
          if (account != null) ...[
            Showcase(
              key: exportID,
              description: "Export an encrypted file of this account",
              child: IconButton(tooltip: "Export Account", onPressed: onExport, icon: Icon(Icons.save)),
            ),
            Showcase(
              key: rewindID,
              description: "Rewind back a few blocks",
              child: IconButton(tooltip: "Rewind to previous checkpoint", onPressed: onRewind, icon: Icon(Icons.fast_rewind)),
            ),
          ],
          Showcase(
            key: resetID,
            description: "Clear and reset account to birth height",
            child: IconButton(tooltip: "Clear Sync Data", onPressed: onReset, icon: Icon(Icons.delete_sweep)),
          ),
        ],
      ),
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
                        onChanged: (account != null) ? onEditName : null,
                      ),
                    ),
                  ),
                  if (account != null) Showcase(key: iconID2, description: "Edit Account Icon", child: account.avatar(onTap: (_) => onEditIcon())),
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
                  inputFormatters: [FilteringTextInputFormatter.digitsOnly],
                  onChanged: (account != null) ? onEditBirth : null,
                ),
              ),
              Showcase(
                key: enableID,
                description: "Enable or disable. Only enabled accounts participate in the global sync",
                child: FormBuilderCheckbox(
                  name: "enabled",
                  title: Text("Enabled"),
                  initialValue: accounts.every((a) => a.enabled == accounts[0].enabled) ? accounts[0].enabled : null,
                  tristate: account == null,
                  onChanged: onEditEnabled,
                ),
              ),
              Showcase(
                key: hideID2,
                description: "Hide this account from the account list",
                child: FormBuilderCheckbox(
                  name: "hidden",
                  title: Text("Hidden"),
                  initialValue: accounts.every((a) => a.hidden == accounts[0].hidden) ? accounts[0].hidden : null,
                  tristate: account == null,
                  onChanged: onEditHidden,
                ),
              ),
              Showcase(
                key: folderID,
                description: "Assign Account to Folder",
                child: FormBuilderDropdown<int>(
                  name: "folder",
                  initialValue: accounts.every((a) => a.folder.id == folder.id) ? folder.id : null,
                  items: folderOptions,
                  onChanged: onEditFolder,
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }

  void onEditName(String? name) async {
    if (name != null) {
      accounts[0] = accounts[0].copyWith(name: name);
      await updateAccount(
        update: AccountUpdate(
          coin: accounts[0].coin,
          id: accounts[0].id,
          name: name,
          folder: accounts[0].folder.id,
        ),
      );
      await appStore.loadAccounts();
      setState(() {});
    }
  }

  void onEditIcon() async {
    final icon = await appWatcher.pickImage();
    var changed = false;
    Uint8List? bytes;
    if (icon != null) {
      bytes = await icon.readAsBytes();
      changed = true;
    } else {
      final remove = await confirmDialog(context, title: "Reset Icon", message: "Do you want to remove the current icon?");
      if (remove) {
        bytes = Uint8List(0);
        changed = true;
      }
    }
    if (changed) {
      accounts[0] = accounts[0].copyWith(icon: bytes?.isNotEmpty == true ? bytes : null);
      await updateAccount(
        update: AccountUpdate(
          coin: accounts[0].coin,
          id: accounts[0].id,
          icon: bytes,
          folder: accounts[0].folder.id,
        ),
      );
      await appStore.loadAccounts();
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
          birth: int.parse(birth),
          folder: accounts[0].folder.id,
        ),
      );
      await appStore.loadAccounts();
      setState(() {});
    }
  }

  void onEditEnabled(bool? v) async {
    if (v == null) return;
    for (var i = 0; i < accounts.length; i++) {
      accounts[i] = accounts[i].copyWith(enabled: v);
      await updateAccount(
        update: AccountUpdate(
          coin: accounts[i].coin,
          id: accounts[i].id,
          enabled: v,
          folder: accounts[i].folder.id,
        ),
      );
    }
    await appStore.loadAccounts();
    setState(() {});
  }

  void onEditHidden(bool? v) async {
    if (v == null) return;
    for (var i = 0; i < accounts.length; i++) {
      accounts[i] = accounts[i].copyWith(hidden: v);
      await updateAccount(
        update: AccountUpdate(
          coin: accounts[i].coin,
          id: accounts[i].id,
          hidden: v,
          folder: accounts[i].folder.id,
        ),
      );
    }
    await appStore.loadAccounts();
    setState(() {});
  }

  void onEditFolder(int? v) async {
    if (v == null) return;
    for (var i = 0; i < accounts.length; i++) {
      accounts[i] = accounts[i].copyWith(folder: appStore.folders.firstWhere((f) => f.id == v, orElse: () => Folder(id: 0, name: "")));
      await updateAccount(
        update: AccountUpdate(
          coin: accounts[i].coin,
          id: accounts[i].id,
          folder: v,
        ),
      );
    }
    await appStore.loadAccounts();
    setState(() {});
  }

  void onExport() async {
    final account = accounts.first;
    final password = await inputPassword(context, title: "Export Account", message: "File Password");
    if (password != null) {
      final res = await exportAccount(id: account.id, passphrase: password);
      await appWatcher.saveFile(title: "Please select an output file for the encrypted account:", fileName: "${account.name}.bin", data: res);
    }
  }

  void onRewind() async {
    final account = accounts.first;
    final confirmed = await confirmDialog(
      context,
      title: "Rewind",
      message:
          "Are you sure you want to rewind this account? This will rollback the account to a previous height. You will not lose any funds, but you will need to resync the account",
    );
    if (!confirmed) return;
    final dbHeight = await getDbHeight();
    await rewindSync(height: dbHeight.height - 60, account: account.id);
    final h = await getDbHeight();
    appStore.heights[account.id]?.set(h.height, h.time);
  }

  void onReset() async {
    final confirmed = await confirmDialog(
      context,
      title: "Reset Account",
      message:
          "Are you sure you want to reset this account? This will clear all sync data and reset the account to the birth height. You will not lose any funds, but you will need to resync the account",
    );
    if (!confirmed) return;
    for (var account in accounts) await resetSync(id: account.id);
    await appStore.loadAccounts();
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
      ),
    );
  }
}

class BalanceWidget extends StatelessWidget {
  final PoolBalance balance;
  final bool showcase;
  final void Function(int)? onPoolSelected;
  const BalanceWidget(this.balance, {super.key, this.showcase = false, this.onPoolSelected});

  @override
  Widget build(BuildContext context) {
    return Row(
      mainAxisAlignment: MainAxisAlignment.center,
      children: [
        zatToText(
          balance.field0[0],
          prefix: "T: ",
          selectable: true,
          onTap: () => onPoolSelected?.call(0),
        ),
        const Gap(8),
        zatToText(
          balance.field0[1],
          prefix: "S: ",
          selectable: true,
          onTap: () => onPoolSelected?.call(1),
        ),
        const Gap(8),
        zatToText(
          balance.field0[2],
          prefix: "O: ",
          selectable: true,
          onTap: () => onPoolSelected?.call(2),
        ),
      ],
    );
  }
}

List<Widget> showTxHistory(List<Tx> transactions) {
  return [
    SliverToBoxAdapter(
      child: Column(
        children: [
          Text("Transaction History (${transactions.length} txs)"),
          const Gap(8),
        ],
      ),
    ),
    SliverFixedExtentList.builder(
      itemCount: transactions.length,
      itemBuilder: (context, index) {
        final tx = transactions[index];
        final tile = ListTile(
          onTap: () => gotoTransaction(context, tx.id),
          leading: Text("${tx.height}"),
          title: Text(getTransactionType(tx.tpe)),
          subtitle: Text(timeToString(tx.time)),
          trailing: zatToText(BigInt.from(tx.value), colored: true, selectable: false),
        );

        return tile;
      },
      itemExtent: 64,
    ),
  ];
}

String getTransactionType(int? tpe) {
  if (tpe == null) return "";
  switch (tpe) {
    case 0:
      return "\u2194 Self Tx";
    case 1:
      return "\u2795 Received";
    case 2:
      return "\u2796 Spent";
    case 4:
      return "\u{1F513} Unshielding";
    case 8:
      return "\u{1F6E1} Shielding";
    case 12:
      return "\u{1F310} \u2194 T. Self Tx";
    default:
      return "Unknown";
  }
}

void gotoTransaction(BuildContext context, int idTx) async {
  await GoRouter.of(context).push("/tx_view", extra: idTx);
}

Uint8List trimTrailingZeros(Uint8List bytes) {
  int end = bytes.length;
  while (end > 0 && bytes[end - 1] == 0x00) {
    end--;
  }
  return bytes.sublist(0, end);
}

Widget showMemos(BuildContext context, List<Memo> memos) {
  return SearchableList(
    initialList: memos,
    itemBuilder: (memo) => MemoWidget(memo),
    filter: (query) => memos.where((m) => query.isEmpty || (m.memo?.contains(query) == true)).toList(),
    inputDecoration: InputDecoration(
      labelText: "Search Memos",
      fillColor: Colors.white,
    ),
  );
}

Widget showNotes(List<TxNote> notes) {
  final t = Theme.of(navigatorKey.currentContext!);
  return ListView.builder(
    itemCount: notes.length + 1,
    itemBuilder: (context, index) {
      if (index == 0)
        return OverflowBar(
          children: [
            IconButton(onPressed: () => onLockRecent(context), tooltip: "Lock recently mined notes", icon: Icon(Icons.table_rows)),
            IconButton(onPressed: () => onUnlockAll(context), tooltip: "Unlock all notes", icon: Icon(Icons.select_all)),
          ],
        );

      final noteIndex = index - 1;
      final note = notes[noteIndex];
      return ListTile(
        key: ValueKey(note.id),
        onTap: () => toggleLock(context, note.id, !note.locked),
        leading: Text("${note.height}"),
        title: Text(poolToString(note.pool)),
        trailing: zatToText(note.value, selectable: false),
        textColor: note.locked ? t.disabledColor : null,
      );
    },
  );
}

void onLockRecent(BuildContext context) async {
  final s = await inputText(context, title: "Enter confirmation threshold");
  final threshold = s?.let((v) => int.tryParse(v));
  if (threshold != null) {
    await lockRecentNotes(
      height: appStore.currentHeight,
      threshold: threshold,
    );
    await appStore.loadNotes();
  }
}

void onUnlockAll(BuildContext context) async {
  final confirmed = await confirmDialog(context, title: "Unlock All", message: "Do you want to unlock every note?");
  if (confirmed) {
    await unlockAllNotes();
    await appStore.loadNotes();
  }
}

void toggleLock(BuildContext context, int id, bool locked) async {
  await lockNote(id: id, locked: locked);
  await appStore.loadNotes();
}

class MemoWidget extends StatelessWidget {
  final Memo memo;
  const MemoWidget(this.memo, {super.key});

  @override
  Widget build(BuildContext context) {
    final t = Theme.of(context);
    final cs = t.colorScheme;
    final incoming = memo.idNote != null;

    return GestureDetector(
      onTap: () => gotoTransaction(context, memo.idTx),
      child: Padding(
        padding: EdgeInsetsGeometry.symmetric(vertical: 4),
        child: Bubble(
          nip: incoming ? BubbleNip.leftTop : BubbleNip.rightTop,
          color: incoming ? cs.surface : cs.secondaryContainer,
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Align(alignment: Alignment.centerRight, child: Text(timeToString(memo.time), style: t.textTheme.labelMedium)),
              Gap(8),
              CopyableText(memo.memo ?? hex.encode(trimTrailingZeros(memo.memoBytes))),
            ],
          ),
        ),
      ),
    );
  }
}

class ViewingKeysPage extends StatefulWidget {
  // the viewing keys page is opened from the edit account page
  // and the account is passed as an argument
  // because the selected account may be different
  final int account;
  const ViewingKeysPage(this.account, {super.key});

  @override
  State<ViewingKeysPage> createState() => ViewingKeysPageState();
}

class ViewingKeysPageState extends State<ViewingKeysPage> {
  int pools = 7;
  String? uvk;
  String? fingerprint;
  Seed? seed;
  int accountPools = 7; // default to all pools
  bool showSeed = false;

  @override
  void initState() {
    super.initState();
    Future(() async {
      fingerprint = await getAccountFingerprint(account: widget.account);
      seed = await getAccountSeed(account: widget.account);
      accountPools = await getAccountPools(account: widget.account);
      setState(() {});
    });
    Future(() => onPoolChanged(pools));
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: Text('Viewing Keys'),
        actions: [if (seed != null) IconButton(tooltip: "Show Seed Phrase", onPressed: onShowSeed, icon: Icon(Icons.key))],
      ),
      body: SingleChildScrollView(
        child: Padding(
          padding: EdgeInsets.symmetric(horizontal: 16),
          child: Column(
            children: [
              if (showSeed && seed != null) ...[
                ListTile(title: Text("Mnemonic"), subtitle: CopyableText(seed!.mnemonic)),
                ListTile(title: Text("Passphrase"), subtitle: CopyableText(seed!.phrase)),
                ListTile(title: Text("Index"), subtitle: CopyableText(seed!.aindex.toString())),
                Divider(),
                Gap(8),
              ],
              Center(child: PoolSelect(enabled: accountPools, initialValue: accountPools, onChanged: onPoolChanged)),
              Gap(32),
              if (uvk != null) CopyableText(uvk!),
              Gap(32),
              if (uvk != null) QrImageView(data: uvk!, size: 200, backgroundColor: Colors.white),
              Gap(8),
              if (fingerprint != null) CopyableText(fingerprint!),
              Gap(16),
              Text("If the account does not include a pool, its receiver will be absent"),
            ],
          ),
        ),
      ),
    );
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

  void onShowSeed() async {
    final authenticated = await authenticate(reason: "Show Seed Phrase");
    if (!authenticated) return;
    setState(() {
      showSeed = true;
    });
  }
}
