import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter_form_builder/flutter_form_builder.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:go_router/go_router.dart';
import 'package:path_provider_platform_interface/path_provider_platform_interface.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:zkool/pages/db.dart';
import 'package:zkool/router.dart' show navigatorKey;
import 'package:zkool/src/rust/api/coin.dart';
import 'package:zkool/src/rust/api/db.dart';
import 'package:zkool/src/rust/frb_generated.dart';
import 'package:zkool/store.dart';

import 'platform_fakes.dart';

// The "Create New Database" dialog lets the user pick the network of the new
// wallet, preselected to the open wallet's, and creates it through the Rust
// create call, where the choice wins over the file name (#1613).

/// The open wallet, on testnet.
class _FakeCoin implements Coin {
  @override
  int get coin => 1;

  @override
  dynamic noSuchMethod(Invocation invocation) => super.noSuchMethod(invocation);
}

class _Api implements RustLibApi {
  final names = <String>['main'];
  final created = <(String, String?, int)>[];
  String? refuseWith;

  @override
  Coin crateApiCoinCoinNew({int? defaultCoin}) => _FakeCoin();

  @override
  Future<List<String>> crateApiDbListDbNames({required String dir}) async => List.of(names);

  @override
  Future<List<DbAccountPreview>> crateApiDbListDbAccounts({required String dbFilepath}) async => [];

  @override
  Future<void> crateApiDbCreateDb({required String dbFilepath, String? password, required int coin}) async {
    if (refuseWith != null) throw AnyhowException(refuseWith!);
    created.add((dbFilepath.split('/').last, password, coin));
    names.add(dbFilepath.split('/').last.replaceAll(RegExp(r'\.db$'), ''));
  }

  @override
  dynamic noSuchMethod(Invocation invocation) => super.noSuchMethod(invocation);
}

void main() {
  late _Api api;
  late Directory dir;

  setUpAll(() {
    silenceRiveHeaders();
    api = _Api();
    RustLib.initMock(api: api);
  });
  tearDownAll(RustLib.dispose);

  setUp(() {
    api.names
      ..clear()
      ..add('main');
    api.created.clear();
    api.refuseWith = null;
    dir = Directory.systemTemp.createTempSync('zkool-dbc-');
    PathProviderPlatform.instance = FakePathProvider(dir);
    useInMemoryPrefs({'database': 'main', 'recovery': true});
  });
  tearDown(() => dir.deleteSync(recursive: true));

  /// AwesomeDialog animates its header for as long as it is shown, so
  /// pumpAndSettle never settles while a dialog is up; pump a fixed span.
  Future<void> settle(WidgetTester tester) async {
    for (var i = 0; i < 12; i++) {
      await tester.pump(const Duration(milliseconds: 100));
    }
  }

  Future<void> openCreateDialog(WidgetTester tester) async {
    final router = GoRouter(navigatorKey: navigatorKey, routes: [
      GoRoute(path: '/', builder: (context, state) => DatabaseManagerPage()),
      GoRoute(path: '/splash', builder: (context, state) => const Scaffold(body: Text('splash'))),
    ]);
    addTearDown(router.dispose);
    await tester.pumpWidget(ProviderScope(child: MaterialApp.router(routerConfig: router)));
    await settle(tester);
    expect(find.text('Database Manager'), findsOneWidget);
    await tester.tap(find.byIcon(Icons.add));
    await settle(tester);
    expect(find.text('Create New Database'), findsOneWidget);
  }

  Future<void> pickNetwork(WidgetTester tester, String name) async {
    await tester.tap(find.byType(FormBuilderDropdown<int>));
    await settle(tester);
    await tester.tap(find.text(name).last);
    await settle(tester);
  }

  Future<void> tapOk(WidgetTester tester) async {
    await tester.tap(find.text('Ok').last);
    await settle(tester);
  }

  testWidgets('the dialog preselects the open wallet\'s network', (tester) async {
    await openCreateDialog(tester);

    // The dropdown shows the selected entry; the others are in the closed menu.
    expect(find.text('Testnet'), findsOneWidget);
    expect(find.text('Mainnet'), findsNothing);
  });

  testWidgets('creating a wallet passes the chosen network and opens it', (tester) async {
    await openCreateDialog(tester);
    await tester.enterText(find.widgetWithText(TextField, 'Name'), 'mine');
    await pickNetwork(tester, 'Regtest');
    await tapOk(tester);

    expect(api.created, [('mine.db', null, 2)]);
    final prefs = SharedPreferencesAsync();
    expect(await prefs.getString('database'), 'mine');
    expect(await prefs.getBool('recovery'), isNull);
    expect(find.text('splash'), findsOneWidget);
  });

  testWidgets('a refused create is reported and nothing is opened', (tester) async {
    api.refuseWith = 'a wallet file already exists at /x/main.db';
    await openCreateDialog(tester);
    await tester.enterText(find.widgetWithText(TextField, 'Name'), 'main');
    await tapOk(tester);

    expect(find.textContaining('already exists'), findsOneWidget);
    expect(api.created, isEmpty);
    expect(await SharedPreferencesAsync().getString('database'), 'main');
    expect(find.text('splash'), findsNothing);
  });

  group('per-network defaults', () {
    test('server', () {
      expect(defaultServerFor('mainnet'), 'https://zec.rocks');
      expect(defaultServerFor('testnet'), 'https://testnet.zec.rocks');
      expect(defaultServerFor('regnet'), 'http://localhost:8137');
      expect(defaultServerFor('zsa'), 'https://zsa.methyl.cc');
    });

    test('block explorer, none on the regtests', () {
      expect(defaultBlockExplorerFor('mainnet'), 'https://zecblock.com/tx/{txid}');
      expect(defaultBlockExplorerFor('testnet'), 'https://testnet.zecblock.com/tx/{txid}');
      expect(defaultBlockExplorerFor('regnet'), '');
      expect(defaultBlockExplorerFor('zsa'), '');
    });
  });
}
