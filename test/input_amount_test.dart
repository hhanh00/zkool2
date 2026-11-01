import 'package:flutter/material.dart';
import 'package:flutter_form_builder/flutter_form_builder.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:zkool/src/rust/api/coin.dart';
import 'package:zkool/src/rust/frb_generated.dart';
import 'package:zkool/store.dart';
import 'package:zkool/widgets/input_amount.dart';

class _FakeCoin implements Coin {
  @override
  dynamic noSuchMethod(Invocation invocation) => super.noSuchMethod(invocation);
}

class _Api implements RustLibApi {
  @override
  Coin crateApiCoinCoinNew({int? defaultCoin}) => _FakeCoin();

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
        currency: 'usd',
        votingConfigUrl: '',
        marketInterval: "1d",
        marketStyle: "candles",
      );
}

/// The widget on its own form, as the send page hosts it.
Future<void> pumpAmount(WidgetTester tester) async {
  await tester.pumpWidget(
    ProviderScope(
      overrides: [appSettingsProvider.overrideWith(_Settings.new)],
      child: MaterialApp(
        home: Scaffold(
          body: FormBuilder(
            key: GlobalKey<FormBuilderState>(),
            child: const InputAmount(name: 'amount'),
          ),
        ),
      ),
    ),
  );
  await tester.pumpAndSettle();
}

Finder field(String name) => find.byWidgetPredicate((w) => w is FormBuilderTextField && w.name == name);

String? valueOf(WidgetTester tester, String name) =>
    (tester.state(field(name)) as FormBuilderFieldState).value as String?;

/// Types into a field and lets the post-frame conversion run.
Future<void> type(WidgetTester tester, String name, String text) async {
  await tester.enterText(field(name), text);
  await tester.pump();
  await tester.pump();
}

/// The inner form of the widget, the one the fiat and Fx fields live on.
FormBuilderState innerForm(WidgetTester tester) => tester.state<FormBuilderState>(find.byType(FormBuilder).last);

void main() {
  setUpAll(() => RustLib.initMock(api: _Api()));
  tearDownAll(RustLib.dispose);

  testWidgets('an Fx rate of zero is refused and a fiat amount typed after it changes nothing', (tester) async {
    await pumpAmount(tester);
    await type(tester, 'fx', '0');
    await type(tester, 'fiat', '1');

    expect(tester.takeException(), isNull);
    expect(valueOf(tester, 'zat'), anyOf(isNull, isEmpty));
    expect(innerForm(tester).validate(), isFalse);
    await tester.pump();
    expect(find.text('Invalid rate'), findsOneWidget);
  });

  testWidgets('a fiat amount that does not parse changes nothing', (tester) async {
    await pumpAmount(tester);
    await type(tester, 'fx', '50');
    await type(tester, 'fiat', 'abc');

    expect(tester.takeException(), isNull);
    expect(valueOf(tester, 'zat'), anyOf(isNull, isEmpty));
  });

  testWidgets('a fiat amount converts to ZEC at the rate', (tester) async {
    await pumpAmount(tester);
    await type(tester, 'fx', '50');
    await type(tester, 'fiat', '1');

    expect(tester.takeException(), isNull);
    expect(valueOf(tester, 'zat'), '0.02000000');
  });

  testWidgets('a ZEC amount still fills the fiat field', (tester) async {
    await pumpAmount(tester);
    await type(tester, 'fx', '50');
    await type(tester, 'zat', '1');

    expect(valueOf(tester, 'fiat'), '50.000');
  });
}
