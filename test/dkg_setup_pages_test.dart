import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:go_router/go_router.dart';
import 'package:zkool/pages/dkg.dart';
import 'package:zkool/router.dart' show navigatorKey;
import 'package:zkool/src/rust/api/coin.dart';
import 'package:zkool/src/rust/frb_generated.dart';
import 'package:zkool/store.dart';

// The first two DKG pages run their setup in a detached future and show
// Cancel and the back button while it is pending. A setup result that lands
// after the user left used to update a disposed page and navigate from a
// dead context (#1605, V12 F-89). It is now dropped.

class _FakeCoin implements Coin {
  @override
  dynamic noSuchMethod(Invocation invocation) => super.noSuchMethod(invocation);
}

class _Api implements RustLibApi {
  /// The setup call the test holds: page 1's DKG-in-progress check, page 2's
  /// address list. The test completes it after leaving the page.
  Completer<bool> dkgParams = Completer<bool>();
  Completer<List<String>> addresses = Completer<List<String>>();

  @override
  Coin crateApiCoinCoinNew({int? defaultCoin}) => _FakeCoin();

  @override
  Future<bool> crateApiFrostHasDkgParams({required Coin c}) => dkgParams.future;

  @override
  Future<void> crateApiFrostInitDkg({required Coin c}) async {}

  @override
  Future<List<String>> crateApiFrostGetDkgAddresses({required Coin c}) => addresses.future;

  @override
  Future<bool> crateApiFrostHasDkgAddresses({required Coin c}) async => true;

  @override
  dynamic noSuchMethod(Invocation invocation) => super.noSuchMethod(invocation);
}

class _Unlocked extends Lifecycle {
  @override
  Future<bool> build() async => false;
}

void main() {
  late _Api api;
  setUpAll(() {
    api = _Api();
    RustLib.initMock(api: api);
  });
  setUp(() {
    api.dkgParams = Completer<bool>();
    api.addresses = Completer<List<String>>();
  });
  tearDownAll(RustLib.dispose);

  /// Opens [page] at [path]; the pages it would redirect to are stubs.
  Future<GoRouter> open(WidgetTester tester, String path, Widget page) async {
    final router = GoRouter(navigatorKey: navigatorKey, initialLocation: path, routes: [
      GoRoute(path: '/', builder: (context, state) => const Scaffold(body: Text('Home'))),
      GoRoute(path: path, builder: (context, state) => page),
      GoRoute(path: '/dkg2', builder: (context, state) => const Scaffold(body: Text('Page 2'))),
      GoRoute(path: '/dkg3', builder: (context, state) => const Scaffold(body: Text('Page 3'))),
    ]);
    addTearDown(router.dispose);
    await tester.pumpWidget(ProviderScope(
      overrides: [
        lifecycleProvider.overrideWith(_Unlocked.new),
        getAccountsProvider.overrideWith((ref) async => []),
      ],
      child: MaterialApp.router(routerConfig: router),
    ));
    await tester.pumpAndSettle();
    return router;
  }

  testWidgets('a DKG-in-progress answer after leaving page 1 is dropped', (tester) async {
    final router = await open(tester, '/dkg1', const DKGPage1());
    expect(find.text('Distributed Key Generation'), findsOneWidget);

    router.go('/');
    await tester.pumpAndSettle();
    expect(find.text('Home'), findsOneWidget);

    // The answer would have redirected to page 2 and updated the page.
    api.dkgParams.complete(true);
    await tester.pumpAndSettle();
    expect(find.text('Home'), findsOneWidget, reason: 'no navigation from the page the user left');
    expect(find.text('Page 2'), findsNothing);
    expect(tester.takeException(), isNull);
  });

  testWidgets('an address list after leaving page 2 is dropped', (tester) async {
    final router = await open(tester, '/dkg2a', const DKGPage2());
    expect(find.text('DKG Addresses'), findsOneWidget);

    router.go('/');
    await tester.pumpAndSettle();
    expect(find.text('Home'), findsOneWidget);

    // The addresses would have updated the page and redirected to page 3.
    api.addresses.complete(['uregtest1a', 'uregtest1b']);
    await tester.pumpAndSettle();
    expect(find.text('Home'), findsOneWidget, reason: 'no navigation from the page the user left');
    expect(find.text('Page 3'), findsNothing);
    expect(tester.takeException(), isNull);
  });
}
