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
  void initState() {
    super.initState();
    Future(() async {
      final settings = await ref.read(appSettingsProvider.future);
      setState(() => this.settings = settings);
    });
  }

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    final route = ModalRoute.of(context);
    if (route is PageRoute) {
      routeObserver.subscribe(this, route);
    }
  }

  @override
  Widget build(BuildContext context) {
    if (settings == null) return blank(context);
    return SettingsForm(
      settings!,
      onChanged: (settings) async {
        final prefs = SharedPreferencesAsync();
        await prefs.setString("database", settings.dbName);
        await putProp(key: "is_light_node", value: settings.isLightNode.toString());
        await putProp(key: "lwd", value: settings.lwd);
        await putProp(key: "block_explorer", value: settings.blockExplorer);
        await putProp(key: "actions_per_sync", value: settings.actionsPerSync);
        await putProp(key: "sync_interval", value: settings.syncInterval);
        await prefs.setBool("pin_lock", settings.needPin);
        await prefs.setBool("offline", settings.offline);
        await prefs.setBool("use_tor", settings.useTor);
        setLwd(lwd: settings.lwd, serverType: settings.isLightNode ? ServerType.lwd : ServerType.zebra);
        setUseTor(useTor: settings.useTor);
        ref.invalidate(appSettingsProvider);
      },
    );
  }
}

class SettingsForm extends ConsumerStatefulWidget {
  final void Function(AppSettings) onChanged;
  final AppSettings settings; // original settings
  const SettingsForm(this.settings, {super.key, required this.onChanged});
  @override
  ConsumerState<SettingsForm> createState() => SettingsFormState();
}

class SettingsFormState extends ConsumerState<SettingsForm> {
  final formKey = GlobalKey<FormBuilderState>();
  late AppSettings settings = widget.settings; // updated settings

  String dbFullPath = "";
  String versionString = "";

  @override
  void initState() {
    super.initState();
    Future(() async {
      dbFullPath = await getFullDatabasePath(settings.dbName);
      final packageInfo = await PackageInfo.fromPlatform();
      final version = packageInfo.version;
      final buildNumber = packageInfo.buildNumber;
      versionString = "$version+$buildNumber";
      setState(() {});
    });
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
                    initialValue: settings.isLightNode,
                    onChanged: onChangedIsLightNode,
                  ),
                ),
                Showcase(
                  key: lwdID,
                  description: "Node server to connect to",
                  child: FormBuilderTextField(
                    name: "lwd",
                    decoration: InputDecoration(labelText: "${settings.isLightNode ? 'Light' : 'Full'} Node Server"),
                    initialValue: settings.lwd,
                    onChanged: onChangedLWD,
                  ),
                ),
                if (settings.isLightNode)
                  Showcase(
                    key: torID,
                    description: "Use TOR to connect to lightwallet server. Need App Restart",
                    child: FormBuilderSwitch(
                      name: "tor",
                      title: Text("Use TOR"),
                      initialValue: settings.useTor,
                      onChanged: onChangedUseTOR,
                    ),
                  ),
                Showcase(
                  key: actionsID,
                  description: "Number actions per synchronization chunk",
                  child: FormBuilderTextField(
                    name: "actions_per_sync",
                    decoration: const InputDecoration(labelText: "Actions per Sync"),
                    initialValue: settings.actionsPerSync,
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
                          initialValue: settings.syncInterval,
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
                  child: FormBuilderSwitch(name: "pin_lock", title: Text("Pin Lock"), initialValue: settings.needPin, onChanged: onPinLockChanged),
                ),
                Gap(8),
                Showcase(
                  key: offlineID,
                  description: "Toggle offline mode",
                  child: FormBuilderSwitch(name: "offline", title: Text("Offline"), initialValue: settings.offline, onChanged: onOfflineChanged),
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
                    initialValue: settings.blockExplorer,
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
    setState(() {
      settings = settings.copyWith(dbName: value);
      widget.onChanged(settings);
    });
  }

  void onChangedLWD(String? value) async {
    if (value == null) return;
    setState(() {
      settings = settings.copyWith(lwd: value);
      widget.onChanged(settings);
    });
  }

  void onChangedBlockExplorer(String? value) async {
    if (value == null) return;
    setState(() {
      settings = settings.copyWith(blockExplorer: value);
      widget.onChanged(settings);
    });
  }

  onChangedIsLightNode(bool? value) async {
    if (value == null) return;
    setState(() {
      settings = settings.copyWith(isLightNode: value);
      widget.onChanged(settings);
    });
  }

  onChangedUseTOR(bool? value) async {
    if (value == null) return;
    setState(() {
      settings = settings.copyWith(useTor: value);
      widget.onChanged(settings);
    });
  }

  onPinLockChanged(bool? value) async {
    if (value == null) return;
    setState(() {
      settings = settings.copyWith(needPin: value);
      widget.onChanged(settings);
    });
  }

  onOfflineChanged(bool? value) async {
    if (value == null) return;
    setState(() {
      settings = settings.copyWith(offline: value);
      widget.onChanged(settings);
    });
  }

  onChangedActionsPerSync(String? value) async {
    if (value == null) return;
    if (int.tryParse(value) == null) {
      return;
    }
    setState(() {
      settings = settings.copyWith(actionsPerSync: value);
      widget.onChanged(settings);
    });
  }

  onChangedSyncInterval(String? value) async {
    if (value == null) return;
    if (int.tryParse(value) == null) {
      return;
    }
    setState(() {
      settings = settings.copyWith(syncInterval: value);
      widget.onChanged(settings);
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
