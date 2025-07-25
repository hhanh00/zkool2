import 'dart:io';

import 'package:awesome_dialog/awesome_dialog.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_form_builder/flutter_form_builder.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge.dart';
import 'package:form_builder_validators/form_builder_validators.dart';
import 'package:gap/gap.dart';
import 'package:go_router/go_router.dart';
import 'package:path_provider/path_provider.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:showcaseview/showcaseview.dart';
import 'package:zkool/router.dart';
import 'package:zkool/src/rust/api/db.dart';
import 'package:zkool/src/rust/api/network.dart';
import 'package:zkool/src/rust/api/sync.dart';
import 'package:zkool/src/rust/coin.dart';
import 'package:zkool/store.dart';
import 'package:zkool/utils.dart';

final databaseID = GlobalKey();
final lightnodeID = GlobalKey();
final lwdID = GlobalKey();
final actionsID = GlobalKey();
final autosyncID = GlobalKey();
final cancelID = GlobalKey();

class SettingsPage extends StatefulWidget {
  const SettingsPage({super.key});

  @override
  State<SettingsPage> createState() => SettingsPageState();
}

class SettingsPageState extends State<SettingsPage> with RouteAware {
  final formKey = GlobalKey<FormBuilderState>();
  String databaseName = appStore.dbName;
  late String currentDatabaseName = databaseName;
  bool isLightNode = appStore.isLightNode;
  String lwd = appStore.lwd;
  String syncInterval = appStore.syncInterval;
  String actionsPerSync = appStore.actionsPerSync;

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    final route = ModalRoute.of(context);
    if (route is PageRoute) {
      routeObserver.subscribe(this, route);
    }
  }

  @override
  void didPop() {
    super.didPop();
    appStore.autoSync();
  }

  void tutorial() async {
    tutorialHelper(context, "tutSettings0",
        [databaseID, lightnodeID, lwdID, actionsID, autosyncID, cancelID]);
  }

  @override
  Widget build(BuildContext context) {
    final t = Theme.of(context).textTheme;

    Future(tutorial);

    return Scaffold(
      appBar: AppBar(
        title: Text("Settings"),
        actions: [
          IconButton(
              tooltip: "Show Tutorials again",
              onPressed: resetTutorial,
              icon: Icon(Icons.school))
        ],
      ),
      body: SingleChildScrollView(
        child: FormBuilder(
          key: formKey,
          child: Padding(
            padding: const EdgeInsets.symmetric(horizontal: 16),
            child: Column(
              children: [
                Row(children: [
                  Expanded(
                      child: Showcase(
                          key: databaseID,
                          description:
                              "Change the database file. This requires a RESTART after",
                          child: FormBuilderTextField(
                            name: "database_name",
                            decoration: const InputDecoration(
                                labelText: "Database Name"),
                            initialValue: databaseName,
                            onChanged: onChangedDatabaseName,
                          ))),
                  if (databaseName != currentDatabaseName) ...[
                    IconButton(
                        tooltip: "Delete Database",
                        onPressed: onDeleteDatabase,
                        icon: Icon(Icons.delete)),
                    IconButton(
                        tooltip: "Change Database Password",
                        onPressed: onChangePassword,
                        icon: Icon(Icons.lock_reset)),
                  ]
                ]),
                Showcase(
                    key: lightnodeID,
                    description: "Whether the server is a light node or not",
                    child: FormBuilderSwitch(
                      name: "light",
                      title: Text("Light Node"),
                      initialValue: isLightNode,
                      onChanged: onChangedIsLightNode,
                    )),
                Showcase(
                    key: lwdID,
                    description: "Node server to connect to",
                    child: FormBuilderTextField(
                      name: "lwd",
                      decoration: InputDecoration(
                          labelText: "${isLightNode ? 'Light' : 'Full'} Node Server"),
                      initialValue: lwd,
                      onChanged: onChangedLWD,
                    )),
                Showcase(
                    key: actionsID,
                    description: "Number actions per synchronization chunk",
                    child: FormBuilderTextField(
                      name: "actions_per_sync",
                      decoration:
                          const InputDecoration(labelText: "Actions per Sync"),
                      initialValue: actionsPerSync,
                      onChanged: onChangedActionsPerSync,
                      validator: FormBuilderValidators.integer(),
                      keyboardType: TextInputType.number,
                      inputFormatters: [FilteringTextInputFormatter.digitsOnly],
                    )),
                Gap(16),
                Row(
                  children: [
                    Expanded(
                        child: Showcase(
                            key: autosyncID,
                            description:
                                "AutoSync interval in blocks. Accounts that are behind by more than this value will start synchronization",
                            child: FormBuilderTextField(
                              name: "autosync",
                              decoration: const InputDecoration(
                                  labelText: "AutoSync Interval"),
                              initialValue: syncInterval,
                              onChanged: onChangedSyncInterval,
                              validator: FormBuilderValidators.integer(),
                              keyboardType: TextInputType.number,
                              inputFormatters: [
                                FilteringTextInputFormatter.digitsOnly
                              ],
                            ))),
                    Showcase(
                        key: cancelID,
                        description:
                            "This will cancel the current sync and disable AutoSync",
                        child: IconButton(
                            tooltip: "Cancel Sync",
                            onPressed: onCancelSync,
                            icon: Icon(Icons.cancel)))
                  ],
                ),
                Gap(16),
                SelectableText(appStore.dbFilepath,
                    style: t.bodySmall),
                Gap(8),
                if (appStore.versionString != null)
                  Text(appStore.versionString!)
              ],
            ),
          ),
        ),
      ),
    );
  }

  void onCancelSync() async {
    final confirmed = await confirmDialog(context,
        title: "Cancel Sync",
        message:
            "Do you want to cancel the current sync? AutoSync will be disabled too");
    if (!confirmed) return;
    formKey.currentState!.fields["autosync"]!.didChange("0");
    await cancelSync();
  }

  void onChangedDatabaseName(String? value) async {
    if (value == null) return;
    final prefs = SharedPreferencesAsync();
    await prefs.setString("database", value);
    appStore.dbName = value;
    setState(() {
      databaseName = value;
    });
  }

  void onChangedLWD(String? value) async {
    if (value == null) return;
    await putProp(key: "lwd", value: value);
    appStore.lwd = value;
    setLwd(
        lwd: value,
        serverType: appStore.isLightNode
            ? ServerType.lwd
            : ServerType.zebra);
    setState(() {
      lwd = value;
    });
  }

  onChangedIsLightNode(bool? value) async {
    if (value == null) return;
    await putProp(key: "is_light_node", value: value.toString());
    appStore.isLightNode = value;
    setLwd(lwd: lwd, serverType: value ? ServerType.lwd : ServerType.zebra);
    setState(() {
      isLightNode = value;
    });
  }

  onChangedActionsPerSync(String? value) async {
    if (value == null) return;
    if (int.tryParse(value) == null) {
      return;
    }
    await putProp(key: "actions_per_sync", value: value);
    appStore.actionsPerSync = value;
    setState(() {
      actionsPerSync = value;
    });
  }

  onChangedSyncInterval(String? value) async {
    if (value == null) return;
    if (int.tryParse(value) == null) {
      return;
    }
    await putProp(key: "sync_interval", value: value);
    appStore.syncInterval = value;
    setState(() {
      syncInterval = value;
    });
  }

  void onDeleteDatabase() async {
    final confirmed = await confirmDialog(context,
        title: "Delete Database",
        message:
            "Do you really want to delete the database $databaseName? This will remove all your data and cannot be undone!");
    if (!confirmed) return;
    final dbDir = await getApplicationDocumentsDirectory();
    final dbFilepath = '${dbDir.path}/$databaseName.db';
    await File(dbFilepath).delete();
    if (!mounted) return;
    await showMessage(context, "Database $databaseName deleted");
    setState(() {
      formKey.currentState!.fields["database_name"]!
          .didChange(currentDatabaseName);
      databaseName = currentDatabaseName;
      appStore.dbName = databaseName;
    });
  }

  void onChangePassword() async {
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

Future<(String, String)?> showChangeDbPassword(BuildContext context,
    {required String databaseName}) async {
  final oldPassword = TextEditingController();
  final newPassword = TextEditingController();

  bool confirmed = await AwesomeDialog(
        context: context,
        dialogType: DialogType.question,
        animType: AnimType.rightSlide,
        title: "Change Database Password",
        body: FormBuilder(
            child: Column(children: [
          Text("Change $databaseName Password",
              style: Theme.of(context).textTheme.headlineSmall),
          Gap(8),
          FormBuilderTextField(
            name: 'old_password',
            decoration: InputDecoration(labelText: 'Old Password'),
            obscureText: true,
            controller: oldPassword,
          ),
          Gap(8),
          FormBuilderTextField(
            name: 'new_password',
            decoration: InputDecoration(labelText: 'New Password'),
            obscureText: true,
            controller: newPassword,
          ),
        ])),
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
    final op = oldPassword.text;
    final np = newPassword.text;
    return (op, np);
  }
  return null;
}
