import 'package:awesome_dialog/awesome_dialog.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_form_builder/flutter_form_builder.dart';
import 'package:form_builder_validators/form_builder_validators.dart';
import 'package:gap/gap.dart';
import 'package:go_router/go_router.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:showcaseview/showcaseview.dart';
import 'package:zkool/router.dart';
import 'package:zkool/src/rust/api/db.dart';
import 'package:zkool/src/rust/api/network.dart';
import 'package:zkool/src/rust/api/sync.dart';
import 'package:zkool/src/rust/coin.dart';
import 'package:zkool/store.dart';
import 'package:zkool/utils.dart';

final logID = GlobalKey();
final lightnodeID = GlobalKey();
final lwdID = GlobalKey();
final torID = GlobalKey();
final actionsID = GlobalKey();
final autosyncID = GlobalKey();
final cancelID = GlobalKey();
final pinLockID = GlobalKey();
final offlineID = GlobalKey();

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
  bool useTor = appStore.useTor;
  bool needPin = appStore.needPin;
  bool offline = appStore.offline;
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
    tutorialHelper(context, "tutSettings0", [logID, lightnodeID, lwdID, torID, actionsID, autosyncID, cancelID, pinLockID, offlineID]);
  }

  @override
  Widget build(BuildContext context) {
    final t = Theme.of(context).textTheme;

    Future(tutorial);

    return Scaffold(
      appBar: AppBar(
        title: Text("Settings"),
        actions: [
          Showcase(
              key: logID,
              description: "Open the App Log",
              child: IconButton(tooltip: "View Log", onPressed: () => onOpenLog(context), icon: Icon(Icons.description))),
          IconButton(tooltip: "Lock", onPressed: lockApp, icon: Icon(Icons.lock)),
          IconButton(tooltip: "Show Tutorials again", onPressed: () => resetTutorial(context), icon: Icon(Icons.school)),
          IconButton(tooltip: "Database Manager", onPressed: onDatabaseManager, icon: Icon(Icons.folder)),
        ],
      ),
      body: SingleChildScrollView(
        child: FormBuilder(
          key: formKey,
          child: Padding(
            padding: const EdgeInsets.symmetric(horizontal: 16),
            child: Column(
              children: [
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
                      decoration: InputDecoration(labelText: "${isLightNode ? 'Light' : 'Full'} Node Server"),
                      initialValue: lwd,
                      onChanged: onChangedLWD,
                    )),
                if (isLightNode)
                  Showcase(
                      key: torID,
                      description: "Use TOR to connect to lightwallet server. Need App Restart",
                      child: FormBuilderSwitch(
                        name: "tor",
                        title: Text("Use TOR"),
                        initialValue: useTor,
                        onChanged: onChangedUseTOR,
                      )),
                Showcase(
                    key: actionsID,
                    description: "Number actions per synchronization chunk",
                    child: FormBuilderTextField(
                      name: "actions_per_sync",
                      decoration: const InputDecoration(labelText: "Actions per Sync"),
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
                            description: "AutoSync interval in blocks. Accounts that are behind by more than this value will start synchronization",
                            child: FormBuilderTextField(
                              name: "autosync",
                              decoration: const InputDecoration(labelText: "AutoSync Interval"),
                              initialValue: syncInterval,
                              onChanged: onChangedSyncInterval,
                              validator: FormBuilderValidators.integer(),
                              keyboardType: TextInputType.number,
                              inputFormatters: [FilteringTextInputFormatter.digitsOnly],
                            ))),
                    Showcase(
                        key: cancelID,
                        description: "This will cancel the current sync and disable AutoSync",
                        child: IconButton(tooltip: "Cancel Sync", onPressed: onCancelSync, icon: Icon(Icons.cancel))),
                  ],
                ),
                Gap(8),
                Showcase(
                    key: pinLockID,
                    description: "Ask for device pin when app opens",
                    child: FormBuilderSwitch(name: "pin_lock", title: Text("Pin Lock"), initialValue: needPin, onChanged: onPinLockChanged)),
                Gap(8),
                Showcase(
                    key: offlineID,
                    description: "Toggle offline mode",
                    child: FormBuilderSwitch(name: "offline", title: Text("Offline"), initialValue: offline, onChanged: onOfflineChanged)),
                Gap(16),
                CopyableText(appStore.dbFilepath, style: t.bodySmall),
                Gap(8),
                if (appStore.versionString != null) Text(appStore.versionString!)
              ],
            ),
          ),
        ),
      ),
    );
  }

  void onOpenLog(BuildContext context) async {
    await GoRouter.of(context).push("/log");
  }

  void onCancelSync() async {
    final confirmed = await confirmDialog(context, title: "Cancel Sync", message: "Do you want to cancel the current sync? AutoSync will be disabled too");
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
    setLwd(lwd: value, serverType: appStore.isLightNode ? ServerType.lwd : ServerType.zebra);
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

  onChangedUseTOR(bool? value) async {
    if (value == null) return;
    final prefs = SharedPreferencesAsync();
    await prefs.setBool("use_tor", value);
    appStore.useTor = value;
    setUseTor(useTor: value);
    setState(() {
      useTor = value;
    });
  }

  onPinLockChanged(bool? value) async {
    if (value == null) return;
    final prefs = SharedPreferencesAsync();
    await prefs.setBool("pin_lock", value);
    appStore.needPin = value;
    setState(() {
      needPin = value;
    });
  }

  onOfflineChanged(bool? value) async {
    if (value == null) return;
    final prefs = SharedPreferencesAsync();
    await prefs.setBool("offline", value);
    appStore.offline = value;
    setState(() {
      offline = value;
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

  void onDatabaseManager() async {
    final confirmed = await confirmDialog(context,
        title: "Database Manager", message: "The Database Manager will open when you restart the app. Do you want to schedule it now?");
    if (confirmed) {
      final prefs = SharedPreferencesAsync();
      await prefs.setBool("recovery", true);
      await showMessage(context, "Restart the app to enter the database manager");
    }
  }
}

Future<(String, String)?> showChangeDbPassword(BuildContext context, {required String databaseName}) async {
  final oldPassword = TextEditingController();
  final newPassword = TextEditingController();

  bool confirmed = await AwesomeDialog(
        context: context,
        dialogType: DialogType.question,
        animType: AnimType.rightSlide,
        title: "Change Database Password",
        body: FormBuilder(
            child: Column(children: [
          Text("Change $databaseName Password", style: Theme.of(context).textTheme.headlineSmall),
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
