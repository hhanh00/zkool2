import 'dart:async';

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:zkool/src/rust/api/account.dart';
import 'package:zkool/src/rust/api/coin.dart';
import 'package:zkool/src/rust/api/sync.dart';
import 'package:zkool/src/rust/frb_generated.dart';
import 'package:zkool/store.dart';

class _FakeCoin implements Coin {
  @override
  dynamic noSuchMethod(Invocation invocation) => super.noSuchMethod(invocation);
}

class _Api implements RustLibApi {
  final streams = <StreamController<SyncEvent>>[];
  final requests = <(List<int>, int)>[];
  int cancels = 0;

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
    requests.add((accounts, currentHeight));
    return stream.stream;
  }

  @override
  Future<int> crateApiNetworkGetCurrentHeight({required Coin c}) async => 100;

  @override
  Future<void> crateApiSyncCancelSync({required Coin c}) async {
    cancels++;
  }

  @override
  Future<void> crateApiSyncFetchTxDetails({required int account, required Coin c}) async {}

  @override
  dynamic noSuchMethod(Invocation invocation) => super.noSuchMethod(invocation);
}

class _Settings extends AppSettingsNotifier {
  final String interval;

  _Settings([this.interval = '']);

  @override
  Future<AppSettings> build() async => AppSettings(
        dbName: '',
        net: '',
        isLightNode: false,
        lwd: '',
        blockExplorer: '',
        syncInterval: interval,
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
        marketInterval: "1d",
        marketStyle: "candles",
      );
}

