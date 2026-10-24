/// Docs screenshots: the MultiSig Accounts / FROST pages of the docs site.
///
/// Not a test of the app but a scripted walkthrough that
/// `tests/tests/test_docs_frost_ui.py` drives on regtest. The app is
/// participant #1 of a 2-of-3 group whose other members are headless
/// `zkool_graphql` instances: it goes through the DKG wizard, shows the
/// group's account, viewing keys page and receive page, the restore form
/// for a backup string, then plans a payment, saves it for the other
/// signers, loads it back as a signer would, and coordinates the signing.
/// Every shot is named after the image it replaces under
/// `docs/frost/images` in the docs site. See `docs_shots_support.dart` for
/// how the shots are recorded.
library;

import 'dart:async';
import 'dart:io';

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
import 'package:zkool/store.dart';
import 'package:zkool/utils.dart';

import 'docs_shots_support.dart';
import 'support.dart';

const _funding = "Funding";
const _restored = "Alice";
const _sendTooltip = "Send funds to one or many addresses";
const _loadTooltip = "Load an unsigned transaction";
const _saveTooltip = "Save the transaction for the group signers";
const _signTooltip = "Sign with the group and send";
const _dkgDone = "The group's key is ready. Its shared address is on this page, and the group's account has been created.";
const _offer = "The transaction is saved. Share the file with the other signers. "
    "Start the signing session now, as the coordinator?";
const _txSent = "The transaction was sent to the network.";
const _waitingCommitments = "Waiting for other participants to send their commitments";
// Long enough for a toast, such as the synchronizer's, to have gone.
const toastGone = Duration(seconds: 6);

