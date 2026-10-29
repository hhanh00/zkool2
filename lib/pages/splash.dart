import 'dart:async';

import 'package:flutter/material.dart';
import 'package:collection/collection.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import 'package:zkool/src/rust/api/coin.dart';
import 'package:zkool/main.dart';
import 'package:zkool/src/rust/api/db.dart';
import 'package:zkool/store.dart';
import 'package:zkool/utils.dart';
import 'package:zkool/widgets/error_display.dart';
import 'package:zkool/router.dart';

class SplashPage extends ConsumerStatefulWidget {
  const SplashPage({super.key});

  @override
  ConsumerState<SplashPage> createState() => SplashPageState();
}

class SplashPageState extends ConsumerState<SplashPage> {
  bool? openDatabaseSuccess;
  int? _restoredAccount;

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
          final account = _restoredAccount;
          if (account != null) {
            GoRouter.of(context).go(accountRoute("/", account));
          } else {
            GoRouter.of(context).go("/accounts");
          }
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
    // A fresh handle: the wallet's network comes from the wallet itself,
    // never from the one that was open before a switch.
    var c = Coin();
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
        // On the first attempt (password is null, i.e. no password was provided),
        // skip the error dialog and go straight to the password prompt.
        // Only show the error dialog if the user already entered a password that was wrong.
        if (password != null && mounted) {
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
    // Restore persisted account ID from the Rust DB
    final savedIdStr = await getProp(key: "selected_account", c: c);
    final savedId = int.tryParse(savedIdStr ?? "") ?? 0;
    ref.read(selectedAccountIdProvider.notifier).set(savedId);
    ref.invalidate(getAccountsProvider);
    final accounts = await ref.read(getAccountsProvider.future);
    final account = accounts.firstWhereOrNull((a) => a.id == savedId);
    _restoredAccount = account?.id;

    final settings = await ref.read(appSettingsProvider.future);
    if (settings.vault && !settings.offline) {
      // initialize the Vault via provider
      await ref.read(vaultProvider.future);
    }
    logger.i("LWD ${settings.lwd}, account ${account?.id}");
    c = c.setLwd(
      serverType: settings.isLightNode ? 0 : 1,
      url: settings.lwd,
    );
    c = c.setTransport(transport: settings.transport);
    c = c.setProxy(proxy: settings.proxy);
    coinContext.set(coin: c);
    final synchronizer = ref.read(synchronizerProvider.notifier);
    synchronizer.autoSync();
    final mempool = ref.read(mempoolProvider.notifier);
    unawaited(Future(mempool.runMempoolListener));
    setState(() => openDatabaseSuccess = true);
  }
}
