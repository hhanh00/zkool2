import 'package:flutter/material.dart';
import 'package:logger/logger.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:showcaseview/showcaseview.dart';
import 'package:toastification/toastification.dart';
import 'package:zkool/router.dart';
import 'package:zkool/src/rust/frb_generated.dart';
import 'package:zkool/store.dart';

var logger = Logger();

const String appName = "zkool";

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();

  final prefs = SharedPreferencesAsync();
  final dbName = await prefs.getString("database") ?? appName;
  await RustLib.init();
  AppStoreBase.instance.dbName = dbName;

  AppStoreBase.instance.init();

  runApp(ToastificationWrapper(
      child: ShowCaseWidget(
          globalTooltipActions: [
        const TooltipActionButton(
            type: TooltipDefaultActionType.skip,
            textStyle: TextStyle(color: Colors.red),
            backgroundColor: Colors.transparent),
        const TooltipActionButton(
            type: TooltipDefaultActionType.next,
            backgroundColor: Colors.transparent),
      ],
          builder: (context) => MaterialApp.router(
              routerConfig: router, debugShowCheckedModeBanner: false))));
}
