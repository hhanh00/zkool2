import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_form_builder/flutter_form_builder.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:go_router/go_router.dart';
import 'package:rive/rive.dart' show RiveFile;
import 'package:zkool/pages/new_account.dart';
import 'package:zkool/router.dart' show navigatorKey;
import 'package:zkool/src/rust/api/account.dart';
import 'package:zkool/src/rust/api/coin.dart';
import 'package:zkool/src/rust/frb_generated.dart';
import 'package:zkool/store.dart';

// The new account page needs the chain tip for two things: the default birth
// height, and the offer to sync a restored account. It used to read a cached
// value that only autosync keeps fresh, so with autosync off the offer never
// came and a new account was born at height 1 (#1473). It fetches the tip
// now, and refuses to guess when the fetch fails.

const tip = 3123456;
const viewingKey = 'uviewtest1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq';

class _FakeCoin implements Coin {
  @override
  dynamic noSuchMethod(Invocation invocation) => super.noSuchMethod(invocation);
}

class _Api implements RustLibApi {
  /// What the server answers for the tip; null makes the call fail.
  int? height = tip;
  final created = <NewAccount>[];
  /// Accounts the wallet already has.
  var existing = <Account>[];

  @override
  Coin crateApiCoinCoinNew({int? defaultCoin}) => _FakeCoin();

  @override
  Future<int> crateApiNetworkGetCurrentHeight({required Coin c}) async {
    final h = height;
    if (h == null) throw AnyhowException('connection refused');
    return h;
  }

  // The pasted key is a valid viewing key, not a seed.
  @override
  bool crateApiKeyIsValidKey({required String key, required Coin c}) => true;
  @override
  bool crateApiKeyIsValidPhrase({required String phrase}) => false;
  @override
  bool crateApiKeyIsValidFvk({required String fvk, required Coin c}) => false;
  @override
  int crateApiKeyGetKeyPools({required String key, required Coin c}) => 7;

  @override
  Future<int> crateApiAccountNewAccount({required NewAccount na, required Coin c}) async {
    created.add(na);
    return 1;
  }

  @override
  Future<void> crateApiSyncCacheBlockTime({required int account, required int height, required Coin c}) async {}
  @override
  Future<bool> crateApiAccountHasTransparentPubKey({required int account, required Coin c}) async => false;
  @override
  Future<Seed?> crateApiAccountGetAccountSeed({required int account, required Coin c}) async => null;
  @override
  Future<void> crateApiDbPutProp({required String key, required String value, required Coin c}) async {}

  @override
  dynamic noSuchMethod(Invocation invocation) => super.noSuchMethod(invocation);
}

class _Unlocked extends Lifecycle {
  @override
  Future<bool> build() async => false;
}

