import 'package:flutter/material.dart';
import 'package:logger/logger.dart';
import 'package:showcaseview/showcaseview.dart';
import 'package:toastification/toastification.dart';
import 'package:zkool/router.dart';
import 'package:zkool/src/rust/frb_generated.dart';
import 'package:zkool/store.dart';

var logger = Logger();

const String appName = "zkool";

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();

  await RustLib.init();
  await AppStoreBase.instance.init();

  runApp(LifecycleWatcher(
      child: ToastificationWrapper(
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
                  debugShowCheckedModeBanner: false)))));
}

class LifecycleWatcher extends StatefulWidget {
  final Widget child;
  const LifecycleWatcher({super.key, required this.child});

  @override
  State<LifecycleWatcher> createState() => LifecycleWatcherState();
}

class LifecycleWatcherState extends State<LifecycleWatcher>
    with WidgetsBindingObserver {
  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addObserver(this);
  }

  @override
  void dispose() {
    WidgetsBinding.instance.removeObserver(this);
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return widget.child;
  }

  @override
  void didChangeAppLifecycleState(AppLifecycleState state) {
    if (state == AppLifecycleState.paused) {
      logger.i("App paused");
    } else if (state == AppLifecycleState.inactive) {
      logger.i("App inactive");
    } else if (state == AppLifecycleState.detached) {
      logger.i("App detached");
    } else if (state == AppLifecycleState.resumed) {
      logger.i("App resumed");
      cancelMempoolListener();
    }
  }
}
