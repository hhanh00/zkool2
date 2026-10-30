import 'dart:async';
import 'dart:typed_data';

import 'package:flutter/material.dart';
import 'package:flutter_form_builder/flutter_form_builder.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:go_router/go_router.dart';
import 'package:zkool/pages/send.dart';
import 'package:zkool/router.dart' show navigatorKey;
import 'package:zkool/src/rust/api/account.dart';
import 'package:zkool/src/rust/api/coin.dart';
import 'package:zkool/src/rust/api/openalias.dart';
import 'package:zkool/src/rust/api/sync.dart';
import 'package:zkool/src/rust/api/zsa.dart';
import 'package:zkool/src/rust/frb_generated.dart';
import 'package:zkool/src/rust/pay.dart';
import 'package:zkool/store.dart';

// An OpenAlias lookup that lands after the user moved on (#1422, V12 F-116):
// the page is locked while the lookup is pending, and a result that no longer
// matches the field is dropped.

const alias = '#alice';
const resolvedAddr = 'u1rrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrr';
const typedAddr = 'u1ttttttttttttttttttttttttttttttttttttttttttttttttttttttttttttttttttttttttttttttttttttttt';

class _FakeCoin implements Coin {
  @override
  dynamic noSuchMethod(Invocation invocation) => super.noSuchMethod(invocation);
}

class _Api implements RustLibApi {
  /// The pending lookups' completers; the test decides when each lands.
  final lookups = <Completer<OpenAliasResolution>>[];

  @override
  Coin crateApiCoinCoinNew({int? defaultCoin}) => _FakeCoin();

  @override
  Future<PoolBalance> crateApiSyncBalance({required int account, required Coin c}) async => _balance();

  @override
  Future<Addresses> crateApiAccountGetAddresses({required int uaPools, required int account, required Coin c}) async =>
      const Addresses(ua: 'u1self', diversifierIndex: 0);

  @override
  Future<List<ZsaHolding>> crateApiZsaListZsaHoldings({required int account, required Coin c}) async => [];

  @override
  bool crateApiKeyIsValidAddress({required String address}) => true;

  @override
  bool crateApiKeyIsValidTransparentAddress({required String address, required Coin c}) => false;

  @override
  List<Recipient>? crateApiPayParsePaymentUri({required String uri}) => null;

