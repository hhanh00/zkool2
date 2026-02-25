import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:logger/logger.dart';
import 'package:path_provider/path_provider.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:showcaseview/showcaseview.dart';
import 'package:toastification/toastification.dart';
import 'package:zkool/router.dart';
import 'package:zkool/src/rust/api/network.dart';
import 'package:zkool/src/rust/api/vote.dart';
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
  final disclaimerAccepted = await prefs.getBool("disclaimer_accepted") ?? false;

  final r = router(disclaimerAccepted, recovery);

  final e = await parseElection(electionJson: '{"start":3155000,"end":3169000,"need_sig":true,"name":"Test Election","questions":[{"title":"Q1. What is your favorite color?","subtitle":"","index":0,"address":"zcv1re3za92mksd4hga0xw6rwxlklkxsqe9nuqqtdws8mu7cynd6gee74863uq4s9aze6q2zywze20y","choices":[{"title":null,"subtitle":null,"answers":["Red","Green","Blue"]}]},{"title":"Q2. Is the earth flat?","subtitle":"","index":1,"address":"zcv1panzgdd6kyygjqtykys6snl9sy59tdnhrpmezdamlt0umxcgs3z4mrndy7eajpkpxerry7tvccv","choices":[{"title":null,"subtitle":null,"answers":["Yes","No"]}]},{"title":"Q3. Do you like pizza?","subtitle":"","index":2,"address":"zcv1yk6u9k8t6087ru4vsjfzepfw9yhhgpnua27r74wmqyqetn35663c62tnfzw46vqqtu2g54jwqt8","choices":[{"title":null,"subtitle":null,"answers":["Yes","No"]}]}]}');
  final c = e.questions[0].choices[0];
  final a = c.answers[0];
  logger.i(a);

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
