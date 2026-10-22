/// Docs screenshots: the Recipe pages of the docs site.
///
/// Not a test of the app but a scripted walkthrough that
/// `tests/tests/test_docs_recipe_ui.py` drives on regtest: the account
/// list with a selection and its menu, the mempool with a transaction the
/// orchestrator leaves unmined, the account page menu and viewing keys,
/// the account edit page, the receive page's scan button, the notes tab,
/// the payment URI, the new account page with the Ledger options, the
/// settings page, the database manager, and the cold wallet round trip:
/// a watch-only account saves an unsigned transaction, the account with the
/// keys signs it offline, and the watch-only account loads the signed one.
/// Every shot is named after the image it replaces under
/// `docs/recipe/images` in the docs site. See `docs_shots_support.dart` for
/// how the shots are recorded.
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
import 'package:zkool/store.dart';
import 'package:zkool/utils.dart';

import 'docs_shots_support.dart';
import 'support.dart';

const _main = "Main Account";
const _watch = "Watch Only";
const _amount = "0.05";
const _memo = "Signed offline";
const _newAccountHint = "Create a new wallet or restore an existing one";
const _sendTooltip = "Send funds to one or many addresses";
const _scanTooltip =
    "Find other transparent addresses. If you restored from a wallet that has address rotation (such as Ledger, Exodus, etc), Tap, then Reset and Sync";
