import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_form_builder/flutter_form_builder.dart';
import 'package:form_builder_validators/form_builder_validators.dart';
import 'package:gap/gap.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:showcaseview/showcaseview.dart';
import 'package:zkool/router.dart';
import 'package:zkool/src/rust/api/db.dart';
import 'package:zkool/src/rust/api/network.dart';
import 'package:zkool/store.dart';
import 'package:zkool/utils.dart';

final databaseID = GlobalKey();
final lwdID = GlobalKey();
final actionsID = GlobalKey();
final autosyncID = GlobalKey();

class SettingsPage extends StatefulWidget {
  const SettingsPage({super.key});

  @override
  State<SettingsPage> createState() => SettingsPageState();
}

class SettingsPageState extends State<SettingsPage> with RouteAware {
  final formKey = GlobalKey<FormBuilderState>();
  String databaseName = AppStoreBase.instance.dbName;
  String lwd = AppStoreBase.instance.lwd;
  String syncInterval = AppStoreBase.instance.syncInterval;
  String actionsPerSync = AppStoreBase.instance.actionsPerSync;

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
    AppStoreBase.instance.autoSync();
  }

  void tutorial() async {
    tutorialHelper(context, "tutSettings0", [databaseID, lwdID, autosyncID]);
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
            onPressed: resetTutorial, icon: Icon(Icons.school))
        ],
      ),
      body: SingleChildScrollView(
        child: FormBuilder(
          key: formKey,
          child: Padding(
            padding: const EdgeInsets.symmetric(horizontal: 16),
            child: Column(
              children: [
                Showcase(key: databaseID, description: "Change the database file. This requires a RESTART after", child:
                FormBuilderTextField(
                  name: "database_name",
                  decoration: const InputDecoration(labelText: "Database Name"),
                  initialValue: databaseName,
                  onChanged: onChangedDatabaseName,
                )),
                Showcase(key: lwdID, description: "Lightwalletd server to connect to", child:
                FormBuilderTextField(
                  name: "lwd",
                  decoration:
                      const InputDecoration(labelText: "Lightwalletd Server"),
                  initialValue: lwd,
                  onChanged: onChangedLWD,
                )),
                Showcase(key: actionsID, description: "Number actions per synchronization chunk", child:
                FormBuilderTextField(
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
                Showcase(key: autosyncID, description: "AutoSync interval in blocks. Accounts that are behind by more than this value will start synchronization.", child:
                FormBuilderTextField(
                  name: "autosync",
                  decoration:
                      const InputDecoration(labelText: "AutoSync Interval"),
                  initialValue: syncInterval,
                  onChanged: onChangedSyncInterval,
                  validator: FormBuilderValidators.integer(),
                  keyboardType: TextInputType.number,
                  inputFormatters: [FilteringTextInputFormatter.digitsOnly],
                )),
                Gap(16),
                SelectableText(AppStoreBase.instance.dbFilepath, style: t.bodySmall),
              ],
            ),
          ),
        ),
      ),
    );
  }

  void onChangedDatabaseName(String? value) async {
    if (value == null) return;
    final prefs = SharedPreferencesAsync();
    await prefs.setString("database", value);
    AppStoreBase.instance.dbName = value;
    setState(() {
      databaseName = value;
    });
  }

  void onChangedLWD(String? value) async {
    if (value == null) return;
    await putProp(key: "lwd", value: value);
    AppStoreBase.instance.lwd = value;
    setLwd(lwd: value);
    setState(() {
      lwd = value;
    });
  }

  onChangedActionsPerSync(String? value) async {
    if (value == null) return;
    if (int.tryParse(value) == null) {
      return;
    }
    await putProp(key: "actions_per_sync", value: value);
    AppStoreBase.instance.actionsPerSync = value;
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
    AppStoreBase.instance.syncInterval = value;
    setState(() {
      syncInterval = value;
    });
  }
}
