/// Integration test: restore a FROST member from its backup string in the
/// Flutter app, then sign with it. Two shapes, picked by `dkg_version` in the
/// rendezvous config:
///
/// - **Legacy (1).** Three headless `zkool_graphql` peers form a dkg-1 group
///   and fund it; participant #3 exports its `zkfrost` string and goes away.
///   The app restores that member through the new-account page (the
///   generation selector defaults to Legacy), sees the snowflake and the
///   "not quantum recoverable" banner, taps Upgrade (which creates the ZIP
///   2005 sibling and plans the sweep), then signs the sweep participant #2
///   coordinates. Once the funds sit in the sibling, neither account shows
///   the banner.
/// - **ZIP 2005 (2).** The peers form a dkg-2 group. The app restores (the
///   selector defaults to ZIP 2005), sees the snowflake and no banner, and
///   signs an ordinary payment participant #2 coordinates.
///
/// On a machine without biometrics it also opens the Viewing Keys page and
/// checks the backup string shown there. `tests/tests/test_frost_restore_ui.py`
/// owns the chain and the peers.
library;

import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_form_builder/flutter_form_builder.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:zkool/main.dart';
import 'package:zkool/pages/tx.dart';
import 'package:zkool/router.dart';
import 'package:zkool/src/rust/api/account.dart';
import 'package:zkool/src/rust/api/frost.dart';
import 'package:zkool/src/rust/api/pay.dart';
import 'package:zkool/store.dart';
import 'package:zkool/widgets/frost_generation_select.dart';

import 'support.dart';

