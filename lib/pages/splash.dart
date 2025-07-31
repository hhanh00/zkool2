import 'dart:io';

import 'package:collection/collection.dart';
import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';
import 'package:zkool/main.dart';
import 'package:zkool/router.dart';
import 'package:zkool/src/rust/api/account.dart';
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
    return FutureBuilder<List<Account>>(
        future: loadAccounts(),
        initialData: [],
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
                final data = snapshot.data;
                if (data != null) {
                  WidgetsBinding.instance.addPostFrameCallback((_) {
                    final account = appStore.selectedAccount;
                    if (account != null) {
                      selectAccount(account);
                      GoRouter.of(context).go("/account", extra: account);
                    } else
                      GoRouter.of(context).go("/");
                  });
                }
              }
            }
            return Center(child: Image.asset("misc/icon.png", scale: 4.0));
          }

          return Material(child: body());
        });
  }
}

Future<List<Account>> loadAccounts() async {
  if (!appStore.loaded) {
    final dbName = appStore.dbName;
    final dbFilepath = await getFullDatabasePath(dbName);
    logger.i('dbFilepath: $dbFilepath');
    appStore.dbFilepath = dbFilepath;

    String? password;
    if (!File(dbFilepath).existsSync()) {
      if (dbName != appName) {
        // do not encrypt default database
        password = await inputPassword(navigatorKey.currentContext!,
            title: "Enter New Database Password");
      }
      if (password != null && password.isEmpty) password = null;
    }

    while (true) {
      try {
        await openDatabase(dbFilepath: dbFilepath, password: password);
        break;
      } catch (_) {
        password = await inputPassword(
          navigatorKey.currentContext!,
          title: "Enter Database Password for $dbName",
        );
        if (password == null) {
          // switch to default database
          await showException(navigatorKey.currentContext!,
              "No password given. Switching to defaut database.");
          appStore.dbName = appName;
          return await loadAccounts();
        }
      }
    }
    await appStore.loadAccounts();
    final accountId = await getSelectedAccount();
    final account = accountId != null
        ? appStore.accounts.firstWhereOrNull((a) => a.id == accountId)
        : null;
    if (account != null) selectAccount(account);

    await appStore.loadSettings();
    setLwd(
        serverType: appStore.isLightNode ? ServerType.lwd : ServerType.zebra,
        lwd: appStore.lwd);
    appStore.autoSync();
    runMempoolListener();
  }

  appStore.loaded = true;
  return appStore.accounts;
}
