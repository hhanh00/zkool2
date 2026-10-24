/// Docs screenshots: the Voting page of the docs site.
///
/// Not a test of the app but a scripted walkthrough that
/// `tests/tests/test_docs_voting_ui.py` drives on regtest with the local
/// voting stack (`tests/tools/voting_stack.sh`): the voting config URL in
/// the settings, the account menu's Voting item, the list of rounds, the
/// ballot one proposal at a time, the submission preview, the submission in
/// progress, waiting on the helper shares and done, the vote report, and the
/// list afterwards. The account
/// holds one note received before the round's snapshot, so the submission
/// proves one note. Every shot is named after its image under
/// `docs/vote/images` in the docs site. See `docs_shots_support.dart` for
/// how the shots are recorded.
library;

import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:zkool/main.dart';
import 'package:zkool/pages/voting.dart';
import 'package:zkool/pages/voting_round.dart';
import 'package:zkool/router.dart';
import 'package:zkool/src/rust/api/account.dart';
import 'package:zkool/store.dart';

import 'docs_shots_support.dart';
import 'support.dart';

const _main = "Main Account";
// The choices made on the three proposals of tests/tools/docs_voting_proposals.json.
const _choices = ["Yes", "Blockchair", "Weekend morning (UTC)"];
const _submitted = "Votes submitted";
const _shares = "Votes cast, confirming helper shares";
// Long enough for a toast, such as the synchronizer's, to have gone.
const toastGone = Duration(seconds: 6);

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();
  // ignore: avoid_print
  debugPrint = (String? message, {int? wrapWidth}) => print(message ?? "");

  testWidgets("Docs: voting screenshots", (tester) async {
    final docsDir = await appDocumentsDir();
    final rendezvous = Rendezvous.fromEnvironment(docsDir);
    expect(rendezvous.dir.existsSync(), isTrue,
        reason: "rendezvous dir ${rendezvous.dir.path} missing — "
            "start this from test_docs_voting_ui.py",);

    final config = rendezvous.readConfig();
    final dbPath = config["db_path"] as String;
    final fund = config["fund"] as String;
    expect(dbPath, contains("regtest"));

    final prefs = SharedPreferencesAsync();
    final savedPrefs = await SavedPrefs.forceTestValues(prefs, defaultTheme: true);
    final shots = Shots(tester, docsDir, config);

    Future<void> waitForText(String text, {required String what, Duration timeout = shortTimeout}) =>
        pumpUntil(tester, () => find.text(text).evaluate().isNotEmpty, timeout: timeout, what: what);

    Future<void> openMenu() async {
      await tester.tap(find.byType(PopupMenuButton<String>).first);
      await pumpFor(tester, const Duration(seconds: 1));
    }

    // The ballot's Stepper builds every proposal's options and buttons,
    // collapsed for the proposals not shown: only what can be hit counts.
    Finder button(String label) => find.widgetWithText(FilledButton, label).hitTestable();
    Finder option(String label) => find.text(label).hitTestable();

    Future<void> tapButton(String label) async {
      await tester.ensureVisible(button(label));
      await tester.pump();
      await tester.tap(button(label));
      await pumpFor(tester, const Duration(seconds: 1));
    }

    try {
      await tester.runAsync(() async {
        await openTestWallet(
          dbPath: dbPath,
          lwd: config["lwd"] as String,
          docsDir: docsDir.path,
        );
      });
      final a = (await tester.runAsync(() => newAccount(
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
        () => getAddresses(uaPools: 7, account: a, c: coinContext.coin),
      ))!;
      rendezvous.publish("a_ua", addresses.ua!);

      // ── Funded, then the round opens on a snapshot past the funding ──────
      appRouter.go(accountRoute("/", a));
      await waitForAccountView(tester, _main);
      await waitForBalance(tester, a, atLeast: zecToZat(fund), what: "the funding to arrive");
      Map<String, dynamic>? voting;
      await pumpUntil(tester, () {
        voting = rendezvous.readOrchestrator();
        return voting?["config_url"] != null;
      }, timeout: const Duration(minutes: 10), what: "the orchestrator's voting round",);
      final configUrl = voting!["config_url"] as String;
      final roundTitle = voting!["round_title"] as String;
      await waitForSync(tester, a);

      // ── Settings: the voting config URL ──────────────────────────────────
      await pumpFor(tester, toastGone);
      unawaited(appRouter.push("/settings"));
      await pumpUntil(tester, () => fieldNamed("voting_config_url").evaluate().isNotEmpty, timeout: shortTimeout, what: "the settings page");
      await tester.ensureVisible(fieldNamed("voting_config_url").first);
      await tester.pump();
      await tester.enterText(fieldNamed("voting_config_url"), configUrl);
      await tester.pump();
      // The page ends with the database path on this machine: blanked.
      await shots.take("vote/images/01.settings", annots: () => [
        shots.box(fieldNamed("voting_config_url")),
        ...shots.blank(find.textContaining("/Documents/")),
      ]);
      await tester.pageBack();
      await waitForAccountView(tester, _main);
      // Typed on the page, and saved here so the wallet reads it whatever
      // the page's save timing.
      await tester.runAsync(() => container.read(appSettingsProvider.notifier).setVotingConfigUrl(configUrl));

      // ── The account menu, the rounds ─────────────────────────────────────
      await openMenu();
      await shots.take("vote/images/02.menu", annots: () => [shots.box(find.text("Voting"))]);
      await tester.tap(find.text("Voting").last);
      await pumpUntil(tester, () => find.byType(VotingPage).evaluate().isNotEmpty && find.text(roundTitle).evaluate().isNotEmpty,
          timeout: const Duration(minutes: 2), what: "the round in the voting list",);
      await shots.take("vote/images/03.rounds", annots: () => [shots.box(find.widgetWithText(FilledButton, "Start voting"))]);
      await tester.tap(find.widgetWithText(FilledButton, "Start voting"));
      await pumpUntil(tester, () => find.byType(VotingRoundPage).evaluate().isNotEmpty && option(_choices.first).evaluate().isNotEmpty,
          timeout: shortTimeout, what: "the ballot",);

      // ── The ballot, one proposal at a time ───────────────────────────────
      for (var i = 0; i < _choices.length; i++) {
        await tester.ensureVisible(option(_choices[i]));
        await tester.pump();
        await tester.tap(option(_choices[i]));
        await pumpFor(tester, const Duration(seconds: 1));
        if (i == 0) {
          await shots.take("vote/images/04.ballot", annots: () => [
            shots.box(option(_choices[i])),
            shots.box(button("Next")),
          ]);
        }
        if (i < _choices.length - 1) {
          await tapButton("Next");
          await pumpUntil(tester, () => option(_choices[i + 1]).evaluate().isNotEmpty, timeout: shortTimeout, what: "proposal ${i + 2}");
        }
      }
      await tapButton("Submit");

      // ── The preview, then the submission ─────────────────────────────────
      await pumpUntil(tester, () => find.text("Submit votes?").evaluate().isNotEmpty && find.textContaining("voting weight").evaluate().isNotEmpty,
          timeout: const Duration(minutes: 2), what: "the submission preview",);
      await shots.take("vote/images/05.submit");
      await tester.tap(button("Submit votes"));
      await pumpUntil(tester, () => find.byType(LinearProgressIndicator).evaluate().isNotEmpty, timeout: shortTimeout, what: "the submission to start");
      await shots.take("vote/images/06.progress", settle: const Duration(milliseconds: 200));
      // The votes are cast within a minute, then the same run confirms the
      // helper shares, which takes the helpers a few more seconds (#1401).
      // The page names the wait; a run so quick that the page never shows
      // it is reported, not failed.
      await pumpUntil(tester, () => find.text(_shares).evaluate().isNotEmpty || find.text(_submitted).evaluate().isNotEmpty,
          timeout: const Duration(minutes: 15), what: "the votes to be cast",);
      if (find.text(_shares).evaluate().isNotEmpty) {
        await shots.take("vote/images/07.shares");
        await waitForText(_submitted, what: "the helper shares to be confirmed", timeout: const Duration(minutes: 15));
      } else {
        debugPrint("[docs-shots] vote/images/07.shares SKIPPED: the page went straight to \"$_submitted\"");
      }
      await shots.take("vote/images/08.done", annots: () => [shots.box(button("Done"))]);

      // ── The vote report ──────────────────────────────────────────────────
      await tapTooltip(tester, "Vote report");
      await pumpUntil(tester, () => find.textContaining("proposals answered").evaluate().isNotEmpty, timeout: shortTimeout, what: "the vote report");
      await shots.take("vote/images/09.report");
      await tester.tap(find.widgetWithText(TextButton, "Close"));
      await pumpFor(tester, const Duration(seconds: 1));
      await tapButton("Done");
      await pumpUntil(tester, () => find.byType(VotingPage).evaluate().isNotEmpty && find.text(roundTitle).evaluate().isNotEmpty,
          timeout: shortTimeout, what: "the voting list again",);
      await shots.take("vote/images/10.voted", settle: const Duration(seconds: 2));

      shots.finish();
      rendezvous.publish("done", "true");
      await pumpFor(tester, const Duration(seconds: 1));
    } finally {
      await savedPrefs.restore(prefs);
    }
  }, timeout: const Timeout(Duration(minutes: 40)));
}