  @override
  Future<OpenAliasResolution> crateApiOpenaliasResolveOpenalias({required String alias, required Coin c}) {
    final completer = Completer<OpenAliasResolution>();
    lookups.add(completer);
    return completer.future;
  }

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
        syncInterval: '',
        actionsPerSync: '',
        transport: 0,
        proxy: '',
        coingecko: '',
        recovery: false,
        needPin: false,
        pinUnlockedAt: DateTime(2026),
        offline: true,
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

PoolBalance _balance() => PoolBalance(field0: Uint64List.fromList([0, 0, 500000000, 0]));

Account _account() => Account(
      coin: 3,
      id: 1,
      name: 'Main',
      aindex: 0,
      dindex: 0,
      useInternal: false,
      birth: 100,
      folder: const Folder(id: 0, name: ''),
      position: 0,
      hidden: false,
      saved: true,
      enabled: true,
      internal: false,
      incomingOnly: false,
      hw: 0,
      frost: false,
      height: 100,
      time: 0,
      balance: BigInt.zero,
    );

AccountData _accountData() => AccountData(
      account: _account(),
      pool: 7,
      balance: _balance(),
      transactions: [],
      memos: [],
      notes: [],
      zsas: [],
    );

OpenAliasResolution _resolution() => OpenAliasResolution(
      recipients: [Recipient(address: resolvedAddr, amount: BigInt.zero, userMemo: 'from alice', assetBase: Uint8List(32))],
      dnssecStatus: 'verified',
    );

void main() {
  final api = _Api();
  setUpAll(() => RustLib.initMock(api: api));
  setUp(api.lookups.clear);
  tearDownAll(RustLib.dispose);

  Future<void> openSend(WidgetTester tester) async {
    final router = GoRouter(navigatorKey: navigatorKey, routes: [
      GoRoute(
        path: '/',
        builder: (context, state) => Consumer(builder: (context, ref, _) {
          final settings = ref.watch(appSettingsProvider);
          return Scaffold(body: Text(settings.hasValue ? 'Home' : 'Loading'));
        }),
      ),
      GoRoute(path: '/send', builder: (context, state) => const SendPage(account: 1)),
    ]);
    addTearDown(router.dispose);
    await tester.pumpWidget(ProviderScope(
      overrides: [
        lifecycleProvider.overrideWith(_Unlocked.new),
        appSettingsProvider.overrideWith(_Settings.new),
        accountProvider(1).overrideWith((ref) async => _accountData()),
        getAccountsProvider.overrideWith((ref) async => [_account()]),
        getContactsProvider.overrideWith((ref) async => []),
      ],
      child: MaterialApp.router(routerConfig: router),
    ));
    await tester.pumpAndSettle();
    expect(find.text('Home'), findsOneWidget);
    unawaited(router.push<void>('/send'));
    await tester.pumpAndSettle();
    expect(find.text('Recipient'), findsOneWidget);
  }

  Finder field(String name) => find.byWidgetPredicate((w) => w is FormBuilderTextField && w.name == name);
  String fieldText(WidgetTester tester, String name) =>
      tester.widget<EditableText>(find.descendant(of: field(name), matching: find.byType(EditableText))).controller.text;
  bool locked(WidgetTester tester) =>
      tester.widget<AbsorbPointer>(find.ancestor(of: find.byType(FormBuilder), matching: find.byType(AbsorbPointer)).first).absorbing;
  // The button wraps its tooltip, so the button is the tooltip's ancestor.
  bool sendEnabled(WidgetTester tester) => tester
          .widget<IconButton>(
              find.ancestor(of: find.byTooltip('Send transaction (including queued recipients)'), matching: find.byType(IconButton)).first)
          .onPressed !=
      null;

  /// Types the alias and completes editing, which moves the focus on: the
  /// lookup's two triggers (editing complete, focus loss) both fire.
  Future<void> typeAliasAndLeave(WidgetTester tester) async {
    await tester.enterText(field('address'), alias);
    await tester.testTextInput.receiveAction(TextInputAction.next);
    await tester.pump();
  }

  testWidgets('one lookup runs per blur and the page is locked until it lands', (tester) async {
    await openSend(tester);
    await typeAliasAndLeave(tester);

    expect(api.lookups, hasLength(1));
    expect(locked(tester), isTrue);
    expect(sendEnabled(tester), isFalse);
    expect(find.byType(CircularProgressIndicator), findsNothing);

    api.lookups.single.complete(_resolution());
    await tester.pumpAndSettle();

    expect(fieldText(tester, 'address'), resolvedAddr);
    expect(fieldText(tester, 'memo'), 'from alice');
    expect(locked(tester), isFalse);
    expect(sendEnabled(tester), isTrue);
  });

  testWidgets('a lookup landing after the field changed does not overwrite it', (tester) async {
    await openSend(tester);
    await typeAliasAndLeave(tester);

    // A writer the lock cannot reach, such as the scanner returning.
    await tester.enterText(field('address'), typedAddr);
    await tester.pump();
    api.lookups.single.complete(_resolution());
    await tester.pumpAndSettle();

    expect(fieldText(tester, 'address'), typedAddr);
    expect(fieldText(tester, 'memo'), '');
    expect(tester.state<SendPageState>(find.byType(SendPage)).address, typedAddr);
    expect(locked(tester), isFalse);
  });

  testWidgets('the spinner appears after a second and not for a fast lookup', (tester) async {
    await openSend(tester);
    await typeAliasAndLeave(tester);

    await tester.pump(const Duration(milliseconds: 900));
    expect(find.byType(CircularProgressIndicator), findsNothing);
    await tester.pump(const Duration(milliseconds: 200));
    expect(find.byType(CircularProgressIndicator), findsOneWidget);

    api.lookups.single.complete(_resolution());
    await tester.pumpAndSettle();
    expect(find.byType(CircularProgressIndicator), findsNothing);

    // A second alias that resolves at once never shows one.
    await tester.enterText(field('address'), '#bob');
    await tester.testTextInput.receiveAction(TextInputAction.next);
    api.lookups.last.complete(_resolution());
    await tester.pump();
    await tester.pump(const Duration(seconds: 2));
    expect(find.byType(CircularProgressIndicator), findsNothing);
    expect(fieldText(tester, 'address'), resolvedAddr);
  });

  testWidgets('a lookup that never answers releases the lock at the timeout', (tester) async {
    await openSend(tester);
    await typeAliasAndLeave(tester);
    expect(locked(tester), isTrue);

    await tester.pump(SendPageState.openaliasTimeout + const Duration(seconds: 1));
    await tester.pump();

    expect(locked(tester), isFalse);
    expect(sendEnabled(tester), isTrue);
    expect(fieldText(tester, 'address'), alias);
    expect(api.lookups, hasLength(1));
  });
}
