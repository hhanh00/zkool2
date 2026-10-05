import 'dart:io';

import 'package:flex_color_scheme/flex_color_scheme.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import 'package:logger/logger.dart';
import 'package:shared_preferences/shared_preferences.dart';

import 'package:toastification/toastification.dart';
import 'package:zkool/error_log_printer.dart';
import 'package:zkool/router.dart';
import 'package:zkool/src/rust/api/network.dart';
import 'package:zkool/src/rust/api/plugin.dart';
import 'package:zkool/src/rust/frb_generated.dart';
import 'package:zkool/store.dart';
import 'package:zkool/utils.dart';
import 'package:zkool/widgets/error_display.dart';

final logger = Logger(filter: ProductionFilter(), printer: ErrorLogPrinter());

const String appName = "zkool";

final appKey = GlobalKey();

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();
  runApp(const BootstrapApp());
}

/// Runs the asynchronous startup work and, on failure, shows a fullscreen
/// error instead of leaving a blank window.
class BootstrapApp extends StatefulWidget {
  const BootstrapApp({super.key});

  @override
  State<BootstrapApp> createState() => _BootstrapAppState();
}

class _BootstrapAppState extends State<BootstrapApp> {
  late final Future<GoRouter> _router = _init();

  Future<GoRouter> _init() async {
    try {
      await RustLib.init();
      final dataDir = await getAppDirectory();
      await initDatadir(directory: dataDir.path);
      initPlugins();
      final prefs = SharedPreferencesAsync();
      final recovery = await prefs.getBool("recovery") ?? false;
      final disclaimerAccepted = await prefs.getBool("disclaimer_accepted") ?? false;
      return router(disclaimerAccepted, recovery);
    } catch (e) {
      stderr.writeln('zkool: startup failed: $e');
      rethrow;
    }
  }

  @override
  Widget build(BuildContext context) {
    return FutureBuilder<GoRouter>(
      future: _router,
      builder: (context, snapshot) {
        if (snapshot.hasError) {
          return MaterialApp(
            debugShowCheckedModeBanner: false,
            home: Scaffold(
              appBar: AppBar(title: const Text(appName)),
              body: ErrorDisplay(
                error: snapshot.error!,
                stackTrace: snapshot.stackTrace,
              ),
            ),
          );
        }
        if (!snapshot.hasData) {
          return const MaterialApp(
            debugShowCheckedModeBanner: false,
            home: Scaffold(body: Center(child: CircularProgressIndicator())),
          );
        }
        return ZkoolApp(router: snapshot.data!);
      },
    );
  }
}

/// The application widget tree, extracted from [main] so that integration
/// tests can pump the exact same tree with a router of their own.
class ZkoolApp extends StatelessWidget {
  final GoRouter router;

  const ZkoolApp({super.key, required this.router});

  @override
  Widget build(BuildContext context) {
    return ProviderScope(
      child: ToastificationConfigProvider(
        config: ToastificationConfig(
          marginBuilder: (c, a) => const EdgeInsets.only(top: 76),
        ),
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
              routerConfig: router,
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
