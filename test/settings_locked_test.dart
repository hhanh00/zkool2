import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter_form_builder/flutter_form_builder.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:package_info_plus/package_info_plus.dart';
import 'package:path_provider_platform_interface/path_provider_platform_interface.dart';
import 'package:zkool/settings.dart';
import 'package:zkool/src/rust/api/coin.dart';
import 'package:zkool/src/rust/api/sapling.dart';
import 'package:zkool/src/rust/frb_generated.dart';
import 'package:zkool/store.dart';

import 'platform_fakes.dart';

// The settings page has no PIN guard and stays usable while locked. The
// controls that could defeat the lock or reach seeds and the wallet file
// refuse input meanwhile, and a stored proxy credential is not readable
// (#1572); the operational settings stay editable.

class _FakeCoin implements Coin {
  @override
  dynamic noSuchMethod(Invocation invocation) => super.noSuchMethod(invocation);
}

class _Api implements RustLibApi {
  @override
  Coin crateApiCoinCoinNew({int? defaultCoin}) => _FakeCoin();

  @override
  SaplingParamsStatus crateApiSaplingCheckSaplingParams() => const SaplingParamsStatus(downloaded: true);

  @override
  bool crateApiNetworkIsValidNymUrl({required String url}) => false;

  @override
  dynamic noSuchMethod(Invocation invocation) => super.noSuchMethod(invocation);
}

class _Locked extends Lifecycle {
  @override
  Future<bool> build() async => true;
}

class _Unlocked extends Lifecycle {
  @override
  Future<bool> build() async => false;
}

AppSettings _settings() => AppSettings(
      dbName: 'zkool',
      net: 'main',
      isLightNode: false,
      lwd: 'https://zec.rocks',
      blockExplorer: '',
      syncInterval: '0',
      actionsPerSync: '',
      transport: 3,
      proxy: 'socks5h://user:pw@127.0.0.1:9050',
      coingecko: '',
      recovery: false,
      needPin: true,
      pinUnlockedAt: DateTime(2026),
      offline: true,
      getFx: false,
      qrSettings: QRSettings(enabled: false, size: 100, ecLevel: 0, delay: 0, repair: 0),
      vault: true,
      expertMode: true,
      paletteName: '',
      darkMode: false,
      transactionTableMode: false,
      collapsePoolBalances: false,
      currency: '',
      votingConfigUrl: '',
      marketInterval: '1d',
      marketStyle: 'candles',
    );

void main() {
  setUpAll(() {
    RustLib.initMock(api: _Api());
    coinContext.set(coin: _FakeCoin());
    PathProviderPlatform.instance = FakePathProvider(Directory.systemTemp.createTempSync('zkool-settings'));
    PackageInfo.setMockInitialValues(
      appName: 'zkool',
      packageName: 'zkool',
      version: '0.0.0',
      buildNumber: '0',
      buildSignature: '',
      installerStore: null,
    );
  });
  tearDownAll(RustLib.dispose);

  Future<void> open(WidgetTester tester, {required bool locked}) async {
    await tester.pumpWidget(ProviderScope(
      overrides: [lifecycleProvider.overrideWith(locked ? _Locked.new : _Unlocked.new)],
      child: MaterialApp(home: SettingsForm(_settings(), onChanged: (_) {})),
    ));
    for (var i = 0; i < 10; i++) {
      await tester.pump(const Duration(milliseconds: 100));
    }
  }

  // The page is a scroll view; most controls sit below the fold, so the
  // finders include offstage widgets.
  Switch switchBeside(WidgetTester tester, String label) => tester.widget<Switch>(find.descendant(
        of: find.ancestor(of: find.text(label, skipOffstage: false), matching: find.byType(Row, skipOffstage: false)).first,
        matching: find.byType(Switch, skipOffstage: false),
        skipOffstage: false,
      ));

  IconButton buttonBeside(WidgetTester tester, String label) => tester.widget<IconButton>(find.descendant(
        of: find.ancestor(of: find.text(label, skipOffstage: false), matching: find.byType(Row, skipOffstage: false)).first,
        matching: find.byType(IconButton, skipOffstage: false),
        skipOffstage: false,
      ));

  // An IconButton wraps its icon in the tooltip, so the button is the
  // tooltip's ancestor.
  IconButton buttonWithTooltip(WidgetTester tester, String tooltip) => tester.widget<IconButton>(find
      .ancestor(of: find.byTooltip(tooltip, skipOffstage: false), matching: find.byType(IconButton, skipOffstage: false))
      .first);

  FormBuilderTextField field(WidgetTester tester, String name) => tester.widget<FormBuilderTextField>(
      find.byWidgetPredicate((w) => w is FormBuilderTextField && w.name == name, skipOffstage: false));

  testWidgets('while locked, the sensitive controls refuse input and the proxy is hidden', (tester) async {
    await open(tester, locked: true);
    expect(switchBeside(tester, 'Pin Lock').onChanged, isNull);
    expect(switchBeside(tester, 'Cloud Vault').onChanged, isNull);
    expect(buttonBeside(tester, 'Recover Accounts from Vault').onPressed, isNull, reason: 'Recover Accounts from Vault');
    expect(buttonBeside(tester, 'Compress Vault').onPressed, isNull, reason: 'Compress Vault');
    expect(buttonWithTooltip(tester, 'Database Manager').onPressed, isNull);
    expect(field(tester, 'proxy').enabled, isFalse);
    expect(field(tester, 'proxy').obscureText, isTrue);
    expect(field(tester, 'lwd').enabled, isTrue, reason: 'operational settings stay editable');
  });

  testWidgets('while unlocked, every control takes input', (tester) async {
    await open(tester, locked: false);
    expect(switchBeside(tester, 'Pin Lock').onChanged, isNotNull);
    expect(switchBeside(tester, 'Cloud Vault').onChanged, isNotNull);
    expect(buttonBeside(tester, 'Recover Accounts from Vault').onPressed, isNotNull);
    expect(buttonBeside(tester, 'Compress Vault').onPressed, isNotNull);
    expect(buttonWithTooltip(tester, 'Database Manager').onPressed, isNotNull);
    expect(field(tester, 'proxy').enabled, isTrue);
    expect(field(tester, 'proxy').obscureText, isFalse);
  });
}