const _notRecoverable = "not quantum recoverable";

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  testWidgets("FROST restore from a backup string through the Flutter UI", (tester) async {
    final docsDir = await appDocumentsDir();
    final rendezvous = Rendezvous.fromEnvironment(docsDir);
    expect(rendezvous.dir.existsSync(), isTrue,
        reason: "rendezvous dir ${rendezvous.dir.path} missing — "
            "start this test from test_frost_restore_ui.py",);

    final config = rendezvous.readConfig();
    final dbPath = config["db_path"] as String;
    final name = config["name"] as String;
    final backup = config["backup"] as String;
    final ufvk = config["ufvk"] as String;
    final coordinator = config["coordinator"] as int;
    final checkBackupPage = config["check_backup_page"] as bool? ?? false;
    final legacy = (config["dkg_version"] as int) < 2;
    final defaultGeneration = legacy ? FrostGeneration.legacy : FrostGeneration.zip2005;
    expect(dbPath, contains("regtest"));
    expect(backup, startsWith("zkfrost1"));

    final prefs = SharedPreferencesAsync();
    final savedPrefs = await SavedPrefs.forceTestValues(prefs);

    try {
      await tester.runAsync(() async {
        final account = await setUpTestWallet(
          dbPath: dbPath,
          lwd: config["lwd"] as String,
          docsDir: docsDir.path,
        );
        final addresses = await getAddresses(uaPools: 6, account: account, c: coinContext.coin);
        debugPrint("[dkg-ui] funding account $account address ${addresses.oaddr}");
        rendezvous.publish("funding_address", addresses.oaddr!);
      });

      // Start on the account list, as the signing test does after its DKG:
      // the root route mounts the account view, which records the selected
      // account from initState, and riverpod's debug check rejects that
      // during the first build.
      final appRouter = router(true, false, initialLocation: "/accounts");
      await tester.pumpWidget(ZkoolApp(router: appRouter));
      await tester.pump(const Duration(seconds: 1));
      final container = ProviderScope.containerOf(tester.element(find.byType(MaterialApp)));
      await primeProviders(tester, container);

      // ── Phase 1: restore from the string and the group's viewing key ──────
      unawaited(appRouter.push("/account/new"));
      await pumpUntil(
        tester,
        () => find.text("Create a new wallet or restore an existing one").evaluate().isNotEmpty,
        timeout: const Duration(seconds: 30),
        what: "the new account page",
      );
      await tester.enterText(fieldNamed("name"), name);
      await tester.pump();
      final restoreSwitch = find.byWidgetPredicate((w) => w is FormBuilderSwitch && w.name == "restore");
      await tester.ensureVisible(restoreSwitch);
      await tester.tap(restoreSwitch);
      await pumpFor(tester, const Duration(seconds: 1));
      await tester.enterText(fieldNamed("key"), backup);
      await tester.pump();
      await pumpUntil(
        tester,
        () => fieldNamed("ufvk").evaluate().isNotEmpty,
        timeout: const Duration(seconds: 10),
        what: "the group viewing key field",
      );
      await tester.ensureVisible(fieldNamed("ufvk"));
      await tester.enterText(fieldNamed("ufvk"), ufvk);
      await tester.pump();
      // The form defaults to the generation the pasted key matches.
      await pumpUntil(
        tester,
        () {
          final found = find.byType(FrostGenerationSelect).evaluate();
          if (found.isEmpty) return false;
          final w = found.first.widget as FrostGenerationSelect;
          return w.initialValue == defaultGeneration.bit;
        },
        timeout: const Duration(seconds: 30),
        what: "the generation selector to default to ${defaultGeneration.label}",
      );
      await tester.tap(find.byTooltip("Save"));
      // The app offers to synchronize when it knows the chain height, which
      // under the test harness it may not; either way it lands on the
      // account page, where a FROST account shows a snowflake by its name.
      await pumpUntil(
        tester,
        () =>
            find.text("FROST Account Restored").evaluate().isNotEmpty ||
            find.byIcon(Icons.ac_unit).evaluate().isNotEmpty,
        timeout: const Duration(minutes: 2),
        what: "the restore to complete",
      );
      if (find.text("FROST Account Restored").evaluate().isNotEmpty) {
        await tester.tap(find.text("Ok")); // synchronize now
        await pumpFor(tester, const Duration(seconds: 2));
      }
      await pumpUntil(
        tester,
        () => find.byIcon(Icons.ac_unit).evaluate().isNotEmpty,
        timeout: const Duration(minutes: 2),
        what: "the restored account page with the snowflake",
      );
      late final int frostAccountId;
      await tester.runAsync(() async {
        container.invalidate(getAccountsProvider);
        final accounts = await container.read(getAccountsProvider.future);
        frostAccountId = accounts.firstWhere((a) => a.name == name).id;
        final backupAgain = await exportFrostBackup(account: frostAccountId, c: coinContext.coin);
        expect(backupAgain, backup, reason: "the restored account exports the same string");
      });
      debugPrint("[dkg-ui] restored FROST account $frostAccountId");
      rendezvous.publish("restored", "$frostAccountId");

      int? siblingId;
      if (legacy) {
        // ── Phase 2: the flag, once the legacy funds are seen ───────────────
        await _syncUntil(
          tester,
          () => find.textContaining(_notRecoverable).evaluate().isNotEmpty,
          what: "the not-recoverable banner",
        );
        debugPrint("[dkg-ui] banner shown");

        // ── Phase 3: Upgrade creates the sibling and plans the sweep ────────
        final upgradeButton = find.widgetWithText(FilledButton, "Upgrade");
        await tester.ensureVisible(upgradeButton);
        await tester.pump();
        await tester.tap(upgradeButton);
        // The confirmation dialog: its OK button is the reliable finder (the
        // dialog package does not render the title as a plain Text).
        await pumpUntil(
          tester,
          () => find.text("Ok").evaluate().isNotEmpty,
          timeout: const Duration(seconds: 30),
          what: "the upgrade confirmation",
        );
        await tester.tap(find.text("Ok"));
        // The app plans the sweep and opens the transaction page with it.
        await pumpUntil(
          tester,
          () => find.byType(TxPage).evaluate().isNotEmpty,
          timeout: const Duration(minutes: 3),
          what: "the transaction page with the planned sweep",
        );
        await tester.runAsync(() async {
          container.invalidate(getAccountsProvider);
          final accounts = await container.read(getAccountsProvider.future);
          siblingId = accounts.firstWhere((a) => a.name == "$name (ZIP 2005)").id;
          final addresses = await getAddresses(uaPools: 6, account: siblingId!, c: coinContext.coin);
          debugPrint("[dkg-ui] sibling account $siblingId address ${addresses.oaddr}");
          rendezvous.publish("sibling_address", addresses.oaddr!);
        });
        // Participant #2 coordinates the sweep; leave our own plan unsent.
        appRouter.go(accountRoute("/", frostAccountId));
        await pumpFor(tester, const Duration(seconds: 2));
      } else {
        // ── Phase 2: a ZIP 2005 group has nothing to fix: funds, no banner ──
        await _syncUntil(
          tester,
          () {
            final data = _accountData(container, frostAccountId);
            return data != null && data.balance.field0[3] > BigInt.zero;
          },
          what: "the restored account to see the group's funds",
        );
        expect(_frostParams(container, frostAccountId)?.quantumRecoverable, isTrue);
        expect(find.textContaining(_notRecoverable), findsNothing);
        debugPrint("[dkg-ui] funds seen, no banner");
      }

      // ── Phase 4: sign what the coordinator prepared (the sweep, or a payment)
      Map<String, dynamic>? fromOrchestrator;
      await pumpUntil(
        tester,
        () {
          fromOrchestrator = rendezvous.readOrchestrator();
          return fromOrchestrator?["pczt"] != null;
        },
        timeout: const Duration(minutes: 15),
        what: "the sweep PCZT from the coordinator",
      );
      late final PcztPackage pczt;
      await tester.runAsync(() async {
        pczt = await unpackTransaction(bytes: _hexToBytes(fromOrchestrator!["pczt"] as String));
      });
      appRouter.go(accountRoute("/frost1", frostAccountId), extra: pczt);
      await pumpUntil(
        tester,
        () => find.text("ID of the coordinator").evaluate().isNotEmpty,
        timeout: const Duration(minutes: 2),
        what: "FROST page 1",
      );
      await selectDropdown(tester, "ID of the coordinator", "$coordinator");
      await selectDropdown(tester, "Funding Account for the FROST messages", "DKG-Fund");
      await tapNext(tester);
      await pumpUntil(
        tester,
        () => find.text("Frost Multi Party Signature").evaluate().isNotEmpty,
        timeout: const Duration(minutes: 2),
        what: "FROST page 2",
      );
      const completed = "Signing completed";
      await awaitStatus(
        tester,
        rendezvous,
        readStatus: () {
          final found = find
              .byWidgetPredicate((w) => _isSigningStatus(_statusText(w)))
              .evaluate();
          if (found.isEmpty) return "";
          return _statusText(found.first.widget);
        },
        isDone: (s) => s == completed,
        timeout: const Duration(minutes: 20),
        what: "signing to complete",
      );
      rendezvous.publish("signing_status", completed);

      // ── Phase 5: once mined, the accounts read right and show no banner ───
      await pumpUntil(
        tester,
        () => rendezvous.readOrchestrator()?["mined"] == true,
        timeout: const Duration(minutes: 15),
        what: "the orchestrator to see the transaction mined",
      );
      appRouter.go(accountRoute("/", frostAccountId));
      await pumpFor(tester, const Duration(seconds: 2));
      await _syncUntil(
        tester,
        () => _frostParams(container, frostAccountId)?.quantumRecoverable == true,
        what: "the account to read as recoverable",
      );
      expect(find.textContaining(_notRecoverable), findsNothing);
      expect(find.byIcon(Icons.ac_unit), findsOneWidget);

      if (siblingId != null) {
        appRouter.go(accountRoute("/", siblingId!));
        await pumpFor(tester, const Duration(seconds: 2));
        await _syncUntil(
          tester,
          () {
            final data = _accountData(container, siblingId!);
            return data != null &&
                data.balance.field0[3] > BigInt.zero &&
                data.frostParams?.quantumRecoverable == true;
          },
          what: "the sibling to hold the swept funds in Ironwood",
        );
        expect(find.textContaining(_notRecoverable), findsNothing);
        expect(find.byIcon(Icons.ac_unit), findsOneWidget);
      }

      // ── Phase 6: the backup string on the Viewing Keys page ───────────────
      if (checkBackupPage) {
        unawaited(appRouter.push("/viewing_keys", extra: frostAccountId));
        await pumpUntil(
          tester,
          () => find.byTooltip("Show Backup String").evaluate().isNotEmpty,
          timeout: const Duration(minutes: 1),
          what: "the viewing keys page",
        );
        await tester.tap(find.byTooltip("Show Backup String"));
        await pumpUntil(
          tester,
          () => find.text(backup).evaluate().isNotEmpty,
          timeout: const Duration(minutes: 1),
          what: "the backup string",
        );
        expect(find.textContaining("device loss"), legacy ? findsOneWidget : findsNothing,
            reason: "only a legacy account carries the device-loss-only note",);
      }

      rendezvous.publish("done", "true");
      await pumpFor(tester, const Duration(seconds: 6));
    } finally {
      await savedPrefs.restore(prefs);
    }
  }, timeout: const Timeout(Duration(minutes: 60)));
}

