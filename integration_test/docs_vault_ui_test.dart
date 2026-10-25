/// Screenshots of the docs site's Cloud Vault pages, taken from the real
/// Flutter app.
///
/// The vault lives on the Settings page behind expert mode. The run shows
/// where it is: the version line whose long press turns expert mode on, and
/// the three vault rows once it is on. Turning the switch on goes through the
/// app's unlock gate, a biometric prompt on a Mac with Touch ID, so the
/// "Enable Vault" dialog is only shot when the orchestrator asks for it
/// (`enable_dialog` in the config) and someone is at the keyboard.
///
/// Started by tests/test_docs_vault_ui.py, which annotates the frames and
/// copies them into the docs site.
library;

import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:go_router/go_router.dart';
import 'package:integration_test/integration_test.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:zkool/main.dart';
import 'package:zkool/router.dart';
import 'package:zkool/src/rust/api/account.dart';
import 'package:zkool/store.dart';

import 'docs_shots_support.dart';
import 'support.dart';

const _main = "Main Account";

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();
  // ignore: avoid_print
  debugPrint = (String? message, {int? wrapWidth}) => print(message ?? "");

  testWidgets("Docs: cloud vault screenshots", (tester) async {
    final docsDir = await appDocumentsDir();
    final rendezvous = Rendezvous.fromEnvironment(docsDir);
    expect(rendezvous.dir.existsSync(), isTrue,
        reason: "rendezvous dir ${rendezvous.dir.path} missing — "
            "start this from test_docs_vault_ui.py");
    final config = rendezvous.readConfig();
    final dbPath = config["db_path"] as String;
    final enableDialog = config["enable_dialog"] == true;
    expect(dbPath, contains("regtest"));

    final prefs = SharedPreferencesAsync();
    final savedPrefs = await SavedPrefs.forceTestValues(prefs, defaultTheme: true);
    // The vault rows only show while the vault is on; the switch itself is
    // reached without signing in. The pref is read when the app starts.
    await prefs.setBool("vault", true);
    final shots = Shots(tester, docsDir, config);

    // The version line at the end of the settings page, "6.35.0+321"
    // (a debug build repeats the version as the build number).
    final versionLine = find.byWidgetPredicate(
      (w) => w is Text && w.data != null && RegExp(r"^\d+\.\d+\.\d+\S*\+\S+$").hasMatch(w.data!),
    );
    // The page ends with the database path on this machine: blanked.
    final dataPath = find.textContaining("/Documents/");
    List<Annot> noPath() => shots.blank(dataPath);
    Finder rowOf(String text) => find.ancestor(of: find.text(text), matching: find.byType(Row)).first;

    late final GoRouter appRouter;
    Future<void> openSettings() async {
      unawaited(appRouter.push("/settings"));
      await pumpUntil(tester, () => fieldNamed("autosync").evaluate().isNotEmpty, timeout: shortTimeout, what: "the settings page");
    }

    try {
      await tester.runAsync(() async {
        await openTestWallet(dbPath: dbPath, lwd: config["lwd"] as String, docsDir: docsDir.path);
      });
      await tester.runAsync(() => newAccount(
            na: const NewAccount(
              name: _main,
              restore: false,
              key: "",
              aindex: 0,
              birth: 1,
              folder: "",
              useInternal: true,
              internal: false,
              hw: 0,
            ),
            c: coinContext.coin,
          ));

      appRouter = router(true, false, initialLocation: "/accounts");
      await tester.pumpWidget(RepaintBoundary(key: shots.boundary, child: ZkoolApp(router: appRouter)));
      await tester.pump(const Duration(seconds: 1));
      final container = ProviderScope.containerOf(tester.element(find.byType(MaterialApp)));
      await primeProviders(tester, container);
      await waitForAccountList(tester);

      // ── Expert mode: the version line ────────────────────────────────────
      await openSettings();
      // The version comes from the package info, a moment after the page.
      await pumpUntil(tester, () => versionLine.evaluate().isNotEmpty, timeout: shortTimeout, what: "the version line");
      await tester.ensureVisible(versionLine);
      await tester.pump();
      await shots.take("vault/images/01.expert", annots: () => [shots.textBox(versionLine), ...noPath()]);
      await tester.longPress(versionLine);
      await pumpUntil(tester, () => find.text("expert").evaluate().isNotEmpty, timeout: shortTimeout, what: "expert mode");

      // ── The vault rows ───────────────────────────────────────────────────
      await tester.ensureVisible(find.text("Compress Vault"));
      await tester.pump();
      await shots.take("vault/images/02.settings", annots: () => [
        shots.boxAround([rowOf("Cloud Vault"), rowOf("Compress Vault")]),
        ...noPath(),
      ]);

      // ── The Enable Vault dialog, when someone can pass the unlock gate ───
      if (enableDialog) {
        await tester.pageBack();
        await waitForAccountList(tester);
        final settings = container.read(appSettingsProvider).requireValue;
        await tester.runAsync(() => container.read(appSettingsProvider.notifier).save(settings.copyWith(vault: false)));
        await openSettings();
        await tester.ensureVisible(find.text("Cloud Vault"));
        await tester.pump();
        await tester.tap(find.descendant(of: rowOf("Cloud Vault"), matching: find.byType(Switch)));
        await pumpUntil(tester, () => find.text("Enable Vault").evaluate().isNotEmpty,
            timeout: const Duration(minutes: 2), what: "the Enable Vault dialog (pass the unlock prompt)");
        await shots.take("vault/images/03.enable");
        await tester.tap(find.byWidgetPredicate(
          (w) => w is Text && w.data != null && w.data!.toLowerCase() == "cancel",
        ));
        await pumpUntil(tester, () => find.text("Enable Vault").evaluate().isEmpty, timeout: shortTimeout, what: "the dialog to close");
      }

      shots.finish();
      rendezvous.publish("done", "true");
      await pumpFor(tester, const Duration(seconds: 1));
    } finally {
      await savedPrefs.restore(prefs);
    }
  }, timeout: const Timeout(Duration(minutes: 10)));
}
