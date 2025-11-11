import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_form_builder/flutter_form_builder.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:form_builder_validators/form_builder_validators.dart';
import 'package:gap/gap.dart';
import 'package:go_router/go_router.dart';
import 'package:package_info_plus/package_info_plus.dart';
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
final blockExplorerID = GlobalKey();

class SettingsPage extends ConsumerStatefulWidget {
  const SettingsPage({super.key});

  @override
  ConsumerState<SettingsPage> createState() => SettingsPageState();
}

class SettingsPageState extends ConsumerState<SettingsPage> with RouteAware {
  AppSettings? settings;

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
    final settingsNotifier = ref.read(appSettingsProvider.notifier);
    settings?.let((s) => settingsNotifier.updateSettings(s));
    // appStore.autoSync();
  }

  @override
  Widget build(BuildContext context) {
    final s = ref.watch(appSettingsProvider);
    switch (s) {
      case AsyncLoading():
        return LinearProgressIndicator();
      case AsyncError(:final error):
        return Text(error.toString());
      default:
    }
    final settings = s.requireValue;
    return SettingsForm(settings);
  }
}

class SettingsForm extends ConsumerStatefulWidget {
  final AppSettings settings;
  const SettingsForm(this.settings, {super.key});
  @override
  ConsumerState<SettingsForm> createState() => SettingsFormState();
}

class SettingsFormState extends ConsumerState<SettingsForm> {
  final formKey = GlobalKey<FormBuilderState>();
  late String databaseName = widget.settings.dbName;
  String dbFullPath = "";
  String versionString = "";
  late bool isLightNode = widget.settings.isLightNode;
  late bool useTor = widget.settings.useTor;
  late bool needPin = widget.settings.needPin;
  late bool offline = widget.settings.offline;
  late String lwd = widget.settings.lwd;
  late String blockExplorer = widget.settings.blockExplorer;
  late String syncInterval = widget.settings.syncInterval;
  late String actionsPerSync = widget.settings.actionsPerSync;

  @override
  void initState() {
    super.initState();
    Future(() async {
      dbFullPath = await getFullDatabasePath(databaseName);
      final packageInfo = await PackageInfo.fromPlatform();
      final version = packageInfo.version;
      final buildNumber = packageInfo.buildNumber;
      versionString = "$version+$buildNumber";
      setState(() {});
    });
  }

  @override
  void didUpdateWidget(covariant SettingsForm oldWidget) {
    super.didUpdateWidget(oldWidget);
    databaseName = widget.settings.dbName;
    isLightNode = widget.settings.isLightNode;
    useTor = widget.settings.useTor;
    needPin = widget.settings.needPin;
    offline = widget.settings.offline;
    lwd = widget.settings.lwd;
    blockExplorer = widget.settings.blockExplorer;
    syncInterval = widget.settings.syncInterval;
    actionsPerSync = widget.settings.actionsPerSync;
  }

