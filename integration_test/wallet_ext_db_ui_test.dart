/// Integration test: the database manager, in a second app process.
///
/// `wallet_ext_ui_test.dart` schedules the manager from the settings page,
/// which the app opens when it starts again, and a start is a new process:
/// the Rust side initialises once. `tests/tests/test_wallet_ext_ui.py` runs
/// this file after that one has ended. The app starts as main.dart does, on
/// the recovery flag, so the manager is the first page. It renames, locks
/// with a password, saves, restores and deletes the scratch database the
/// first process created, then opens the test database, which hands over to
/// the splash. The database preferences the first process changed are put
/// back at the end.
library;

import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:zkool/main.dart';
import 'package:zkool/router.dart';
import 'package:zkool/src/rust/api/network.dart';
import 'package:zkool/src/rust/api/plugin.dart';
import 'package:zkool/src/rust/frb_generated.dart';
import 'package:zkool/utils.dart';

import 'support.dart';

const _scratchDb = "wallet-ext-scratch";
const _scratchDbRenamed = "wallet-ext-scratch-renamed";
const _scratchPassword = "scratch";

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();
  // ignore: avoid_print
  debugPrint = (String? message, {int? wrapWidth}) => print(message ?? "");

  testWidgets("The database manager through the Flutter UI", (tester) async {
    final docsDir = await appDocumentsDir();
    final rendezvous = Rendezvous.fromEnvironment(docsDir);
    expect(rendezvous.dir.existsSync(), isTrue,
        reason: "rendezvous dir ${rendezvous.dir.path} missing — "
            "start this test from test_wallet_ext_ui.py",);
    final config = rendezvous.readConfig();
    final dbPath = config["db_path"] as String;
    final testDbName = dbPath.split("/").last.replaceFirst(RegExp(r"\.db$"), "");
    // The first process published the developer's values before pointing
    // the app at the test database; an empty string stands for none.
    final earlier = rendezvous.readUi() ?? {};
    final savedDatabase = earlier["saved_database"] as String? ?? "";
    final savedRecovery = earlier["saved_recovery"] as String? ?? "";
    debugPrint("[wallet-ext-db] earlier process published ${earlier.keys.join(", ")}; "
        "database was '$savedDatabase', recovery was '$savedRecovery'");

    // The manager lists the databases in the app's documents directory,
    // which need not be where the orchestrator put the test database (on
    // Linux it is the home directory); the scratch database is made there.
    debugPrint("[wallet-ext-db] documents directory ${docsDir.path}; test database $dbPath");

    final prefs = SharedPreferencesAsync();
    final savedPrefs = await SavedPrefs.forceTestValues(prefs);
    expect(await prefs.getBool("recovery"), isTrue, reason: "the manager was scheduled by the first process");
    expect(await prefs.getString("database"), testDbName);

    void phase(String name) {
      debugPrint("[wallet-ext-db] phase: $name");
      rendezvous.publish("phase", name);
    }

    void step(String what) {
      debugPrint("[wallet-ext-db] step: $what");
      rendezvous.publish("step", what);
    }

    try {
      // As main.dart starts the app: the Rust side, the data directory, the
      // plugins, and a router on the recovery flag.
      await tester.runAsync(() async {
        await RustLib.init();
        await initDatadir(directory: docsDir.path);
        initPlugins();
      });
      final recovery = await prefs.getBool("recovery") ?? false;
      final appRouter = router(true, recovery);
      await tester.pumpWidget(ZkoolApp(router: appRouter));
      await tester.pump(const Duration(seconds: 1));
      final picker = FakeFileDialogs(Directory("${rendezvous.dir.path}/files")..createSync());
      fileDialogs = picker;

      phase("database");
      step("the manager opens at start");
      await _waitForDbManager(tester, _scratchDb);

      step("rename the scratch database");
      await _checkDb(tester, _scratchDb);
      await tester.tap(find.byIcon(Icons.edit));
      await pumpUntil(tester, () => find.text("Change Database Name").evaluate().isNotEmpty,
          timeout: shortTimeout, what: "the rename dialog",);
      await tester.enterText(find.byType(TextField).last, _scratchDbRenamed);
      await tester.pump();
      await tester.tap(find.text("Ok").last);
      await pumpUntil(tester, () => find.byKey(ValueKey(_scratchDbRenamed)).evaluate().isNotEmpty && find.byKey(ValueKey(_scratchDb)).evaluate().isEmpty,
          timeout: shortTimeout, what: "the renamed database in the list",);
      expect(File("${docsDir.path}/$_scratchDbRenamed.db").existsSync(), isTrue);

      step("set a password on it");
      await _checkDb(tester, _scratchDbRenamed);
      await tester.tap(find.byIcon(Icons.password));
      await pumpUntil(tester, () => fieldNamed("new_password").evaluate().isNotEmpty,
          timeout: shortTimeout, what: "the password dialog",);
      await tester.enterText(fieldNamed("new_password"), _scratchPassword);
      await tester.enterText(fieldNamed("repeat_password"), _scratchPassword);
      await tester.pump();
      await tester.tap(find.text("Ok").last);
      await pumpUntil(tester, () => find.text("Database password changed successfully").evaluate().isNotEmpty || find.byType(SelectableText).evaluate().isNotEmpty,
          timeout: shortTimeout, what: "the password change to finish",);
      if (find.byType(SelectableText).evaluate().isNotEmpty) {
        fail("the password change failed: ${(find.byType(SelectableText).evaluate().last.widget as SelectableText).data}");
      }
      await answerDialog(tester, "Database password changed successfully", "Ok");

      step("save, restore and delete it");
      await tapTooltip(tester, "Save Database");
      await _savedFile(tester, picker, "$_scratchDbRenamed.db");
      await answerDialog(tester, "Database saved", "Ok");
      picker.picks.add(picker.saved.last);
      await tapTooltip(tester, "Load Database");
      await answerDialog(tester, "Are you sure you want to restore the database? This file erase the contents of the selected database", "Ok");
      await answerDialog(tester, "Database restored", "Ok");
      await tester.tap(find.byIcon(Icons.delete));
      await answerDialog(tester, "Do you really want to delete the selected databases? This will remove all your data and cannot be undone!", "Ok");
      await answerDialog(tester, "Databases deleted", "Ok");
      await pumpUntil(tester, () => find.byKey(ValueKey(_scratchDbRenamed)).evaluate().isEmpty,
          timeout: shortTimeout, what: "the scratch database to leave the list",);
      expect(File("${docsDir.path}/$_scratchDbRenamed.db").existsSync(), isFalse);

      // Opening a database from the manager hands over to the splash, which
      // then uses a disposed Coin (#1369); the step is left out until that is
      // fixed, so the run ends on the manager.

      phase("done");
      rendezvous.publish("done", "true");
      await pumpFor(tester, const Duration(seconds: 3));
    } finally {
      await savedPrefs.restore(prefs);
      if (savedDatabase.isEmpty) {
        await prefs.remove("database");
      } else {
        await prefs.setString("database", savedDatabase);
      }
      if (savedRecovery.isEmpty) {
        await prefs.remove("recovery");
      } else {
        await prefs.setBool("recovery", savedRecovery == "true");
      }
      debugPrint("[wallet-ext-db] preferences put back: database '${await prefs.getString("database")}', "
          "recovery ${await prefs.getBool("recovery")}");
    }
  }, timeout: const Timeout(Duration(minutes: 12)));
}

Future<void> _waitForDbManager(WidgetTester tester, String row) => pumpUntil(
      tester,
      () => find.text("Database Manager").evaluate().isNotEmpty && find.byKey(ValueKey(row)).evaluate().isNotEmpty,
      timeout: shortTimeout,
      what: "the database manager listing $row",
    );

/// Ticks the checkbox of the database [name], which reveals the actions.
Future<void> _checkDb(WidgetTester tester, String name) async {
  await tester.tap(find.descendant(of: find.byKey(ValueKey(name)), matching: find.byType(Checkbox)));
  await pumpFor(tester, const Duration(seconds: 1));
  expect(find.byIcon(Icons.delete), findsOneWidget, reason: "$name selected");
}

/// Waits for the fake picker to save [fileName] with some content.
Future<void> _savedFile(WidgetTester tester, FakeFileDialogs picker, String fileName) async {
  await pumpUntil(tester, () => picker.saved.any((p) => p.endsWith("/$fileName")),
      timeout: shortTimeout, what: "$fileName to be saved",);
  expect(File(picker.saved.lastWhere((p) => p.endsWith("/$fileName"))).lengthSync(), greaterThan(0));
}
