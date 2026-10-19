/// Integration test: the pages the everyday wallet test leaves out, one app
/// and no peers.
///
/// Opens every page in scope that `wallet_ui_test.dart` never reaches and taps
/// through its actions, checking each one's effect: settings and its
/// sub-pages, the account view tabs, contacts, the address list, the error
/// display and the database manager. The flows the base test already covers
/// (creating accounts, receiving, paying) are not repeated: the accounts are
/// created through the bridge and `tests/tests/test_wallet_ext_ui.py` funds
/// one of them directly, with notes in every pool, a dust note and a memo.
///
/// Every setting the test changes is changed back through the same page, and
/// the stored value is checked both times, so the chain, the server and the
/// developer's own preferences are as before when the run ends.
///
/// Two accounts: A (funded, the pages are opened on it) and B (a contact).
library;


import 'package:flutter/material.dart';
import 'package:flutter_form_builder/flutter_form_builder.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:zkool/main.dart';
import 'package:zkool/router.dart';
import 'package:zkool/src/rust/api/account.dart';
import 'package:zkool/src/rust/api/db.dart';
import 'package:zkool/store.dart';

import 'support.dart';

const _accountA = "Alice";
const _accountB = "Bob";

// Values no setting has by default, so a saved change is told from the original.
const _proxy = "socks5h://127.0.0.1:19050";
const _votingUrl = "https://example.invalid/voting-config.json";
const _actionsPerSync = "123";
const _coingeckoKey = "wallet-ext-key";
const _blockExplorer = "https://example.invalid/tx/";
const _qrDelay = "250";
const _qrRepair = "7";

