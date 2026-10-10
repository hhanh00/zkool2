import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:go_router/go_router.dart';
import 'package:zkool/pages/account.dart';
import 'package:zkool/router.dart' show navigatorKey;
import 'package:zkool/src/rust/api/account.dart';
import 'package:zkool/src/rust/api/coin.dart';
import 'package:zkool/src/rust/frb_generated.dart';
import 'package:zkool/store.dart';

class _Api implements RustLibApi {
  final updates = <AccountUpdate>[];
  final resets = <int>[];

  @override
  Coin crateApiCoinCoinNew({int? defaultCoin}) => const Coin.raw(
        coin: 3,
        account: 0,
        dbFilepath: '',
        url: '',
        serverType: 0,
        transport: 0,
        proxy: '',
      );

  @override
  Future<void> crateApiAccountUpdateAccount({required AccountUpdate update, required Coin c}) async {
    updates.add(update);
  }

  @override
  Future<void> crateApiAccountResetSync({required int id, required Coin c}) async {
    resets.add(id);
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
      );
}

Account _account() => Account(
      coin: 3,
      id: 1,
      name: 'Original',
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
      hw: 0,
      height: 100,
      time: 0,
      balance: BigInt.zero,
    );

void main() {
  late _Api api;
  setUpAll(() {
    api = _Api();
    RustLib.initMock(api: api);
  });
  setUp(() {
    api.updates.clear();
    api.resets.clear();
  });
  tearDownAll(RustLib.dispose);

  Future<GoRouter> openEditor(WidgetTester tester) async {
    final key = GlobalKey<AccountEditPageState>();
    final router = GoRouter(navigatorKey: navigatorKey, routes: [
      GoRoute(path: '/', builder: (context, state) => const Scaffold(body: Text('Account list'))),
      GoRoute(
        path: '/edit',
        builder: (context, state) => AccountEditPage([_account()], key: key),
        onExit: (context, state) => key.currentState?.confirmLeave() ?? true,
      ),
    ]);
    addTearDown(router.dispose);
    await tester.pumpWidget(ProviderScope(
      overrides: [
        lifecycleProvider.overrideWith(_Unlocked.new),
        appSettingsProvider.overrideWith(_Settings.new),
        getFoldersProvider.overrideWith((ref) async => []),
      ],
      child: MaterialApp.router(routerConfig: router),
    ));
    unawaited(router.push<void>('/edit'));
    await tester.pumpAndSettle();
    return router;
  }

  testWidgets('unchanged account leaves without a prompt or update', (tester) async {
    final router = await openEditor(tester);
    router.pop();
    await tester.pumpAndSettle();
    expect(find.text('Account list'), findsOneWidget);
    expect(api.updates, isEmpty);
  });

  testWidgets('edits stay local; stay blocks exit and discard leaves without saving', (tester) async {
    final router = await openEditor(tester);
    await tester.enterText(find.byType(TextField).first, 'Draft');
    expect(api.updates, isEmpty);
    router.pop();
    await tester.pumpAndSettle();
    await tester.tap(find.text('Stay'));
    await tester.pumpAndSettle();
    expect(find.text('Account Edit'), findsOneWidget);
    expect(find.text('Draft'), findsOneWidget);
    router.go('/');
    await tester.pumpAndSettle();
    await tester.tap(find.text('Leave without saving'));
    await tester.pumpAndSettle();
    expect(find.text('Account list'), findsOneWidget);
    expect(api.updates, isEmpty);
    expect(api.resets, isEmpty);
  });

  testWidgets('saving a name change does not reset sync data', (tester) async {
    final router = await openEditor(tester);
    await tester.enterText(find.byType(TextField).first, 'Renamed');
    router.pop();
    await tester.pumpAndSettle();
    expect(find.textContaining('clear the affected accounts'), findsNothing);
    await tester.tap(find.text('Save and leave'));
    await tester.pumpAndSettle();
    expect(api.updates.single.name, 'Renamed');
    expect(api.resets, isEmpty);
  });

  testWidgets('invalid birth height keeps the form open without saving', (tester) async {
    final router = await openEditor(tester);
    await tester.enterText(find.byType(TextField).at(1), '');
    router.pop();
    await tester.pumpAndSettle();
    await tester.tap(find.text('Save and leave'));
    await tester.pumpAndSettle();
    expect(find.text('Account Edit'), findsOneWidget);
    expect(find.text('Enter a valid birth height'), findsOneWidget);
    expect(api.updates, isEmpty);
    expect(api.resets, isEmpty);
  });

  testWidgets('birth height changes warn and reset only after save confirmation', (tester) async {
    final router = await openEditor(tester);
    await tester.enterText(find.byType(TextField).at(1), '200');
    expect(api.updates, isEmpty);
    expect(api.resets, isEmpty);
    router.pop();
    await tester.pumpAndSettle();
    expect(find.textContaining('clear the affected accounts'), findsOneWidget);
    await tester.tap(find.text('Save and leave'));
    await tester.pumpAndSettle();
    expect(api.updates.single.birth, 200);
    expect(api.resets, [1]);
    expect(find.text('Account list'), findsOneWidget);
  });

  testWidgets('internal change is staged and reset on confirmed exit', (tester) async {
    final router = await openEditor(tester);
    await tester.tap(find.text('Use Internal Change'));
    await tester.pump();
    expect(api.updates, isEmpty);
    router.pop();
    await tester.pumpAndSettle();
    expect(find.textContaining('clear the affected accounts'), findsOneWidget);
    await tester.tap(find.text('Save and leave'));
    await tester.pumpAndSettle();
    expect(api.updates.single.useInternal, true);
    expect(api.resets, [1]);
  });
}
