import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:logger/logger.dart';
import 'package:path_provider/path_provider.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:showcaseview/showcaseview.dart';
import 'package:toastification/toastification.dart';
import 'package:zkool/router.dart';
import 'package:zkool/src/rust/api/network.dart';
import 'package:zkool/src/rust/frb_generated.dart';
import 'package:zkool/utils.dart';

final logger = Logger(filter: ProductionFilter());

const String appName = "zkool";

final appKey = GlobalKey();

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();

  await RustLib.init();
  final dataDir = await getApplicationDocumentsDirectory();
  await initDatadir(directory: dataDir.path);
  final prefs = SharedPreferencesAsync();
  final recovery = await prefs.getBool("recovery") ?? false;
  final r = router(recovery);

  // Future<AppSettings> initApp(WidgetRef ref, AppSettings s) async {
  //   appWatcher = LifecycleWatcher(ref);
  //   return s;
  // }
  runApp(
    ProviderScope(
      child: ToastificationWrapper(
        child: ShowCaseWidget(
          globalTooltipActions: [
            const TooltipActionButton(type: TooltipDefaultActionType.skip, textStyle: TextStyle(color: Colors.red), backgroundColor: Colors.transparent),
            const TooltipActionButton(type: TooltipDefaultActionType.next, backgroundColor: Colors.transparent),
          ],
          builder: (context) {
            return MaterialApp.router(
              key: appKey,
              routerConfig: r,
              themeMode: ThemeMode.system,
              theme: ThemeData.light(),
              darkTheme: ThemeData.dark(),
              debugShowCheckedModeBanner: false,
            );
          },
        ),
      ),
    ),
  );
}

class PinLock extends ConsumerStatefulWidget {
  const PinLock({
    super.key,
  });

  @override
  ConsumerState<PinLock> createState() => PinLockState();
}

class PinLockState extends ConsumerState<PinLock> {
  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: Text("Locked")),
      body: Center(
        child: InkWell(
          onTap: () => onUnlock(ref),
          child: Image.asset("misc/icon.png", width: 200),
        ),
      ),
    );
  }
}
