import 'dart:typed_data';

import 'package:flutter/material.dart';
import 'package:flutter_mobx/flutter_mobx.dart';
import 'package:go_router/go_router.dart';
import 'package:zkool/main.dart';
import 'package:zkool/pages/account.dart';
import 'package:zkool/pages/accounts.dart';
import 'package:zkool/pages/db.dart';
import 'package:zkool/pages/disclaimer.dart';
import 'package:zkool/pages/dkg.dart';
import 'package:zkool/pages/frost.dart';
import 'package:zkool/pages/log.dart';
import 'package:zkool/pages/market.dart';
import 'package:zkool/pages/new_account.dart';
import 'package:zkool/pages/receive.dart';
import 'package:zkool/pages/send.dart';
import 'package:zkool/pages/splash.dart';
import 'package:zkool/pages/tx.dart';
import 'package:zkool/pages/tx_view.dart';
import 'package:zkool/settings.dart';
import 'package:zkool/src/rust/api/account.dart';
import 'package:zkool/src/rust/api/pay.dart';
import 'package:zkool/src/rust/pay.dart';
import 'package:zkool/store.dart';
import 'package:zkool/widgets/scanner.dart';

final navigatorKey = GlobalKey<NavigatorState>();
final RouteObserver<ModalRoute<void>> routeObserver =
    RouteObserver<ModalRoute<void>>();

final router = GoRouter(
  initialLocation: '/splash',
  observers: [routeObserver],
  navigatorKey: navigatorKey,
  routes: [
    GoRoute(
        path: '/',
        builder: (context, state) => pinLock(AccountListPage()),
        routes: [
          GoRoute(
            path: 'account',
            builder: (context, state) => pinLock(AccountViewPage()),
          ),
        ]),
    GoRoute(
      path: '/account/edit',
      builder: (context, state) =>
          pinLock(AccountEditPage(state.extra as List<Account>)),
    ),
    GoRoute(
      path: '/account/new',
      builder: (context, state) => pinLock(NewAccountPage()),
    ),
    GoRoute(
      path: '/viewing_keys',
      builder: (context, state) => pinLock(ViewingKeysPage(state.extra as int)),
    ),
    GoRoute(
      path: '/receive',
      builder: (context, state) => pinLock(ReceivePage()),
    ),
    GoRoute(
      path: '/transparent_addresses',
      builder: (context, state) => pinLock(TransparentAddressesPage(
          txCounts: state.extra as List<TAddressTxCount>)),
    ),
    GoRoute(
      path: '/send',
      builder: (context, state) => pinLock(SendPage()),
    ),
    GoRoute(
      path: '/send2',
      builder: (context, state) =>
          pinLock(Send2Page(state.extra as List<Recipient>)),
    ),
    GoRoute(
        path: '/tx',
        builder: (context, state) =>
            pinLock(TxPage(state.extra as PcztPackage))),
    GoRoute(
        path: '/tx_view',
        builder: (context, state) => pinLock(TxView(state.extra as int))),
    GoRoute(path: '/log', builder: (context, state) => pinLock(LogviewPage())),
    GoRoute(
        path: '/scanner',
        builder: (context, state) => pinLock(
            ScannerPage(validator: state.extra as String? Function(String?)))),
    GoRoute(
        path: '/qr',
        builder: (context, state) {
          final args = state.extra as Map<String, dynamic>;
          return pinLock(QRPage(text: args["text"], title: args["title"]));
        }),
    GoRoute(path: '/splash', builder: (context, state) => SplashPage()),
    GoRoute(path: '/market', builder: (context, state) => MarketPrice()),
    GoRoute(path: '/mempool', builder: (context, state) => MempoolPage()),
    GoRoute(
        path: '/mempool_view',
        builder: (context, state) =>
            MempoolTxViewPage(state.extra as Uint8List)),
    GoRoute(path: '/dkg1', builder: (context, state) => pinLock(DKGPage1())),
    GoRoute(path: '/dkg2', builder: (context, state) => pinLock(DKGPage2())),
    GoRoute(path: '/dkg3', builder: (context, state) => pinLock(DKGPage3())),
    GoRoute(
        path: '/frost1',
        builder: (context, state) =>
            pinLock(FrostPage1(state.extra as PcztPackage))),
    GoRoute(
        path: '/frost2', builder: (context, state) => pinLock(FrostPage2())),
    GoRoute(
        path: '/settings',
        builder: (context, state) => SettingsPage()), // Authenticated by caller
    GoRoute(
      path: '/database_manager',
      builder: (context, state) => DatabaseManagerPage()),
    GoRoute(path: '/disclaimer', builder: (context, state) => DisclaimerPage()),
  ],
);

Widget pinLock(Widget child) => Observer(
    builder: (context) =>
        (appStore.unlocked == null && appStore.needPin) ? PinLock() : child);