Account _account(int id) => Account(
      coin: 0,
      id: id,
      name: 'account $id',
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

  Future<ProviderContainer> setup({String interval = '', List<Account> accounts = const []}) async {
    api.streams.clear();
    api.requests.clear();
    api.cancels = 0;
    final container = ProviderContainer(overrides: [
      appSettingsProvider.overrideWith(() => _Settings(interval)),
      getAccountsProvider.overrideWith((ref) async => accounts),
    ]);
    await container.read(appSettingsProvider.future);
    return container;
  }

  test('an autosync interval of 0 means never, and a positive one means behind by that much', () async {
    // 0: a block arrives, nothing is due, even for an account far behind.
    var container = await setup(interval: '0', accounts: [_account(1)]);
    var notifier = container.read(synchronizerProvider.notifier);
    await notifier.syncIfNeeded(1000, now: false);
    expect(api.streams, isEmpty, reason: 'an interval of 0 turns autosync off');
    // "Sync now" still works with autosync off.
    final forced = notifier.syncIfNeeded(1000, now: true);
    await Future<void>.delayed(Duration.zero);
    expect(api.requests.map((r) => r.$1).toList(), [[1]]);
    unawaited(api.streams.single.close());
    await forced;
    container.dispose();

    // 30: due once the account is 30 blocks behind the tip, not before.
    container = await setup(interval: '30', accounts: [_account(1)]);
    addTearDown(container.dispose);
    notifier = container.read(synchronizerProvider.notifier);
    await notifier.syncIfNeeded(129, now: false);
    expect(api.streams, isEmpty, reason: 'the account at 100 is 29 blocks behind');
    final due = notifier.syncIfNeeded(130, now: false);
    await Future<void>.delayed(Duration.zero);
    expect(api.requests.map((r) => r.$1).toList(), [[1]]);
    unawaited(api.streams.single.close());
    await due;
  });

  test('cancelling stays busy until Rust closes, then allows a new sync', () async {
    final container = await setup();
    addTearDown(container.dispose);
    final notifier = container.read(synchronizerProvider.notifier);
    final task = notifier.startSynchronize([], currentHeight: 100);
    await Future<void>.delayed(Duration.zero);
    await notifier.cancelSynchronization();
    expect(container.read(synchronizerProvider).cancelling, isTrue);
    expect(notifier.syncInProgress, isTrue);
    await notifier.startSynchronize([], currentHeight: 100);
    expect(api.streams, hasLength(1));
    await Future<void>.delayed(Duration.zero);
    expect(notifier.syncInProgress, isTrue);
    expect(container.read(synchronizerProvider).cancelling, isTrue);
    unawaited(api.streams.single.close());
    await Future<void>.delayed(Duration.zero);
    await task;
    expect(notifier.retryCount, 0);
    expect(notifier.syncInProgress, isFalse);
    expect(container.read(synchronizerProvider).cancelling, isFalse);
    expect(container.read(synchronizerProvider).end, 0);
    final next = notifier.startSynchronize([], currentHeight: 101);
    await Future<void>.delayed(Duration.zero);
    expect(api.streams, hasLength(2));
    unawaited(api.streams.last.close());
    await Future<void>.delayed(Duration.zero);
    await next;
  });

  test('a request made while a sync runs is queued and runs next with its accounts', () async {
    final container = await setup();
    addTearDown(container.dispose);
    final notifier = container.read(synchronizerProvider.notifier);
    final first = notifier.startSynchronize([_account(1)], currentHeight: 100);
    await Future<void>.delayed(Duration.zero);
    expect(api.streams, hasLength(1));
    // Two requests while busy: merged into one, highest height wins.
    await notifier.startSynchronize([_account(2)], currentHeight: 101);
    await notifier.startSynchronize([_account(3), _account(2)], currentHeight: 90);
    expect(api.streams, hasLength(1));
    unawaited(api.streams.single.close());
    for (var i = 0; i < 5 && api.streams.length < 2; i++) {
      await Future<void>.delayed(const Duration(milliseconds: 10));
    }
    expect(api.streams, hasLength(2));
    expect(api.requests.map((r) => r.$1).toList(), [[1], [2, 3]]);
    expect(api.requests.map((r) => r.$2).toList(), [100, 101]);
    expect(notifier.syncInProgress, isTrue);
    unawaited(api.streams.last.close());
    await first;
    expect(notifier.syncInProgress, isFalse);
    expect(api.streams, hasLength(2));
  });

  test('requested cancellation closes normally without retrying', () async {
    final container = await setup();
    addTearDown(container.dispose);
    final notifier = container.read(synchronizerProvider.notifier);
    final task = notifier.startSynchronize([], currentHeight: 100);
    await Future<void>.delayed(Duration.zero);
    await notifier.cancelSynchronization();
    expect(notifier.syncInProgress, isTrue);
    expect(container.read(synchronizerProvider).cancelling, isTrue);
    await api.streams.single.close();
    await task;
    expect(notifier.retryCount, 0);
    expect(notifier.syncInProgress, isFalse);
    expect(container.read(synchronizerProvider).cancelling, isFalse);
    expect(container.read(synchronizerProvider).end, 0);
    expect(api.streams, hasLength(1));
  });

  test('a manual sync that fails waits before retrying, visibly and still busy, then success exits', () async {
    final container = await setup();
    addTearDown(container.dispose);
    final notifier = container.read(synchronizerProvider.notifier);
    notifier.retryDelay = (_) => const Duration(milliseconds: 200);
    final task = notifier.startSynchronize([], currentHeight: 100);
    await Future<void>.delayed(Duration.zero);
    api.streams.single.addError(Exception('Network failed'));
    await api.streams.single.close();
    await Future<void>.delayed(Duration.zero);
    expect(notifier.retryCount, 1);
    // The wait is part of the sync: busy, and shown as the retry to come.
    expect(notifier.syncInProgress, isTrue);
    final waiting = container.read(synchronizerProvider);
    expect(waiting.kind, SyncKind.manual);
    expect(waiting.attempt, 1);
    expect(waiting.retryInSeconds, 0, reason: 'a 200 ms wait rounds down');
    expect(waiting.cancelling, isFalse);
    expect(api.streams, hasLength(1));
    await Future<void>.delayed(const Duration(milliseconds: 300));
    expect(api.streams, hasLength(2));
    await api.streams.last.close();
    await task;
    expect(notifier.syncInProgress, isFalse);
    expect(container.read(synchronizerProvider).end, 0);
    expect(container.read(synchronizerProvider).kind, isNull);
  });

  // Auto and manual syncs are different things (#1474, part 2).

  test('an auto sync that fails does not retry', () async {
    final container = await setup();
    addTearDown(container.dispose);
    final notifier = container.read(synchronizerProvider.notifier);
    notifier.retryDelay = (_) => const Duration(milliseconds: 50);
    final task = notifier.startSynchronize([_account(1)], currentHeight: 100, kind: SyncKind.auto);
    await Future<void>.delayed(Duration.zero);
    expect(container.read(synchronizerProvider).kind, SyncKind.auto);
    api.streams.single.addError(Exception('stream ended at height 50, 100 asked'));
    await api.streams.single.close();
    await task;
    await Future<void>.delayed(const Duration(milliseconds: 200));
    expect(api.streams, hasLength(1), reason: 'the next schedule retries, not this run');
    expect(notifier.syncInProgress, isFalse);
    expect(container.read(synchronizerProvider).kind, isNull);
  });

  test('a cancel during the backoff wait ends the manual sync without another attempt', () async {
    final container = await setup();
    addTearDown(container.dispose);
    final notifier = container.read(synchronizerProvider.notifier);
    notifier.retryDelay = (_) => const Duration(seconds: 30);
    final task = notifier.startSynchronize([], currentHeight: 100);
    await Future<void>.delayed(Duration.zero);
    api.streams.single.addError(Exception('Network failed'));
    await api.streams.single.close();
    await Future<void>.delayed(Duration.zero);
    expect(container.read(synchronizerProvider).attempt, 1);
    await notifier.cancelSynchronization();
    await task;
    expect(api.streams, hasLength(1));
    expect(notifier.syncInProgress, isFalse);
    expect(container.read(synchronizerProvider).kind, isNull);
    expect(container.read(synchronizerProvider).cancelling, isFalse);
  });

  test('a request made during the backoff wait is queued, not run alongside', () async {
    final container = await setup();
    addTearDown(container.dispose);
    final notifier = container.read(synchronizerProvider.notifier);
    notifier.retryDelay = (_) => const Duration(milliseconds: 200);
    final task = notifier.startSynchronize([_account(1)], currentHeight: 100);
    await Future<void>.delayed(Duration.zero);
    api.streams.single.addError(Exception('Network failed'));
    await api.streams.single.close();
    await Future<void>.delayed(Duration.zero);
    // During the wait: queued, since the sync is still in progress.
    await notifier.startSynchronize([_account(2)], currentHeight: 101, kind: SyncKind.auto);
    expect(api.streams, hasLength(1));
    await Future<void>.delayed(const Duration(milliseconds: 300));
    expect(api.streams, hasLength(2), reason: 'the retry of the first request');
    await api.streams.last.close();
    for (var i = 0; i < 5 && api.streams.length < 3; i++) {
      await Future<void>.delayed(const Duration(milliseconds: 10));
    }
    expect(api.streams, hasLength(3), reason: 'then the queued request');
    expect(api.requests.map((r) => r.$1).toList(), [[1], [1], [2]]);
    await api.streams.last.close();
    await task;
    expect(notifier.syncInProgress, isFalse);
  });

  test('a queued request is manual if either request was manual', () async {
    final container = await setup();
    addTearDown(container.dispose);
    final notifier = container.read(synchronizerProvider.notifier);
    final task = notifier.startSynchronize([_account(1)], currentHeight: 100, kind: SyncKind.auto);
    await Future<void>.delayed(Duration.zero);
    await notifier.startSynchronize([_account(2)], currentHeight: 100, kind: SyncKind.auto);
    await notifier.startSynchronize([_account(3)], currentHeight: 100);
    await api.streams.single.close();
    for (var i = 0; i < 5 && api.streams.length < 2; i++) {
      await Future<void>.delayed(const Duration(milliseconds: 10));
    }
    expect(api.requests.map((r) => r.$1).toList(), [[1], [2, 3]]);
    expect(container.read(synchronizerProvider).kind, SyncKind.manual);
    await api.streams.last.close();
    await task;
  });

  test('a manual sync gives up after the last retry', () async {
    final container = await setup();
    addTearDown(container.dispose);
    final notifier = container.read(synchronizerProvider.notifier);
    notifier.retryDelay = (_) => const Duration(milliseconds: 10);
    final task = notifier.startSynchronize([], currentHeight: 100);
    for (var attempt = 0; attempt <= SynchronizerNotifier.maxRetries; attempt++) {
      for (var i = 0; i < 20 && api.streams.length < attempt + 1; i++) {
        await Future<void>.delayed(const Duration(milliseconds: 5));
      }
      expect(api.streams, hasLength(attempt + 1));
      api.streams.last.addError(Exception('Network failed'));
      await api.streams.last.close();
    }
    await task;
    expect(api.streams, hasLength(SynchronizerNotifier.maxRetries + 1), reason: 'the first run plus ten retries');
    expect(notifier.syncInProgress, isFalse);
    expect(container.read(synchronizerProvider).kind, isNull);
  });
}
