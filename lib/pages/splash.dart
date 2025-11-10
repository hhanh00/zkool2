import 'package:collection/collection.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import 'package:zkool/main.dart';
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
    final accountsAV = ref.watch(getAccountsProvider);
    switch (accountsAV) {
      case AsyncError(:final error):
        return SingleChildScrollView(
          child: Center(
            child: Text(
              error.toString(),
              style: TextStyle(color: Colors.red),
            ),
          ),
        );
      case AsyncLoading():
        break;
      case AsyncValue(:final hasValue):
        final settings = ref.watch(appSettingsProvider);
        if (!settings.disclaimerAccepted) {
          WidgetsBinding.instance.addPostFrameCallback((_) => GoRouter.of(context).go("/disclaimer"));
        } else {
          WidgetsBinding.instance.addPostFrameCallback((_) async {
            if (!hasValue)
              GoRouter.of(context).go('/database_manager');
            else {
              final selectedAccount = ref.read(selectedAccountProvider);
              if (selectedAccount != null) {
                GoRouter.of(context).go("/account", extra: account);
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
    String? password;
    while (true) {
      final settings = ref.read(appSettingsProvider);
      final dbName = settings.dbName;
      final dbFilepath = await getFullDatabasePath(dbName);
      logger.i('dbFilepath: $dbFilepath');
      try {
        await openDatabase(dbFilepath: dbFilepath, password: password);
        break;
      } catch (e) {
        logger.e(e);
        password = await inputPassword(context, title: "Enter Database Password for $dbName", btnCancelText: "Database Manager");
        if (password == null) setState(() => openDatabaseSuccess = false);
      }
    }
    // Read selected account from sharedPrefs
    final selectedAccountId = null;
    final accounts = await ref.read(getAccountsProvider.future);
    final account = selectedAccountId != null ? accounts.firstWhereOrNull((a) => a.id == selectedAccountId) : null;
    if (account != null) {
      final selectedAccount = ref.read(selectedAccountProvider.notifier);
      selectedAccount.selectAccount(account);
    }

    final settings = ref.read(appSettingsProvider);
    setLwd(
      serverType: settings.isLightNode ? ServerType.lwd : ServerType.zebra,
      lwd: settings.lwd,
    );
    setUseTor(useTor: settings.useTor);
    final synchronizer = ref.read(synchronizerProvider.notifier);
    synchronizer.autoSync(ref);
    // TODO runMempoolListener();
  }
}