  void tutorial() async {
    tutorialHelper(context, "tutSettings0", [logID, lightnodeID, lwdID, torID, actionsID, autosyncID, cancelID, pinLockID, offlineID, blockExplorerID]);
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
            child: IconButton(tooltip: "View Log", onPressed: () => onOpenLog(context), icon: Icon(Icons.description)),
          ),
          IconButton(tooltip: "Lock", onPressed: () => lockApp(ref), icon: Icon(Icons.lock)),
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
                  ),
                ),
                Showcase(
                  key: lwdID,
                  description: "Node server to connect to",
                  child: FormBuilderTextField(
                    name: "lwd",
                    decoration: InputDecoration(labelText: "${isLightNode ? 'Light' : 'Full'} Node Server"),
                    initialValue: lwd,
                    onChanged: onChangedLWD,
                  ),
                ),
                if (isLightNode)
                  Showcase(
                    key: torID,
                    description: "Use TOR to connect to lightwallet server. Need App Restart",
                    child: FormBuilderSwitch(
                      name: "tor",
                      title: Text("Use TOR"),
                      initialValue: useTor,
                      onChanged: onChangedUseTOR,
                    ),
                  ),
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
                  ),
                ),
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
                        ),
                      ),
                    ),
                    Showcase(
                      key: cancelID,
                      description: "This will cancel the current sync and disable AutoSync",
                      child: IconButton(tooltip: "Cancel Sync", onPressed: onCancelSync, icon: Icon(Icons.cancel)),
                    ),
                  ],
                ),
                Gap(8),
                Showcase(
                  key: pinLockID,
                  description: "Ask for device pin when app opens",
                  child: FormBuilderSwitch(name: "pin_lock", title: Text("Pin Lock"), initialValue: needPin, onChanged: onPinLockChanged),
                ),
                Gap(8),
                Showcase(
                  key: offlineID,
                  description: "Toggle offline mode",
                  child: FormBuilderSwitch(name: "offline", title: Text("Offline"), initialValue: offline, onChanged: onOfflineChanged),
                ),
                Gap(8),
                Showcase(
                  key: blockExplorerID,
                  description: "Block Explorer URL",
                  child: FormBuilderTextField(
                    name: "block_explorer",
                    decoration: InputDecoration(
                      label: Text("Block Explorer"),
                    ),
                    initialValue: blockExplorer,
                    onChanged: onChangedBlockExplorer,
                  ),
                ),
                Gap(16),
                CopyableText(dbFullPath, style: t.bodySmall),
                Gap(8),
                Text(versionString),
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
    ref.invalidate(appSettingsProvider);
    setState(() {
      databaseName = value;
    });
  }

  void onChangedLWD(String? value) async {
    if (value == null) return;
    await putProp(key: "lwd", value: value);
    setLwd(lwd: value, serverType: isLightNode ? ServerType.lwd : ServerType.zebra);
    ref.invalidate(appSettingsProvider);
    setState(() {
      lwd = value;
    });
  }

  void onChangedBlockExplorer(String? value) async {
    if (value == null) return;
    await putProp(key: "block_explorer", value: value);
    ref.invalidate(appSettingsProvider);
    setState(() {
      blockExplorer = value;
    });
  }

  onChangedIsLightNode(bool? value) async {
    if (value == null) return;
    await putProp(key: "is_light_node", value: value.toString());
    ref.invalidate(appSettingsProvider);
    setLwd(lwd: lwd, serverType: value ? ServerType.lwd : ServerType.zebra);
    setState(() {
      isLightNode = value;
    });
  }

  onChangedUseTOR(bool? value) async {
    if (value == null) return;
    final prefs = SharedPreferencesAsync();
    await prefs.setBool("use_tor", value);
    ref.invalidate(appSettingsProvider);
    setUseTor(useTor: value);
    setState(() {
      useTor = value;
    });
  }

  onPinLockChanged(bool? value) async {
    if (value == null) return;
    final prefs = SharedPreferencesAsync();
    await prefs.setBool("pin_lock", value);
    ref.invalidate(appSettingsProvider);
    setState(() {
      needPin = value;
    });
  }

  onOfflineChanged(bool? value) async {
    if (value == null) return;
    final prefs = SharedPreferencesAsync();
    await prefs.setBool("offline", value);
    ref.invalidate(appSettingsProvider);
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
    ref.invalidate(appSettingsProvider);
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
    ref.invalidate(appSettingsProvider);
    setState(() {
      syncInterval = value;
    });
  }

  void onDatabaseManager() async {
    final confirmed = await confirmDialog(
      context,
      title: "Database Manager",
      message: "The Database Manager will open when you restart the app. Do you want to schedule it now?",
    );
    if (confirmed) {
      final prefs = SharedPreferencesAsync();
      await prefs.setBool("recovery", true);
      await showMessage(context, "Restart the app to enter the database manager");
    }
  }
}
