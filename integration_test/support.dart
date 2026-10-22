/// Shared scaffolding for the Flutter-side FROST integration tests.
///
/// `dkg_ui_test.dart` and `frost_ui_test.dart` run the real app as one
/// participant while `zkool_graphql` instances play the others, orchestrated by
/// pytest; `wallet_ui_test.dart` runs it alone. This file holds the parts they
/// have in common: the JSON-file rendezvous with the orchestrator, pumping
/// helpers that work on a route with live timers, wallet setup, and the DKG
/// flow itself (signing needs a completed DKG first).
library;

import 'dart:convert';
import 'dart:io';
import 'dart:typed_data';

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
import 'package:zkool/src/rust/api/sync.dart';
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

  /// What an earlier app process published, for a test that runs after it.
  Map<String, dynamic>? readUi() => _readJson(_ui);

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
  final bool? getFx;
  /// Expert mode is toggled by the extended test; a run that failed in
  /// between leaves it on for the next, which expects it off.
  final bool? expertMode;
  /// The theme, forced to the app's defaults by the docs screenshot runs.
  final String? paletteName;
  final bool? darkMode;

  SavedPrefs(this.pinLock, this.offline, this.vault, this.getFx, this.expertMode, this.paletteName, this.darkMode);

  /// [getFx] turns on the market price fetch at startup; the wallet test
  /// points it at a mock through the `coingecko_url` prop.
  /// [defaultTheme] puts the app in its default theme (the blue palette,
  /// dark mode), whatever this machine's wallet has chosen: the docs
  /// screenshots show what a new user sees.
  static Future<SavedPrefs> forceTestValues(
    SharedPreferencesAsync prefs, {
    bool getFx = false,
    bool defaultTheme = false,
  }) async {
    final saved = SavedPrefs(
      await prefs.getBool("pin_lock"),
      await prefs.getBool("offline"),
      await prefs.getBool("vault"),
      await prefs.getBool("get_fx"),
      await prefs.getBool("expert_mode"),
      await prefs.getString("palette_name"),
      await prefs.getBool("dark_mode"),
    );
    await prefs.setBool("pin_lock", false);
    await prefs.setBool("offline", false);
    await prefs.setBool("vault", false);
    await prefs.setBool("get_fx", getFx);
    await prefs.setBool("expert_mode", false);
    if (defaultTheme) {
      await prefs.setString("palette_name", "blue");
      await prefs.setBool("dark_mode", true);
    }
    return saved;
  }

  Future<void> restore(SharedPreferencesAsync prefs) async {
    await _restore(prefs, "pin_lock", pinLock);
    await _restore(prefs, "offline", offline);
    await _restore(prefs, "vault", vault);
    await _restore(prefs, "get_fx", getFx);
    await _restore(prefs, "expert_mode", expertMode);
    if (paletteName == null) {
      await prefs.remove("palette_name");
    } else {
      await prefs.setString("palette_name", paletteName!);
    }
    await _restore(prefs, "dark_mode", darkMode);
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

/// Opens a fresh test database with no accounts, pointed at [lwd]. Extra
/// [props] are written to the wallet's props table (e.g. `coingecko_url`).
/// With [fresh] false, an existing database is opened as it is, e.g. one an
/// orchestrator prepared with `zkool_graphql`.
/// Must run inside `tester.runAsync` — it is all real async I/O.
Future<void> openTestWallet({
  required String dbPath,
  required String lwd,
  required String docsDir,
  Map<String, String> props = const {},
  bool fresh = true,
}) async {
  await RustLib.init();
  await initDatadir(directory: docsDir);

  final dbFile = File(dbPath);
  if (fresh && dbFile.existsSync()) dbFile.deleteSync();

  var c = await Coin().openDatabase(dbFilepath: dbPath, password: null);
  await putProp(key: "lwd", value: lwd, c: c);
  await putProp(key: "is_light_node", value: "true", c: c);
  for (final e in props.entries) {
    await putProp(key: e.key, value: e.value, c: c);
  }
  c = c.setLwd(serverType: 0, url: lwd);
  coinContext.set(coin: c);
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
  await openTestWallet(dbPath: dbPath, lwd: lwd, docsDir: docsDir);

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

Finder switchNamed(String name) =>
    find.byWidgetPredicate((w) => w is FormBuilderSwitch && w.name == name);

Finder checkboxNamed(String name) =>
    find.byWidgetPredicate((w) => w is FormBuilderCheckbox && w.name == name);

Finder dropdownNamed(String name) =>
    find.byWidgetPredicate((w) => w is FormBuilderDropdown<int> && w.name == name);

/// Taps an [IconButton] by its tooltip, scrolling it into view first.
Future<void> tapTooltip(WidgetTester tester, String tooltip) async {
  final button = find.byTooltip(tooltip);
  await tester.ensureVisible(button.first);
  await tester.pump();
  await tester.tap(button.first, warnIfMissed: false);
  await pumpFor(tester, const Duration(seconds: 1));
}

/// Picks [value] on a dropdown found by [dropdown] (see [selectDropdown] for
/// the label-based variant).
Future<void> pickDropdown(WidgetTester tester, Finder dropdown, String value) async {
  await tester.ensureVisible(dropdown.first);
  await tester.pump();
  await tester.tap(dropdown.first, warnIfMissed: false);
  await pumpFor(tester, const Duration(seconds: 1));
  await tester.tap(find.text(value).last);
  await pumpFor(tester, const Duration(seconds: 1));
}

/// Pumps real frames until [done] holds. `pumpAndSettle` is unusable on the
/// FROST routes: both DKGPage3 and FrostPage2 install a 30s periodic timer and
/// the synchronizer keeps scheduling frames, so the tree never settles.
/// [beforeEach] runs before every check, for a wait that has to nudge the
/// page first, such as scrolling a lazy list to the row it looks for.
Future<void> pumpUntil(
  WidgetTester tester,
  bool Function() done, {
  required Duration timeout,
  required String what,
  Future<void> Function()? beforeEach,
}) async {
  final deadline = DateTime.now().add(timeout);
  while (DateTime.now().isBefore(deadline)) {
    if (beforeEach != null) await beforeEach();
    if (done()) return;
    await tester.pump(const Duration(milliseconds: 500));
  }
  debugPrint("[ui-test] visible texts: ${visibleTexts().join(" | ")}");
  fail("timed out after $timeout waiting for $what");
}

/// The distinct strings of every [Text] on screen, for the log when a wait
/// times out: it says which page or dialog the app is actually showing.
List<String> visibleTexts({int limit = 60}) {
  final texts = <String>{};
  for (final e in find.byType(Text).evaluate()) {
    final data = (e.widget as Text).data;
    if (data != null && data.trim().isNotEmpty) texts.add(data.trim());
    if (texts.length >= limit) break;
  }
  return texts.toList();
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
  await dismissLowFundingWarning(tester);
}

/// The DKG and FROST first pages warn, on Next, when the funding account
/// holds under 0.001 ZEC. The orchestrator funds the app's account only
/// after the app has published its address, so on the DKG page the warning
/// is expected: answer it the way a user who knows the funds are coming
/// would. The check waits a moment for the balance lookup behind the dialog.
Future<void> dismissLowFundingWarning(WidgetTester tester) async {
  // The dialog's title is not a plain Text; its message is.
  final warning = find.textContaining("Continue anyway?");
  if (warning.evaluate().isEmpty) {
    await pumpFor(tester, const Duration(seconds: 1));
    if (warning.evaluate().isEmpty) return;
  }
  debugPrint("[ui-test] dismissing the low funding balance warning");
  await tester.tap(find.text("Ok"));
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

/// Stands in for the system file dialogs, which a test cannot drive: a save
/// lands in [dir] under the name the app proposes, and a pick hands out the
/// next path queued in [picks]. Installed with `fileDialogs = FakeFileDialogs(dir)`.
class FakeFileDialogs implements FileDialogs {
  final Directory dir;
  final List<String> picks = [];
  final List<String> saved = [];

  /// When set, the next saves throw it instead of writing, as a read-only
  /// folder would.
  Object? failWith;

  FakeFileDialogs(this.dir);

  @override
  Future<Uint8List?> open({String? title}) async {
    if (picks.isEmpty) return null;
    return File(picks.removeAt(0)).readAsBytes();
  }

  @override
  Future<String?> save({String? title, String? fileName, required Uint8List data}) async {
    if (failWith != null) throw failWith!;
    final path = "${dir.path}/${fileName ?? "file-${saved.length}"}";
    File(path).writeAsBytesSync(data);
    saved.add(path);
    return path;
  }
}

// ── Wallet pages ─────────────────────────────────────────────────────────────
// Helpers for the account list, account view and send page, shared by the
// wallet tests.

const shortTimeout = Duration(seconds: 30);
const chainTimeout = Duration(minutes: 4);

BigInt zecToZat(String zec) => BigInt.from((double.parse(zec) * 1e8).round());

Future<Map<String, int>> accountIds() async {
  final accounts = await listAccounts(c: coinContext.coin);
  return {for (final a in accounts) a.name: a.id};
}

Future<BigInt> totalBalance(int account) async {
  final b = await balance(account: account, c: coinContext.coin);
  return b.field0.fold<BigInt>(BigInt.zero, (sum, v) => sum + v);
}

String? fieldValue(WidgetTester tester, String name) {
  final found = fieldNamed(name).evaluate();
  if (found.isEmpty) return null;
  final state = (found.first as StatefulElement).state as FormBuilderFieldState;
  return state.value as String?;
}

Future<void> waitForAccountList(WidgetTester tester) => pumpUntil(
      tester,
      () => find.text("Account List").evaluate().isNotEmpty,
      timeout: shortTimeout,
      what: "the account list",
    );

Future<void> waitForAccountView(WidgetTester tester, String name) => pumpUntil(
      tester,
      () => find.text(name).evaluate().isNotEmpty && find.byTooltip("Sync this account").evaluate().isNotEmpty,
      timeout: shortTimeout,
      what: "the account view of $name",
    );

/// Taps the sync button and waits until the account's total reaches [atLeast].
/// Logs every poll: a stall here is the first thing to read in the log.
Future<void> waitForBalance(WidgetTester tester, int account, {required BigInt atLeast, required String what}) async {
  final deadline = DateTime.now().add(chainTimeout);
  var polls = 0;
  while (DateTime.now().isBefore(deadline)) {
    debugPrint("[ui-test] ${DateTime.now().toIso8601String()} poll ${++polls}: reading balance of $account");
    final total = await tester.runAsync(() => totalBalance(account).timeout(const Duration(seconds: 60)));
    debugPrint("[ui-test] balance of $account: $total (want $atLeast)");
    if (total! >= atLeast) return;
    if (polls % 3 == 1) await tapTooltip(tester, "Sync this account");
    await pumpFor(tester, const Duration(seconds: 5));
  }
  debugPrint("[ui-test] visible texts: ${visibleTexts().join(" | ")}");
  fail("timed out waiting for $what");
}

/// Syncs until the account is at the chain tip, so the send page needs no
/// sync warning.
Future<void> waitForSync(WidgetTester tester, int account) async {
  final deadline = DateTime.now().add(chainTimeout);
  var polls = 0;
  while (DateTime.now().isBefore(deadline)) {
    final tip = await tester.runAsync(() => getCurrentHeight(c: coinContext.coin).timeout(const Duration(seconds: 60)));
    final accounts = await tester.runAsync(() => listAccounts(c: coinContext.coin).timeout(const Duration(seconds: 60)));
    final height = accounts!.firstWhere((x) => x.id == account).height;
    debugPrint("[ui-test] account $account at $height, tip $tip");
    if (height >= tip!) return;
    if (polls++ % 3 == 0) await tapTooltip(tester, "Sync this account");
    await pumpFor(tester, const Duration(seconds: 5));
  }
  fail("timed out waiting for account $account to reach the chain tip");
}

Future<void> tapTab(WidgetTester tester, String name) async {
  await tester.tap(find.widgetWithText(Tab, name));
  await pumpFor(tester, const Duration(seconds: 1));
}

Future<void> openMenuItem(WidgetTester tester, String item) async {
  await tester.tap(find.byType(PopupMenuButton<String>).first);
  await pumpFor(tester, const Duration(seconds: 1));
  await tester.tap(find.text(item).last);
  await pumpFor(tester, const Duration(seconds: 1));
}

/// Selects an account in the list by tapping its avatar (rows are keyed by
/// id) and taps the [action] icon the selection reveals. The list clears its
/// selection whenever the page rebuilds with a new account list, which a
/// height or price update does, so the two taps are retried as a unit.
Future<void> selectAndTap(WidgetTester tester, int id, IconData action) async {
  for (var attempt = 1; attempt <= 5; attempt++) {
    final row = find.byKey(ValueKey(id));
    await tester.ensureVisible(row);
    final avatar = find.descendant(of: row, matching: find.byType(CircleAvatar)).first;
    await tester.tap(avatar);
    await tester.pump(const Duration(milliseconds: 300));
    if (find.byIcon(action).evaluate().isNotEmpty) {
      await tester.tap(find.byIcon(action));
      await pumpFor(tester, const Duration(seconds: 1));
      return;
    }
    debugPrint("[ui-test] selection of $id lost before the action, attempt $attempt");
    await pumpFor(tester, const Duration(seconds: 1));
  }
  fail("could not select account $id and tap its action");
}

Finder copyableWith(String text) => find.byWidgetPredicate((w) => w is CopyableText && w.text == text);

/// Every ListTile on screen as its plain texts (Text and RichText), for the log.
List<String> tileTexts() => [
      for (final tile in find.byType(ListTile).evaluate())
        [
          for (final e in find.descendant(of: find.byWidget(tile.widget), matching: find.byType(RichText)).evaluate())
            (e.widget as RichText).text.toPlainText(),
        ].join(" / "),
    ];

/// Waits for an AwesomeDialog showing [message] (its title is not a Text
/// widget), taps the [button] once it has slid in, and waits for the dialog
/// to go: a tap during its exit lands on its barrier.
Future<void> answerDialog(WidgetTester tester, String message, String button) async {
  await pumpUntil(
    tester,
    () => find.text(message).evaluate().isNotEmpty && find.text(button).evaluate().isNotEmpty,
    timeout: shortTimeout,
    what: "the dialog '$message'",
  );
  await tester.tap(find.text(button).last);
  await pumpUntil(
    tester,
    () => find.text(message).evaluate().isEmpty,
    timeout: shortTimeout,
    what: "the dialog '$message' to close",
  );
  await pumpFor(tester, const Duration(milliseconds: 500));
}

Future<void> enterAmount(WidgetTester tester, String amount) async {
  // InputAmount nests a form whose ZEC field is named "zat".
  final field = fieldNamed("zat");
  await tester.ensureVisible(field.first);
  final editableFinder = find.descendant(of: field.first, matching: find.byType(EditableText));
  // With a queued recipient loaded for editing, the tap enterText makes does
  // not focus the field; ask its focus node directly.
  tester.widget<EditableText>(editableFinder).focusNode.requestFocus();
  await tester.pump();
  await tester.enterText(field.first, amount);
  await tester.pump();
  await pumpUntil(tester, () => fieldValue(tester, "zat") == amount,
      timeout: const Duration(seconds: 5), what: "the ZEC amount to read $amount",);
}

Future<void> enterRecipient(WidgetTester tester, String address, String amount, {String? memo}) async {
  await tester.enterText(fieldNamed("address"), address);
  await tester.pump();
  await pumpFor(tester, const Duration(seconds: 1)); // address validation and memo visibility
  await enterAmount(tester, amount);
  if (memo != null) {
    await tester.enterText(fieldNamed("memo"), memo);
    await tester.pump();
  }
}
