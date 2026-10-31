import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:zkool/src/rust/api/coin.dart';
import 'package:zkool/src/rust/frb_generated.dart';
import 'package:zkool/store.dart';

import 'platform_fakes.dart';

// Saving the settings is a long sequence of preference and database writes.
// When one fails, the save must surface the failure rather than drop it, and
// the provider must reload what was actually stored (#1511, V12 F-65).

class _FakeCoin implements Coin {
  @override
  dynamic noSuchMethod(Invocation invocation) => super.noSuchMethod(invocation);
}

/// A Rust API whose `proxy` property write fails; every other write lands.
class _Api implements RustLibApi {
  final written = <String, String>{};

  @override
  Coin crateApiCoinCoinNew({int? defaultCoin}) => _FakeCoin();

  @override
  Future<void> crateApiDbPutProp({required String key, required String value, required Coin c}) async {
    if (key == 'proxy') throw Exception('database is busy');
    written[key] = value;
  }

  @override
  Future<String?> crateApiDbGetProp({required String key, required Coin c}) async => written[key];

  @override
  Future<String> crateApiNetworkGetNetworkName({required Coin c}) async => 'regtest';

  @override
  void crateApiInitSetExpertMode({required bool enabled}) {}

  @override
  dynamic noSuchMethod(Invocation invocation) => super.noSuchMethod(invocation);
}

void main() {
  final api = _Api();
  setUpAll(() => RustLib.initMock(api: api));
  tearDownAll(RustLib.dispose);

  test('a failing write surfaces and the reload shows what was stored', () async {
    useInMemoryPrefs({});
    coinContext.set(coin: _FakeCoin());
    final container = ProviderContainer();
    addTearDown(container.dispose);
    final notifier = container.read(appSettingsProvider.notifier);
    final loaded = await container.read(appSettingsProvider.future);
    expect(loaded.transport, 0);

    final draft = loaded.copyWith(lwd: 'https://new.example', transport: 3, proxy: 'socks5h://127.0.0.1:9050');
    await expectLater(notifier.save(draft), throwsA(isA<Exception>()));

    expect(api.written['lwd'], 'https://new.example', reason: 'writes before the failure landed');
    expect(api.written.containsKey('currency'), isFalse, reason: 'writes after the failure did not run');

    notifier.reload();
    final reloaded = await container.read(appSettingsProvider.future);
    expect(reloaded.transport, 3, reason: 'the transport preference was written before the failure');
    expect(reloaded.lwd, 'https://zec.rocks', reason: 'with no database open the loader falls back to the default');
  });
}
