import 'package:collection/collection.dart';
import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';
import 'package:zkool/main.dart';
import 'package:zkool/src/rust/api/db.dart';
import 'package:zkool/src/rust/api/network.dart';
import 'package:zkool/src/rust/coin.dart';
import 'package:zkool/store.dart';
import 'package:zkool/utils.dart';

class SplashPage extends StatefulWidget {
  const SplashPage({super.key});

  @override
  State<StatefulWidget> createState() => SplashPageState();
}

class SplashPageState extends State<SplashPage> {
  @override
  Widget build(BuildContext context) {
    return FutureBuilder<bool>(
        future: loadAccounts(context),
        builder: (context, snapshot) {
          Widget body() {
            if (snapshot.hasError) {
              return SingleChildScrollView(
                  child: Center(
                child: Text(
                  snapshot.error.toString(),
                  style: TextStyle(color: Colors.red),
                ),
              ));
            }
            if (snapshot.hasData) {
              if (!appStore.disclaimerAccepted) {
                WidgetsBinding.instance.addPostFrameCallback((_) => GoRouter.of(context).go("/disclaimer"));
              } else {
                final loaded = snapshot.data ?? false;
                WidgetsBinding.instance.addPostFrameCallback((_) async {
                  if (!loaded)
                    GoRouter.of(context).go('/database_manager');
                  else {
                    final account = appStore.selectedAccount;
                    if (account != null) {
                      await selectAccount(account);
                      GoRouter.of(context).go("/account", extra: account);
                    } else
                      GoRouter.of(context).go("/");
                  }
                });
              }
            }
            return Center(child: Image.asset("misc/icon.png", scale: 4.0));
          }

          return Material(child: body());
        });
  }
}

Future<bool> loadAccounts(BuildContext context) async {
  if (!appStore.loaded) {
    String? password;
    while (true) {
      final dbName = appStore.dbName;
      final dbFilepath = await getFullDatabasePath(dbName);
      logger.i('dbFilepath: $dbFilepath');
      appStore.dbFilepath = dbFilepath;
      try {
        await openDatabase(dbFilepath: dbFilepath, password: password);
        break;
      } catch (_) {
        password = await inputPassword(context, title: "Enter Database Password for $dbName", btnCancelText: "Database Manager");
      }
    }
    await appStore.loadAccounts();
    final accountId = await getSelectedAccount();
    final account = accountId != null ? appStore.accounts.firstWhereOrNull((a) => a.id == accountId) : null;
    if (account != null) await selectAccount(account);

    await appStore.loadSettings();
    setLwd(serverType: appStore.isLightNode ? ServerType.lwd : ServerType.zebra, lwd: appStore.lwd);
    setUseTor(useTor: appStore.useTor);
    appStore.autoSync();
    runMempoolListener();
  }

  appStore.loaded = true;
  return true;
}