const _transportLabels = {0: "Direct", 1: "Tor", 2: "Nym", 3: "Proxy"};

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();
  // debugPrint throttles and drops queued lines when the test dies; the log
  // must show the last steps before a failure.
  // ignore: avoid_print
  debugPrint = (String? message, {int? wrapWidth}) => print(message ?? "");

  testWidgets("Remaining wallet pages through the Flutter UI", (tester) async {
    final docsDir = await appDocumentsDir();
    final rendezvous = Rendezvous.fromEnvironment(docsDir);
    expect(rendezvous.dir.existsSync(), isTrue,
        reason: "rendezvous dir ${rendezvous.dir.path} missing — "
            "start this test from test_wallet_ext_ui.py",);

    final config = rendezvous.readConfig();
    final dbPath = config["db_path"] as String;
    final coingeckoUrl = config["coingecko_url"] as String;
    final fundTaddr = config["fund_taddr"] as String;
    final fundSaddr = config["fund_saddr"] as String;
    final fundOaddr = config["fund_oaddr"] as String;
    final fundDust = config["fund_dust"] as String;
    // The lock screen opens on a tap where the device has no biometrics
    // (Linux, CI); on macOS it prompts Touch ID, so the lock step is skipped.
    final biometricGate = config["biometric_gate"] as bool? ?? false;
    expect(dbPath, contains("regtest"));

    final prefs = SharedPreferencesAsync();
    final savedPrefs = await SavedPrefs.forceTestValues(prefs);
    // The settings page shows and edits the key; start from a known value.
    final savedApiKey = await prefs.getString("coingecko");
    await prefs.setString("coingecko", "");
    // The settings phase puts these back through the UI; a run that dies
    // halfway must not leave the developer's app with the test values.
    final savedTransport = await prefs.getInt("transport");
    final savedDarkMode = await prefs.getBool("dark_mode");
    final savedPalette = await prefs.getString("palette_name");

    void phase(String name) {
      debugPrint("[wallet-ext] phase: $name");
      rendezvous.publish("phase", name);
    }

    // The runner drops log lines printed just before a failure; a step written
    // to the rendezvous file survives, and pytest prints it.
    void step(String what) {
      debugPrint("[wallet-ext] step: $what");
      rendezvous.publish("step", what);
    }

    try {
      // ── Set up ───────────────────────────────────────────────────────────
      // Accounts come from the bridge: the new-account page is the base
      // test's job, and the budget here goes to the pages under test.
      final (a, b) = (await tester.runAsync(() async {
        await openTestWallet(
          dbPath: dbPath,
          lwd: config["lwd"] as String,
          docsDir: docsDir.path,
          props: {"coingecko_url": coingeckoUrl},
        );
        return (await _newAccount(_accountA), await _newAccount(_accountB));
      }))!;
      debugPrint("[wallet-ext] accounts A=$a B=$b");

      final appRouter = router(true, false, initialLocation: "/accounts");
      await tester.pumpWidget(ZkoolApp(router: appRouter));
      await tester.pump(const Duration(seconds: 1));
      final container = ProviderScope.containerOf(tester.element(find.byType(MaterialApp)));
      await primeProviders(tester, container);
      await waitForAccountList(tester);

      // ── Fund ─────────────────────────────────────────────────────────────
      // pytest pays A at every pool in one transaction, plus a dust note.
      phase("fund");
      final aAddresses = (await tester.runAsync(
        () => getAddresses(uaPools: 7, account: a, c: coinContext.coin),
      ))!;
      rendezvous.publish("a_taddr", aAddresses.taddr!);
      rendezvous.publish("a_saddr", aAddresses.saddr!);
      rendezvous.publish("a_oaddr", aAddresses.oaddr!);
      appRouter.go(accountRoute("/", a));
      await waitForAccountView(tester, _accountA);
      await waitForBalance(tester, a,
          atLeast: zecToZat(fundTaddr) + zecToZat(fundSaddr) + zecToZat(fundOaddr) + zecToZat(fundDust),
          what: "A to receive the funding",);

      // ── Settings ─────────────────────────────────────────────────────────
      // The page saves when it is left, so the fields are changed together,
      // the page left, the stored values checked, and the same again to put
      // the originals back. The light-node switch and the server are only
      // toggled on the page: saved, they would point the sync elsewhere.
      phase("settings");
      final original = container.read(appSettingsProvider).value!;
      final palette = original.paletteName == "red" ? "blue" : "red";
      final changed = original.copyWith(
        transport: 3,
        proxy: _proxy,
        votingConfigUrl: _votingUrl,
        actionsPerSync: _actionsPerSync,
        syncInterval: "0",
        offline: !original.offline,
        getFx: !original.getFx,
        coingecko: _coingeckoKey,
        blockExplorer: _blockExplorer,
        qrSettings: original.qrSettings.copyWith(
          enabled: !original.qrSettings.enabled,
          delay: int.parse(_qrDelay),
          repair: int.parse(_qrRepair),
        ),
        darkMode: !original.darkMode,
        paletteName: palette,
      );

      step("open settings from the account menu");
      await openMenuItem(tester, "Settings");
      await _waitForSettings(tester);

      step("light node switch relabels the server field");
      expect(find.text("Light Node Server"), findsOneWidget);
      await _tapSwitch(tester, switchNamed("light"));
      expect(find.text("Full Node Server"), findsOneWidget);
      await _tapSwitch(tester, switchNamed("light"));
      expect(find.text("Light Node Server"), findsOneWidget);

      step("server picker: regtest has no public list");
      await tapTooltip(tester, "Select from server list");
      await pumpUntil(tester, () => find.text("Select Lightwalletd Server").evaluate().isNotEmpty,
          timeout: shortTimeout, what: "the server picker",);
      await pumpUntil(tester, () => find.text("No servers available").evaluate().isNotEmpty,
          timeout: shortTimeout, what: "the empty server list",);
      await _back(tester);
      await _waitForSettings(tester);
      expect(fieldValue(tester, "lwd"), original.lwd, reason: "leaving the picker keeps the server");

      step("proxy transport enables the proxy field");
      expect(_proxyEnabled(tester), original.transport == 3);
      await _selectTransport(tester, "Proxy", proxyEnabled: true);
      await _enterField(tester, "proxy", _proxy);
      await _enterField(tester, "voting_config_url", _votingUrl);
      await _enterField(tester, "actions_per_sync", _actionsPerSync);

      step("cancel sync zeroes the autosync interval");
      await tapTooltip(tester, "This will cancel the current sync and disable AutoSync");
      await answerDialog(tester, "Do you want to cancel the current sync? AutoSync will be disabled too", "Ok");
      expect(fieldValue(tester, "autosync"), "0");

      await _tapSwitch(tester, switchNamed("offline"));
      await _tapSwitch(tester, switchNamed("fx"));
      await _enterField(tester, "coingecko", _coingeckoKey);
      await _enterField(tester, "block_explorer", _blockExplorer);

      step("QR settings page");
      await _editQrSettings(tester, enabled: changed.qrSettings.enabled, delay: _qrDelay, repair: _qrRepair);

      step("theme page saves at once");
      await _setTheme(tester, container, darkMode: changed.darkMode, palette: palette);

      step("log page");
      await tapTooltip(tester, "View Log");
      await pumpUntil(tester, () => find.text("Log").evaluate().isNotEmpty,
          timeout: shortTimeout, what: "the log page",);
      await _back(tester);
      await _waitForSettings(tester);

      step("expert mode: long-press the version");
      expect(find.text("expert"), findsNothing);
      await _longPressVersion(tester);
      expect(find.text("expert"), findsOneWidget);
      expect(find.text("Cloud Vault"), findsOneWidget, reason: "expert mode shows the vault switch");
      await _longPressVersion(tester);
      expect(find.text("expert"), findsNothing);

      step("leave settings: saves the changes");
      await _back(tester);
      await waitForAccountView(tester, _accountA);
      await _expectSettingsSaved(tester, container, prefs, changed, what: "the changed settings");

      if (!biometricGate) {
        step("lock the app, unlock with a tap");
        await openMenuItem(tester, "Settings");
        await _waitForSettings(tester);
        await tapTooltip(tester, "Lock");
        await _back(tester);
        await pumpUntil(tester, () => find.text("Locked").evaluate().isNotEmpty,
            timeout: shortTimeout, what: "the lock screen",);
        await tester.tap(find.byType(InkWell).first);
        await waitForAccountView(tester, _accountA);
      }

      step("put the settings back");
      await openMenuItem(tester, "Settings");
      await _waitForSettings(tester);
      await _selectTransport(tester, _transportLabels[original.transport]!, proxyEnabled: original.transport == 3);
      await _enterField(tester, "proxy", original.proxy);
      await _enterField(tester, "voting_config_url", original.votingConfigUrl);
      await _enterField(tester, "actions_per_sync", original.actionsPerSync);
      await _enterField(tester, "autosync", original.syncInterval);
      await _tapSwitch(tester, switchNamed("offline"));
      await _tapSwitch(tester, switchNamed("fx"));
      await _enterField(tester, "coingecko", original.coingecko);
      await _enterField(tester, "block_explorer", original.blockExplorer);
      await _editQrSettings(tester,
          enabled: original.qrSettings.enabled,
          delay: original.qrSettings.delay.toString(),
          repair: original.qrSettings.repair.toString(),);
      await _setTheme(tester, container, darkMode: original.darkMode, palette: original.paletteName);
      await _back(tester);
      await waitForAccountView(tester, _accountA);
      await _expectSettingsSaved(tester, container, prefs, original, what: "the original settings");

      phase("done");
      rendezvous.publish("done", "true");
      await pumpFor(tester, const Duration(seconds: 3));
    } finally {
      await savedPrefs.restore(prefs);
      await _restorePref(prefs, "coingecko", savedApiKey);
      await _restorePref(prefs, "transport", savedTransport);
      await _restorePref(prefs, "dark_mode", savedDarkMode);
      await _restorePref(prefs, "palette_name", savedPalette);
    }
  }, timeout: const Timeout(Duration(minutes: 12)));
}

