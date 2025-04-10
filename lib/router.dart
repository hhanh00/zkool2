import 'package:go_router/go_router.dart';
import 'package:zkool/pages/account.dart';
import 'package:zkool/pages/accounts.dart';
import 'package:zkool/pages/new_account.dart';
import 'package:zkool/pages/send.dart';
import 'package:zkool/pages/tx.dart';
import 'package:zkool/src/rust/api/account.dart';
import 'package:zkool/src/rust/pay.dart';

final router = GoRouter(
  initialLocation: "/",
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
      path: '/send',
      builder: (context, state) => SendPage(),
    ),
    GoRoute(
        path: '/tx',
        builder: (context, state) => TxPage(state.extra as TxPlan)),
  ],
);
