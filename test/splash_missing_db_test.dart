import 'dart:async';
import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:go_router/go_router.dart';
import 'package:path_provider_platform_interface/path_provider_platform_interface.dart';
import 'package:zkool/main.dart' show appKey;
import 'package:zkool/pages/splash.dart';
import 'package:zkool/router.dart' show navigatorKey;
import 'package:zkool/src/rust/api/coin.dart';
import 'package:zkool/src/rust/api/init.dart';
import 'package:zkool/src/rust/frb_generated.dart';

import 'platform_fakes.dart';

// The splash opens the wallet named by the `database` preference, and the
// Rust open creates a missing file. After a rename or delete of that wallet
// the start must go to the manager instead (#1428, V12 F-85).

class _FakeCoin implements Coin {
  @override
  dynamic noSuchMethod(Invocation invocation) => super.noSuchMethod(invocation);
}

class _Api implements RustLibApi {
  final opened = <String>[];

  @override
  Coin crateApiCoinCoinNew({int? defaultCoin}) => _FakeCoin();

  // The splash subscribes to the log at start; nothing is logged here.
  @override
  Stream<LogMessage> crateApiInitSetLogStream() => StreamController<LogMessage>().stream;

  @override
  dynamic noSuchMethod(Invocation invocation) => super.noSuchMethod(invocation);
}

void main() {
  late Directory dir;

  setUpAll(() {
    silenceRiveHeaders();
    RustLib.initMock(api: _Api());
  });
  tearDownAll(RustLib.dispose);

  setUp(() {
    dir = Directory.systemTemp.createTempSync('zkool-splash-');
    PathProviderPlatform.instance = FakePathProvider(dir);
  });
  tearDown(() => dir.deleteSync(recursive: true));

  group('missingPreferredDatabase', () {
    test('is null when nothing is preferred yet: the first start creates the default wallet', () async {
      useInMemoryPrefs({});
      expect(await missingPreferredDatabase(), isNull);
    });

    test('is null when the preferred wallet file is there', () async {
      useInMemoryPrefs({'database': 'main'});
      File('${dir.path}/main.db').writeAsStringSync('');
      expect(await missingPreferredDatabase(), isNull);
    });

    test('names the preferred wallet when its file is gone', () async {
      useInMemoryPrefs({'database': 'main'});
      expect(await missingPreferredDatabase(), 'main');
    });
  });

  testWidgets('the splash tells the user and opens the manager instead of creating the wallet', (tester) async {
    useInMemoryPrefs({'database': 'main'});
    final router = GoRouter(navigatorKey: navigatorKey, initialLocation: '/splash', routes: [
      GoRoute(path: '/splash', builder: (context, state) => const SplashPage()),
      GoRoute(path: '/database_manager', builder: (context, state) => const Scaffold(body: Text('manager'))),
    ]);
    addTearDown(router.dispose);
    // The log listener looks the provider scope up through appKey, as the app does.
    await tester.pumpWidget(ProviderScope(child: MaterialApp.router(key: appKey, routerConfig: router)));
    for (var i = 0; i < 12; i++) {
      await tester.pump(const Duration(milliseconds: 100));
    }

    expect(find.text("Wallet 'main' was not found"), findsOneWidget);
    await tester.tap(find.text('Ok'));
    for (var i = 0; i < 12; i++) {
      await tester.pump(const Duration(milliseconds: 100));
    }

    expect(find.text('manager'), findsOneWidget);
    expect(File('${dir.path}/main.db').existsSync(), isFalse);
  });
}