Future<void> _restorePref(SharedPreferencesAsync prefs, String key, Object? value) => switch (value) {
      null => prefs.remove(key),
      final bool v => prefs.setBool(key, v),
      final int v => prefs.setInt(key, v),
      final String v => prefs.setString(key, v),
      _ => throw ArgumentError("unsupported preference type for $key"),
    };

Future<int> _newAccount(String name) => newAccount(
      na: NewAccount(
        name: name,
        restore: false,
        key: "",
        aindex: 0,
        birth: 1,
        folder: "",
        useInternal: false,
        internal: false,
        hw: 0,
      ),
      c: coinContext.coin,
    );

Future<void> _waitForSettings(WidgetTester tester) => pumpUntil(
      tester,
      () => find.text("Settings").evaluate().isNotEmpty && fieldNamed("lwd").evaluate().isNotEmpty,
      timeout: shortTimeout,
      what: "the settings page",
    );

/// Taps the back arrow and lets the route transition finish: a tap during
/// the transition lands on the page that is leaving.
Future<void> _back(WidgetTester tester) async {
  await tester.pageBack();
  await pumpFor(tester, const Duration(seconds: 1));
}

bool _proxyEnabled(WidgetTester tester) => tester.widget<FormBuilderTextField>(fieldNamed("proxy")).enabled;

