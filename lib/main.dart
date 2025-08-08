import 'package:flutter/material.dart';
import 'package:logger/logger.dart';
import 'package:mobx/mobx.dart';
import 'package:path_provider/path_provider.dart';
import 'package:showcaseview/showcaseview.dart';
import 'package:toastification/toastification.dart';
import 'package:zkool/router.dart';
import 'package:zkool/src/rust/api/network.dart';
import 'package:zkool/src/rust/frb_generated.dart';
import 'package:zkool/store.dart';
import 'package:zkool/utils.dart';

var logger = Logger(filter: ProductionFilter());

const String appName = "zkool";

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();

  await RustLib.init();
  await appStore.init();
  await appStore.loadAppSettings();
  final torDir = await getApplicationDocumentsDirectory();
  if (appStore.useTor)
    await initTor(directory: torDir.path);

  final appWatcher = LifecycleWatcher();
  appWatcher.init();

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

class LifecycleWatcher with WidgetsBindingObserver {
  void init() {
    WidgetsBinding.instance.addObserver(this);
  }

  @override
  void didChangeAppLifecycleState(AppLifecycleState state) {
    if (state == AppLifecycleState.resumed) {
      cancelMempoolListener();
      if (appStore.unlocked != null &&  DateTime.now().difference(appStore.unlocked!).inSeconds >= 5) {
        runInAction(() => appStore.unlocked = null);
      }
    }
  }
}

class PinLock extends StatefulWidget {
  const PinLock({
    super.key,
  });

  @override
  State<StatefulWidget> createState() => PinLockState();
}

class PinLockState extends State<PinLock> {
  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) => onUnlock());
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
        appBar: AppBar(title: Text("Locked")),
        body: Center(
            child: IconButton(
          onPressed: onUnlock,
          icon: Icon(
            Icons.lock,
            size: 200,
          ),
        )));
  }

  void onUnlock() async {
    final authenticated = await authenticate(reason: "Unlock the App");
    if (authenticated) {
      runInAction(() => appStore.unlocked = DateTime.now());
    }
  }
}
