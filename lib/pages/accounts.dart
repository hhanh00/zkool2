import 'package:awesome_dialog/awesome_dialog.dart';
import 'package:data_table_2/data_table_2.dart';
import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';
import 'package:zkool/pages/account.dart';
import 'package:zkool/src/rust/api/account.dart';
import 'package:zkool/store.dart';
import 'package:zkool/widgets/editable_list.dart';

class AccountListPage extends StatelessWidget {
  final int coin;
  AccountListPage({required this.coin, super.key});

  final columns = [
    DataColumn2(
      label: Text('Icon'),
      size: ColumnSize.S,
    ),
    DataColumn2(
      label: Text('Name'),
      size: ColumnSize.L,
    ),
    DataColumn2(
      label: Text('Balance'),
      size: ColumnSize.M,
    ),
  ];

  @override
  Widget build(BuildContext context) {
    return EditableList<Account>(
      observable: () => appStore.accounts,
      builder: (context, index, account, {selected, onSelectChanged}) =>
          DataRow2(
        selected: selected ?? false,
        cells: [
          DataCell(account.avatar),
          DataCell(Text(account.name)),
          DataCell(Text("0.000")),
        ],
        onSelectChanged: onSelectChanged,
        onTap: () => onOpen(context, account),
      ),
      title: "Account List",
      onCreate: () => AppStoreBase.loadAccounts(coin),
      createBuilder: (context) {},
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
            deleteAccount(account: a);
          }
          AppStoreBase.loadAccounts(accounts[0].coin);
        }
      },
      columns: columns,
    );
  }

  onOpen(BuildContext context, Account account) {
    GoRouter.of(context).push('/account', extra: account);
  }
}
