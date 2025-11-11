import 'dart:io';

import 'package:awesome_dialog/awesome_dialog.dart';
import 'package:collection/collection.dart';
import 'package:flutter/material.dart';
import 'package:flutter_form_builder/flutter_form_builder.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge.dart';
import 'package:form_builder_validators/form_builder_validators.dart';
import 'package:gap/gap.dart';
import 'package:go_router/go_router.dart';
import 'package:path_provider/path_provider.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:zkool/main.dart';
import 'package:zkool/src/rust/api/db.dart';
import 'package:zkool/store.dart';
import 'package:zkool/utils.dart';

class DatabaseManagerPage extends ConsumerStatefulWidget {
  const DatabaseManagerPage({super.key});

  @override
  ConsumerState<DatabaseManagerPage> createState() => DatabaseManagerState();
}

class DatabaseManagerState extends ConsumerState<DatabaseManagerPage> {
  List<(String, bool)> dbNames = [];

  @override
  void initState() {
    super.initState();
    Future(refresh);
  }

  Future<void> refresh() async {
    final dbDir = await getApplicationDocumentsDirectory();
    dbNames = (await listDbNames(dir: dbDir.path)).sorted().map((n) => (n, false)).toList();
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
            IconButton(tooltip: "Load Database", onPressed: onOpenDatabase, icon: Icon(Icons.file_open)),
            IconButton(tooltip: "Save Database", onPressed: onSaveDatabase, icon: Icon(Icons.save)),
            IconButton(onPressed: onChangeName, icon: Icon(Icons.edit)),
            IconButton(onPressed: onChangePassword, icon: Icon(Icons.password)),
          ],
          if (hasSelection) IconButton(onPressed: onDeleteDatabases, icon: Icon(Icons.delete)),
          IconButton(onPressed: onOK, icon: Icon(Icons.check)),
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
              },
            ),
            title: Text(dbName.$1),
            onTap: () => onSelect(dbName.$1),
          );
        },
      ),
    );
  }

  Iterable<String> get selection => dbNames.where((a) => a.$2).map((a) => a.$1);
  bool get hasSingleSelection => selection.length == 1;
  bool get hasSelection => selection.isNotEmpty;

  void onSelect(String dbName) async {
    await selectDatabase(ref, dbName);
    await showMessage(context, "Database $dbName selected");
  }

  void onNewDatabase() async {
    final formKey = GlobalKey<FormBuilderState>();
    final res = await inputData<(String, String?)>(
      context,
      builder: (context) => FormBuilder(
        key: formKey,
        child: Column(
          children: [
            Text("Create New Database", style: Theme.of(context).textTheme.headlineSmall),
            Gap(8),
            FormBuilderTextField(
              name: "name",
              decoration: InputDecoration(labelText: 'Name'),
              validator: FormBuilderValidators.required(),
            ),
            Gap(8),
            FormBuilderTextField(
              name: "password",
              decoration: InputDecoration(labelText: 'Password'),
              obscureText: true,
            ),
            Gap(8),
            FormBuilderTextField(
              name: "repeat_password",
              decoration: InputDecoration(labelText: 'Repeat Password'),
              obscureText: true,
              validator: (v) {
                final newPassword = formKey.currentState!.fields["password"]!.value as String?;
                if (newPassword != v) return "Passwords do not match";
                return null;
              },
            ),
          ],
        ),
      ),
      validate: () => formKey.currentState!.validate(),
      onConfirmed: () {
        final fields = formKey.currentState!.fields;
        final name = fields["name"]!.value as String;
        final password = fields["password"]!.value as String?;
        return (name, password);
      },
    );

    if (res != null) {
      final (name, password) = res;
      final dbFilepath = await getFullDatabasePath(name);
      await openDatabase(dbFilepath: dbFilepath, password: password);
      await refresh();
    }
  }

  void onSaveDatabase() async {
    final databaseName = selection.first;
    final db = File(await getFullDatabasePath(databaseName));
    final data = await db.readAsBytes();
    final res = await appWatcher.saveFile(title: "Save Database", fileName: "$databaseName.db", data: data);
    if (!mounted) return;
    if (res != null) await showMessage(context, "Database saved");
  }

  void onOpenDatabase() async {
    final databaseName = selection.first;
    final data = await appWatcher.openFile(title: "Open Database");
    if (data == null) return;
    if (!mounted) return;
    final confirmed = await confirmDialog(
      context,
      title: "Restore Database",
      message: "Are you sure you want to restore the database? This file erase the contents of the selected database",
    );
    if (!confirmed) return;
    final db = File(await getFullDatabasePath(databaseName));
    await db.writeAsBytes(data);
    if (!mounted) return;
    await showMessage(context, "Database restored");
  }

  Future<void> onDeleteDatabases() async {
    final confirmed = await confirmDialog(
      context,
      title: "Delete Databases",
      message: "Do you really want to delete the selected databases? This will remove all your data and cannot be undone!",
    );
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
          body: Column(
            children: [
              Text("Change Database Name", style: Theme.of(context).textTheme.headlineSmall),
              Gap(8),
              TextField(
                decoration: InputDecoration(labelText: 'Name'),
                controller: name,
              ),
            ],
          ),
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
        oldPassword: oldPassword ?? "",
        newPassword: newPassword ?? "",
      );
    } on AnyhowException catch (e) {
      if (!mounted) return;
      await showException(context, "Failed to change database password: $e");
      return;
    }
    if (!mounted) return;
    await showMessage(context, "Database password changed successfully");
  }

  Future<void> onOK() async {
    final prefs = SharedPreferencesAsync();
    await prefs.remove("recovery");
    GoRouter.of(context).go("/splash");
  }
}

Future<void> selectDatabase(WidgetRef ref, String dbName) async {
  final prefs = SharedPreferencesAsync();
  await prefs.setString("database", dbName);
  final settings = ref.read(appSettingsProvider.notifier);
  settings.setDbName(dbName);
}

Future<(String?, String?)?> showChangeDbPassword(BuildContext context, {required String databaseName}) async {
  final formKey = GlobalKey<FormBuilderState>();

  return await inputData<(String?, String?)>(
    context,
    builder: (BuildContext context) => FormBuilder(
      key: formKey,
      child: Column(
        children: [
          Text("Change $databaseName Password", style: Theme.of(context).textTheme.headlineSmall),
          Gap(8),
          FormBuilderTextField(
            name: 'old_password',
            decoration: InputDecoration(labelText: 'Old Password'),
            obscureText: true,
          ),
          Gap(8),
          FormBuilderTextField(
            name: 'new_password',
            decoration: InputDecoration(labelText: 'New Password'),
            obscureText: true,
          ),
          Gap(8),
          FormBuilderTextField(
            name: 'repeat_password',
            decoration: InputDecoration(labelText: 'Repeat New Password'),
            obscureText: true,
            validator: (v) {
              final newPassword = formKey.currentState!.fields["new_password"]!.value as String?;
              if (newPassword != v) return "New password does not match";
              return null;
            },
          ),
        ],
      ),
    ),
    validate: () => formKey.currentState!.validate(),
    onConfirmed: () {
      final fields = formKey.currentState!.fields;
      final oldPassword = fields["old_password"]!.value as String?;
      final newPassword = fields["new_password"]!.value as String?;
      logger.i("$oldPassword $newPassword");
      return (oldPassword, newPassword);
    },
  );
}