/// The messages FrostPage2 can show; used to pick the status Text out of the
/// page without matching the AppBar title or the stepper labels.
const _signingStatuses = [
  "Announcing our mailbox address to the participants",
  "Waiting for the coordinator to announce its mailbox address",
  "Waiting for our last message to be mined",
  _waitingCommitments,
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

String _statusText(Widget w) => switch (w) {
      Text(:final data) => data ?? "",
      SelectableText(:final data) => data ?? "",
      _ => "",
    };

String _readSigningStatus() {
  final found = find.byWidgetPredicate((w) => _isSigningStatus(_statusText(w))).evaluate();
  return found.isEmpty ? "" : _statusText(found.first.widget);
}

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();
  // ignore: avoid_print
  debugPrint = (String? message, {int? wrapWidth}) => print(message ?? "");

  testWidgets("Docs: FROST screenshots", (tester) async {
    final docsDir = await appDocumentsDir();
    final rendezvous = Rendezvous.fromEnvironment(docsDir);
    expect(rendezvous.dir.existsSync(), isTrue,
        reason: "rendezvous dir ${rendezvous.dir.path} missing — "
            "start this from test_docs_frost_ui.py",);

    final config = rendezvous.readConfig();
    final dbPath = config["db_path"] as String;
    final name = config["name"] as String;
    final n = config["n"] as int;
    final threshold = config["t"] as int;
    final myId = config["my_id"] as int;
    final participantFunding = config["participant_funding"] as String;
    final sharedFunding = config["shared_funding"] as String;
    final amount = config["amount"] as String;
    expect(dbPath, contains("regtest"));

    final prefs = SharedPreferencesAsync();
    final savedPrefs = await SavedPrefs.forceTestValues(prefs, defaultTheme: true);
    final shots = Shots(tester, docsDir, config);
    // The unsigned transaction goes through the save dialog, which the test
    // cannot drive: a fake saves into the rendezvous directory, where the
    // orchestrator picks it up for the other signer, and loads it back.
    final files = FakeFileDialogs(Directory("${rendezvous.dir.path}/files")..createSync());
    fileDialogs = files;

    Future<void> waitForText(String text, {required String what}) =>
        pumpUntil(tester, () => find.text(text).evaluate().isNotEmpty, timeout: shortTimeout, what: what);

    // A badge just right of the widget [finder] matches, outside it.
    Annot besides(Finder finder, int n) => shots.badgeAt(shots.rectOf(finder).centerRight + const Offset(26, 0), n);

    Future<void> waitForOrchestrator(String key, {required Duration timeout, required String what}) =>
        pumpUntil(tester, () => rendezvous.readOrchestrator()?[key] != null, timeout: timeout, what: what);

    /// Takes [shot] once [when] holds, or without it after [timeout]: a
    /// status the peers may race past is still worth the page as it is.
    /// Shot at once, before the status moves on.
    Future<void> takeWhen(String shot, bool Function() when, {required Duration timeout, List<Annot> Function()? annots}) async {
      try {
        await pumpUntil(tester, when, timeout: timeout, what: shot);
      } on TestFailure {
        debugPrint("[docs-shots] $shot taken without its expected status");
      }
      await shots.take(shot, annots: annots, settle: const Duration(milliseconds: 100));
    }

    try {
      await tester.runAsync(() async {
        await openTestWallet(
          dbPath: dbPath,
          lwd: config["lwd"] as String,
          docsDir: docsDir.path,
        );
      });
      final fundingId = (await tester.runAsync(() => newAccount(
            na: const NewAccount(
              name: _funding,
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
          )))!;

      final appRouter = router(true, false, initialLocation: "/accounts");
      await tester.pumpWidget(RepaintBoundary(key: shots.boundary, child: ZkoolApp(router: appRouter)));
      await tester.pump(const Duration(seconds: 1));
      final container = ProviderScope.containerOf(tester.element(find.byType(MaterialApp)));
      await primeProviders(tester, container);
      await waitForAccountList(tester);
      await tester.runAsync(() => container.read(currentHeightProvider.notifier).fetch(force: true));
      await pumpUntil(tester, () => find.text("Height: N/A").evaluate().isEmpty, timeout: shortTimeout, what: "the chain height");
      final addresses = (await tester.runAsync(
        () => getAddresses(uaPools: 7, account: fundingId, c: coinContext.coin),
      ))!;
      rendezvous.publish("funding_address", addresses.ua!);

      // ── The funding account, with its ZEC ────────────────────────────────
      appRouter.go(accountRoute("/", fundingId));
      await waitForAccountView(tester, _funding);
      await waitForBalance(tester, fundingId, atLeast: zecToZat(participantFunding), what: "the funding to arrive");
      await waitForSync(tester, fundingId);
      appRouter.go("/accounts");
      await waitForAccountList(tester);
      await shots.take("frost/images/01.dkg", settle: toastGone);

      // ── The new account page: the DKG button ─────────────────────────────
      await tapTooltip(tester, "Create new account");
      await pumpUntil(tester, () => find.byTooltip("Start Distributed Key Generation").evaluate().isNotEmpty,
          timeout: shortTimeout, what: "the new account page",);
      await shots.take("frost/images/02.dkg", annots: () => [shots.box(find.byTooltip("Start Distributed Key Generation"))]);
      await tapTooltip(tester, "Start Distributed Key Generation");
      await waitForText("Number of Participants", what: "DKG page 1");

      // ── The DKG form ─────────────────────────────────────────────────────
      await tester.enterText(fieldNamed("name"), name);
      await tester.pump();
      await selectDropdown(tester, "Number of Participants", "$n");
      await selectDropdown(tester, "Your Participant ID", "$myId");
      await selectDropdown(tester, "Number of Signers Required (Threshold)", "$threshold");
      await selectDropdown(tester, "Funding Account for the DKG messages", _funding);
      await shots.take("frost/images/03.form", annots: () => [
        shots.badge(fieldNamed("name"), 1),
        shots.badge(dropdownNamed("participants"), 2),
        shots.badge(dropdownNamed("id"), 3),
        shots.badge(dropdownNamed("threshold"), 4),
        shots.badge(dropdownNamed("account"), 5),
      ]);
      await tapNext(tester);

      // ── The participants' addresses ──────────────────────────────────────
      await pumpUntil(tester, () => find.text("DKG Addresses").evaluate().isNotEmpty && fieldNamed("${myId - 1}").evaluate().isNotEmpty,
          timeout: const Duration(minutes: 2), what: "DKG page 2 with our address",);
      final myAddress = tester.widget<FormBuilderTextField>(fieldNamed("${myId - 1}")).initialValue!;
      expect(myAddress, isNotEmpty);
      await shots.take("frost/images/04.addresses");
      rendezvous.publish("dkg_address", myAddress);
      Map<String, dynamic>? peers;
      await pumpUntil(tester, () {
        peers = rendezvous.readPeers();
        return peers != null && peers!.length == n - 1;
      }, timeout: const Duration(minutes: 10), what: "the peers' DKG addresses",);
      for (final entry in peers!.entries) {
        final field = fieldNamed("${int.parse(entry.key) - 1}");
        await tester.ensureVisible(field);
        await tester.enterText(field, entry.value as String);
        await tester.pump();
      }
      await shots.take("frost/images/05.addresses_filled");
      await tapNext(tester);

      // ── The rounds, then the shared address ──────────────────────────────
      await waitForText("Distributed Key Generation", what: "DKG page 3");
      await takeWhen(
        "frost/images/06.dkg_r1",
        () => readCopyableStatus().startsWith("Waiting for other participants"),
        timeout: const Duration(minutes: 2),
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
      rendezvous.publish("shared_address", sharedAddress);
      await answerDialog(tester, _dkgDone, "Ok");
      await shots.take("frost/images/07.dkg_end", annots: () => [
        shots.box(find.byWidgetPredicate((w) => w is CopyableText && w.text.startsWith(sharedAddressPrefix))),
        shots.box(find.widgetWithText(ElevatedButton, "Dismiss")),
      ]);
      await tester.tap(find.widgetWithText(ElevatedButton, "Dismiss"));
      await pumpFor(tester, const Duration(seconds: 2));
      // Away from the DKG page, so its protocol driver is cancelled.
      appRouter.go("/accounts");
      await waitForAccountList(tester);
      container.invalidate(getAccountsProvider);
      final frostId = (await tester.runAsync(() => accountIds()))![name]!;

      // ── The group's account ──────────────────────────────────────────────
      appRouter.go(accountRoute("/", frostId));
      await waitForAccountView(tester, name);
      await pumpUntil(tester, () => find.byIcon(Icons.ac_unit).evaluate().isNotEmpty, timeout: shortTimeout, what: "the group icon");
      await shots.take("frost/images/09.multisig", settle: toastGone, annots: () => [shots.box(find.byIcon(Icons.ac_unit))]);

      // ── Backup: the viewing keys page ────────────────────────────────────
      await openMenuItem(tester, "Backup");
      await pumpUntil(tester, () => find.byTooltip("Show Backup String").evaluate().isNotEmpty && find.text("Incoming").evaluate().isNotEmpty,
          timeout: shortTimeout, what: "the group's viewing keys page",);
      await shots.take("frost/images/16.backup", annots: () => [shots.box(find.byTooltip("Show Backup String"))]);
      // The string itself sits behind the device authentication: read it
      // from the wallet for the restore form.
      final backup = (await tester.runAsync(() => exportFrostBackup(account: frostId, c: coinContext.coin)))!;
      await tester.pageBack();
      await waitForAccountView(tester, name);

      // ── Restore: a backup string on the new account page ─────────────────
      unawaited(appRouter.push("/account/new"));
      await waitForText("Create a new wallet or restore an existing one", what: "the new account page");
      await tester.enterText(fieldNamed("name"), _restored);
      await tester.pump();
      await tester.tap(switchNamed("restore"));
      await pumpUntil(tester, () => fieldNamed("key").evaluate().isNotEmpty, timeout: shortTimeout, what: "the key field");
      await tester.enterText(fieldNamed("key"), backup);
      await tester.pump();
      await pumpUntil(tester, () => fieldNamed("ufvk").evaluate().isNotEmpty, timeout: shortTimeout, what: "the group viewing key field");
      await tester.ensureVisible(find.text("Accounts to restore"));
      await tester.pump();
      await shots.take("frost/images/17.restore", annots: () => [
        shots.badge(fieldNamed("key"), 1),
        shots.badge(fieldNamed("ufvk"), 2),
        besides(find.text("Accounts to restore"), 3),
      ]);
      appRouter.go(accountRoute("/", frostId));
      await waitForAccountView(tester, name);
      // A tap during the route's transition lands on nothing.
      await pumpFor(tester, const Duration(seconds: 1));

      // ── Receive ──────────────────────────────────────────────────────────
      await tapTooltip(tester, "Show the account receiving addresses");
      await pumpUntil(tester, () => find.text("Receive Funds").evaluate().isNotEmpty && copyableWith(sharedAddress).evaluate().isNotEmpty,
          timeout: shortTimeout, what: "the receive page with the shared address",);
      await shots.take("frost/images/08.addresses", annots: () => [
        shots.box(find.byTooltip("Generate a new set of addresses (transparent/sapling and orchard). Previous addresses can still receive funds")),
      ]);
      await tester.pageBack();
      await waitForAccountView(tester, name);

      // ── Funded ───────────────────────────────────────────────────────────
      await waitForOrchestrator("receiver", timeout: const Duration(minutes: 15), what: "the receiver's address from the orchestrator");
      final receiver = rendezvous.readOrchestrator()!["receiver"] as String;
      await waitForBalance(tester, frostId, atLeast: zecToZat(sharedFunding), what: "the shared address to be funded");
      await waitForSync(tester, frostId);

      // ── The payment, saved for the signers ───────────────────────────────
      await tapTooltip(tester, _sendTooltip);
      await pumpUntil(tester, () => fieldNamed("address").evaluate().isNotEmpty && find.byTooltip(_loadTooltip).evaluate().isNotEmpty,
          timeout: shortTimeout, what: "the send page",);
      await shots.take("frost/images/13.load", settle: toastGone, annots: () => [shots.box(find.byTooltip(_loadTooltip))]);
      await enterRecipient(tester, receiver, amount);
      await tapTooltip(tester, "Send transaction (including queued recipients)");
      await waitForText("Extra Options", what: "the send options page");
      await tapTooltip(tester, "Send (Summary and Confirmation)");
      await pumpUntil(tester, () => find.byType(TxPage).evaluate().isNotEmpty && find.byTooltip(_saveTooltip).evaluate().isNotEmpty,
          timeout: const Duration(minutes: 2), what: "the plan with its save button",);
      await shots.take("frost/images/10.send", annots: () => [shots.box(find.byTooltip(_saveTooltip))]);
      await tapTooltip(tester, _saveTooltip);
      await pumpUntil(tester, () => files.saved.isNotEmpty, timeout: shortTimeout, what: "the unsigned transaction file");
      rendezvous.publish("pczt_file", files.saved.last);
      await pumpUntil(tester, () => find.text(_offer).evaluate().isNotEmpty && find.text("Start").evaluate().isNotEmpty,
          timeout: shortTimeout, what: "the offer to coordinate",);
      await shots.take("frost/images/18.offer");
      // The long way round, as a signer who received the file would go.
      await tester.tap(find.text("Not now").last);
      await pumpUntil(tester, () => find.text(_offer).evaluate().isEmpty, timeout: shortTimeout, what: "the offer to close");
      appRouter.go(accountRoute("/", frostId));
      await waitForAccountView(tester, name);
      await pumpFor(tester, const Duration(seconds: 1));

      // ── The file loaded back, and signed with the group ──────────────────
      await tapTooltip(tester, _sendTooltip);
      await pumpUntil(tester, () => find.byTooltip(_loadTooltip).evaluate().isNotEmpty, timeout: shortTimeout, what: "the send page again");
      files.picks.add(files.saved.last);
      await tapTooltip(tester, _loadTooltip);
      await pumpUntil(tester, () => find.byType(TxPage).evaluate().isNotEmpty && find.byTooltip(_signTooltip).evaluate().isNotEmpty,
          timeout: const Duration(minutes: 2), what: "the loaded transaction with the group button",);
      await shots.take("frost/images/19.loaded", annots: () => [shots.box(find.byTooltip(_signTooltip))]);
      await tapTooltip(tester, _signTooltip);
      await waitForText("ID of the coordinator", what: "FROST page 1");
      await selectDropdown(tester, "ID of the coordinator", "$myId");
      await selectDropdown(tester, "Funding Account for the FROST messages", _funding);
      await shots.take("frost/images/11.send", annots: () => [
        shots.badge(dropdownNamed("coordinator"), 1),
        shots.badge(dropdownNamed("account"), 2),
      ]);
      await tapNext(tester);
      await waitForText("Frost Multi Party Signature", what: "FROST page 2");
      // The other signer starts once the coordinator is in its session.
      rendezvous.publish("signing", "true");
      await takeWhen(
        "frost/images/12.commitments",
        () => _readSigningStatus() == _waitingCommitments,
        timeout: const Duration(minutes: 5),
      );
      final last = await awaitStatus(
        tester,
        rendezvous,
        readStatus: _readSigningStatus,
        isDone: (s) => s.startsWith("TX ID: "),
        timeout: const Duration(minutes: 20),
        what: "the transaction to be sent",
      );
      rendezvous.publish("signing_status", last);
      await pumpUntil(tester, () => find.textContaining(_txSent).evaluate().isNotEmpty && find.text("Ok").evaluate().isNotEmpty,
          timeout: shortTimeout, what: "the transaction sent popup",);
      await tester.tap(find.text("Ok").last);
      await pumpUntil(tester, () => find.textContaining(_txSent).evaluate().isEmpty, timeout: shortTimeout, what: "the popup to close");
      await shots.take("frost/images/14.tx", annots: () => [
        shots.box(find.byWidgetPredicate((w) => w is CopyableText && w.text.startsWith("TX ID: "))),
      ]);

      // ── The payment in the group's history ───────────────────────────────
      await waitForOrchestrator("paid", timeout: const Duration(minutes: 15), what: "the orchestrator to see the receiver paid");
      appRouter.go(accountRoute("/", frostId));
      await waitForAccountView(tester, name);
      await waitForSync(tester, frostId);
      await tapTab(tester, "Transactions");
      await pumpUntil(tester, () => find.text("Sent").evaluate().isNotEmpty, timeout: const Duration(minutes: 2), what: "the payment in the history");
      await shots.take("frost/images/15.history", settle: toastGone);

      shots.finish();
      rendezvous.publish("done", "true");
      await pumpFor(tester, const Duration(seconds: 1));
    } finally {
      fileDialogs = const SystemFileDialogs();
      await savedPrefs.restore(prefs);
    }
  }, timeout: const Timeout(Duration(minutes: 60)));
}
