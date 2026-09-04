/// Shared scaffolding for the Flutter-side FROST integration tests.
///
/// Both `dkg_ui_test.dart` and `frost_ui_test.dart` run the real app as one
/// participant while `zkool_graphql` instances play the others, orchestrated by
/// pytest. This file holds the parts they have in common: the JSON-file
/// rendezvous with the orchestrator, pumping helpers that work on a route with
/// live timers, and the DKG flow itself (signing needs a completed DKG first).
library;

import 'dart:convert';
import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter_form_builder/flutter_form_builder.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:path_provider/path_provider.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:zkool/src/rust/api/account.dart';
import 'package:zkool/src/rust/api/coin.dart' hide initDatadir;
import 'package:zkool/src/rust/api/db.dart';
import 'package:zkool/src/rust/api/network.dart';
import 'package:zkool/src/rust/frb_generated.dart';
import 'package:zkool/store.dart';
import 'package:zkool/utils.dart';

/// Directory shared with the Python orchestrator, passed as
/// `--dart-define=ZKOOL_TEST_RENDEZVOUS=<dir>`.
const rendezvousOverride = String.fromEnvironment("ZKOOL_TEST_RENDEZVOUS");

/// JSON file rendezvous with the Python orchestrator.
///
/// pytest cannot reach the app over the network, and both sides need values
/// from the other while `flutter test` is running, so they exchange JSON
/// documents in a shared directory. Writes go to a temp file and are renamed so
/// a reader never sees a partial document.
class Rendezvous {
  final Directory dir;

  Rendezvous(this.dir);

  factory Rendezvous.fromEnvironment(Directory docsDir) => Rendezvous(
        Directory(
          rendezvousOverride.isNotEmpty
              ? rendezvousOverride
              : "${docsDir.path}/dkg_ui_rendezvous",
        ),
      );

  File get _config => File("${dir.path}/config.json");
  File get _ui => File("${dir.path}/ui.json");
  File get _peers => File("${dir.path}/peers.json");

  Map<String, dynamic> readConfig() =>
      jsonDecode(_config.readAsStringSync()) as Map<String, dynamic>;

  Map<String, dynamic>? readPeers() => _readJson(_peers);

  /// Values the orchestrator publishes for us, e.g. the PCZT to sign.
  Map<String, dynamic>? readOrchestrator() =>
      _readJson(File("${dir.path}/orchestrator.json"));

  Map<String, dynamic>? _readJson(File f) {
    if (!f.existsSync()) return null;
    try {
      return jsonDecode(f.readAsStringSync()) as Map<String, dynamic>;
    } on FormatException {
      return null; // renamed atomically, but be safe
    }
  }

  final Map<String, dynamic> _uiState = {};

  /// Publishes [key] to the orchestrator.
  void publish(String key, String value) {
    _uiState[key] = value;
    final tmp = File("${_ui.path}.tmp");
    tmp.writeAsStringSync(jsonEncode(_uiState));
    tmp.renameSync(_ui.path);
  }
}

/// Preference values the tests force, and their originals, so a run does not
/// leave the developer's own app settings changed.
class SavedPrefs {
  final bool? pinLock;
  final bool? offline;
  final bool? vault;

  SavedPrefs(this.pinLock, this.offline, this.vault);

  static Future<SavedPrefs> forceTestValues(SharedPreferencesAsync prefs) async {
    final saved = SavedPrefs(
      await prefs.getBool("pin_lock"),
      await prefs.getBool("offline"),
      await prefs.getBool("vault"),
    );
    await prefs.setBool("pin_lock", false);
    await prefs.setBool("offline", false);
    await prefs.setBool("vault", false);
    return saved;
  }

  Future<void> restore(SharedPreferencesAsync prefs) async {
    await _restore(prefs, "pin_lock", pinLock);
    await _restore(prefs, "offline", offline);
    await _restore(prefs, "vault", vault);
  }

