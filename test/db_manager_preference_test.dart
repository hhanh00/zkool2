import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:go_router/go_router.dart';
import 'package:path_provider_platform_interface/path_provider_platform_interface.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:zkool/pages/db.dart';
import 'package:zkool/router.dart' show navigatorKey;
import 'package:zkool/src/rust/api/coin.dart';
import 'package:zkool/src/rust/api/db.dart';
import 'package:zkool/src/rust/frb_generated.dart';

import 'platform_fakes.dart';

// The `database` preference names the wallet the next start opens. The
// manager's rename and delete must keep it in step (#1428, V12 F-85).

class _FakeCoin implements Coin {
  @override
  dynamic noSuchMethod(Invocation invocation) => super.noSuchMethod(invocation);
}

class _Api implements RustLibApi {
  final names = <String>['main', 'other'];
  final renamed = <(String, String)>[];
  final deleted = <String>[];

  @override
  Coin crateApiCoinCoinNew({int? defaultCoin}) => _FakeCoin();

  @override
  Future<List<String>> crateApiDbListDbNames({required String dir}) async => List.of(names);

  @override
  Future<List<DbAccountPreview>> crateApiDbListDbAccounts({required String dbFilepath}) async => [];

  @override
  Future<void> crateApiDbRenameDb({required String dbFilepath, required String newDbFilepath, required Coin c}) async {
    final from = _stem(dbFilepath);
    final to = _stem(newDbFilepath);
    renamed.add((from, to));
    names[names.indexOf(from)] = to;
  }

  @override
  Future<void> crateApiDbDeleteDb({required String dbFilepath, required Coin c}) async {
    final name = _stem(dbFilepath);
    deleted.add(name);
    names.remove(name);
  }

  String _stem(String path) => path.split('/').last.replaceAll(RegExp(r'\.db$'), '');

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
      ..addAll(['main', 'other']);
    api.renamed.clear();
    api.deleted.clear();
    dir = Directory.systemTemp.createTempSync('zkool-dbm-');
    PathProviderPlatform.instance = FakePathProvider(dir);
    useInMemoryPrefs({'database': 'main'});
  });
  tearDown(() => dir.deleteSync(recursive: true));

  /// AwesomeDialog animates its header for as long as it is shown, so
  /// pumpAndSettle never settles while a dialog is up; pump a fixed span.
  Future<void> settle(WidgetTester tester) async {
    for (var i = 0; i < 12; i++) {
      await tester.pump(const Duration(milliseconds: 100));
    }
  }

  Future<void> openManager(WidgetTester tester) async {
    final router = GoRouter(navigatorKey: navigatorKey, routes: [
      GoRoute(path: '/', builder: (context, state) => DatabaseManagerPage()),
      GoRoute(path: '/splash', builder: (context, state) => const Scaffold(body: Text('splash'))),
    ]);
    addTearDown(router.dispose);
    await tester.pumpWidget(ProviderScope(child: MaterialApp.router(routerConfig: router)));
    await settle(tester);
    expect(find.text('Database Manager'), findsOneWidget);
    expect(find.byKey(const ValueKey('main')), findsOneWidget);
  }

  Finder row(String name) => find.byKey(ValueKey(name));

  Future<void> check(WidgetTester tester, String name) async {
    await tester.tap(find.descendant(of: row(name), matching: find.byType(Checkbox)));
    await settle(tester);
  }

  Future<void> tapOk(WidgetTester tester) async {
    await tester.tap(find.text('Ok').last);
    await settle(tester);
  }

  Future<void> rename(WidgetTester tester, String from, String to) async {
    await check(tester, from);
    await tester.tap(find.byIcon(Icons.edit));
    await settle(tester);
    await tester.enterText(find.byType(TextField).last, to);
    await tapOk(tester);
  }

  Future<void> delete(WidgetTester tester, List<String> names) async {
    for (final n in names) {
      await check(tester, n);
    }
    await tester.tap(find.byIcon(Icons.delete));
    await settle(tester);
    await tapOk(tester); // the confirmation
    await tapOk(tester); // "Databases deleted"
  }

  /// The preferred row carries the check mark.
  Finder marked(String name) => find.descendant(of: row(name), matching: find.byIcon(Icons.check));

  testWidgets('renaming the preferred wallet moves the preference to the new name', (tester) async {
    await openManager(tester);
    expect(marked('main'), findsOneWidget);

    await rename(tester, 'main', 'renamed');

    expect(api.renamed, [('main', 'renamed')]);
    expect(await SharedPreferencesAsync().getString('database'), 'renamed');
    expect(row('main'), findsNothing);
    expect(marked('renamed'), findsOneWidget);
  });

  testWidgets('renaming another wallet leaves the preference alone', (tester) async {
    await openManager(tester);

    await rename(tester, 'other', 'renamed');

    expect(api.renamed, [('other', 'renamed')]);
    expect(await SharedPreferencesAsync().getString('database'), 'main');
    expect(marked('main'), findsOneWidget);
    expect(marked('renamed'), findsNothing);
  });

  testWidgets('deleting the preferred wallet clears the preference and schedules the manager', (tester) async {
    await openManager(tester);

    await delete(tester, ['main', 'other']);

    expect(api.deleted, unorderedEquals(['main', 'other']));
    final prefs = SharedPreferencesAsync();
    expect(await prefs.getString('database'), isNull);
    expect(await prefs.getBool('recovery'), isTrue);
    expect(find.byIcon(Icons.check), findsNothing);
  });

  testWidgets('deleting another wallet leaves the preference alone', (tester) async {
    await openManager(tester);

    await delete(tester, ['other']);

    expect(api.deleted, ['other']);
    final prefs = SharedPreferencesAsync();
    expect(await prefs.getString('database'), 'main');
    expect(await prefs.getBool('recovery'), isNull);
    expect(marked('main'), findsOneWidget);
  });
}
