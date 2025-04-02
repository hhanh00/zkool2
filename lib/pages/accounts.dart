import 'package:data_table_2/data_table_2.dart';
import 'package:flutter/material.dart';
import 'package:zkool/src/rust/api/account.dart';
import 'package:collection/collection.dart';

class AccountList extends StatefulWidget {
  const AccountList({super.key});

  @override
  State<AccountList> createState() => AccountListState();
}

class AccountListState extends State<AccountList> {
  List<bool> selected = [];
  List<Account> accounts = [];

  @override
  void initState() {
    super.initState();
    Future(() async {
      final accountList = await listAccounts(coin: 0);
      setState(() {
        accounts = accountList;
        selected = List.generate(accountList.length, (index) => false);
      });
    });
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
              label: Text('Name'),
              size: ColumnSize.L,
            ),
            DataColumn2(
              label: Text('Balance'),
              size: ColumnSize.M,
            ),
          ],
          rows: accounts.asMap().entries
              .map((e) => DataRow(selected: selected[e.key], cells: [
                    DataCell(Text(e.value.name)),
                    DataCell(Text("0.000")),
                  ],
                  onSelectChanged: (v) => setState(() => selected[e.key] = v!)
              ))
              .toList()),
    );
  }
}
