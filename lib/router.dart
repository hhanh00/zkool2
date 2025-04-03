import 'package:go_router/go_router.dart';
import 'package:zkool/pages/account.dart';
import 'package:zkool/pages/accounts.dart';
import 'package:zkool/pages/new_account.dart';
import 'package:zkool/src/rust/api/account.dart';

final router = GoRouter(
  initialLocation: "/",
  routes: [
    GoRoute(
      path: '/',
      builder: (context, state) => AccountListPage(coin: 0),
    ),
    GoRoute(
      path: '/account/edit',
      builder: (context, state) => AccountEditPage(state.extra as Account),
    ),
    GoRoute(
      path: '/account/new',
      builder: (context, state) => NewAccountPage(),
    ),
  ],
);
