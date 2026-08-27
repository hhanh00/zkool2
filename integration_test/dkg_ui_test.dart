/// Integration test: run the Flutter app as FROST DKG participant #1 against a
/// regtest chain, while the other participants run headless as `zkool_graphql`
/// instances driven by `tests/tests/test_dkg_ui.py`.
///
/// The Python side owns the chain (funding, mining) and the peer participants.
/// Both sides exchange addresses through JSON files in a rendezvous directory
/// that lives inside the macOS app sandbox container — see
/// `tests/tests/dkg.py` for the other half of the protocol.
library;

import 'dart:convert';
import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter_form_builder/flutter_form_builder.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:path_provider/path_provider.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:zkool/main.dart';
import 'package:zkool/router.dart';
import 'package:zkool/src/rust/api/account.dart';
import 'package:zkool/src/rust/api/coin.dart' hide initDatadir;
import 'package:zkool/src/rust/api/db.dart';
import 'package:zkool/src/rust/api/network.dart';
import 'package:zkool/src/rust/frb_generated.dart';
import 'package:zkool/store.dart';
import 'package:zkool/utils.dart';

/// Directory shared with the Python orchestrator, passed as
/// `--dart-define=ZKOOL_TEST_RENDEZVOUS=<dir>`. Defaults to
/// `<documents>/dkg_ui_rendezvous`, which is where the Python side puts it.
const rendezvousOverride = String.fromEnvironment("ZKOOL_TEST_RENDEZVOUS");

/// JSON file rendezvous with the Python orchestrator.
class Rendezvous {
  final Directory dir;

  Rendezvous(this.dir);

  File get _config => File("${dir.path}/config.json");
  File get _ui => File("${dir.path}/ui.json");
  File get _peers => File("${dir.path}/peers.json");

  Map<String, dynamic> readConfig() =>
      jsonDecode(_config.readAsStringSync()) as Map<String, dynamic>;

  Map<String, dynamic>? readPeers() {
    if (!_peers.existsSync()) return null;
    try {
      return jsonDecode(_peers.readAsStringSync()) as Map<String, dynamic>;
    } on FormatException {
      return null; // partially written; the writer renames atomically but be safe
    }
  }

  final Map<String, dynamic> _uiState = {};

