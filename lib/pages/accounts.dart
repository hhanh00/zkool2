import 'dart:async';
import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter_mobx/flutter_mobx.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated_io.dart';
import 'package:gap/gap.dart';
import 'package:go_router/go_router.dart';
import 'package:path_provider/path_provider.dart';
import 'package:showcaseview/showcaseview.dart';
import 'package:zkool/main.dart';
import 'package:zkool/pages/account.dart';
import 'package:zkool/router.dart';
import 'package:zkool/src/rust/api/account.dart';
import 'package:zkool/src/rust/api/db.dart';
import 'package:zkool/src/rust/api/network.dart';
import 'package:zkool/store.dart';
import 'package:zkool/utils.dart';
import 'package:zkool/widgets/editable_list.dart';

final heightID = GlobalKey();
final mempoolID = GlobalKey();
final settingsID = GlobalKey();
final syncID = GlobalKey();
final hideID = GlobalKey();

final accountListID = GlobalKey();
final avatarID = GlobalKey();

class AccountListPage extends StatelessWidget {
  const AccountListPage({super.key});

  Future<List<Account>> loadAccounts() async {
    if (!AppStoreBase.instance.loaded) {
      final dbName = AppStoreBase.instance.dbName;
      final dbDir = await getApplicationDocumentsDirectory();
      final dbFilepath = '${dbDir.path}/$dbName.db';
      logger.i('dbFilepath: $dbFilepath');
      AppStoreBase.instance.dbFilepath = dbFilepath;

      String? password;
      if (!File(dbFilepath).existsSync()) {
        if (dbName != appName) {
          // do not encrypt default database
          password = await inputPassword(navigatorKey.currentContext!,
              title: "Enter New Database Password",
              message: "Password CANNOT be changed later");
        }
        if (password != null && password.isEmpty) password = null;
      }

      while (true) {
        try {
          await openDatabase(dbFilepath: dbFilepath, password: password);
          break;
        } catch (_) {
          password = await inputPassword(
            navigatorKey.currentContext!,
            title: "Enter Database Password for $dbName",
          );
          if (password == null) {
            // switch to default database
            await showException(navigatorKey.currentContext!,
                "No password given. Switching to defaut database.");
            AppStoreBase.instance.dbName = appName;
            return await loadAccounts();
          }
        }
      }
      await AppStoreBase.instance.loadSettings();
      setLwd(lwd: AppStoreBase.instance.lwd);
      AppStoreBase.instance.autoSync();
      runMempoolListener();
    }

    AppStoreBase.instance.loaded = true;
    final accounts = await AppStoreBase.instance.loadAccounts();
    return accounts;
  }

  @override
  Widget build(BuildContext context) {
    return FutureBuilder<List<Account>>(
        future: loadAccounts(),
        initialData: [],
        builder: (context, snapshot) {
          final data = snapshot.data;
          if (data != null) return AccountListPage2(snapshot.data!);
          return SizedBox.shrink();
        });
  }
}

class AccountListPage2 extends StatefulWidget {
  final List<Account> accounts;
  const AccountListPage2(this.accounts, {super.key});

  @override
  State<AccountListPage2> createState() => AccountListPage2State();
}

class AccountListPage2State extends State<AccountListPage2> {
  var includeHidden = false;
  final listKey = GlobalKey<EditableListState<Account>>();
  double? price;

  @override
  void initState() {
    super.initState();
    if (!AppStoreBase.instance.disclaimerAccepted) {
      WidgetsBinding.instance.addPostFrameCallback((_) {
        logger.i("Disclaimer not accepted");
        GoRouter.of(context).push("/disclaimer");
      });
    }
  }

  @override
  void didUpdateWidget(covariant AccountListPage2 oldWidget) {
    super.didUpdateWidget(oldWidget);
    for (var account in widget.accounts) {
      AppStoreBase.instance.heights[account.id] = account.height;
    }
  }

  void refreshHeight() async {
    try {
      final height = await getCurrentHeight();
      AppStoreBase.instance.currentHeight = height;
      final p = await getCoingeckoPrice();
      setState(() => price = p);
    } on AnyhowException catch (e) {
      if (mounted) await showException(context, e.message);
    }
  }

  List<Account> get accounts => AppStoreBase.instance.accounts
      .where((a) => !a.internal && (includeHidden || !a.hidden))
      .toList();

  void tutorial() async {
    if (!AppStoreBase.instance.disclaimerAccepted) return;
    tutorialHelper(context, "tutMain0",
        [newAccountId, settingsID, syncID, hideID, heightID, mempoolID]);
    if (accounts.isNotEmpty)
      tutorialHelper(context, "tutMain1", [accountListID, avatarID]);
  }

