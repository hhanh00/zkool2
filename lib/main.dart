import 'package:flutter/material.dart';
import 'package:logger/logger.dart';
import 'package:showcaseview/showcaseview.dart';
import 'package:toastification/toastification.dart';
import 'package:zkool/router.dart';
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
  DateTime? unlocked;

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
    final needPin = appStore.pinLock;
    if (needPin && (unlocked == null ||
        DateTime.now().difference(unlocked!).inSeconds >= 5)) {
      return MaterialApp(
          home: PinLock(
              onUnlock: () => setState(() => unlocked = DateTime.now())));
    }
    return widget.child;
  }

  @override
  void didChangeAppLifecycleState(AppLifecycleState state) {
    if (state == AppLifecycleState.resumed) {
      cancelMempoolListener();
      setState(() {});
    }
  }
}

class PinLock extends StatefulWidget {
  final void Function() onUnlock;

  const PinLock({
    super.key,
    required this.onUnlock,
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
          icon: Icon(Icons.lock, size: 200,),
        )));
  }

  void onUnlock() async {
    final authenticated = await authenticate(reason: "Unlock the App");
    if (authenticated) widget.onUnlock();
  }
}
