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
import 'package:zkool/src/rust/api/sync.dart';
import 'package:zkool/src/rust/api/zsa.dart';
import 'package:zkool/src/rust/frb_generated.dart';
import 'package:zkool/src/rust/pay.dart';
import 'package:zkool/store.dart';

// Editing a queued ZSA recipient must keep its asset (#1416, V12 F-170).
// The form resets to ZEC after every Add, and the edit used to restore the
// amount as ZEC without restoring the asset, so finishing the edit rebuilt
// the row as a ZEC payment.

const addrA = 'u1aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa';

final tokenBase = Uint8List.fromList(List.generate(32, (i) => i + 1));

ZsaHolding _token() => ZsaHolding(
      idAsset: 1,
      assetDescHash: Uint8List.fromList(List.generate(32, (i) => 0xA0 + i)),
      assetName: 'TOKEN',
      ik: Uint8List(32),
      assetBase: tokenBase,
      finalized: true,
      firstSeenHeight: 10,
      balance: BigInt.from(1000000000),
    );

class _FakeCoin implements Coin {
  @override
  dynamic noSuchMethod(Invocation invocation) => super.noSuchMethod(invocation);
}

class _Api implements RustLibApi {
  @override
  Coin crateApiCoinCoinNew({int? defaultCoin}) => _FakeCoin();

  @override
  Future<PoolBalance> crateApiSyncBalance({required int account, required Coin c}) async => _balance();

  @override
  Future<Addresses> crateApiAccountGetAddresses({required int uaPools, required int account, required Coin c}) async =>
      const Addresses(ua: 'u1self', diversifierIndex: 0);

  // The wallet holds one asset, so the Currency dropdown offers it.
  @override
  Future<List<ZsaHolding>> crateApiZsaListZsaHoldings({required int account, required Coin c}) async => [_token()];

  @override
  bool crateApiKeyIsValidAddress({required String address}) => true;

  @override
  bool crateApiKeyIsValidTransparentAddress({required String address, required Coin c}) => false;

  @override
  List<Recipient>? crateApiPayParsePaymentUri({required String uri}) => null;

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
      zsas: [_token()],
    );

void main() {
  setUpAll(() => RustLib.initMock(api: _Api()));
  tearDownAll(RustLib.dispose);

  /// Opens the send page; [sent] receives the recipients passed to /send2.
  Future<void> openSend(WidgetTester tester, List<List<Recipient>> sent) async {
    final router = GoRouter(navigatorKey: navigatorKey, routes: [
      GoRoute(
        path: '/',
        builder: (context, state) => Consumer(builder: (context, ref, _) {
          final settings = ref.watch(appSettingsProvider);
          return Scaffold(body: Text(settings.hasValue ? 'Home' : 'Loading'));
        }),
      ),
      GoRoute(path: '/send', builder: (context, state) => const SendPage(account: 1)),
      GoRoute(
        path: '/send2',
        builder: (context, state) {
          final (recipients, _) = state.extra as (List<Recipient>, bool);
          sent.add(List.of(recipients));
          return const Scaffold(body: Text('send2'));
        },
      ),
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
  Finder tile(String address) => find.widgetWithText(ListTile, address);
  String fieldText(WidgetTester tester, String name) =>
      tester.widget<EditableText>(find.descendant(of: field(name), matching: find.byType(EditableText))).controller.text;
  Finder dropdown() => find.byWidgetPredicate((w) => w is FormBuilderDropdown<Uint8List> && w.name == 'asset');
  SendPageState page(WidgetTester tester) => tester.state<SendPageState>(find.byType(SendPage));

  /// The asset the next recipient is built from, which must agree with the
  /// Currency dropdown's value.
  Uint8List selectedValue(WidgetTester tester) {
    final state = page(tester);
    expect(state.formKey.currentState!.fields['asset']!.value, state.selectedAssetBase);
    return state.selectedAssetBase;
  }

  Future<void> chooseCurrency(WidgetTester tester, String label) async {
    await tester.tap(dropdown());
    await tester.pumpAndSettle();
    await tester.tap(find.text(label).last);
    await tester.pumpAndSettle();
  }

  Future<void> queueButton(WidgetTester tester) async {
    await tester.tap(find.byTooltip('Queue this recipient to create a multi send'));
    await tester.pumpAndSettle();
  }

  testWidgets('editing a queued asset recipient keeps its asset and amount', (tester) async {
    final sent = <List<Recipient>>[];
    await openSend(tester, sent);

    await chooseCurrency(tester, 'TOKEN');
    expect(find.text('Amount in TOKEN'), findsOneWidget);
    await tester.enterText(field('address'), addrA);
    await tester.enterText(field('zat'), '500000000');
    await tester.pumpAndSettle();
    await queueButton(tester);
    expect(find.text('500000000 TOKEN'), findsOneWidget);
    expect(selectedValue(tester), zecBase, reason: 'Add resets the form to ZEC');
    expect(find.text('Amount in ZEC'), findsOneWidget);

    // Edit: the form takes the row back, asset included.
    await tester.tap(tile(addrA));
    await tester.pumpAndSettle();
    expect(fieldText(tester, 'zat'), '500000000', reason: 'whole units, not 5.00000000 ZEC');
    expect(selectedValue(tester), tokenBase);
    expect(find.text('Amount in TOKEN'), findsOneWidget);

    // Finish the edit by queueing it again: the row is still the asset.
    await queueButton(tester);
    expect(find.text('500000000 TOKEN'), findsOneWidget);
    expect(find.textContaining('5.00000000'), findsNothing);

    await tester.tap(find.byTooltip('Send transaction (including queued recipients)'));
    await tester.pumpAndSettle();
    final recipient = sent.single.single;
    expect(recipient.address, addrA);
    expect(recipient.amount, BigInt.from(500000000));
    expect(recipient.assetBase, tokenBase);
    expect(recipient.assetName, 'TOKEN');
  });

  testWidgets('clearing the form goes back to ZEC', (tester) async {
    final sent = <List<Recipient>>[];
    await openSend(tester, sent);
    await chooseCurrency(tester, 'TOKEN');
    await tester.enterText(field('address'), addrA);
    await tester.enterText(field('zat'), '7');
    await tester.pumpAndSettle();
    await queueButton(tester);

    // A ZEC recipient typed after the reset is a ZEC recipient.
    await tester.enterText(field('address'), addrA);
    await tester.enterText(field('zat'), '0.1');
    await tester.pumpAndSettle();
    await queueButton(tester);
    expect(find.text('7 TOKEN'), findsOneWidget);
    final zec = tester.state<SendPageState>(find.byType(SendPage)).recipients.last;
    expect(zec.assetBase.every((b) => b == 0), isTrue);
    expect(zec.amount, BigInt.from(10000000));
  });
}