class _Settings extends AppSettingsNotifier {
  @override
  Future<AppSettings> build() async => AppSettings(
        dbName: '',
        net: '',
        isLightNode: false,
        lwd: '',
        blockExplorer: '',
        syncInterval: '0', // autosync off: nothing polls the height
        actionsPerSync: '',
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

Account _account({int height = tip}) => Account(
      coin: 1,
      id: 1,
      name: 'restored',
      aindex: 0,
      dindex: 0,
      useInternal: false,
      birth: height,
      folder: const Folder(id: 0, name: ''),
      position: 0,
      hidden: false,
      saved: true,
      enabled: true,
      internal: false,
      incomingOnly: true,
      hw: 0,
      frost: false,
      height: height,
      time: 0,
      balance: BigInt.zero,
    );

void main() {
  late _Api api;
  setUpAll(() {
    api = _Api();
    RustLib.initMock(api: api);
    // The dialogs are AwesomeDialogs with a Rive header; take Rive's one
    // initialisation failure in a guarded zone, as the other page tests do.
    runZonedGuarded(() => unawaited(RiveFile.initialize()), (_, __) {});
  });
  setUp(() {
    api.created.clear();
    api.existing = [];
  });
  tearDownAll(RustLib.dispose);

  /// Pumps through dialog animations; pumpAndSettle never settles on them.
  Future<void> settle(WidgetTester tester) async {
    for (var i = 0; i < 10; i++) {
      await tester.pump(const Duration(milliseconds: 200));
    }
  }

  Future<void> openPage(WidgetTester tester) async {
    final router = GoRouter(navigatorKey: navigatorKey, routes: [
      GoRoute(
        path: '/',
        builder: (context, state) => Consumer(builder: (context, ref, _) {
          final settings = ref.watch(appSettingsProvider);
          return Scaffold(body: Text(settings.hasValue ? 'Home' : 'Loading'));
        }),
      ),
      GoRoute(path: '/new_account', builder: (context, state) => const NewAccountPage()),
    ]);
    addTearDown(router.dispose);
    await tester.pumpWidget(ProviderScope(
      overrides: [
        lifecycleProvider.overrideWith(_Unlocked.new),
        appSettingsProvider.overrideWith(_Settings.new),
        getAccountsProvider.overrideWith((ref) async => [...api.existing, if (api.created.isNotEmpty) _account()]),
      ],
      child: MaterialApp.router(routerConfig: router),
    ));
    await tester.pumpAndSettle();
    expect(find.text('Home'), findsOneWidget);
    unawaited(router.push<void>('/new_account'));
    await tester.pumpAndSettle();
    expect(find.byTooltip('Save'), findsOneWidget);
  }

  Finder field(String name) => find.byWidgetPredicate((w) => w is FormBuilderTextField && w.name == name);

  Future<void> fillRestore(WidgetTester tester, {String? birth}) async {
    await tester.tap(find.byWidgetPredicate((w) => w is FormBuilderSwitch && w.name == 'restore'));
    await tester.pumpAndSettle();
    await tester.enterText(field('key'), viewingKey);
    if (birth != null) await tester.enterText(field('birth'), birth);
    await tester.pumpAndSettle();
  }

  Future<void> save(WidgetTester tester) async {
    await tester.tap(find.byTooltip('Save'));
    await settle(tester);
  }

  Future<void> tapDialog(WidgetTester tester, String label) async {
    expect(find.text(label), findsOneWidget);
    await tester.tap(find.text(label));
    await settle(tester);
  }

  // First, while nothing has ever fetched a height: the fallback to the
  // cached value must find nothing either.
  testWidgets('no tip and no birth height: the account is not created', (tester) async {
    api.height = null;
    await openPage(tester);
    await fillRestore(tester);
    await save(tester);

    expect(find.textContaining('Enter a birth height'), findsOneWidget);
    expect(find.text('No Birth Height'), findsNothing, reason: 'the question is moot without a tip');
    expect(api.created, isEmpty);
    await tapDialog(tester, 'Ok');
  });

  testWidgets('no tip: a new account is born at the highest height the wallet has synced to', (tester) async {
    api.height = null;
    api.existing = [_account(height: 2000000)];
    await openPage(tester);
    // Restore off: a new account, nothing to type.
    await save(tester);

    final na = api.created.single;
    expect(na.restore, isFalse);
    expect(na.birth, 2000000);
    expect(find.textContaining('Would you like to synchronize'), findsNothing, reason: 'nothing to sync offline');
    expect(find.text('Home'), findsOneWidget);
  });

  testWidgets('a restored account with no birth height is born at the fetched tip and gets the sync offer',
      (tester) async {
    api.height = tip;
    await openPage(tester);
    await fillRestore(tester);
    await save(tester);

    await tapDialog(tester, 'Ok'); // "No Birth Height": sure
    await settle(tester);
    expect(find.textContaining('Would you like to synchronize it now?'), findsOneWidget);
    await tapDialog(tester, 'Cancel');

    final na = api.created.single;
    expect(na.restore, isTrue);
    expect(na.birth, tip);
    expect(find.text('Home'), findsOneWidget, reason: 'back to the account after the offer');
  });

  testWidgets('a typed birth height is kept and the sync offer still comes', (tester) async {
    api.height = tip;
    await openPage(tester);
    await fillRestore(tester, birth: '1');
    await save(tester);

    expect(find.text('No Birth Height'), findsNothing);
    expect(find.textContaining('Would you like to synchronize it now?'), findsOneWidget);
    await tapDialog(tester, 'Cancel');
    expect(api.created.single.birth, 1);
  });
}
