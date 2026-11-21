import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:go_router/go_router.dart';
import 'package:rive/rive.dart' show RiveFile;
import 'package:zkool/pages/account.dart';
import 'package:zkool/pages/accounts.dart';
import 'package:zkool/router.dart' show AccountRouteState, accountRoute, navigatorKey;
import 'package:zkool/src/rust/api/account.dart';
import 'package:zkool/src/rust/api/coin.dart';
import 'package:zkool/src/rust/frb_generated.dart';
import 'package:zkool/store.dart';

// The account list opened from an account page's menu is pushed over that
// page. When the account under it is deleted, the list becomes the root
// page, as at startup with no accounts, instead of staying stacked with a
// back button to a dead page (#1681). The page under the list is hidden,
// so it does not see the deletion; the list handles it.

class _FakeCoin implements Coin {
  @override
  dynamic noSuchMethod(Invocation invocation) => super.noSuchMethod(invocation);
}

class _Api implements RustLibApi {
  final deleted = <int>[];

  @override
  Future<void> crateApiAccountDeleteAccount({required int account, required Coin c}) async {
    deleted.add(account);
    accounts = accounts.where((a) => a.id != account).toList();
  }

  @override
  Coin crateApiCoinCoinNew({int? defaultCoin}) => _FakeCoin();

  @override
  Future<bool> crateApiZsaIsZsaAvailable({required Coin c}) async => false;

  // The account page records itself as the one to reopen at launch.
  @override
  Future<void> crateApiDbPutProp({required String key, required String value, required Coin c}) async {}

  @override
  dynamic noSuchMethod(Invocation invocation) => super.noSuchMethod(invocation);
}

class _Settings extends AppSettingsNotifier {
  @override
  Future<AppSettings> build() async => AppSettings(
        dbName: '',
        net: '',
        isLightNode: false,
        lwd: '',
        blockExplorer: '',
        syncInterval: '0',
        feePerAction: '',
        transport: 0,
        proxy: '',
        coingecko: '',
        recovery: false,
        needPin: false,
        pinUnlockedAt: DateTime(2026),
        offline: false,
        getFx: false,
        qrSettings: QRSettings(enabled: false, size: 100, ecLevel: 0, delay: 0, repair: 0),
        vault: false,
        expertMode: false,
        paletteName: '',
        darkMode: false,
        transactionTableMode: false,
        collapsePoolBalances: false,
        currency: '',
        votingConfigUrl: '',
        marketInterval: '1d',
        marketStyle: 'candles',
      );
}

Account _account(int id) => Account(
      coin: 0,
      id: id,
      name: 'Account $id',
      aindex: 0,
      dindex: 0,
      useInternal: false,
      birth: 1,
      folder: const Folder(id: 0, name: ''),
      position: id,
      hidden: false,
      saved: true,
      enabled: true,
      internal: false,
      hw: 0,
      height: 1,
      time: 0,
      balance: BigInt.zero,
      incomingOnly: false,
      frost: false,
      fixedInternalChange: false,
    );

/// The accounts in the fake database; deleting through the API removes one.
late List<Account> accounts;

void main() {
  late _Api api;
  setUpAll(() {
    api = _Api();
    RustLib.initMock(api: api);
    // confirmDialog is an AwesomeDialog with a Rive header; take Rive's one
    // initialisation failure in a guarded zone, as the other page tests do.
    runZonedGuarded(() => unawaited(RiveFile.initialize()), (_, __) {});
  });
  setUp(() => api.deleted.clear());
  tearDownAll(RustLib.dispose);

  late GoRouter router;

  /// The account page of [shown], with the account list pushed over it, as
  /// from the "Account Manager" menu item. The account page's own data never
  /// loads; it is hidden under the list for the whole test.
  Future<void> pumpStackedList(WidgetTester tester, int shown) async {
    router = GoRouter(navigatorKey: navigatorKey, initialLocation: accountRoute('/', shown), routes: [
      GoRoute(
        path: '/',
        builder: (context, state) => AccountViewPage(account: state.accountIdOrNull!),
      ),
      GoRoute(
        path: '/accounts',
        builder: (context, state) => const AccountListPage(),
      ),
    ]);
    addTearDown(router.dispose);
    await tester.pumpWidget(ProviderScope(
      overrides: [
        appSettingsProvider.overrideWith(_Settings.new),
        getAccountsProvider.overrideWith((ref) async => accounts),
        fullAccountPageDataProvider(shown).overrideWith((ref) => Completer<FullAccountPageData>().future),
      ],
      child: MaterialApp.router(routerConfig: router),
    ));
    await settle(tester);
    unawaited(router.push('/accounts'));
    await settle(tester);
    expect(find.text('Account List'), findsOneWidget);
    expect(find.byType(BackButton), findsOneWidget);
    expect(router.canPop(), isTrue);
  }

  /// Selects the account's row by its avatar, taps delete and confirms, then
  /// lets the pages and the row removal animate out. No frame on the way
  /// shows an error: the deleted row is still on screen while it animates
  /// out, after its account is gone.
  Future<void> deleteAccount(WidgetTester tester, int id) async {
    final row = find.ancestor(of: find.text('Account $id'), matching: find.byType(GestureDetector)).first;
    await tester.tap(find.descendant(of: row, matching: find.byType(CircleAvatar)));
    await settle(tester);
    await tester.tap(find.byIcon(Icons.delete));
    await settle(tester);
    await tester.tap(find.text('Ok'));
    for (var i = 0; i < 250; i++) {
      await tester.pump(const Duration(milliseconds: 20));
      expect(find.textContaining('Error'), findsNothing, reason: 'frame $i');
    }
    expect(api.deleted, [id]);
    expect(find.text('Account $id'), findsNothing);
  }

  testWidgets('deleting the last account makes the list the root page', (tester) async {
    accounts = [_account(7)];
    await pumpStackedList(tester, 7);

    await deleteAccount(tester, 7);
    expect(find.text('Account List'), findsOneWidget);
    expect(find.byType(BackButton), findsNothing);
    expect(router.canPop(), isFalse);
  });

  testWidgets('deleting the account under the list makes the list the root page', (tester) async {
    accounts = [_account(7), _account(8)];
    await pumpStackedList(tester, 7);

    await deleteAccount(tester, 7);
    expect(find.text('Account List'), findsOneWidget);
    expect(find.byType(BackButton), findsNothing);
    expect(router.canPop(), isFalse);
  });

  testWidgets('deleting another account leaves the list stacked', (tester) async {
    accounts = [_account(7), _account(8)];
    await pumpStackedList(tester, 7);

    await deleteAccount(tester, 8);
    expect(find.text('Account List'), findsOneWidget);
    expect(find.byType(BackButton), findsOneWidget);
    expect(router.canPop(), isTrue);
  });
}

Future<void> settle(WidgetTester tester) async {
  for (var i = 0; i < 50; i++) {
    await tester.pump(const Duration(milliseconds: 100));
  }
}