  @override
  Widget build(BuildContext context) {
    Future(tutorial);

    return Observer(builder: (context) {
      final tt = Theme.of(context).textTheme;
      final t = tt.bodyMedium!.copyWith(fontFamily: "monospace");
      AppStoreBase.instance.accounts;

      return Showcase(
          key: accountListID,
          description:
              "List of Accounts. Tap on a row to select. Long tap then drag and drop to reorder",
          child: EditableList<Account>(
              key: listKey,
              items: accounts,
              headerBuilder: (context) => [
                    Showcase(
                        key: heightID,
                        description:
                            "Current Block Height. Refreshed automatically every 15 seconds. Tap to update manually",
                        child: Observer(
                            builder: (context) => ElevatedButton(
                                onPressed: () => Future(refreshHeight),
                                child: Text(
                                    "Height: ${AppStoreBase.instance.currentHeight}")))),
                    const Gap(8),
                    Showcase(key: mempoolID, description: "Show the transactions in the mempool", child:
                      ElevatedButton(onPressed: onMempool, child: Text("Mempool"))),
                    const Gap(8),
                    if (price != null) Text("Price: $price USD"),
                    const Gap(8),
                  ],
              builder: (context, index, account, {selected, onSelectChanged}) {
                final avatar = account.avatar(
                    selected: selected ?? false, onTap: onSelectChanged);
                return Material(
                    key: ValueKey(account.id),
                    child: GestureDetector(
                      child: ListTile(
                        leading: index == 0
                            ? Showcase(
                                key: avatarID,
                                description: "Tap to select for edit/delete",
                                child: avatar)
                            : avatar,
                        title: Text(account.name,
                            style: !account.enabled
                                ? TextStyle(color: Colors.grey)
                                : null),
                        subtitle: Text(zatToString(account.balance),
                            style: t.copyWith(fontWeight: FontWeight.w700)),
                        trailing: Observer(
                            builder: (context) => Text(
                                  AppStoreBase.instance.heights[account.id]
                                      .toString(),
                                  textAlign: TextAlign.end,
                                )),
                      ),
                      onTap: () => onOpen(context, account),
                    ));
              },
              title: "Account List",
              onCreate: () => widget.accounts,
              createBuilder: (context) =>
                  GoRouter.of(context).push("/account/new"),
              editBuilder: (context, a) =>
                  GoRouter.of(context).push("/account/edit", extra: a),
              deleteBuilder: (context, accounts) async {
                final confirmed = await confirmDialog(context,
                    title: "Delete Account(s)",
                    message: "Are you sure you want to delete these accounts?");
                if (confirmed) {
                  for (var a in accounts) {
                    await deleteAccount(account: a.id);
                  }
                  await AppStoreBase.instance.loadAccounts();
                }
              },
              isEqual: (a, b) => a.id == b.id,
              onReorder: onReorder,
              buttons: [
                Showcase(
                    key: settingsID,
                    description: "Open Settings",
                    child: IconButton(
                        onPressed: onSettings, icon: Icon(Icons.settings))),
                Showcase(
                    key: syncID,
                    description:
                        "Synchronize all enabled accounts or the accounts currently selected",
                    child:
                        IconButton(onPressed: onSync, icon: Icon(Icons.sync))),
                Showcase(
                    key: hideID,
                    description: "Show/Hide hidden accounts",
                    child: IconButton(
                        onPressed: onHide,
                        icon: Icon(includeHidden
                            ? Icons.visibility
                            : Icons.visibility_off))),
              ]));
    });
  }

  onMempool() => GoRouter.of(context).push('/mempool');

  onHide() async {
    final authenticated =
        await authenticate(reason: "Show/Hide Hidden Accounts");
    if (!authenticated) return;
    setState(() {
      includeHidden = !includeHidden;
    });
  }

  onSync() async {
    try {
      final listState = listKey.currentState!;
      List<int> accountIds = [];
      final hasSelection = listState.selected.any((s) => s);
      if (hasSelection) {
        // if any selection, use the selection, otherwise use the enabled flag
        for (var i = 0; i < listState.selected.length; i++) {
          if (listState.selected[i]) accountIds.add(accounts[i].id);
        }
      } else {
        // no selection, use the enabled flag
        for (var a in AppStoreBase.instance.accounts) {
          if (a.enabled) accountIds.add(a.id);
        }
      }
      await AppStoreBase.instance.startSynchronize(
          accountIds, int.parse(AppStoreBase.instance.actionsPerSync));
    } on AnyhowException catch (e) {
      if (mounted) await showException(context, e.message);
    }
  }

  onOpen(BuildContext context, Account account) {
    setAccount(account: account.id);
    AppStoreBase.instance.selectedAccount = account;
    GoRouter.of(context).push('/account', extra: account);
  }

  onReorder(int oldIndex, int newIndex) async {
    await reorderAccount(
        oldPosition: accounts[oldIndex].position,
        newPosition: accounts[newIndex].position);
    await AppStoreBase.instance.loadAccounts();
  }

  onSettings() {
    GoRouter.of(context).push('/settings');
  }
}