  static Future<void> _restore(
    SharedPreferencesAsync prefs,
    String key,
    bool? value,
  ) async {
    if (value == null) {
      await prefs.remove(key);
    } else {
      await prefs.setBool(key, value);
    }
  }
}

/// Opens the test database and creates the account that pays for the protocol
/// memos. Must run inside `tester.runAsync` — it is all real async I/O.
///
/// Returns the id of the funding account.
Future<int> setUpTestWallet({
  required String dbPath,
  required String lwd,
  required String docsDir,
}) async {
  await RustLib.init();
  await initDatadir(directory: docsDir);

  final dbFile = File(dbPath);
  if (dbFile.existsSync()) dbFile.deleteSync();

  var c = await Coin().openDatabase(dbFilepath: dbPath, password: null);
  await putProp(key: "lwd", value: lwd, c: c);
  await putProp(key: "is_light_node", value: "true", c: c);
  c = c.setLwd(serverType: 0, url: lwd);
  coinContext.set(coin: c);

  final account = await newAccount(
    na: NewAccount(
      name: "DKG-Fund",
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
  await coinContext.setAccount(account: account);
  return account;
}

/// Splash normally initialises these; without it `appSettingsProvider` stays in
/// AsyncLoading and `startSynchronize`'s `requireValue` throws.
Future<void> primeProviders(WidgetTester tester, ProviderContainer container) async {
  // Read the settings first: it watches hasDbProvider, which keeps that
  // (auto-dispose) notifier alive across the setHasDb call below.
  container.read(appSettingsProvider);
  container.read(hasDbProvider.notifier).setHasDb();
  await pumpUntil(
    tester,
    () => container.read(appSettingsProvider).hasValue,
    timeout: const Duration(seconds: 30),
    what: "app settings to load",
  );
}

Finder fieldNamed(String name) =>
    find.byWidgetPredicate((w) => w is FormBuilderTextField && w.name == name);

/// Pumps real frames until [done] holds. `pumpAndSettle` is unusable on the
/// FROST routes: both DKGPage3 and FrostPage2 install a 30s periodic timer and
/// the synchronizer keeps scheduling frames, so the tree never settles.
Future<void> pumpUntil(
  WidgetTester tester,
  bool Function() done, {
  required Duration timeout,
  required String what,
}) async {
  final deadline = DateTime.now().add(timeout);
  while (DateTime.now().isBefore(deadline)) {
    if (done()) return;
    await tester.pump(const Duration(milliseconds: 500));
  }
  fail("timed out after $timeout waiting for $what");
}

/// Pumps real frames for [duration]; see [pumpUntil] for why not pumpAndSettle.
Future<void> pumpFor(WidgetTester tester, Duration duration) async {
  final deadline = DateTime.now().add(duration);
  while (DateTime.now().isBefore(deadline)) {
    await tester.pump(const Duration(milliseconds: 100));
  }
}

Future<void> selectDropdown(WidgetTester tester, String label, String value) async {
  final dropdown = find.ancestor(
    of: find.text(label),
    matching: find.byType(FormBuilderDropdown<int>),
  );
  await tester.ensureVisible(dropdown.first);
  await tester.pump();
  await tester.tap(dropdown.first, warnIfMissed: false);
  await pumpFor(tester, const Duration(seconds: 1));
  await tester.tap(find.text(value).last);
  await pumpFor(tester, const Duration(seconds: 1));
}

Future<void> tapNext(WidgetTester tester) async {
  final next = find.widgetWithText(ElevatedButton, "Next");
  await tester.ensureVisible(next);
  await tester.pump();
  await tester.tap(next);
  await pumpFor(tester, const Duration(seconds: 1));
}

/// Watches a status line, publishing every change so the run is observable from
/// the orchestrator and the log, and returns once [isDone] accepts one.
///
/// [readStatus] pulls the current message out of whatever widget the page uses
/// (DKGPage3 renders a CopyableText, FrostPage2 a plain Text).
Future<String> awaitStatus(
  WidgetTester tester,
  Rendezvous rendezvous, {
  required String Function() readStatus,
  required bool Function(String) isDone,
  required Duration timeout,
  required String what,
}) async {
  var last = "";
  String? finalStatus;
  await pumpUntil(
    tester,
    () {
      final text = readStatus();
      if (text.isNotEmpty && text != last) {
        last = text;
        debugPrint("[dkg-ui] status: $text");
        rendezvous.publish("status", text);
      }
      if (!isDone(text)) return false;
      finalStatus = text;
      return true;
    },
    timeout: timeout,
    what: what,
  );
  return finalStatus!;
}

/// Current message on DKGPage3, which renders it in a [CopyableText].
String readCopyableStatus() {
  final found = find.byType(CopyableText).evaluate();
  if (found.isEmpty) return "";
  return (found.first.widget as CopyableText).text;
}

const sharedAddressPrefix = "The shared address is: ";

/// Drives the DKG wizard: parameters, address exchange, then the rounds.
/// Returns the shared address rendered on the final page.
Future<String> driveDkg(
  WidgetTester tester,
  Rendezvous rendezvous, {
  required String name,
  required int n,
  required int t,
  required int myId,
}) async {
  await pumpUntil(
    tester,
    () => find.text("Number of Participants").evaluate().isNotEmpty,
    timeout: const Duration(seconds: 30),
    what: "DKG page 1",
  );
  await tester.enterText(fieldNamed("name"), name);
  await tester.pump();
  await selectDropdown(tester, "Number of Participants", "$n");
  await selectDropdown(tester, "Your Participant ID", "$myId");
  await selectDropdown(tester, "Number of Signers Required (Threshold)", "$t");
  await selectDropdown(tester, "Funding Account for the DKG messages", "DKG-Fund");
  await tapNext(tester);

  await pumpUntil(
    tester,
    () => find.text("DKG Addresses").evaluate().isNotEmpty,
    timeout: const Duration(minutes: 2),
    what: "DKG page 2",
  );
  await pumpUntil(
    tester,
    () => fieldNamed("${myId - 1}").evaluate().isNotEmpty,
    timeout: const Duration(minutes: 2),
    what: "own DKG address field",
  );

  final myAddress =
      tester.widget<FormBuilderTextField>(fieldNamed("${myId - 1}")).initialValue!;
  expect(myAddress, isNotEmpty);
  debugPrint("[dkg-ui] my DKG address $myAddress");
  rendezvous.publish("dkg_address", myAddress);

  Map<String, dynamic>? peers;
  await pumpUntil(
    tester,
    () {
      peers = rendezvous.readPeers();
      return peers != null && peers!.length == n - 1;
    },
    timeout: const Duration(minutes: 10),
    what: "peer DKG addresses",
  );

  for (final entry in peers!.entries) {
    final field = fieldNamed("${int.parse(entry.key) - 1}");
    await tester.ensureVisible(field);
    await tester.enterText(field, entry.value as String);
    await tester.pump();
  }
  await tapNext(tester);

  await pumpUntil(
    tester,
    () => find.text("Distributed Key Generation").evaluate().isNotEmpty,
    timeout: const Duration(minutes: 2),
    what: "DKG page 3",
  );

  final status = await awaitStatus(
    tester,
    rendezvous,
    readStatus: readCopyableStatus,
    isDone: (s) => s.startsWith(sharedAddressPrefix),
    timeout: const Duration(minutes: 15),
    what: "the shared address",
  );

  final sharedAddress = status.substring(sharedAddressPrefix.length).trim();
  expect(sharedAddress, isNotEmpty);
  debugPrint("[dkg-ui] shared address $sharedAddress");
  return sharedAddress;
}

/// Documents directory the app actually uses, which is where the rendezvous and
/// the test database live (the macOS app is sandboxed and cannot use /tmp).
Future<Directory> appDocumentsDir() => getApplicationDocumentsDirectory();
