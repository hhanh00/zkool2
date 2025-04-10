import 'dart:async';

import 'package:awesome_dialog/awesome_dialog.dart';
import 'package:flutter/material.dart';
import 'package:gap/gap.dart';
import 'package:go_router/go_router.dart';
import 'package:zkool/main.dart';
import 'package:zkool/pages/account.dart';
import 'package:zkool/src/rust/api/account.dart';
import 'package:zkool/src/rust/api/network.dart';
import 'package:zkool/src/rust/api/sync.dart';
import 'package:zkool/store.dart';
import 'package:zkool/widgets/editable_list.dart';

class AccountListPage extends StatefulWidget {
  const AccountListPage({super.key});

  @override
  State<AccountListPage> createState() => AccountListPageState();
}

class AccountListPageState extends State<AccountListPage> {
  @override
  Widget build(BuildContext context) {
    return EditableList<Account>(
      observable: () => appStore.accounts,
      builder: (context, index, account, {selected, onSelectChanged}) =>
      Material(key: ValueKey(account.id), child: GestureDetector(child:
        SizedBox(height: 60, child: Row(children: [
          Checkbox(value: selected, onChanged: onSelectChanged),
          const Gap(8),
          SizedBox(width: 24, child: Text(account.position.toString(), textAlign: TextAlign.end,)),
          const Gap(8),
          account.avatar,
          const Gap(8),
          Expanded(child: Text(account.name, style: !account.enabled ? TextStyle(color: Colors.grey) : null)),
          Text(account.height.toString()),
          const Gap(8),
        ])),
        onTap: () => onOpen(context, account),
      )),
      title: "Account List",
      onCreate: () => AppStoreBase.loadAccounts(),
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
            await deleteAccount(account: a);
          }
          await AppStoreBase.loadAccounts();
        }
      },
      isEqual:(a, b) => a.id == b.id,
      onReorder: onReorder,
      buttons: [
        IconButton(onPressed: onSync, icon: Icon(Icons.sync)),
      ]
    );
  }

  onSync() async {
    final accountIds = appStore.accounts.where((a) => a.enabled).map((a) => a.id).toList();
    final syncProgress = await startSync(accountIds: accountIds);
    syncProgress.listen(null, onDone: () {
      if (mounted) {
        AppStoreBase.loadAccounts();
      }
    });
  }

  onOpen(BuildContext context, Account account) {
    GoRouter.of(context).push('/account', extra: account);
  }

  onReorder(int oldIndex, int newIndex) async {
    logger.i("Reorder $oldIndex to $newIndex");
    
    await reorderAccount(
      oldPosition: appStore.accounts[oldIndex].position, 
      newPosition: appStore.accounts[newIndex].position);
    await AppStoreBase.loadAccounts();
  }
}

Future<Stream<SyncProgress>> startSync({required List<int> accountIds}) async {
  final currentHeight = await getCurrentHeight();
  return synchronize(accounts: accountIds, currentHeight: currentHeight);
}

