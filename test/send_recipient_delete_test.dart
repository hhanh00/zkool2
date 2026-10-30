import 'dart:async';

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
import 'package:zkool/utils.dart';

// Deleting a queued recipient while one is being edited (#1420, V12 F-127):
// the form must not hold values Send then appends as a new recipient.

const addrA = 'u1aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa';
const addrB = 'u1bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb';

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

  @override
  Future<List<ZsaHolding>> crateApiZsaListZsaHoldings({required int account, required Coin c}) async => [];

  // Any address the tests type is a valid shielded address.
  @override
  bool crateApiKeyIsValidAddress({required String address}) => true;

  @override
  bool crateApiKeyIsValidTransparentAddress({required String address, required Coin c}) => false;

  // A plain address is not a payment URI.
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
      zsas: [],
    );

void main() {
  setUpAll(() => RustLib.initMock(api: _Api()));
  tearDownAll(RustLib.dispose);

  /// Opens the send page; [sent] receives the recipients passed to /send2.
  Future<void> openSend(WidgetTester tester, List<List<Recipient>> sent) async {
    final router = GoRouter(navigatorKey: navigatorKey, routes: [
      // The home route warms the settings, as the app has them loaded before
      // the send page opens; the page's amount field relies on that.
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

  Future<void> typeRecipient(WidgetTester tester, String address, String amount) async {
    await tester.enterText(field('address'), address);
    await tester.enterText(field('zat'), amount);
    await tester.pumpAndSettle();
  }

  Future<void> queue(WidgetTester tester, String address, String amount) async {
    await typeRecipient(tester, address, amount);
    await tester.tap(find.byTooltip('Queue this recipient to create a multi send'));
    await tester.pumpAndSettle();
    expect(tile(address), findsAtLeastNWidgets(1));
  }

  Future<void> edit(WidgetTester tester, String address) async {
    await tester.tap(tile(address));
    await tester.pumpAndSettle();
    expect(fieldText(tester, 'address'), address);
  }

  /// Taps the delete button of the tile at [index].
  Future<void> delete(WidgetTester tester, int index) async {
    await tester.tap(find.byIcon(Icons.delete).at(index));
    await tester.pumpAndSettle();
  }

  Future<void> send(WidgetTester tester) async {
    await tester.tap(find.byTooltip('Send transaction (including queued recipients)'));
    await tester.pumpAndSettle();
    expect(find.text('send2'), findsOneWidget);
  }

  testWidgets('deleting the recipient being edited clears the form and Send does not add it back', (tester) async {
    final sent = <List<Recipient>>[];
    await openSend(tester, sent);
    await queue(tester, addrA, '0.1');
    await queue(tester, addrB, '0.2');

    await edit(tester, addrA);
    await delete(tester, 0);

    expect(tile(addrA), findsNothing);
    expect(fieldText(tester, 'address'), '');
    // The amount widget keeps showing the last edited amount after any form
    // reset (its initialValue follows the page's amount); what matters here
    // is that the form holds nothing Send would queue.
    expect(tester.state<SendPageState>(find.byType(SendPage)).formKey.currentState!.isDirty, isFalse);

    await send(tester);
    expect(sent.single.map((r) => r.address), [addrB]);
    expect(sent.single.single.amount, stringToZat('0.2'));
  });

  testWidgets('deleting another recipient while editing does not pay the edited one twice', (tester) async {
    final sent = <List<Recipient>>[];
    await openSend(tester, sent);
    await queue(tester, addrA, '0.1');
    await queue(tester, addrB, '0.2');

    await edit(tester, addrB);
    await delete(tester, 0);

    expect(tile(addrA), findsNothing);
    expect(tile(addrB), findsOneWidget);
    expect(tester.widget<ListTile>(tile(addrB)).selected, isTrue);
    expect(fieldText(tester, 'address'), addrB);

    await send(tester);
    expect(sent.single.map((r) => r.address), [addrB]);
    expect(sent.single.single.amount, stringToZat('0.2'));
  });

  testWidgets('the edit follows the recipient when an earlier one is deleted', (tester) async {
    final sent = <List<Recipient>>[];
    await openSend(tester, sent);
    await queue(tester, addrA, '0.1');
    await queue(tester, addrB, '0.2');

    await edit(tester, addrB);
    await delete(tester, 0);
    await tester.enterText(field('zat'), '0.3');
    await tester.pumpAndSettle();

    await send(tester);
    expect(sent.single.map((r) => r.address), [addrB]);
    expect(sent.single.single.amount, stringToZat('0.3'));
  });

  testWidgets('deleting a later recipient keeps the edit on the same one', (tester) async {
    final sent = <List<Recipient>>[];
    await openSend(tester, sent);
    await queue(tester, addrA, '0.1');
    await queue(tester, addrB, '0.2');

    await edit(tester, addrA);
    await delete(tester, 1);
    expect(tile(addrB), findsNothing);
    expect(tester.widget<ListTile>(tile(addrA)).selected, isTrue);
    await tester.enterText(field('zat'), '0.3');
    await tester.pumpAndSettle();

    await send(tester);
    expect(sent.single.map((r) => r.address), [addrA]);
    expect(sent.single.single.amount, stringToZat('0.3'));
  });

  testWidgets('deleting one of two equal recipients removes only that tile', (tester) async {
    final sent = <List<Recipient>>[];
    await openSend(tester, sent);
    await queue(tester, addrA, '0.1');
    await queue(tester, addrA, '0.1');
    expect(tile(addrA), findsNWidgets(2));

    await delete(tester, 1);
    expect(tile(addrA), findsOneWidget);

    await send(tester);
    expect(sent.single.map((r) => r.address), [addrA]);
  });
}
