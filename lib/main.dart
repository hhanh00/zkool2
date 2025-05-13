import 'package:flutter/material.dart';
import 'package:logger/logger.dart';
import 'package:showcaseview/showcaseview.dart';
import 'package:toastification/toastification.dart';
import 'package:zkool/router.dart';
import 'package:zkool/src/rust/api/frost.dart';
import 'package:zkool/src/rust/frb_generated.dart';
import 'package:zkool/store.dart';

var logger = Logger();

const String appName = "zkool";

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();

  await RustLib.init();
  await AppStoreBase.instance.init();
  testFrost();

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
              routerConfig: router,
              themeMode: ThemeMode.system,
              theme: ThemeData.light(),
              darkTheme: ThemeData.dark(),
              debugShowCheckedModeBanner: false))));
}
