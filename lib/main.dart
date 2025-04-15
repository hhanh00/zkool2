import 'package:flutter/material.dart';
import 'package:logger/logger.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:zkool/router.dart';
import 'package:zkool/src/rust/api/network.dart';
import 'package:zkool/src/rust/frb_generated.dart';
import 'package:zkool/store.dart';

var logger = Logger();

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();
  final prefs = await SharedPreferences.getInstance();
  final dbName = prefs.getString("database") ?? "zkool";
  await RustLib.init();
  AppStoreBase.instance.dbName = dbName;

  // setLwd(lwd: "https://lwd4.zcash-infra.com:9067");
  setLwd(lwd: "https://zec.rocks");
  AppStoreBase.instance.init();

  runApp(MaterialApp.router(
      routerConfig: router, debugShowCheckedModeBanner: false));
}
