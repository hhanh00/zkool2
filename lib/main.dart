import 'package:flutter/material.dart';
import 'package:logger/logger.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:toastification/toastification.dart';
import 'package:zkool/router.dart';
import 'package:zkool/src/rust/frb_generated.dart';
import 'package:zkool/store.dart';

var logger = Logger();

const String appName = "zkool";

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();
  final prefs = await SharedPreferences.getInstance();
  final dbName = prefs.getString("database") ?? appName;
  await RustLib.init();
  AppStoreBase.instance.dbName = dbName;

  AppStoreBase.instance.init();

  runApp(ToastificationWrapper(child: MaterialApp.router(
      routerConfig: router, debugShowCheckedModeBanner: false)));
}
