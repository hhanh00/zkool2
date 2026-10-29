import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:zkool/pages/account.dart';
import 'package:zkool/src/rust/api/account.dart';
import 'package:zkool/src/rust/api/coin.dart';
import 'package:zkool/src/rust/frb_generated.dart';
import 'package:zkool/store.dart';
import 'package:zkool/widgets/pool_select.dart';

class _FakeCoin implements Coin {
  @override
  dynamic noSuchMethod(Invocation invocation) => super.noSuchMethod(invocation);
}

/// Key fetches the test completes by hand, in any order.
class _Api implements RustLibApi {
  final fetches = <(int pools, bool incoming, Completer<String>)>[];

  @override
  Coin crateApiCoinCoinNew({int? defaultCoin}) => _FakeCoin();

  @override
  Future<Seed?> crateApiAccountGetAccountSeed({required int account, required Coin c}) async => null;

  @override
  Future<int> crateApiAccountGetAccountPools({required int account, required Coin c}) async => 7;

  @override
  Future<FrostParams?> crateApiAccountGetAccountFrostParams({required int account, required Coin c}) async => null;

  @override
  Future<String> crateApiAccountGetAccountUfvk({required int account, required int pools, required Coin c}) {
    final done = Completer<String>();
    fetches.add((pools, false, done));
    return done.future;
  }

  @override
  Future<String> crateApiAccountGetAccountUivk({required int account, required int pools, required Coin c}) {
    final done = Completer<String>();
    fetches.add((pools, true, done));
    return done.future;
  }

  @override
  dynamic noSuchMethod(Invocation invocation) => super.noSuchMethod(invocation);
}

class _Unlocked extends Lifecycle {
  @override
  Future<bool> build() async => false;
}

Account _account(int id) => Account(
      coin: 0,
      id: id,
      name: 'account $id',
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
      frost: false,
      hw: 0,
      height: 100,
      time: 0,
      balance: BigInt.zero,
    );

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();
  final api = _Api();
  setUpAll(() => RustLib.initMock(api: api));
  tearDownAll(RustLib.dispose);

  PoolSelect selector(WidgetTester tester) => tester.widget<PoolSelect>(find.byType(PoolSelect));

  testWidgets('the pool selector follows the key shown, not the tap', (tester) async {
    api.fetches.clear();
    await tester.pumpWidget(ProviderScope(
      overrides: [
        lifecycleProvider.overrideWith(_Unlocked.new),
        getAccountsProvider.overrideWith((ref) async => [_account(1)]),
      ],
      child: const MaterialApp(home: ViewingKeysPage(1)),
    ));
    await tester.pumpAndSettle();

    // The page opens with a fetch for the account's pools.
    expect(api.fetches, hasLength(1));
    expect(api.fetches.single.$1, 7);
    api.fetches.single.$3.complete('ufvk-all');
    await tester.pumpAndSettle();
    expect(find.textContaining('ufvk-all'), findsWidgets);
    expect(selector(tester).value, 7);
    expect(selector(tester).enabled, 7);

    // Deselect transparent: the fetch starts, the selector does not move
    // and takes no more input until the key lands.
    await tester.tap(find.text('Trp'));
    await tester.pump();
    expect(api.fetches, hasLength(2));
    expect(api.fetches.last.$1, 6);
    expect(selector(tester).value, 7, reason: 'still the pools of the key shown');
    expect(selector(tester).enabled, 0, reason: 'refusing input while the fetch is in flight');
    await tester.tap(find.text('Sap'), warnIfMissed: false);
    await tester.pump();
    expect(api.fetches, hasLength(2), reason: 'no overlapping fetch');

    api.fetches.last.$3.complete('ufvk-shielded');
    await tester.pumpAndSettle();
    expect(find.textContaining('ufvk-shielded'), findsWidgets);
    expect(find.textContaining('ufvk-all'), findsNothing);
    expect(selector(tester).value, 6, reason: 'moved with the key');
    expect(selector(tester).enabled, 7);
  });
}
