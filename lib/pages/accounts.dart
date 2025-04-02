import 'package:data_table_2/data_table_2.dart';
import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';
import 'package:mobx/mobx.dart';
import 'package:zkool/pages/account.dart';
import 'package:zkool/src/rust/api/account.dart';

class AccountListPage extends StatefulWidget {
  const AccountListPage({super.key});

  @override
  State<AccountList> createState() => AccountListState();
}

final accountListTrigger = Observable(0);
void updateList() {
  runInAction(() => accountListTrigger.value += 1);
}

class AccountListPageState extends State<AccountListPage> {
  List<bool> selected = [];
  List<Account> accounts = [];
  ReactionDisposer? reaction;

  @override
  void initState() {
    super.initState();

    reaction = autorun((_) {
      accountListTrigger;
      print("Account list trigger: ${accountListTrigger.value}");
      Future(() async {
        final accountList = await listAccounts(coin: 0);
        setState(() {
          accounts = accountList;
          selected = List.generate(accountList.length, (index) => false);
        });
      });
    });

    updateList();
  }

  @override
  void dispose() {
    reaction?.call();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: Text('Account List'),
      ),
      body: DataTable2(
          columnSpacing: 12,
          horizontalMargin: 12,
          minWidth: 600,
          headingCheckboxTheme: const CheckboxThemeData(
              side: BorderSide(color: Colors.white, width: 2.0)),
          columns: [
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
          ],
          rows: accounts
              .asMap()
              .entries
              .map((e) => DataRow(
                  selected: selected[e.key],
                  cells: [
                    DataCell(e.value.avatar),
                    DataCell(Text(e.value.name)),
                    DataCell(Text("0.000")),
                  ],
                  onSelectChanged: (v) => setState(() => selected[e.key] = v!),
                  onLongPress: () => GoRouter.of(context)
                      .push("/account/edit", extra: e.value)))
              .toList()),
    );
  }
}
