import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import 'package:zkool/main.dart';
import 'package:zkool/store.dart';
import 'package:zkool/utils.dart';
import 'package:zkool/widgets/error_display.dart';

class SplashPage extends ConsumerStatefulWidget {
  const SplashPage({super.key});

  @override
  ConsumerState<SplashPage> createState() => SplashPageState();
}

class SplashPageState extends ConsumerState<SplashPage> {
  bool? openDatabaseSuccess;

  @override
  void initState() {
    super.initState();
    runLogListener();
    LifecycleWatcher.instance.init();
    Future(tryOpenDatabase);
  }

  @override
  Widget build(BuildContext context) {
    if (openDatabaseSuccess != null) {
      WidgetsBinding.instance.addPostFrameCallback((_) async {
        if (!openDatabaseSuccess!)
          GoRouter.of(context).go('/database_manager');
        else {
          final selectedAccount = await ref.read(selectedAccountProvider.future);
          if (selectedAccount != null) {
            GoRouter.of(context).go("/account", extra: selectedAccount);
          } else
            GoRouter.of(context).go("/");
        }
      });
    }
    return Center(
      child: Image.asset(
        "misc/icon.png",
        width: 200,
      ),
    );
  }

  void tryOpenDatabase() async {
    String? password;
    var c = coinContext.coin;
    while (true) {
      final settings = await ref.read(appSettingsProvider.future);
      final dbName = settings.dbName;
      final dbFilepath = await getFullDatabasePath(dbName);
      logger.i("dbFilepath: $dbFilepath");
      try {
        c = await c.openDatabase(dbFilepath: dbFilepath, password: password);
        break;
      } catch (e, s) {
        logger.e(e);
        if (mounted) {
          await ErrorDialog.show(
            context,
            error: e,
            stackTrace: s,
          );
        }
        password = await inputPassword(context, title: "Enter Database Password for $dbName", btnCancelText: "Database Manager");
        if (password == null) {
          setState(() => openDatabaseSuccess = false);
          return;
        }
      }
    }
    coinContext.set(coin: c);
    final hasDb = ref.read(hasDbProvider.notifier);
    hasDb.setHasDb();
    ref.invalidate(appSettingsProvider);
    final account = await ref.read(selectedAccountProvider.future);
    if (account != null) c = await c.setAccount(account: account.id);

    final settings = await ref.read(appSettingsProvider.future);
    if (settings.vault && !settings.offline) {
      // initialize the Vault via provider
      await ref.read(vaultProvider.future);
    }
    logger.i("LWD ${settings.lwd}");
    c = c.setLwd(
      serverType: settings.isLightNode ? 0 : 1,
      url: settings.lwd,
    );
    c = await c.setUseTor(useTor: settings.useTor);
    coinContext.set(coin: c);
    final synchronizer = ref.read(synchronizerProvider.notifier);
    synchronizer.autoSync();
    final mempool = ref.read(mempoolProvider.notifier);
    unawaited(Future(mempool.runMempoolListener));
    setState(() => openDatabaseSuccess = true);
  }
}
