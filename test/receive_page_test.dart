import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:zkool/pages/receive.dart';
import 'package:zkool/src/rust/api/account.dart';
import 'package:zkool/src/rust/api/coin.dart';
import 'package:zkool/src/rust/api/sync.dart';
import 'package:zkool/src/rust/frb_generated.dart';
import 'package:zkool/store.dart';
import 'package:zkool/widgets/pool_select.dart';

// The receive page and the address browser applied whatever result landed
// last, so two quick selection changes whose fetches completed out of
// order showed, and copied, an address or a row set for the other
// selection (#1483). Each page now takes one request at a time and its
// selectors refuse input while a fetch is in flight, the pattern of the
// viewing key page.

class _FakeCoin implements Coin {
  @override
  dynamic noSuchMethod(Invocation invocation) => super.noSuchMethod(invocation);
}

class _Api implements RustLibApi {
  final addressFetches = <(int pools, Completer<Addresses>)>[];
  final rowFetches = <(int pools, bool aggregate, Completer<List<TAddressTxCount>>)>[];

  @override
  Coin crateApiCoinCoinNew({int? defaultCoin}) => _FakeCoin();

  @override
  Future<Addresses> crateApiAccountGetAddresses({required int uaPools, required int account, required Coin c}) {
    final done = Completer<Addresses>();
    addressFetches.add((uaPools, done));
    return done.future;
  }

  @override
  Future<List<TAddressTxCount>> crateApiAccountFetchAddressTxCount(
      {required bool aggregate, required int poolFilter, required int account, required Coin c}) {
    final done = Completer<List<TAddressTxCount>>();
    rowFetches.add((poolFilter, aggregate, done));
    return done.future;
  }

  @override
  dynamic noSuchMethod(Invocation invocation) => super.noSuchMethod(invocation);
}

class _Unlocked extends Lifecycle {
  @override
  Future<bool> build() async => false;
}

Account _account() => Account(
      coin: 0,
      id: 1,
      name: 'account',
      aindex: 0,
      dindex: 0,
      useInternal: true,
      birth: 100,
      folder: const Folder(id: 0, name: ''),
      position: 0,
      hidden: false,
      saved: true,
      enabled: true,
      internal: false,
      incomingOnly: false,
      fixedInternalChange: false,
      frost: false,
      hw: 0,
      height: 100,
      time: 0,
      balance: BigInt.zero,
    );

Addresses _addresses(int pools) => Addresses(
      taddr: 't1-$pools',
      saddr: 'zs-$pools',
      oaddr: 'uo-$pools',
      ua: 'u-$pools',
      diversifierIndex: 0,
    );

TAddressTxCount _row(int pool) => TAddressTxCount(
      pool: pool,
      address: 'addr-$pool',
      scope: 0,
      dindex: 0,
      amount: BigInt.zero,
      txCount: 0,
      time: 0,
    );

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();
  final api = _Api();
  setUpAll(() {
    RustLib.initMock(api: api);
    coinContext.set(coin: _FakeCoin());
  });
  setUp(() {
    api.addressFetches.clear();
    api.rowFetches.clear();
  });
  tearDownAll(RustLib.dispose);

  PoolSelect selector(WidgetTester tester) => tester.widget<PoolSelect>(find.byType(PoolSelect));

  /// Bounded pumps: the page animates its QR code, so pumpAndSettle never settles.
  Future<void> settle(WidgetTester tester) async {
    for (var i = 0; i < 10; i++) {
      await tester.pump(const Duration(milliseconds: 100));
    }
  }

  testWidgets('the receive page refuses a second selection while an address fetch is pending', (tester) async {
    await tester.pumpWidget(ProviderScope(
      overrides: [
        lifecycleProvider.overrideWith(_Unlocked.new),
        accountProvider(1).overrideWith((ref) async => AccountData(
              account: _account(),
              pool: 7,
              balance: PoolBalance(field0: Uint64List.fromList([0, 0, 0, 0])),
              transactions: [],
              memos: [],
              notes: [],
              zsas: [],
            )),
      ],
      child: const MaterialApp(home: ReceivePage(account: 1)),
    ));
    await settle(tester);

    // The page opens with a fetch for the shielded pools.
    expect(api.addressFetches, hasLength(1));
    expect(api.addressFetches.single.$1, 6);
    api.addressFetches.single.$2.complete(_addresses(6));
    await settle(tester);
    expect(find.text('u-6'), findsOneWidget);
    expect(selector(tester).value, 6);
    expect(selector(tester).enabled, 7);

    // Add transparent: the fetch starts, the selector keeps the shown
    // selection and takes no more input until the address lands.
    await tester.tap(find.text('Trp'));
    await tester.pump();
    expect(api.addressFetches, hasLength(2));
    expect(api.addressFetches.last.$1, 7);
    expect(selector(tester).value, 6, reason: 'still the pools of the address shown');
    expect(selector(tester).enabled, 0, reason: 'refusing input while the fetch is in flight');
    await tester.tap(find.text('Sap'), warnIfMissed: false);
    await tester.pump();
    expect(api.addressFetches, hasLength(2), reason: 'no overlapping fetch');

    api.addressFetches.last.$2.complete(_addresses(7));
    await settle(tester);
    expect(find.text('u-7'), findsOneWidget);
    expect(find.text('u-6'), findsNothing);
    expect(selector(tester).value, 7, reason: 'moved with the address');
    expect(selector(tester).enabled, 7);
  });

  testWidgets('the address browser refuses a filter change while its rows are loading', (tester) async {
    await tester.pumpWidget(ProviderScope(
      overrides: [lifecycleProvider.overrideWith(_Unlocked.new)],
      child: MaterialApp(
        home: AddressesPage(txCounts: [_row(0), _row(1), _row(2)], availablePools: 7, c: _FakeCoin(), account: 1),
      ),
    ));
    await settle(tester);
    expect(find.text('addr-0'), findsOneWidget);
    expect(selector(tester).enabled, 7);

    // Drop transparent: the rows reload, the selector refuses input meanwhile.
    await tester.tap(find.text('Trp'));
    await tester.pump();
    expect(api.rowFetches, hasLength(1));
    expect(api.rowFetches.single.$1, 6);
    expect(selector(tester).enabled, 0);
    await tester.tap(find.text('Sap'), warnIfMissed: false);
    await tester.pump();
    expect(api.rowFetches, hasLength(1), reason: 'no overlapping fetch');

    api.rowFetches.single.$3.complete([_row(1), _row(2)]);
    await settle(tester);
    expect(find.text('addr-0'), findsNothing, reason: 'the rows match the controls');
    expect(find.text('addr-1'), findsOneWidget);
    expect(selector(tester).enabled, 7);
  });
}