/// Picks [label] on the transport selector and waits for the proxy field
/// to follow, retrying the tap: an overlay (a tooltip, a transition) can
/// take the first one. The texts on screen are logged when it does.
Future<void> _selectTransport(WidgetTester tester, String label, {required bool proxyEnabled}) async {
  for (var attempt = 1; attempt <= 5; attempt++) {
    final segment = find.text(label);
    await tester.ensureVisible(segment);
    await tester.pump();
    await tester.tap(segment, warnIfMissed: false);
    await pumpFor(tester, const Duration(seconds: 1));
    if (_proxyEnabled(tester) == proxyEnabled) return;
    debugPrint("[wallet-ext] transport $label not taken, attempt $attempt; visible: ${visibleTexts().join(" | ")}");
  }
  fail("the transport $label was not selected");
}

Future<void> _tapSwitch(WidgetTester tester, Finder finder) async {
  await tester.ensureVisible(finder);
  await tester.pump();
  await tester.tap(finder);
  await pumpFor(tester, const Duration(seconds: 1));
}

Future<void> _enterField(WidgetTester tester, String name, String value) async {
  final field = fieldNamed(name);
  await tester.ensureVisible(field);
  await tester.pump();
  if (value.isEmpty) {
    // enterText("") leaves the field as it was on the desktop binding; the
    // form field follows its controller, so clear that instead.
    tester.widget<TextField>(find.descendant(of: field, matching: find.byType(TextField))).controller!.clear();
  } else {
    await tester.enterText(field, value);
  }
  await tester.pump();
  expect(fieldValue(tester, name), value, reason: "field $name reads back");
}

/// Opens the QR settings sub-page, sets its fields and leaves; the page hands
/// its values to the settings page when it pops.
Future<void> _editQrSettings(WidgetTester tester, {required bool enabled, required String delay, required String repair}) async {
  final row = find.ancestor(of: find.text("File Transmission via QR Codes"), matching: find.byType(Row)).first;
  await tester.ensureVisible(row);
  await tester.pump();
  await tester.tap(find.descendant(of: row, matching: find.byIcon(Icons.chevron_right)));
  await pumpUntil(tester, () => find.text("QR Codes").evaluate().isNotEmpty && switchNamed("enabled").evaluate().isNotEmpty,
      timeout: shortTimeout, what: "the QR settings page",);
  final state = (switchNamed("enabled").evaluate().first as StatefulElement).state as FormBuilderFieldState;
  if ((state.value as bool) != enabled) await _tapSwitch(tester, switchNamed("enabled"));
  await _enterField(tester, "delay", delay);
  await _enterField(tester, "repair", repair);
  await _back(tester);
  await _waitForSettings(tester);
}

