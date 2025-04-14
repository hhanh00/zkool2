import 'package:flutter/material.dart';
import 'package:flutter_form_builder/flutter_form_builder.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:zkool/store.dart';

class SettingsPage extends StatefulWidget {
  const SettingsPage({super.key});

  @override
  State<SettingsPage> createState() => SettingsPageState();
}

class SettingsPageState extends State<SettingsPage> {
  final formKey = GlobalKey<FormBuilderState>();
  String databaseName = AppStoreBase.instance.dbName;

  @override
  Widget build(BuildContext context) {
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
              ],
            ),
          ),
        ),
      ),
    );
  }

  void onChangedDatabaseName(String? value) async {
    final prefs = await SharedPreferences.getInstance();
    await prefs.setString("database", value!);
    setState(() {
      databaseName = value;
      AppStoreBase.instance.dbName = value;
    });
  }
}
