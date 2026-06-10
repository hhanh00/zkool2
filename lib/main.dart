import 'package:flex_color_scheme/flex_color_scheme.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:logger/logger.dart';
import 'package:path_provider/path_provider.dart';
import 'package:shared_preferences/shared_preferences.dart';

import 'package:toastification/toastification.dart';
import 'package:zkool/router.dart';
import 'package:zkool/src/rust/api/network.dart';
import 'package:zkool/src/rust/frb_generated.dart';
import 'package:zkool/store.dart';
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

  runApp(
    ProviderScope(
      child: ToastificationWrapper(
        child: Consumer(builder: (context, ref, _) {
          final settings = ref.watch(appSettingsProvider).value;
          final scheme = settings?.let((s) {
                try {
                  return FlexScheme.values.byName(s.paletteName);
                } catch (_) {
                  return FlexScheme.blue;
                }
              }) ??
              FlexScheme.blue;
          final theme = FlexThemeData.light(scheme: scheme).copyWith(useMaterial3: true);
          final darkTheme = FlexThemeData.dark(scheme: scheme).copyWith(useMaterial3: true);
          return MaterialApp.router(
            key: appKey,
            routerConfig: r,
            builder: (context, child) => SafeArea(
              top: false,
              left: false,
              right: false,
              child: child!,
            ),
            themeMode: settings?.darkMode == true ? ThemeMode.dark : ThemeMode.light,
            theme: theme,
            darkTheme: darkTheme,
            debugShowCheckedModeBanner: false,
          );
        }),
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
