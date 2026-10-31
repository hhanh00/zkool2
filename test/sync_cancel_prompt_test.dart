import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:go_router/go_router.dart';
import 'package:rive/rive.dart' show RiveFile;
import 'package:zkool/router.dart' show navigatorKey;
import 'package:zkool/src/rust/api/coin.dart';
import 'package:zkool/src/rust/api/sync.dart';
import 'package:zkool/src/rust/frb_generated.dart';
import 'package:zkool/store.dart';

// Cancelling from a page (#1474, part 2): an auto sync asks whether to turn
// auto sync off and persists the answer; a manual sync just stops.

class _FakeCoin implements Coin {
  @override
  dynamic noSuchMethod(Invocation invocation) => super.noSuchMethod(invocation);
}

class _Api implements RustLibApi {
  final streams = <StreamController<SyncEvent>>[];
  final props = <(String, String)>[];

  @override
  Coin crateApiCoinCoinNew({int? defaultCoin}) => _FakeCoin();

  @override
  Stream<SyncEvent> crateApiSyncSynchronize({
    required List<int> accounts,
    required int currentHeight,
    required int actionsPerSync,
    required int transparentLimit,
    required int checkpointAge,
    required bool fast,
    required Coin c,
  }) {
    final stream = StreamController<SyncEvent>();
    streams.add(stream);
    return stream.stream;
  }

  // Rust ends the run when asked to stop. Not awaited: the test drives the
  // widget tree under fake async, where a close awaited here would wait on
  // its own listener.
  @override
  Future<void> crateApiSyncCancelSync({required Coin c}) async {
    for (final s in streams) {
      if (!s.isClosed) unawaited(s.close());
    }
  }

  @override
  Future<void> crateApiDbPutProp({required String key, required String value, required Coin c}) async {
    props.add((key, value));
  }

  @override
  Future<void> crateApiSyncFetchTxDetails({required int account, required Coin c}) async {}

  @override
  dynamic noSuchMethod(Invocation invocation) => super.noSuchMethod(invocation);
}

class _Settings extends AppSettingsNotifier {
  @override
  Future<AppSettings> build() async => AppSettings(
        dbName: '',
        net: '',
        isLightNode: false,
        lwd: '',
        blockExplorer: '',
        syncInterval: '30',
        actionsPerSync: '100',
        transport: 0,
        proxy: '',
        coingecko: '',
        recovery: false,
        needPin: false,
        pinUnlockedAt: DateTime(2026),
        offline: false,
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

class _Started {
  late Future<void> run;
}

void main() {
  late _Api api;
  setUpAll(() {
    api = _Api();
    RustLib.initMock(api: api);
    // confirmDialog is an AwesomeDialog with a Rive header; take Rive's one
    // initialisation failure in a guarded zone, as the other page tests do.
    runZonedGuarded(() => unawaited(RiveFile.initialize()), (_, __) {});
  });
  setUp(() {
    api.streams.clear();
    api.props.clear();
  });
  tearDownAll(RustLib.dispose);

  /// A page with the cancel control, and the container to drive the sync.
  /// Routed, since confirmDialog dismisses through GoRouter, and on the
  /// app's navigator key, since the synchronizer's snackbars go through it.
  Future<ProviderContainer> pumpPage(WidgetTester tester) async {
    late ProviderContainer container;
    final router = GoRouter(navigatorKey: navigatorKey, routes: [
      GoRoute(
        path: '/',
        builder: (context, state) => Scaffold(
          body: Consumer(
            builder: (context, ref, _) => ElevatedButton(
              onPressed: () => cancelSyncFromPage(context, ref),
              child: const Text('Cancel'),
            ),
          ),
        ),
      ),
    ]);
    addTearDown(router.dispose);
    await tester.pumpWidget(ProviderScope(
      overrides: [
        appSettingsProvider.overrideWith(_Settings.new),
        getAccountsProvider.overrideWith((ref) async => []),
      ],
      child: Builder(builder: (context) {
        container = ProviderScope.containerOf(context);
        return MaterialApp.router(routerConfig: router);
      }),
    ));
    await tester.pumpAndSettle();
    await container.read(appSettingsProvider.future);
    return container;
  }

  Future<void> settle(WidgetTester tester) async {
    for (var i = 0; i < 10; i++) {
      await tester.pump(const Duration(milliseconds: 200));
    }
  }

  /// Starts a sync of [kind] with real async, so its first steps run. The
  /// run's future comes back in a holder: an async function cannot return a
  /// future as its value.
  Future<_Started> start(WidgetTester tester, ProviderContainer container, SyncKind kind) async {
    final started = _Started();
    await tester.runAsync(() async {
      started.run = container.read(synchronizerProvider.notifier).startSynchronize([], currentHeight: 100, kind: kind);
      await Future<void>.delayed(const Duration(milliseconds: 20));
    });
    await tester.pump();
    return started;
  }

  /// Lets the run finish with real async, then pumps the result.
  Future<void> finish(WidgetTester tester, _Started started) async {
    await tester.runAsync(() => started.run);
    await settle(tester);
  }

  testWidgets('cancelling an auto sync offers to turn auto sync off, and Disable persists it', (tester) async {
    final container = await pumpPage(tester);
    final run = await start(tester, container, SyncKind.auto);
    expect(container.read(synchronizerProvider).kind, SyncKind.auto);

    await tester.tap(find.text('Cancel'));
    await finish(tester, run);
    // The dialog's title is not a plain Text; its body and buttons are.
    expect(find.textContaining('Turn auto sync off?'), findsOneWidget);
    expect(find.text('Disable'), findsOneWidget);
    expect(api.streams.single.isClosed, isTrue, reason: 'the run was stopped before the question');

    await tester.tap(find.text('Disable'));
    await settle(tester);
    expect(api.props, [('sync_interval', '0')]);
    expect(container.read(appSettingsProvider).requireValue.syncInterval, '0');
    expect(container.read(synchronizerProvider).kind, isNull);
  });

  testWidgets('Keep leaves auto sync on', (tester) async {
    final container = await pumpPage(tester);
    final run = await start(tester, container, SyncKind.auto);

    await tester.tap(find.text('Cancel'));
    await finish(tester, run);
    await tester.tap(find.text('Keep'));
    await settle(tester);
    expect(api.props, isEmpty);
    expect(container.read(appSettingsProvider).requireValue.syncInterval, '30');
  });

  testWidgets('cancelling a manual sync asks nothing', (tester) async {
    final container = await pumpPage(tester);
    final run = await start(tester, container, SyncKind.manual);
    expect(container.read(synchronizerProvider).kind, SyncKind.manual);

    await tester.tap(find.text('Cancel'));
    await finish(tester, run);
    expect(find.textContaining('Turn auto sync off?'), findsNothing);
    expect(find.text('Disable'), findsNothing);
    expect(api.props, isEmpty);
    expect(container.read(synchronizerProvider).kind, isNull);
  });
}
