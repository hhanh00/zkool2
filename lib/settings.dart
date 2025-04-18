import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_form_builder/flutter_form_builder.dart';
import 'package:form_builder_validators/form_builder_validators.dart';
import 'package:gap/gap.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:zkool/router.dart';
import 'package:zkool/src/rust/api/db.dart';
import 'package:zkool/src/rust/api/network.dart';
import 'package:zkool/store.dart';

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
    AppStoreBase.instance.setSyncTimer();
  }

  @override
  Widget build(BuildContext context) {
    final t = Theme.of(context).textTheme;

    return Scaffold(
      appBar: AppBar(
        title: Text("Settings"),
      ),
      body: SingleChildScrollView(
        child: FormBuilder(
          key: formKey,
          child: Padding(
            padding: const EdgeInsets.symmetric(horizontal: 16),
            child: Column(
              children: [
                FormBuilderTextField(
                  name: "database_name",
                  decoration: const InputDecoration(labelText: "Database Name"),
                  initialValue: databaseName,
                  onChanged: onChangedDatabaseName,
                ),
                FormBuilderTextField(
                  name: "lwd",
                  decoration:
                      const InputDecoration(labelText: "Lightwalletd Server"),
                  initialValue: lwd,
                  onChanged: onChangedLWD,
                ),
                FormBuilderTextField(
                  name: "autosync",
                  decoration:
                      const InputDecoration(labelText: "AutoSync Interval"),
                  initialValue: syncInterval,
                  onChanged: onChangedSyncInterval,
                  validator: FormBuilderValidators.integer(),
                  keyboardType: TextInputType.number,
                  inputFormatters: [FilteringTextInputFormatter.digitsOnly],
                ),
                Gap(16),
                Text(AppStoreBase.instance.dbFilepath, style: t.bodySmall),
              ],
            ),
          ),
        ),
      ),
    );
  }

  void onChangedDatabaseName(String? value) async {
    if (value == null) return;
    final prefs = await SharedPreferences.getInstance();
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