/// Opens the theme sub-page and picks [darkMode] and [palette]; unlike the
/// rest of the settings, the theme is stored as soon as it changes.
Future<void> _setTheme(WidgetTester tester, ProviderContainer container, {required bool darkMode, required String palette}) async {
  await tapTooltip(tester, "Theme");
  await pumpUntil(tester, () => find.text("Theme").evaluate().isNotEmpty && find.text("Dark Mode").evaluate().isNotEmpty,
      timeout: shortTimeout, what: "the theme page",);
  if (container.read(appSettingsProvider).value!.darkMode != darkMode) {
    await tester.tap(find.text("Dark Mode"));
    await pumpFor(tester, const Duration(seconds: 1));
  }
  final button = find.widgetWithText(ElevatedButton, palette);
  await tester.ensureVisible(button);
  await tester.pump();
  await tester.tap(button);
  await pumpUntil(tester, () {
    final s = container.read(appSettingsProvider).value;
    return s != null && s.darkMode == darkMode && s.paletteName == palette;
  }, timeout: shortTimeout, what: "the theme $palette (dark: $darkMode) to be stored",);
  await _back(tester);
  await _waitForSettings(tester);
}

/// Long-presses the version line (`version+build`) at the bottom of the
/// settings page; the page fills it in after an async lookup.
Future<void> _longPressVersion(WidgetTester tester) async {
  final version = find.byWidgetPredicate((w) => w is Text && w.data != null && RegExp(r"^\d+(\.\d+)*\+").hasMatch(w.data!));
  await pumpUntil(tester, () => version.evaluate().isNotEmpty,
      timeout: shortTimeout, what: "the version line on the settings page",);
  await tester.ensureVisible(version);
  await tester.pump();
  await tester.longPress(version);
  await pumpFor(tester, const Duration(seconds: 1));
}

/// Checks the settings the page writes when it is left: the provider, the
/// wallet props and the preferences.
Future<void> _expectSettingsSaved(
  WidgetTester tester,
  ProviderContainer container,
  SharedPreferencesAsync prefs,
  AppSettings expected, {
  required String what,
}) async {
  await pumpUntil(tester, () => container.read(appSettingsProvider).value?.proxy == expected.proxy,
      timeout: shortTimeout, what: "$what to be saved",);
  final s = container.read(appSettingsProvider).value!;
  expect(s.transport, expected.transport, reason: "transport");
  expect(s.proxy, expected.proxy, reason: "proxy");
  expect(s.votingConfigUrl, expected.votingConfigUrl, reason: "voting config url");
  expect(s.actionsPerSync, expected.actionsPerSync, reason: "actions per sync");
  expect(s.syncInterval, expected.syncInterval, reason: "autosync interval");
  expect(s.offline, expected.offline, reason: "offline");
  expect(s.getFx, expected.getFx, reason: "auto fetch price");
  expect(s.coingecko, expected.coingecko, reason: "coingecko key");
  expect(s.blockExplorer, expected.blockExplorer, reason: "block explorer");
  expect(s.qrSettings.enabled, expected.qrSettings.enabled, reason: "qr enabled");
  expect(s.qrSettings.delay, expected.qrSettings.delay, reason: "qr delay");
  expect(s.qrSettings.repair, expected.qrSettings.repair, reason: "qr repair");
  expect(s.darkMode, expected.darkMode, reason: "dark mode");
  expect(s.paletteName, expected.paletteName, reason: "palette");
  expect(s.lwd, expected.lwd, reason: "server unchanged");
  expect(s.isLightNode, expected.isLightNode, reason: "light node unchanged");
  final stored = (await tester.runAsync(() async => (
        proxy: await getProp(key: "proxy", c: coinContext.coin),
        syncInterval: await getProp(key: "sync_interval", c: coinContext.coin),
        qrDelay: await getProp(key: "qr_delay", c: coinContext.coin),
        offline: await prefs.getBool("offline"),
        transport: await prefs.getInt("transport"),
        coingecko: await prefs.getString("coingecko"),
      )))!;
  expect(stored.proxy, expected.proxy, reason: "proxy prop");
  expect(stored.syncInterval, expected.syncInterval, reason: "sync_interval prop");
  expect(stored.qrDelay, expected.qrSettings.delay.toString(), reason: "qr_delay prop");
  expect(stored.offline, expected.offline, reason: "offline pref");
  expect(stored.transport, expected.transport, reason: "transport pref");
  expect(stored.coingecko, expected.coingecko, reason: "coingecko pref");
}
