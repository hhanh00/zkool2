import 'dart:async';
import 'dart:io';

import 'package:awesome_dialog/awesome_dialog.dart';
import 'package:flutter/material.dart';
import 'package:flutter_mobx/flutter_mobx.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated_io.dart';
import 'package:gap/gap.dart';
import 'package:go_router/go_router.dart';
import 'package:path_provider/path_provider.dart';
import 'package:zkool/main.dart';
import 'package:zkool/pages/account.dart';
import 'package:zkool/router.dart';
import 'package:zkool/src/rust/api/account.dart';
import 'package:zkool/src/rust/api/db.dart';
import 'package:zkool/src/rust/api/network.dart';
import 'package:zkool/store.dart';
import 'package:zkool/utils.dart';
import 'package:zkool/widgets/editable_list.dart';

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
        logger.i("Creating database file: $dbFilepath with password: $password");
        await createDatabase(
            dbFilepath: dbFilepath, password: password, coin: 0);
        logger.i("Database file created: $dbFilepath");
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
  var height = 0;
  final listKey = GlobalKey<EditableListState<Account>>();

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
      if (mounted) setState(() => this.height = height);
    } on AnyhowException catch (e) {
      if (mounted) await showException(context, e.message);
    }
  }

  List<Account> get accounts => AppStoreBase.instance.accounts
      .where((a) => includeHidden || !a.hidden)
      .toList();

  @override
  Widget build(BuildContext context) {
    return Observer(builder: (context) {
      final tt = Theme.of(context).textTheme;
      final t = tt.bodyMedium!.copyWith(fontFamily: "monospace");

      return EditableList<Account>(
          key: listKey,
          items: accounts,
          headerBuilder: (context) => [
                ElevatedButton(
                    onPressed: () => Future(refreshHeight),
                    child: Text("Height: $height")),
                const Gap(8),
              ],
          builder: (context, index, account, {selected, onSelectChanged}) {
            return Material(
                key: ValueKey(account.id),
                child: GestureDetector(
                  child: ListTile(
                    leading: account.avatar(
                        selected: selected ?? false, onTap: onSelectChanged),
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
          createBuilder: (context) => GoRouter.of(context).push("/account/new"),
          editBuilder: (context, a) =>
              GoRouter.of(context).push("/account/edit", extra: a),
          deleteBuilder: (context, accounts) async {
            final confirmed = await AwesomeDialog(
                context: context,
                dialogType: DialogType.warning,
                animType: AnimType.rightSlide,
                title: 'Delete Account(s)',
                desc: 'Are you sure you want to delete these accounts?',
                btnCancelOnPress: () {},
                btnOkOnPress: () {},
                autoDismiss: false,
                onDismissCallback: (d) {
                  final res = (() {
                    switch (d) {
                      case DismissType.btnOk:
                        return true;
                      default:
                        return false;
                    }
                  })();
                  GoRouter.of(context).pop(res);
                }).show() as bool;
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
            IconButton(onPressed: onSettings, icon: Icon(Icons.settings)),
            IconButton(onPressed: onSync, icon: Icon(Icons.sync)),
            IconButton(
                onPressed: onHide,
                icon: Icon(
                    includeHidden ? Icons.visibility : Icons.visibility_off)),
          ]);
    });
  }

  onHide() async {
    setState(() {
      includeHidden = !includeHidden;
    });
  }

  onSync() async {
    try {
      final listState = listKey.currentState!;
      final accounts = AppStoreBase.instance.accounts;
      List<int> accountIds = [];
      final hasSelection = listState.selected.any((s) => s);
      for (var i = 0; i < accounts.length; i++) {
        // if any selection, use the selection, otherwise use the enabled flag
        if ((hasSelection && listState.selected[i]) ||
            (!hasSelection && accounts[i].enabled))
          accountIds.add(accounts[i].id);
      }
      final syncProgress =
          await AppStoreBase.instance.startSynchronize(accountIds);
      if (syncProgress == null) return;
      syncProgress.listen((progress) {
        for (var id in accountIds) {
          AppStoreBase.instance.heights[id] = progress.height;
        }
      }, onDone: () {
        if (mounted) {
          AppStoreBase.instance.loadAccounts();
        }
      });
    } on AnyhowException catch (e) {
      if (mounted) await showException(context, e.message);
    }
  }

  onOpen(BuildContext context, Account account) {
    GoRouter.of(context).push('/account', extra: account);
  }

  onReorder(int oldIndex, int newIndex) async {
    logger.i("Reorder $oldIndex to $newIndex");

    await reorderAccount(
        oldPosition: accounts[oldIndex].position,
        newPosition: accounts[newIndex].position);
    await AppStoreBase.instance.loadAccounts();
  }

  onSettings() {
    GoRouter.of(context).push('/settings');
  }
}
