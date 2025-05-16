import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';
import 'package:zkool/pages/account.dart';
import 'package:zkool/pages/accounts.dart';
import 'package:zkool/pages/disclaimer.dart';
import 'package:zkool/pages/dkg.dart';
import 'package:zkool/pages/frost.dart';
import 'package:zkool/pages/log.dart';
import 'package:zkool/pages/new_account.dart';
import 'package:zkool/pages/receive.dart';
import 'package:zkool/pages/send.dart';
import 'package:zkool/pages/tx.dart';
import 'package:zkool/pages/tx_view.dart';
import 'package:zkool/settings.dart';
import 'package:zkool/src/rust/api/account.dart';
import 'package:zkool/src/rust/api/frost.dart';
import 'package:zkool/src/rust/api/pay.dart';
import 'package:zkool/src/rust/pay.dart';
import 'package:zkool/widgets/scanner.dart';

final navigatorKey = GlobalKey<NavigatorState>();
final RouteObserver<ModalRoute<void>> routeObserver = RouteObserver<ModalRoute<void>>();

final router = GoRouter(
  initialLocation: '/',
  observers: [routeObserver],
  navigatorKey: navigatorKey,
  routes: [
    GoRoute(
      path: '/',
      builder: (context, state) => AccountListPage(),
      routes: [
        GoRoute(
          path: 'account',
          builder: (context, state) => AccountViewPage(),
        ),
      ]
    ),
    GoRoute(
      path: '/account/edit',
      builder: (context, state) => AccountEditPage(state.extra as List<Account>),
    ),
    GoRoute(
      path: '/account/new',
      builder: (context, state) => NewAccountPage(),
    ),
    GoRoute(
      path: '/viewing_keys',
      builder: (context, state) => ViewingKeysPage(state.extra as int),
    ),
    GoRoute(
      path: '/receive',
      builder: (context, state) => ReceivePage(),
    ),
    GoRoute(
      path: '/send',
      builder: (context, state) => SendPage(),
    ),
    GoRoute(
      path: '/send2',
      builder: (context, state) => Send2Page(state.extra as List<Recipient>),
    ),
    GoRoute(
        path: '/tx',
        builder: (context, state) => TxPage(state.extra as PcztPackage)),
    GoRoute(
        path: '/tx_view',
        builder: (context, state) => TxView(state.extra as int)),
    GoRoute(path: '/log', builder: (context, state) => LogviewPage()),
    GoRoute(
        path: '/scanner',
        builder: (context, state) =>
            ScannerPage(validator: state.extra as String? Function(String?))),
    GoRoute(
        path: '/qr',
        builder: (context, state) {
          final args = state.extra as Map<String, dynamic>;
          return QRPage(text: args["text"], title: args["title"]);
        }),
    GoRoute(path: '/dkg1', builder: (context, state) => DKGPage1()),
    GoRoute(path: '/dkg2', builder: (context, state) => DKGPage2(state.extra as FrostPackage)),
    GoRoute(path: '/dkg3', builder: (context, state) => DKGPage3()),
    GoRoute(path: '/frost1', builder: (context, state) => FrostPage1(state.extra as PcztPackage)),
    GoRoute(path: '/frost2', builder: (context, state) => FrostPage2()),
    GoRoute(path: '/settings', builder: (context, state) => SettingsPage()),
    GoRoute(path: '/disclaimer', builder: (context, state) => DisclaimerPage()),
  ],
);
