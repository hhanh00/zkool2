import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:zkool/src/rust/api/coin.dart';
import 'package:zkool/src/rust/frb_generated.dart';
import 'package:zkool/store.dart';

// Every settings build and every save reconfigures the auto FX fetch. It
// used to add a periodic timer each time without cancelling the previous
// one, so each visit to the settings page left one more price request per
// minute running until the process exited (#1530).

class _FakeCoin implements Coin {
  @override
  dynamic noSuchMethod(Invocation invocation) => super.noSuchMethod(invocation);
}

class _Api implements RustLibApi {
  int fetches = 0;

  @override
  Coin crateApiCoinCoinNew({int? defaultCoin}) => _FakeCoin();

  @override
  Future<double> crateApiNetworkGetCoingeckoPrice({required String api, required String currency, required Coin c}) async {
    fetches += 1;
    return 1.0;
  }

  @override
  dynamic noSuchMethod(Invocation invocation) => super.noSuchMethod(invocation);
}

void main() {
  final api = _Api();
  setUpAll(() {
    RustLib.initMock(api: api);
    coinContext.set(coin: _FakeCoin());
  });
  setUp(() => api.fetches = 0);
  tearDownAll(RustLib.dispose);

  // Inside testWidgets the periodic timer runs on the test clock, which
  // tester.pump advances.
  PriceNotifier notifier(WidgetTester tester) {
    final container = ProviderContainer();
    addTearDown(container.dispose);
    return container.read(priceProvider.notifier);
  }

  testWidgets('reconfiguring twice leaves one periodic fetch', (tester) async {
    final price = notifier(tester);
    price.setAutoFetchFx(true, 'api', 'usd');
    price.setAutoFetchFx(true, 'api', 'usd');
    await tester.pump();
    expect(api.fetches, 2, reason: 'one immediate fetch per call');
    await tester.pump(const Duration(minutes: 3));
    expect(api.fetches, 5, reason: 'three minutes of one timer, not two');
    price.setAutoFetchFx(false, 'api', 'usd');
  });

  testWidgets('a disable that lands during the first fetch stops the timer', (tester) async {
    final price = notifier(tester);
    price.setAutoFetchFx(true, 'api', 'usd');
    price.setAutoFetchFx(false, 'api', 'usd');
    await tester.pump();
    await tester.pump(const Duration(minutes: 3));
    expect(api.fetches, 1, reason: 'the immediate fetch, then nothing');
  });

  testWidgets('disabling cancels the running timer', (tester) async {
    final price = notifier(tester);
    price.setAutoFetchFx(true, 'api', 'usd');
    await tester.pump(const Duration(minutes: 1));
    expect(api.fetches, 2);
    price.setAutoFetchFx(false, 'api', 'usd');
    await tester.pump(const Duration(minutes: 3));
    expect(api.fetches, 2);
  });
}