/// Taps the account page's sync button until [done] holds. The restored
/// account is born at Orchard activation and the chain is short, so each
/// sync is quick; the data changes only once the chain has what we wait for.
/// Taps are spaced out: a sync that overlaps a protocol pass makes one of
/// them fail on the SQLite lock, which the app only reports as a warning.
Future<void> _syncUntil(WidgetTester tester, bool Function() done, {required String what}) async {
  final deadline = DateTime.now().add(const Duration(minutes: 10));
  while (DateTime.now().isBefore(deadline)) {
    if (done()) return;
    final sync = find.byTooltip("Sync this account");
    if (sync.evaluate().isNotEmpty) {
      await tester.tap(sync, warnIfMissed: false);
    }
    await pumpFor(tester, const Duration(seconds: 20));
    if (done()) return;
  }
  fail("timed out waiting for $what");
}

AccountData? _accountData(ProviderContainer container, int account) =>
    container.read(accountProvider(account)).value;

FrostParams? _frostParams(ProviderContainer container, int account) =>
    _accountData(container, account)?.frostParams;

const _signingStatuses = [
  "Announcing our mailbox address to the participants",
  "Waiting for the coordinator to announce its mailbox address",
  "Waiting for our last message to be mined",
  "Waiting for other participants to send their commitments",
  "Sending our commitments to the coordinator",
  "Broadcasting the signing package to all participants",
  "Waiting for the signing package from the coordinator",
  "Sending our signature share to the coordinator",
  "Signing completed",
  "Waiting for the signature share from the other participants",
  "Assembling the transaction",
  "Sending the transaction to the network",
];

bool _isSigningStatus(String s) =>
    _signingStatuses.contains(s) || s.startsWith("TX ID: ") || s.startsWith("This transaction expired");

/// A status line's text: a plain Text, or the SelectableText the page uses
/// for the transaction id so it can be copied. Empty for anything else.
String _statusText(Widget w) => switch (w) {
      Text(:final data) => data ?? "",
      SelectableText(:final data) => data ?? "",
      _ => "",
    };

List<int> _hexToBytes(String hex) => [
      for (var i = 0; i < hex.length; i += 2) int.parse(hex.substring(i, i + 2), radix: 16),
    ];
