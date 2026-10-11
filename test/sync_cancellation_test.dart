import 'dart:async';

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:zkool/src/rust/api/coin.dart';
import 'package:zkool/src/rust/api/sync.dart';
import 'package:zkool/src/rust/frb_generated.dart';
import 'package:zkool/store.dart';

class _Api implements RustLibApi {
  final streams = <StreamController<SyncProgress>>[];
  int cancels = 0;

  @override
  Coin crateApiCoinCoinNew({int? defaultCoin}) => const Coin.raw(
        coin: 0,
        account: 0,
        dbFilepath: '',
        url: '',
        serverType: 0,
        transport: 0,
        proxy: '',
      );

  @override
  Stream<SyncProgress> crateApiSyncSynchronize({
    required List<int> accounts,
    required int currentHeight,
    required int actionsPerSync,
    required int transparentLimit,
    required int checkpointAge,
    required bool fast,
    required Coin c,
  }) {
    final stream = StreamController<SyncProgress>();
    streams.add(stream);
    return stream.stream;
  }

  @override
  Future<int> crateApiNetworkGetCurrentHeight({required Coin c}) async => 100;

  @override
  Future<void> crateApiSyncCancelSync() async {
    cancels++;
  }

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

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();
  final api = _Api();
  setUpAll(() => RustLib.initMock(api: api));
  tearDownAll(RustLib.dispose);

  Future<ProviderContainer> setup() async {
    api.streams.clear();
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