  /// Publishes [key] to the orchestrator. Written to a temp file then renamed
  /// so the reader never observes a partial document.
  void publish(String key, String value) {
    _uiState[key] = value;
    final tmp = File("${_ui.path}.tmp");
    tmp.writeAsStringSync(jsonEncode(_uiState));
    tmp.renameSync(_ui.path);
  }
}

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  testWidgets("DKG participant #1 through the Flutter UI", (tester) async {
    final docsDir = await getApplicationDocumentsDirectory();
    final rendezvous = Rendezvous(Directory(
        rendezvousOverride.isNotEmpty ? rendezvousOverride : "${docsDir.path}/dkg_ui_rendezvous",),);
    expect(rendezvous.dir.existsSync(), isTrue,
        reason: "rendezvous dir ${rendezvous.dir.path} missing — start this test from test_dkg_ui.py",);

    final config = rendezvous.readConfig();
    final dbPath = config["db_path"] as String;
    final lwd = config["lwd"] as String;
    final n = config["n"] as int;
    final t = config["t"] as int;
    final myId = config["my_id"] as int;
    final dkgName = config["name"] as String;

    // The database filename must contain "regtest": that substring is what
    // selects Network::Regtest in rust/src/api/coin.rs.
    expect(dbPath, contains("regtest"));

    final prefs = SharedPreferencesAsync();
    // Saved so the developer's own app settings survive the test run.
    final savedPinLock = await prefs.getBool("pin_lock");
    final savedOffline = await prefs.getBool("offline");
    final savedVault = await prefs.getBool("vault");

    try {
      // ── Setup: Rust bridge, database, funding account ─────────────────────
      await tester.runAsync(() async {
        await RustLib.init();
        await initDatadir(directory: docsDir.path);

        final dbFile = File(dbPath);
        if (dbFile.existsSync()) dbFile.deleteSync();

        await prefs.setBool("pin_lock", false);
        await prefs.setBool("offline", false);
        await prefs.setBool("vault", false);

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
            ledger: false,
          ),
          c: coinContext.coin,
        );
        await coinContext.setAccount(account: account);
        // uaPools 6 = sapling|orchard, matching the GraphQL `ironwood` address.
        final addresses = await getAddresses(uaPools: 6, c: coinContext.coin);
        final fundingAddress = addresses.oaddr!;
        debugPrint("[dkg-ui] funding account $account address $fundingAddress");
        rendezvous.publish("funding_address", fundingAddress);
      });

      // ── Start the app directly on the DKG wizard ──────────────────────────
      await tester.pumpWidget(
          ZkoolApp(router: router(true, false, initialLocation: "/dkg1")),);
      await tester.pump(const Duration(seconds: 1));

      final view = tester.view;
      final logicalSize = view.physicalSize / view.devicePixelRatio;
      debugPrint(
        "[dkg-ui] window ${logicalSize.width.toStringAsFixed(0)}"
        "x${logicalSize.height.toStringAsFixed(0)} logical "
        "(dpr ${view.devicePixelRatio})",
      );

      // Splash normally does this; without it the settings provider stays in
      // AsyncLoading and `startSynchronize`'s requireValue would throw.
      final container = ProviderScope.containerOf(tester.element(find.byType(MaterialApp)));
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

      // ── Page 1: DKG parameters ────────────────────────────────────────────
      await pumpUntil(tester, () => find.text("Number of Participants").evaluate().isNotEmpty,
          timeout: const Duration(seconds: 30), what: "DKG page 1",);
      await tester.enterText(fieldNamed("name"), dkgName);
      await tester.pump();
      await selectDropdown(tester, "Number of Participants", "$n");
      await selectDropdown(tester, "Your Participant ID", "$myId");
      await selectDropdown(tester, "Number of Signers Required (Threshold)", "$t");
      await selectDropdown(tester, "Funding Account for the DKG messages", "DKG-Fund");
      await tapNext(tester);

      // ── Page 2: exchange DKG addresses ────────────────────────────────────
      await pumpUntil(tester, () => find.text("DKG Addresses").evaluate().isNotEmpty,
          timeout: const Duration(minutes: 2), what: "DKG page 2",);
      await pumpUntil(tester, () => fieldNamed("${myId - 1}").evaluate().isNotEmpty,
          timeout: const Duration(minutes: 2), what: "own DKG address field",);

      final myAddress =
          tester.widget<FormBuilderTextField>(fieldNamed("${myId - 1}")).initialValue!;
      expect(myAddress, isNotEmpty);
      debugPrint("[dkg-ui] my DKG address $myAddress");
      rendezvous.publish("dkg_address", myAddress);

      Map<String, dynamic>? peers;
      await pumpUntil(tester, () {
        peers = rendezvous.readPeers();
        return peers != null && peers!.length == n - 1;
      }, timeout: const Duration(minutes: 10), what: "peer DKG addresses",);

      for (final entry in peers!.entries) {
        final index = int.parse(entry.key) - 1;
        final field = fieldNamed("$index");
        await tester.ensureVisible(field);
        await tester.enterText(field, entry.value as String);
        await tester.pump();
      }
      await tapNext(tester);

      // ── Page 3: run the DKG rounds ────────────────────────────────────────
      await pumpUntil(tester, () => find.text("Distributed Key Generation").evaluate().isNotEmpty,
          timeout: const Duration(minutes: 2), what: "DKG page 3",);

      const sharedPrefix = "The shared address is: ";
      CopyableText? status;
      var lastStatus = "";
      await pumpUntil(
        tester,
        () {
          final found = find.byType(CopyableText).evaluate();
          if (found.isEmpty) return false;
          final w = found.first.widget as CopyableText;
          // Publish every status change: it makes the round progression
          // visible to the orchestrator and to anyone reading the log.
          if (w.text.isNotEmpty && w.text != lastStatus) {
            lastStatus = w.text;
            debugPrint("[dkg-ui] status: ${w.text}");
            rendezvous.publish("status", w.text);
          }
          if (!w.text.startsWith(sharedPrefix)) return false;
          status = w;
          return true;
        },
        timeout: const Duration(minutes: 15),
        what: "the shared address",
      );

      final sharedAddress = status!.text.substring(sharedPrefix.length).trim();
      expect(sharedAddress, isNotEmpty);
      debugPrint("[dkg-ui] shared address $sharedAddress");
      rendezvous.publish("shared_address", sharedAddress);

      // Hold the finished screen briefly so the "Finalize" step and the shared
      // address are actually observable (on screen, and in a screen recording)
      // instead of vanishing the instant the assertion passes.
      await pumpFor(tester, const Duration(seconds: 6));
    } finally {
      await restoreBool(prefs, "pin_lock", savedPinLock);
      await restoreBool(prefs, "offline", savedOffline);
      await restoreBool(prefs, "vault", savedVault);
    }
  }, timeout: const Timeout(Duration(minutes: 40)),);
}

Future<void> restoreBool(SharedPreferencesAsync prefs, String key, bool? value) async {
  if (value == null) {
    await prefs.remove(key);
  } else {
    await prefs.setBool(key, value);
  }
}

Finder fieldNamed(String name) =>
    find.byWidgetPredicate((w) => w is FormBuilderTextField && w.name == name);

/// Pumps real frames until [done] holds. `pumpAndSettle` is unusable on the DKG
/// route: DKGPage3 installs a 30s periodic timer and the synchronizer keeps
/// scheduling frames, so the tree never settles.
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

/// Pumps real frames for [duration]. Used instead of `pumpAndSettle`, which
/// cannot be trusted anywhere on this route (see [pumpUntil]).
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
