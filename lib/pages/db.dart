import 'dart:io';

import 'package:awesome_dialog/awesome_dialog.dart';
import 'package:collection/collection.dart';
import 'package:flutter/material.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge.dart';
import 'package:gap/gap.dart';
import 'package:go_router/go_router.dart';
import 'package:path_provider/path_provider.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:zkool/main.dart';
import 'package:zkool/settings.dart';
import 'package:zkool/src/rust/api/db.dart';
import 'package:zkool/store.dart';
import 'package:zkool/utils.dart';

class DatabaseManagerPage extends StatefulWidget {
  const DatabaseManagerPage({super.key});

  @override
  State<StatefulWidget> createState() => DatabaseManagerState();
}

class DatabaseManagerState extends State<DatabaseManagerPage> {
  List<(String, bool)> dbNames = [];

  @override
  void initState() {
    super.initState();
    Future(refresh);
  }

  Future<void> refresh() async {
    final dbDir = await getApplicationDocumentsDirectory();
    dbNames = (await listDbNames(dir: dbDir.path, dbName: appStore.dbName))
        .sorted()
        .map((n) => (n, false))
        .toList();
    setState(() {});
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
        appBar: AppBar(
          title: Text("Database Manager"),
          actions: [
            IconButton(onPressed: onNewDatabase, icon: Icon(Icons.add)),
            if (hasSingleSelection) ...[
              IconButton(
                  tooltip: "Load Database",
                  onPressed: onOpenDatabase,
                  icon: Icon(Icons.file_open)),
              IconButton(
                  tooltip: "Save Database",
                  onPressed: onSaveDatabase,
                  icon: Icon(Icons.save)),
              IconButton(onPressed: onChangeName, icon: Icon(Icons.edit)),
              IconButton(
                  onPressed: onChangePassword, icon: Icon(Icons.password))
            ],
            if (hasSelection)
              IconButton(
                  onPressed: onDeleteDatabases, icon: Icon(Icons.delete)),
          ],
        ),
        body: ListView.builder(
          itemCount: dbNames.length,
          itemBuilder: (context, index) {
            final dbName = dbNames[index];
            return ListTile(
              leading: Checkbox(
                  value: dbName.$2,
                  onChanged: (v) {
                    setState(() => dbNames[index] = (dbName.$1, v ?? false));
                  }),
              title: Text(dbName.$1),
              onTap: () => onSelect(dbName.$1),
            );
          },
        ));
  }

  Iterable<String> get selection => dbNames.where((a) => a.$2).map((a) => a.$1);
  bool get hasSingleSelection => selection.length == 1;
  bool get hasSelection => selection.isNotEmpty;

  void onSelect(String dbName) async {
    await selectDatabase(dbName);
    GoRouter.of(context).pop(dbName);
  }

  void onNewDatabase() async {
    final name = TextEditingController();
    final password = TextEditingController();

    bool confirmed = await AwesomeDialog(
          context: context,
          dialogType: DialogType.question,
          animType: AnimType.rightSlide,
          body: Column(children: [
            Text("Create New Database",
                style: Theme.of(context).textTheme.headlineSmall),
            Gap(8),
            TextField(
              decoration: InputDecoration(labelText: 'Name'),
              controller: name,
            ),
            Gap(8),
            TextField(
              decoration: InputDecoration(labelText: 'Password'),
              obscureText: true,
              controller: password,
            )
          ]),
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
          dismissOnTouchOutside: false,
          autoDismiss: false,
        ).show() ??
        false;
    if (confirmed) {
      final dbFilepath = await getFullDatabasePath(name.text);
      final p = password.text.isNotEmpty ? password.text : null;
      await openDatabase(dbFilepath: dbFilepath, password: p);
      await refresh();
    }
  }

  void onSaveDatabase() async {
    final databaseName = selection.first;
    final db = File(await getFullDatabasePath(databaseName));
    final data = await db.readAsBytes();
    final res = await appWatcher.saveFile(
        title: "Save Database", fileName: "$databaseName.db", data: data);
    if (!mounted) return;
    if (res != null) await showMessage(context, "Database saved");
  }

  void onOpenDatabase() async {
    final databaseName = selection.first;
    final data = await appWatcher.openFile(title: "Open Database");
    if (data == null) return;
    if (!mounted) return;
    final confirmed = await confirmDialog(context,
        title: "Restore Database",
        message:
            "Are you sure you want to restore the database? This file erase the contents of the selected database");
    if (!confirmed) return;
    final db = File(await getFullDatabasePath(databaseName));
    await db.writeAsBytes(data);
    if (!mounted) return;
    await showMessage(context, "Database restored");
  }

  Future<void> onDeleteDatabases() async {
    final confirmed = await confirmDialog(context,
        title: "Delete Databases",
        message:
            "Do you really want to delete the selected databases? This will remove all your data and cannot be undone!");
    if (!confirmed) return;

    for (var dbName in selection) {
      final db = await getFullDatabasePath(dbName);
      await File(db).delete();
    }

    if (!mounted) return;
    await showMessage(context, "Databases deleted");
    await refresh();
  }

  Future<void> onChangeName() async {
    final name = TextEditingController(text: selection.first);
    bool confirmed = await AwesomeDialog(
          context: context,
          dialogType: DialogType.question,
          animType: AnimType.rightSlide,
          body: Column(children: [
            Text("Change Database Name",
                style: Theme.of(context).textTheme.headlineSmall),
            Gap(8),
            TextField(
              decoration: InputDecoration(labelText: 'Name'),
              controller: name,
            ),
          ]),
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
          dismissOnTouchOutside: false,
          autoDismiss: false,
        ).show() ??
        false;
    if (confirmed) {
      try {
        final oldDbFilepath = await getFullDatabasePath(selection.first);
        final newDbFilepath = await getFullDatabasePath(name.text);
        await File(oldDbFilepath).rename(newDbFilepath);
        await refresh();
      } on AnyhowException catch (e) {
        if (!mounted) return;
        await showException(context, "Failed to rename database: $e");
        return;
      }
    }
  }

  void onChangePassword() async {
    final databaseName = selection.first;
    final res = await showChangeDbPassword(context, databaseName: databaseName);
    if (res == null) return;
    final (oldPassword, newPassword) = res;
    try {
      await changeDbPassword(
          dbFilepath: await getFullDatabasePath(databaseName),
          tmpDir: (await getTemporaryDirectory()).path,
          oldPassword: oldPassword,
          newPassword: newPassword);
    } on AnyhowException catch (e) {
      if (!mounted) return;
      await showException(context, "Failed to change database password: $e");
      return;
    }
    if (!mounted) return;
    await showMessage(context, "Database password changed successfully");
  }
}

Future<void> selectDatabase(String dbName) async {
  final prefs = SharedPreferencesAsync();
  await prefs.setString("database", dbName);
  appStore.dbName = dbName;
}
