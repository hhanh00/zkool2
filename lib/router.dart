import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';
import 'package:zkool/pages/account.dart';
import 'package:zkool/pages/accounts.dart';
import 'package:zkool/pages/log.dart';
import 'package:zkool/pages/new_account.dart';
import 'package:zkool/pages/receive.dart';
import 'package:zkool/pages/send.dart';
import 'package:zkool/pages/tx.dart';
import 'package:zkool/src/rust/api/account.dart';
import 'package:zkool/src/rust/pay.dart';
import 'package:zkool/widgets/scanner.dart';

final RouteObserver<ModalRoute<void>> routeObserver = RouteObserver<ModalRoute<void>>();

final router = GoRouter(
  initialLocation: "/",
  observers: [routeObserver],
  routes: [
    GoRoute(
      path: '/',
      builder: (context, state) => AccountListPage(),
    ),
    GoRoute(
      path: '/account',
      builder: (context, state) => AccountViewPage(state.extra as Account),
    ),
    GoRoute(
      path: '/account/edit',
      builder: (context, state) => AccountEditPage(state.extra as Account),
    ),
    GoRoute(
      path: '/account/new',
      builder: (context, state) => NewAccountPage(),
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
        builder: (context, state) => TxPage(state.extra as TxPlan)),
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
  ],
);
