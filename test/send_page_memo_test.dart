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
import 'package:zkool/src/rust/api/contacts.dart';
import 'package:zkool/src/rust/api/sync.dart';
import 'package:zkool/src/rust/api/zsa.dart';
import 'package:zkool/src/rust/frb_generated.dart';
import 'package:zkool/src/rust/pay.dart';
import 'package:zkool/store.dart';

// A reference (`!account`, `@contact`, `#alias`) hides the memo field while
// it is typed, so a memo written for an earlier shielded address must not
// ride along to the resolved address (#1533, V12 F-109). One shielded
// address replacing another keeps the memo: it stays visible.

const _me = 'u1me';
const _alice = 'u1alice';
const _bob = 'u1bob';
const _carol = 'u1carol';

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
      Addresses(ua: account == 1 ? _me : _bob, diversifierIndex: 0);

  // `!bob` resolves through the account's own address list.
  @override
  Future<Addresses> crateApiAccountGetAccountAddresses({required int account, required int uaPools, required Coin c}) async =>
      Addresses(ua: account == 1 ? _me : _bob, diversifierIndex: 0);

  @override
  Future<List<ZsaHolding>> crateApiZsaListZsaHoldings({required int account, required Coin c}) async => [];

  @override
  bool crateApiKeyIsValidAddress({required String address, required Coin c}) => address.startsWith('u1');

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

Account _account(int id, String name) => Account(
      coin: 0,
      id: id,
      name: name,
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

AccountData _accountData(Account account) => AccountData(
      account: account,
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

  Future<void> openSend(WidgetTester tester) async {
    final me = _account(1, 'me');
    final bob = _account(2, 'bob');
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
        accountProvider(1).overrideWith((ref) async => _accountData(me)),
        getAccountsProvider.overrideWith((ref) async => [me, bob]),
        getContactsProvider.overrideWith((ref) async => [const Contact(id: 1, name: 'bob', addresses: [_bob], notes: '')]),
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

  Future<void> type(WidgetTester tester, String name, String text) async {
    await tester.enterText(field(name), text);
    await tester.pumpAndSettle();
  }

  /// A memo typed while a shielded address made the field visible, then
  /// [address] in its place.
  Future<void> memoThenAddress(WidgetTester tester, String address) async {
    await type(tester, 'address', _alice);
    await type(tester, 'zat', '1');
    expect(field('memo').hitTestable(), findsOneWidget, reason: 'a shielded address shows the memo field');
    await type(tester, 'memo', 'hello');
    await type(tester, 'address', address);
  }

  Future<Recipient> queue(WidgetTester tester) async {
    await tester.tap(find.byTooltip('Queue this recipient to create a multi send'));
    await tester.pumpAndSettle();
    final state = tester.state<SendPageState>(find.byType(SendPage));
    expect(state.recipients, hasLength(1), reason: 'the recipient was queued');
    return state.recipients.single;
  }

  testWidgets('a memo typed before a @contact reference is not sent to the contact', (tester) async {
    await openSend(tester);
    await memoThenAddress(tester, '@bob');
    expect(field('memo').hitTestable(), findsNothing, reason: 'the reference hides the memo field');
    final recipient = await queue(tester);
    expect(recipient.address, _bob);
    expect(recipient.userMemo, isNull);
  });

  testWidgets('a memo typed before a !account reference is not sent to the account', (tester) async {
    await openSend(tester);
    await memoThenAddress(tester, '!bob');
    final recipient = await queue(tester);
    expect(recipient.address, _bob);
    expect(recipient.userMemo, isNull);
  });

  testWidgets('a memo stays when one shielded address replaces another', (tester) async {
    await openSend(tester);
    await memoThenAddress(tester, _carol);
    expect(field('memo').hitTestable(), findsOneWidget, reason: 'still visible, the user can read it');
    final recipient = await queue(tester);
    expect(recipient.address, _carol);
    expect(recipient.userMemo, 'hello');
  });
}
