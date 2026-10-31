import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_form_builder/flutter_form_builder.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:go_router/go_router.dart';
import 'package:zkool/pages/new_account.dart';
import 'package:zkool/router.dart' show navigatorKey;
import 'package:zkool/src/rust/api/coin.dart';
import 'package:zkool/src/rust/frb_generated.dart';
import 'package:zkool/store.dart';
import 'package:zkool/widgets/frost_generation_select.dart';

// Restoring a FROST member: the group viewing key is matched to pick the
// generation to restore. The match ran on every keystroke and its result
// landed whatever the field held by then (#1523, V12 F-87). It now runs
// under the page's gate once editing completes, and is applied only if the
// field still holds the key it matched.

const backup = 'zkfrost1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq';
const keyA = 'uview1aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa';
const keyB = 'uview1bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb';

class _FakeCoin implements Coin {
  @override
  dynamic noSuchMethod(Invocation invocation) => super.noSuchMethod(invocation);
}

class _Api implements RustLibApi {
  /// The pending match: the test completes it when it wants the result in.
  Completer<int>? match;
  final matched = <String>[];

  @override
  Coin crateApiCoinCoinNew({int? defaultCoin}) => _FakeCoin();

  @override
  Future<int> crateApiFrostMatchFrostBackup({required String backup, required String ufvk, required Coin c}) {
    matched.add(ufvk);
    match = Completer<int>();
    return match!.future;
  }

  @override
  bool crateApiKeyIsValidKey({required String key, required Coin c}) => true;
  @override
  bool crateApiKeyIsValidPhrase({required String phrase}) => false;
  @override
  bool crateApiKeyIsValidFvk({required String fvk, required Coin c}) => false;
  @override
  int crateApiKeyGetKeyPools({required String key, required Coin c}) => 7;

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
        syncInterval: '0',
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

void main() {
  late _Api api;
  setUpAll(() {
    api = _Api();
    RustLib.initMock(api: api);
  });
  setUp(() {
    api.match = null;
    api.matched.clear();
  });
  tearDownAll(RustLib.dispose);

  Future<void> openPage(WidgetTester tester) async {
    final router = GoRouter(navigatorKey: navigatorKey, routes: [
      GoRoute(path: '/', builder: (context, state) => const Scaffold(body: Text('Home'))),
      GoRoute(path: '/new_account', builder: (context, state) => const NewAccountPage()),
    ]);
    addTearDown(router.dispose);
    await tester.pumpWidget(ProviderScope(
      overrides: [
        lifecycleProvider.overrideWith(_Unlocked.new),
        appSettingsProvider.overrideWith(_Settings.new),
        getAccountsProvider.overrideWith((ref) async => []),
      ],
      child: MaterialApp.router(routerConfig: router),
    ));
    await tester.pumpAndSettle();
    unawaited(router.push<void>('/new_account'));
    await tester.pumpAndSettle();
    expect(find.byTooltip('Save'), findsOneWidget);
  }

  Finder field(String name) => find.byWidgetPredicate((w) => w is FormBuilderTextField && w.name == name);

  /// Types the key into the group viewing key field and finishes editing.
  Future<void> enterKey(WidgetTester tester, String key) async {
    await tester.enterText(field('ufvk'), key);
    await tester.testTextInput.receiveAction(TextInputAction.done);
    await tester.pump();
  }

  bool locked(WidgetTester tester) =>
      tester.widgetList<AbsorbPointer>(find.byType(AbsorbPointer)).any((w) => w.absorbing);

  int generations(WidgetTester tester) =>
      tester.widget<FrostGenerationSelect>(find.byType(FrostGenerationSelect)).initialValue;

  Future<void> showFrostFields(WidgetTester tester) async {
    await tester.enterText(field('key'), backup);
    await tester.pumpAndSettle();
    expect(field('ufvk'), findsOneWidget);
  }

  testWidgets('the match runs when editing completes, locks the page, and sets the generation', (tester) async {
    await openPage(tester);
    await showFrostFields(tester);
    expect(generations(tester), FrostGeneration.zip2005.bit, reason: 'the default');

    await tester.enterText(field('ufvk'), keyA);
    await tester.pump();
    expect(api.matched, isEmpty, reason: 'typing alone does not match');

    await tester.testTextInput.receiveAction(TextInputAction.done);
    await tester.pump();
    expect(api.matched, [keyA]);
    expect(locked(tester), isTrue, reason: 'the page is locked while the match is pending');

    api.match!.complete(0);
    await tester.pumpAndSettle();
    expect(locked(tester), isFalse);
    expect(generations(tester), FrostGeneration.legacy.bit);
  });

  testWidgets('a match for a key the field no longer holds is not applied', (tester) async {
    await openPage(tester);
    await showFrostFields(tester);
    await enterKey(tester, keyA);
    expect(api.matched, [keyA]);

    // The field changes under the pending match (the text input bypasses
    // the pointer lock, as a paste from the keyboard would).
    await tester.enterText(field('ufvk'), keyB);
    await tester.pump();
    api.match!.complete(0);
    await tester.pumpAndSettle();
    expect(generations(tester), FrostGeneration.zip2005.bit, reason: 'the stale result is dropped');

    // Finishing the edit matches the current key, which is then applied.
    await tester.testTextInput.receiveAction(TextInputAction.done);
    await tester.pump();
    expect(api.matched, [keyA, keyB]);
    api.match!.complete(0);
    await tester.pumpAndSettle();
    expect(generations(tester), FrostGeneration.legacy.bit);
  });
}
