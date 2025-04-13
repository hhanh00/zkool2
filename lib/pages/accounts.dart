import 'dart:async';
import 'dart:io';

import 'package:awesome_dialog/awesome_dialog.dart';
import 'package:file_picker/file_picker.dart';
import 'package:flutter/material.dart';
import 'package:flutter_form_builder/flutter_form_builder.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated_io.dart';
import 'package:gap/gap.dart';
import 'package:go_router/go_router.dart';
import 'package:zkool/main.dart';
import 'package:zkool/pages/account.dart';
import 'package:zkool/src/rust/api/account.dart';
import 'package:zkool/src/rust/api/network.dart';
import 'package:zkool/src/rust/api/sync.dart';
import 'package:zkool/store.dart';
import 'package:zkool/utils.dart';
import 'package:zkool/widgets/editable_list.dart';

class AccountListPage extends StatefulWidget {
  const AccountListPage({super.key});

  @override
  State<AccountListPage> createState() => AccountListPageState();
}

class AccountListPageState extends State<AccountListPage> {
  var hiding = true;
  var height = 0;
  Timer? heightPollingTimer;

  @override
  void initState() {
    super.initState();
    Future(refreshHeight);
  }

  @override
  void dispose() {
    heightPollingTimer?.cancel();
    super.dispose();
  }

  void refreshHeight() async {
    try {
      final height = await getCurrentHeight();
      if (mounted) setState(() => this.height = height);
    } on AnyhowException catch (e) {
      if (mounted) await showException(context, e.message);
    }
  }

  @override
  Widget build(BuildContext context) {
    return EditableList<Account>(
        observable: () => AppStoreBase.instance.accounts,
        headerBuilder: (context) => [
              ElevatedButton(
                  onPressed: () => Future(refreshHeight),
                  child: Text("Height: $height")),
              const Gap(8),
            ],
        builder: (context, index, account, {selected, onSelectChanged}) =>
            Material(
                key: ValueKey(account.id),
                child: GestureDetector(
                  child: SizedBox(
                      height: 60,
                      child: Row(children: [
                        Checkbox(value: selected, onChanged: onSelectChanged),
                        const Gap(8),
                        SizedBox(
                            width: 24,
                            child: Text(
                              account.position.toString(),
                              textAlign: TextAlign.end,
                            )),
                        const Gap(8),
                        account.avatar,
                        const Gap(8),
                        Expanded(
                            child: Text(account.name,
                                style: !account.enabled
                                    ? TextStyle(color: Colors.grey)
                                    : null)),
                        Text(account.height.toString()),
                        const Gap(8),
                      ])),
                  onTap: () => onOpen(context, account),
                )),
        title: "Account List",
        onCreate: () => AppStoreBase.instance.loadAccounts(),
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
          IconButton(onPressed: onImport, icon: Icon(Icons.file_download)),
          IconButton(onPressed: onSync, icon: Icon(Icons.sync)),
          IconButton(
              onPressed: onHide,
              icon: Icon(hiding ? Icons.visibility : Icons.visibility_off)),
        ]);
  }

  onImport() async {
    try {
      final files = await FilePicker.platform.pickFiles(
        dialogTitle: 'Please select an encrypted account file for import',
      );
      if (files == null) return;
      if (!mounted) return;
      final file = files.files.first;
      final password = TextEditingController();
      bool confirmed = await AwesomeDialog(
            context: context,
            dialogType: DialogType.question,
            animType: AnimType.rightSlide,
            body: FormBuilder(
                child: FormBuilderTextField(
              name: 'password',
              decoration: InputDecoration(labelText: 'Password'),
              obscureText: true,
              controller: password,
            )),
            btnCancelOnPress: () {},
            btnOkOnPress: () {},
            onDismissCallback: (type) {
              final res = (() {
                switch (type) {
                  case DismissType.btnOk:
                    return true;
                  default:
                    return false;
                }
              })();
              GoRouter.of(context).pop(res);
            },
            autoDismiss: false,
          ).show() ??
          false;
      if (confirmed) {
        final p = password.text;
        final encryptedFile = File(file.path!);
        final encrypted = encryptedFile.readAsBytesSync();
        await importAccount(passphrase: p, data: encrypted);
        await AppStoreBase.instance.loadAccounts();
      }
    } on AnyhowException catch (e) {
      if (mounted) await showException(context, e.message);
    }
  }

  onHide() async {
    setState(() {
      hiding = !hiding;
      AppStoreBase.instance.includeHidden = !hiding;
      AppStoreBase.instance.loadAccounts();
    });
  }

  onSync() async {
    try {
      final accountIds = AppStoreBase.instance.accounts
          .where((a) => a.enabled)
          .map((a) => a.id)
          .toList();
      final syncProgress = await startSync(accountIds: accountIds);
      syncProgress.listen(null, onDone: () {
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
        oldPosition: AppStoreBase.instance.accounts[oldIndex].position,
        newPosition: AppStoreBase.instance.accounts[newIndex].position);
    await AppStoreBase.instance.loadAccounts();
  }
}

Future<Stream<SyncProgress>> startSync({required List<int> accountIds}) async {
  final currentHeight = await getCurrentHeight();
  return synchronize(accounts: accountIds, currentHeight: currentHeight);
}
