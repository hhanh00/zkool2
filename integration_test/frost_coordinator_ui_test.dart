/// FROST signing with the Flutter app as the coordinator.
///
/// Mirrors `frost_ui_test.dart` up to the shared key, then the app plans a
/// payment from the group account on the send page, saves the unsigned
/// transaction for the other signers, accepts the offer to coordinate, and
/// drives /frost1 and /frost2 as the coordinator while the headless peers
/// (`tests/tests/test_frost_coordinator_ui.py`) sign the file it saved.
library;

import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter_form_builder/flutter_form_builder.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:zkool/main.dart';
import 'package:zkool/router.dart';
import 'package:zkool/src/rust/api/account.dart';
import 'package:zkool/store.dart';
import 'package:zkool/utils.dart';

import 'support.dart';

const _offer = "The transaction is saved. Share the file with the other signers. "
    "Start the signing session now, as the coordinator?";

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();
  // ignore: avoid_print
  debugPrint = (String? message, {int? wrapWidth}) => print(message ?? "");

  testWidgets("FROST coordinator through the Flutter UI", (tester) async {
    final docsDir = await appDocumentsDir();
    final rendezvous = Rendezvous.fromEnvironment(docsDir);
    expect(rendezvous.dir.existsSync(), isTrue,
        reason: "rendezvous dir ${rendezvous.dir.path} missing — "
            "start this test from test_frost_coordinator_ui.py",);

    final config = rendezvous.readConfig();
    final dbPath = config["db_path"] as String;
    final myId = config["my_id"] as int;
    final name = config["name"] as String;
    final sharedFunding = config["shared_funding"] as String;
    final sendAmount = config["send_amount"] as String;
    expect(dbPath, contains("regtest"));

    final prefs = SharedPreferencesAsync();
    final savedPrefs = await SavedPrefs.forceTestValues(prefs);
    // The unsigned transaction goes through the save dialog, which the test
    // cannot drive: a fake saves into the rendezvous directory, where the
    // orchestrator picks it up for the other signers.
    final picker = FakeFileDialogs(Directory("${rendezvous.dir.path}/files")..createSync());
    fileDialogs = picker;

    try {
      // ── Phase 1: DKG, to get a shared key to sign with ────────────────────
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

      final appRouter = router(true, false, initialLocation: "/dkg1");
      await tester.pumpWidget(ZkoolApp(router: appRouter));
      await tester.pump(const Duration(seconds: 1));

      final container = ProviderScope.containerOf(tester.element(find.byType(MaterialApp)));
      await primeProviders(tester, container);

      final sharedAddress = await driveDkg(
        tester,
        rendezvous,
        name: name,
        n: config["n"] as int,
        t: config["t"] as int,
        myId: myId,
      );
      rendezvous.publish("shared_address", sharedAddress);

      // Leave the DKG page so its protocol driver is cancelled (see
      // frost_ui_test.dart).
      appRouter.go("/accounts");
      await pumpFor(tester, const Duration(seconds: 2));

      container.invalidate(getAccountsProvider);
      final frostAccountId = (await tester.runAsync(() async {
        final accounts = await container.read(getAccountsProvider.future);
        return accounts
            .firstWhere(
              (a) => a.name == name,
              orElse: () => throw StateError("no FROST account named $name after the DKG"),
            )
            .id;
      }))!;
      debugPrint("[dkg-ui] FROST account $frostAccountId");

      // ── Phase 2: the orchestrator funds the shared address and names the
      // receiver ────────────────────────────────────────────────────────────
      String? receiver;
      await pumpUntil(
        tester,
        () {
          receiver = rendezvous.readOrchestrator()?["receiver"] as String?;
          return receiver != null;
        },
        timeout: const Duration(minutes: 15),
        what: "the receiver's address from the orchestrator",
      );
      appRouter.go(accountRoute("/", frostAccountId));
      await waitForAccountView(tester, name);
      await waitForBalance(tester, frostAccountId, atLeast: zecToZat(sharedFunding), what: "the shared address to be funded");
      await waitForSync(tester, frostAccountId);

      // ── Phase 3: plan the payment and save it for the signers ─────────────
      await tapTooltip(tester, "Send funds to one or many addresses");
      await pumpUntil(tester, () => fieldNamed("address").evaluate().isNotEmpty,
          timeout: shortTimeout, what: "the send page",);
      await enterRecipient(tester, receiver!, sendAmount);
      await tapTooltip(tester, "Send transaction (including queued recipients)");
      await pumpUntil(tester, () => find.text("Extra Options").evaluate().isNotEmpty,
          timeout: shortTimeout, what: "the send options page",);
      await tapTooltip(tester, "Send (Summary and Confirmation)");
      await pumpUntil(tester, () => find.byTooltip("Save the transaction for the group signers").evaluate().isNotEmpty,
          timeout: const Duration(minutes: 2), what: "the plan with its save button",);
      await tapTooltip(tester, "Save the transaction for the group signers");
      await pumpUntil(tester, () => picker.saved.isNotEmpty, timeout: shortTimeout, what: "the unsigned transaction file");
      rendezvous.publish("pczt_file", picker.saved.last);

      // ── Phase 4: coordinate from the offer ────────────────────────────────
      await answerDialog(tester, _offer, "Start");
      await pumpUntil(
        tester,
        () => find.text("ID of the coordinator").evaluate().isNotEmpty,
        timeout: const Duration(minutes: 2),
        what: "FROST page 1",
      );
      // Opened from the offer, the page proposes this member as the
      // coordinator.
      final coordinatorField = (dropdownNamed("coordinator").evaluate().single as StatefulElement).state as FormBuilderFieldState;
      expect(coordinatorField.value, myId, reason: "the coordinator defaults to this member");
      // Every member of the group is on offer, and no one else (#1393).
      expect(tester.widget<FormBuilderDropdown<int>>(dropdownNamed("coordinator")).items, hasLength(config["n"] as int),
          reason: "the coordinator dropdown lists the group's members",);
      await selectDropdown(tester, "Funding Account for the FROST messages", "DKG-Fund");
      await tapNext(tester);
      await pumpUntil(
        tester,
        () => find.text("Frost Multi Party Signature").evaluate().isNotEmpty,
        timeout: const Duration(minutes: 2),
        what: "FROST page 2",
      );

      // The coordinator ends by sending the transaction and showing its id.
      final status = await awaitStatus(
        tester,
        rendezvous,
        readStatus: () {
          final found = find.byWidgetPredicate((w) => _isSigningStatus(_statusText(w))).evaluate();
          if (found.isEmpty) return "";
          return _statusText(found.first.widget);
        },
        isDone: (s) => s == "Signing completed" || s.startsWith("TX ID: "),
        timeout: const Duration(minutes: 20),
        what: "signing to complete",
      );
      rendezvous.publish("signing_status", status);

      await pumpFor(tester, const Duration(seconds: 6));
    } finally {
      fileDialogs = const SystemFileDialogs();
      await savedPrefs.restore(prefs);
    }
  }, timeout: const Timeout(Duration(minutes: 60)));
}

/// The messages FrostPage2 can show; used to pick the status Text out of the
/// page without matching the AppBar title or the stepper labels.
const _signingStatuses = [
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
