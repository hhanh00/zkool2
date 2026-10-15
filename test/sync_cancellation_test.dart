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
  @override
  Future<AppSettings> build() async => AppSettings(
        dbName: '',
        net: '',
        isLightNode: false,
        lwd: '',
        blockExplorer: '',
        syncInterval: '',
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

  Future<ProviderContainer> setup() async {
    api.streams.clear();
    api.requests.clear();
    api.cancels = 0;
    final container = ProviderContainer(overrides: [
      appSettingsProvider.overrideWith(_Settings.new),
    ]);
    await container.read(appSettingsProvider.future);
    return container;
  }

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

  test('failure waits before retrying, then success exits', () async {
    final container = await setup();
    addTearDown(container.dispose);
    final notifier = container.read(synchronizerProvider.notifier);
    final task = notifier.startSynchronize([], currentHeight: 100);
    await Future<void>.delayed(Duration.zero);
    api.streams.single.addError(Exception('Network failed'));
    await api.streams.single.close();
    await Future<void>.delayed(Duration.zero);
    expect(notifier.retryCount, 1);
    expect(notifier.syncInProgress, isFalse);
    expect(container.read(synchronizerProvider).cancelling, isFalse);
    expect(api.streams, hasLength(1));
    await Future<void>.delayed(const Duration(seconds: 33));
    expect(api.streams, hasLength(2));
    await api.streams.last.close();
    await task;
    expect(notifier.syncInProgress, isFalse);
    expect(container.read(synchronizerProvider).end, 0);
  }, timeout: const Timeout(Duration(minutes: 1)));
}