// Long enough for a toast, such as the synchronizer's, to have gone.
const toastGone = Duration(seconds: 6);

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();
  // ignore: avoid_print
  debugPrint = (String? message, {int? wrapWidth}) => print(message ?? "");

  testWidgets("Docs: recipe screenshots", (tester) async {
    final docsDir = await appDocumentsDir();
    final rendezvous = Rendezvous.fromEnvironment(docsDir);
    expect(rendezvous.dir.existsSync(), isTrue,
        reason: "rendezvous dir ${rendezvous.dir.path} missing — "
            "start this from test_docs_recipe_ui.py",);

    final config = rendezvous.readConfig();
    final dbPath = config["db_path"] as String;
    final destination = config["destination"] as String;
    final fund = config["fund"] as String;
    expect(dbPath, contains("regtest"));

    final prefs = SharedPreferencesAsync();
    final savedPrefs = await SavedPrefs.forceTestValues(prefs, defaultTheme: true);
    final shots = Shots(tester, docsDir, config);
    // The cold wallet round trip saves and loads transaction files.
    final files = FakeFileDialogs(shots.dir);
    fileDialogs = files;

    Future<void> waitForText(String text, {required String what}) =>
        pumpUntil(tester, () => find.text(text).evaluate().isNotEmpty, timeout: shortTimeout, what: what);

    // The edit page's birth height is an int field, not a text field.
    final birthField = find.byWidgetPredicate((w) => w is FormBuilderField<int> && w.name == "birth");

    // A tooltip whose wording follows a toggle's state.
    Finder eitherTooltip(String a, String b) => find.byWidgetPredicate((w) => w is Tooltip && (w.message == a || w.message == b));

    // A badge just right of the widget [finder] matches, outside it.
    Annot besides(Finder finder, int n) => shots.badgeAt(shots.rectOf(finder).centerRight + const Offset(26, 0), n);

    // A badge on the lower right corner of the icon [finder] matches, for a
    // row of icons with content right under it.
    Annot under(Finder finder, int n) => shots.badgeAt(shots.rectOf(finder).bottomRight + const Offset(-6, -6), n);

    Future<void> openMenu() async {
      await tester.tap(find.byType(PopupMenuButton<String>).first);
      await pumpFor(tester, const Duration(seconds: 1));
    }

    Future<void> closeMenu() async {
      await tester.tapAt(const Offset(20, 700));
      await pumpFor(tester, const Duration(seconds: 1));
    }

    Future<void> setOffline(ProviderContainer container, bool offline) async {
      final settings = container.read(appSettingsProvider).requireValue;
      await container.read(appSettingsProvider.notifier).save(settings.copyWith(offline: offline));
      await pumpFor(tester, const Duration(seconds: 1));
    }

    Future<void> sendTo(String address) async {
      await tapTooltip(tester, _sendTooltip);
      await pumpUntil(tester, () => fieldNamed("address").evaluate().isNotEmpty, timeout: shortTimeout, what: "the send page");
      await enterRecipient(tester, address, _amount, memo: _memo);
      await tapTooltip(tester, "Send transaction (including queued recipients)");
      await waitForText("Extra Options", what: "the send options page");
    }

    Future<void> waitForTxPage(IconData action, {required String what}) => pumpUntil(
          tester,
          () => find.byType(TxPage).evaluate().isNotEmpty && find.byIcon(action).evaluate().isNotEmpty,
          timeout: const Duration(minutes: 2),
          what: what,
        );

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
      // The splash normally starts the mempool listener. Started from the
      // tester's real async zone, where the Rust stream delivers; the
      // wrapper returns at once so the test is not held.
      await tester.runAsync(() async {
        unawaited(Future(container.read(mempoolProvider.notifier).runMempoolListener));
      });
      final addresses = (await tester.runAsync(
        () => getAddresses(uaPools: 7, account: a, c: coinContext.coin),
      ))!;
      rendezvous.publish("a_ua", addresses.ua!);

      // ── Mempool: the orchestrator pays while nothing mines ───────────────
      unawaited(appRouter.push("/mempool"));
      await waitForText("Mempool", what: "the mempool page");
      rendezvous.publish("mempool_ready", "true");
      // The app's mempool stream does not reliably report the payment on
      // regtest (the headless server does, with the same Rust): the shot is
      // skipped, and the image kept as it is, when nothing shows in time.
      try {
        await pumpUntil(
          tester,
          () => find.descendant(of: find.byType(MempoolPage), matching: find.byType(ListTile)).evaluate().isNotEmpty,
          timeout: const Duration(seconds: 60),
          what: "an unconfirmed transaction",
        );
        await shots.take("recipe/images/23.mempool");
      } on TestFailure {
        debugPrint("[docs-shots] recipe/images/23.mempool SKIPPED: no unconfirmed transaction shown");
      }
      rendezvous.publish("mempool_done", "true");
      await tester.pageBack();
      await waitForAccountList(tester);

      // ── Funding ──────────────────────────────────────────────────────────
      appRouter.go(accountRoute("/", a));
      await waitForAccountView(tester, _main);
      await waitForBalance(tester, a, atLeast: zecToZat(fund), what: "the funding to arrive");
      await waitForSync(tester, a);

      // ── Account list: sync button, selection, menu ───────────────────────
      appRouter.go("/accounts");
      await waitForAccountList(tester);
      await shots.take("recipe/images/12.sync", settle: toastGone, annots: () => [
        shots.box(find.byTooltip("Synchronize all enabled accounts or the accounts currently selected")),
      ]);
      final avatar = find.descendant(of: find.byKey(ValueKey(a)), matching: find.byType(CircleAvatar)).first;
      await tester.tap(avatar);
      await pumpUntil(tester, () => find.byIcon(Icons.edit).evaluate().isNotEmpty, timeout: shortTimeout, what: "the selection actions");
      await shots.take("recipe/images/27.edit", annots: () => [
        shots.boxAround([find.byIcon(Icons.edit), find.byIcon(Icons.delete)]),
      ]);
      await tester.tap(avatar);
      await pumpFor(tester, const Duration(seconds: 1));
      await openMenu();
      await shots.take("recipe/images/24.mempool1", annots: () => [shots.box(find.text("Mempool"))]);
      await shots.take("recipe/images/26.folders", annots: () => [shots.box(find.text("Folders"))]);
      await closeMenu();

      // ── Account page menu, viewing keys ──────────────────────────────────
      appRouter.go(accountRoute("/", a));
      await waitForAccountView(tester, _main);
      await waitForSync(tester, a);
      await openMenu();
      await shots.take("recipe/images/29.csv", annots: () => [shots.box(find.text("Export Transactions"))]);
      await closeMenu();
      await openMenuItem(tester, "Backup");
      await waitForText("Viewing Keys", what: "the viewing keys page");
      await shots.take("recipe/images/36.view_keys");
      await shots.take("recipe/images/06.ufvk");
      final ufvk = (await tester.runAsync(() => getAccountUfvk(account: a, pools: 7, c: coinContext.coin)))!;
      await tester.pageBack();
      await waitForAccountView(tester, _main);

      // ── Account edit page ────────────────────────────────────────────────
      await openMenuItem(tester, "Edit Account");
      await waitForText("Account Edit", what: "the account edit page");
      // The form fills in once the account has loaded.
      await pumpUntil(tester, () => dropdownNamed("folder").evaluate().isNotEmpty, timeout: shortTimeout, what: "the account edit form");
      await shots.take("recipe/images/35.edit", annots: () => [
        shots.badge(fieldNamed("name"), 1, align: const Alignment(0.6, 0)),
        shots.badge(find.byType(CircleAvatar), 2, align: const Alignment(-1.8, 0)),
        shots.badge(birthField, 3),
        shots.badgeAfterText(find.text("Enabled"), 4),
        shots.badgeAfterText(find.text("Hidden"), 5),
        shots.badgeAfterText(find.text("Use Internal Change"), 6),
        shots.badge(dropdownNamed("folder"), 7),
      ]);
      await shots.take("recipe/images/13.edit", annots: () => [shots.box(birthField)]);
      await shots.take("recipe/images/16.rewind", annots: () => [shots.box(find.byTooltip("Rewind back a few blocks"))]);
      await shots.take("recipe/images/20.export", annots: () => [shots.box(find.byTooltip("Export an encrypted file of this account"))]);
      await tester.pageBack();
      await waitForAccountView(tester, _main);

      // ── Receive page: the transparent scan ───────────────────────────────
      await tapTooltip(tester, "Show the account receiving addresses");
      await waitForText("Receive Funds", what: "the receive page");
      await shots.take("recipe/images/14.scan", annots: () => [shots.box(find.byTooltip(_scanTooltip))]);
      await tester.pageBack();
      await waitForAccountView(tester, _main);

      // ── Notes tab ────────────────────────────────────────────────────────
      await tapTab(tester, "Notes");
      await pumpUntil(tester, () => find.byTooltip("Lock recently mined notes").evaluate().isNotEmpty, timeout: shortTimeout, what: "the notes toolbar");
      await shots.take("recipe/images/30.coin", annots: () => [
        under(find.byTooltip("Lock recently mined notes"), 1),
        under(find.byTooltip("Toggle all notes"), 2),
        under(find.byTooltip("Unlock all notes"), 3),
        under(eitherTooltip("Show dust notes", "Hide dust notes"), 4),
        under(eitherTooltip("Group by pool", "Ungroup notes"), 5),
      ]);
      await tester.tap(find.byType(ListTile).first);
      await pumpFor(tester, const Duration(seconds: 1));
      await shots.take("recipe/images/15.notes", annots: () => [shots.box(find.byType(ListTile).first)]);
      await tester.tap(find.byType(ListTile).first);
      await pumpFor(tester, const Duration(seconds: 1));
      await tapTab(tester, "Transactions");

      // ── Payment URI ──────────────────────────────────────────────────────
      await sendTo(destination);
      await shots.take("recipe/images/25.puri", annots: () => [
        shots.box(find.ancestor(of: find.text("Payment URI"), matching: find.byType(InputDecorator)).first),
      ]);
      appRouter.go(accountRoute("/", a));
      await waitForAccountView(tester, _main);

      // ── New account page: import from file, Ledger ───────────────────────
      unawaited(appRouter.push("/account/new"));
      await waitForText(_newAccountHint, what: "the new account page");
      await shots.take("recipe/images/21.load", annots: () => [shots.box(find.byTooltip("Import an account from file"))]);
      await tester.enterText(fieldNamed("name"), "Ledger");
      await tester.pump();
      await tester.tap(switchNamed("restore"));
      await pumpFor(tester, const Duration(milliseconds: 500));
      await tester.tap(switchNamed("advanced"));
      await pumpUntil(tester, () => switchNamed("ledger").evaluate().isNotEmpty, timeout: shortTimeout, what: "the Ledger switch");
      await tester.ensureVisible(switchNamed("ledger").first);
      await tester.tap(switchNamed("ledger"));
      await waitForText("Official (Ironwood)", what: "the Ledger app selector");
      await tester.ensureVisible(find.text("Official (Ironwood)"));
      await tester.pump();
      await shots.take("recipe/images/32.import", annots: () => [
        besides(find.text("H/W Ledger"), 2),
        besides(find.text("Zondax (Sapling)"), 3),
        shots.badge(fieldNamed("aindex"), 4),
      ]);
      appRouter.go(accountRoute("/", a));
      await waitForAccountView(tester, _main);

      // ── Settings ─────────────────────────────────────────────────────────
      unawaited(appRouter.push("/settings"));
      await pumpUntil(tester, () => fieldNamed("autosync").evaluate().isNotEmpty, timeout: shortTimeout, what: "the settings page");
      // The page ends with the database path on this machine: blanked.
      final dataPath = find.textContaining("/Documents/");
      List<Annot> noPath() => shots.blank(dataPath);
      await shots.take("recipe/images/17.database", annots: () => [shots.box(find.byTooltip("Database Manager")), ...noPath()]);
      await tester.ensureVisible(find.text("Proxy"));
      await tester.pump();
      await shots.take("recipe/images/19.tor", annots: () => [shots.boxAround([find.text("Direct"), find.text("Proxy")]), ...noPath()]);
      await tester.ensureVisible(fieldNamed("autosync").first);
      await tester.pump();
      await shots.take("recipe/images/11.autosync", annots: () => [shots.box(fieldNamed("autosync")), ...noPath()]);
      await tester.ensureVisible(find.text("Pin Lock"));
      await tester.pump();
      await shots.take("recipe/images/22.pinlock", annots: () => [
        shots.box(find.ancestor(of: find.text("Pin Lock"), matching: find.byType(Row)).first),
        ...noPath(),
      ]);
      await tester.pageBack();
      await waitForAccountView(tester, _main);

      // ── Database manager ─────────────────────────────────────────────────
      unawaited(appRouter.push("/database_manager"));
      await waitForText("Database Manager", what: "the database manager");
      // The manager lists every database of this machine's app: the run's
      // own row is checked and the others blanked.
      final dbName = dbPath.split("/").last.replaceAll(".db", "");
      // The rows are built lazily, keyed by database name.
      final ownRow = find.byKey(ValueKey(dbName));
      await tester.scrollUntilVisible(ownRow, 200, scrollable: find.byType(Scrollable).first);
      await tester.pump();
      await tester.tap(find.descendant(of: ownRow, matching: find.byType(Checkbox)).first);
      await pumpUntil(tester, () => find.byTooltip("Load Database").evaluate().isNotEmpty, timeout: shortTimeout, what: "the database actions");
      await shots.take("recipe/images/18.manager", annots: () => [
        ...shots.blank(
          find.byWidgetPredicate((w) => w is ListTile && w.title is Text && (w.title as Text).data != dbName),
          within: find.byType(ListView),
        ),
        // Nothing sits under this app bar once the rows are blanked.
        shots.badge(find.byIcon(Icons.add), 1, align: const Alignment(0, 2.2)),
        shots.badge(find.byTooltip("Load Database"), 2, align: const Alignment(0, 2.2)),
        shots.badge(find.byTooltip("Save Database"), 3, align: const Alignment(0, 2.2)),
        shots.badge(find.byIcon(Icons.edit), 4, align: const Alignment(0, 2.2)),
        shots.badge(find.byIcon(Icons.password), 5, align: const Alignment(0, 2.2)),
        shots.badge(find.byIcon(Icons.delete), 6, align: const Alignment(0, 2.2)),
      ]);
      await tester.pageBack();
      await waitForAccountView(tester, _main);

      // ── Cold wallet: unsigned on the watch-only account ──────────────────
      final w = (await tester.runAsync(() => newAccount(
            na: NewAccount(
              name: _watch,
              restore: true,
              key: ufvk,
              aindex: 0,
              birth: 1,
              folder: "",
              useInternal: true,
              internal: false,
              hw: 0,
            ),
            c: coinContext.coin,
          )))!;
      container.invalidate(getAccountsProvider);
      appRouter.go(accountRoute("/", w));
      await waitForAccountView(tester, _watch);
      await waitForBalance(tester, w, atLeast: zecToZat(fund), what: "the watch-only account to see the funds");
      await waitForSync(tester, w);
      await sendTo(destination);
      await tapTooltip(tester, "Send (Summary and Confirmation)");
      await waitForTxPage(Icons.save, what: "the transaction plan with its save button");
      await shots.take("recipe/images/07.save", annots: () => [shots.box(find.byIcon(Icons.save))]);
      await tapTooltip(tester, "Save the unsigned transaction");
      await pumpUntil(tester, () => files.saved.isNotEmpty, timeout: shortTimeout, what: "the unsigned transaction file");

      // ── Signed offline on the account with the keys ──────────────────────
      await setOffline(container, true);
      appRouter.go(accountRoute("/", a));
      await waitForAccountView(tester, _main);
      await tapTooltip(tester, _sendTooltip);
      await pumpUntil(tester, () => fieldNamed("address").evaluate().isNotEmpty, timeout: shortTimeout, what: "the send page");
      await shots.take("recipe/images/08.load", annots: () => [shots.box(find.byTooltip("Load an unsigned transaction"))]);
      files.picks.add(files.saved.last);
      await tapTooltip(tester, "Load an unsigned transaction");
      await waitForTxPage(Icons.draw, what: "the loaded transaction with its sign button");
      await shots.take("recipe/images/09.sign", annots: () => [shots.box(find.byIcon(Icons.draw))]);
      await tapTooltip(tester, "Sign and save the transaction");
      await pumpUntil(tester, () => files.saved.length >= 2, timeout: const Duration(minutes: 3), what: "the signed transaction file");
      await setOffline(container, false);

      // ── Broadcast from the watch-only account ────────────────────────────
      appRouter.go(accountRoute("/", w));
      await waitForAccountView(tester, _watch);
      await tapTooltip(tester, _sendTooltip);
      await pumpUntil(tester, () => fieldNamed("address").evaluate().isNotEmpty, timeout: shortTimeout, what: "the send page");
      files.picks.add(files.saved.last);
      await tapTooltip(tester, "Load an unsigned transaction");
      await waitForTxPage(Icons.send, what: "the signed transaction with its send button");
      await shots.take("recipe/images/10.broadcast", annots: () => [shots.box(find.byIcon(Icons.send))]);
      await tapTooltip(tester, "Cancel, do NOT send");

      shots.finish();
      rendezvous.publish("done", "true");
      await pumpFor(tester, const Duration(seconds: 1));
    } finally {
      fileDialogs = const SystemFileDialogs();
      await savedPrefs.restore(prefs);
    }
  }, timeout: const Timeout(Duration(minutes: 20)));
}
