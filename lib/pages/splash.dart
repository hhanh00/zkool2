import 'package:collection/collection.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import 'package:zkool/main.dart';
import 'package:zkool/src/rust/api/account.dart';
import 'package:zkool/src/rust/api/db.dart';
import 'package:zkool/src/rust/api/network.dart';
import 'package:zkool/src/rust/coin.dart';
import 'package:zkool/store.dart';
import 'package:zkool/utils.dart';

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
    Future(tryOpenDatabase);
  }

  @override
  Widget build(BuildContext context) {
    if (openDatabaseSuccess != null) {
      final settings = ref.watch(appSettingsProvider).requireValue;
      if (!settings.disclaimerAccepted) {
        WidgetsBinding.instance.addPostFrameCallback((_) => GoRouter.of(context).go("/disclaimer"));
      } else {
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
    }
    return Center(
      child: Image.asset(
        "misc/icon.png",
        width: 200,
      ),
    );
  }

  void tryOpenDatabase() async {
    logger.i("tryOpenDatabase");
    String? password;
    while (true) {
      final settings = ref.read(appSettingsProvider).requireValue;
      final dbName = settings.dbName;
      final dbFilepath = await getFullDatabasePath(dbName);
      logger.i('dbFilepath: $dbFilepath');
      try {
        await openDatabase(dbFilepath: dbFilepath, password: password);
        break;
      } catch (e) {
        logger.e(e);
        password = await inputPassword(context, title: "Enter Database Password for $dbName", btnCancelText: "Database Manager");
        if (password == null) {
          setState(() => openDatabaseSuccess = false);
          return;
        }
      }
    }
    final hasDb = ref.read(hasDbProvider.notifier);
    hasDb.setHasDb();
    final account = await ref.read(selectedAccountProvider.future);
    if (account != null) await setAccount(account: account.id);

    final settings = ref.read(appSettingsProvider).requireValue;
    setLwd(
      serverType: settings.isLightNode ? ServerType.lwd : ServerType.zebra,
      lwd: settings.lwd,
    );
    setUseTor(useTor: settings.useTor);
    // final synchronizer = ref.read(synchronizerProvider.notifier);
    // synchronizer.autoSync(ref);
    // TODO runMempoolListener();
    setState(() => openDatabaseSuccess = true);
  }
}
