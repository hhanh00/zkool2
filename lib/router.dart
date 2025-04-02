import 'package:go_router/go_router.dart';
import 'package:zkool/pages/account.dart';
import 'package:zkool/pages/accounts.dart';
import 'package:zkool/src/rust/api/account.dart';

final router = GoRouter(
  routes: [
    GoRoute(
      path: '/',
      builder: (context, state) => AccountListPage(),
    ),
    GoRoute(
      path: '/account/edit',
      builder: (context, state) => AccountEditPage(state.extra as Account),
    ),
  ],
);
