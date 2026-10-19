/// Integration test: the everyday wallet flows, one app and no peers.
///
/// Creates accounts through the new-account page, receives a payment the
/// orchestrator sends, edits and hides an account, pays two recipients in one
/// transaction built from the send page's queue, checks the receiver, shows
/// fiat prices from a mock CoinGecko and switches the currency, and deletes an
/// account. `tests/tests/test_wallet_ui.py` owns the chain, the funding wallet
/// and the mock, and verifies the transaction on the node.
///
/// Three accounts: A (fresh, funded by pytest, pays), B (fresh, the receiver),
/// C (restored from a fixed mnemonic, edited, hidden and deleted).
library;

import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:go_router/go_router.dart';
import 'package:integration_test/integration_test.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:zkool/main.dart';
import 'package:zkool/pages/tx.dart';
import 'package:zkool/pages/tx_view.dart';
import 'package:zkool/router.dart';
import 'package:zkool/src/rust/api/account.dart';
import 'package:zkool/store.dart';
import 'package:zkool/utils.dart';

import 'support.dart';

const _accountA = "Alice";
const _accountB = "Bob";
const _accountC = "Carol";
const _accountCRenamed = "Carol-renamed";
const _folderName = "Tests";
// A valid BIP-39 phrase that no other test funds.
const _mnemonicC = "zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo wrong";
const _memo = "wallet-ui memo";

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();
  // debugPrint throttles and drops queued lines when the test dies; the log
  // must show the last steps before a failure.
  // ignore: avoid_print
  debugPrint = (String? message, {int? wrapWidth}) => print(message ?? "");

  testWidgets("Everyday wallet flows through the Flutter UI", (tester) async {
    final docsDir = await appDocumentsDir();
    final rendezvous = Rendezvous.fromEnvironment(docsDir);
    expect(rendezvous.dir.existsSync(), isTrue,
        reason: "rendezvous dir ${rendezvous.dir.path} missing — "
            "start this test from test_wallet_ui.py",);

    final config = rendezvous.readConfig();
    final dbPath = config["db_path"] as String;
    final coingeckoUrl = config["coingecko_url"] as String;
    final usdPrice = (config["usd_price"] as num).toDouble();
    final eurPrice = (config["eur_price"] as num).toDouble();
    final fundUa = config["fund_ua"] as String;
    final fundTaddr = config["fund_taddr"] as String;
    // "Show All" sits behind authenticate(), which prompts Touch ID on macOS;
    // on Linux (CI) the device has no biometrics and the gate opens by itself.
    final hiddenFlow = !(config["biometric_gate"] as bool? ?? false);
    expect(dbPath, contains("regtest"));

    final prefs = SharedPreferencesAsync();
    final savedPrefs = await SavedPrefs.forceTestValues(prefs, getFx: true);
    // The mock takes any key; an empty one keeps the real default untouched.
    final savedApiKey = await prefs.getString("coingecko");
    await prefs.setString("coingecko", "");

    void phase(String name) {
      debugPrint("[wallet-ui] phase: $name");
      rendezvous.publish("phase", name);
    }

    // The runner drops log lines printed just before a failure; a step written
    // to the rendezvous file survives, and pytest prints it.
    void step(String what) {
      debugPrint("[wallet-ui] step: $what");
      rendezvous.publish("step", what);
    }

    try {
      await tester.runAsync(() async {
        await openTestWallet(
          dbPath: dbPath,
          lwd: config["lwd"] as String,
          docsDir: docsDir.path,
          props: {"coingecko_url": coingeckoUrl},
        );
      });

      final appRouter = router(true, false, initialLocation: "/accounts");
      await tester.pumpWidget(ZkoolApp(router: appRouter));
      await tester.pump(const Duration(seconds: 1));
      final container = ProviderScope.containerOf(tester.element(find.byType(MaterialApp)));
      await primeProviders(tester, container);

      // ── Create ───────────────────────────────────────────────────────────
      phase("create");
      await _createAccount(tester, appRouter, name: _accountA);
      await _createAccount(tester, appRouter, name: _accountB);
      await _createAccount(tester, appRouter, name: _accountC, mnemonic: _mnemonicC);
      final ids = await tester.runAsync(() => accountIds());
      final a = ids![_accountA]!, b = ids[_accountB]!, c = ids[_accountC]!;
      debugPrint("[wallet-ui] accounts A=$a B=$b C=$c");
      for (final name in [_accountA, _accountB, _accountC]) {
        expect(find.text(name), findsOneWidget, reason: "$name in the account list");
      }

      // ── Receive ──────────────────────────────────────────────────────────
      phase("receive");
      final aAddresses = (await tester.runAsync(
        () => getAddresses(uaPools: 7, account: a, c: coinContext.coin),
      ))!;
      final bAddresses = (await tester.runAsync(
        () => getAddresses(uaPools: 7, account: b, c: coinContext.coin),
      ))!;
      appRouter.go(accountRoute("/receive", a));
      await pumpUntil(tester, () => find.text("Receive Funds").evaluate().isNotEmpty,
          timeout: shortTimeout, what: "the receive page",);
      await pumpUntil(tester, () => copyableWith(aAddresses.ua!).evaluate().isNotEmpty && copyableWith(aAddresses.taddr!).evaluate().isNotEmpty,
          timeout: shortTimeout, what: "A's addresses on the receive page",);
      rendezvous.publish("a_ua", aAddresses.ua!);
      rendezvous.publish("a_taddr", aAddresses.taddr!);

      appRouter.go(accountRoute("/", a));
      await waitForAccountView(tester, _accountA);
      await waitForBalance(tester, a, atLeast: zecToZat(fundUa) + zecToZat(fundTaddr),
          what: "A to receive the funding",);
      // pytest pays both addresses in one transaction: one entry in the
      // history, two notes (Ironwood from the unified address, transparent).
      await tapTab(tester, "Transactions");
      await pumpUntil(tester, () => find.text("Receive").evaluate().isNotEmpty,
          timeout: shortTimeout, what: "the incoming transaction in A's history",);
      final aNotes = (await tester.runAsync(() => listNotes(account: a, c: coinContext.coin)))!;
      expect(aNotes.map((n) => (n.pool, n.value)).toSet(), {(3, zecToZat(fundUa)), (0, zecToZat(fundTaddr))},
          reason: "one Ironwood note and one transparent note",);

      // ── Edit ─────────────────────────────────────────────────────────────
      // The folders page is pushed over the account list and left with pop:
      // the selected folder is an auto-dispose provider the list page keeps
      // alive, and `go` would rebuild that page and lose the selection.
      phase("edit");
      appRouter.go("/accounts");
      await waitForAccountList(tester);
      step("await openMenuItem(tester, 'Folders');");
      await openMenuItem(tester, "Folders");
      await pumpUntil(tester, () => find.text("Folders").evaluate().isNotEmpty,
          timeout: shortTimeout, what: "the folders page",);
      step('await tester.tap(find.byIcon(Icons.add));');
      await tester.tap(find.byIcon(Icons.add));
      await pumpUntil(tester, () => find.byType(TextField).evaluate().isNotEmpty && find.text("Ok").evaluate().isNotEmpty,
          timeout: shortTimeout, what: "the new folder dialog",);
      await tester.enterText(find.byType(TextField).last, _folderName);
      await tester.pump();
      step("await tester.tap(find.text('Ok'));");
      await tester.tap(find.text("Ok"));
      // The typed name also matches as text while the dialog is open: wait
      // for the dialog to close and the folder to show as a list tile before
      // leaving, or the pop would close the dialog with no result.
      await pumpUntil(tester, () => find.byType(TextField).evaluate().isEmpty && _folderTile().evaluate().isNotEmpty,
          timeout: shortTimeout, what: "the new folder in the list",);
      step('appRouter.pop();');
      appRouter.pop();
      await waitForAccountList(tester);
      step("select Carol and edit");
      await selectAndTap(tester, c, Icons.edit);
      await pumpUntil(tester, () => fieldNamed("name").evaluate().isNotEmpty,
          timeout: shortTimeout, what: "the account edit page",);
      await tester.enterText(fieldNamed("name"), _accountCRenamed);
      await tester.pump();
      await pickDropdown(tester, dropdownNamed("folder"), _folderName);
      if (hiddenFlow) {
        await tester.ensureVisible(checkboxNamed("hidden"));
        await tester.tap(checkboxNamed("hidden"));
        await pumpFor(tester, const Duration(seconds: 1));
      }
      appRouter.pop();
      await pumpUntil(tester, () => find.text("Save and leave").evaluate().isNotEmpty,
          timeout: shortTimeout, what: "the unsaved changes dialog",);
      step("await tester.tap(find.text('Save and leave'));");
      await tester.tap(find.text("Save and leave"));
      await waitForAccountList(tester);
      final carol = (await tester.runAsync(() => listAccounts(c: coinContext.coin)))!
          .firstWhere((x) => x.id == c);
      expect(carol.name, _accountCRenamed, reason: "Carol renamed");
      expect(carol.hidden, hiddenFlow);
      expect(carol.folder.name, _folderName);
      // Not in the root folder any more, and hidden inside hers until "Show All".
      // The list refreshes asynchronously after the save.
      await pumpUntil(tester, () => find.text(_accountCRenamed).evaluate().isEmpty && find.text(_accountC).evaluate().isEmpty,
          timeout: shortTimeout, what: "Carol to leave the root folder's list",);
      step("await openMenuItem(tester, 'Folders');");
      await openMenuItem(tester, "Folders");
      await pumpUntil(tester, () => _folderTile().evaluate().isNotEmpty,
          timeout: shortTimeout, what: "the folders page to pick Tests",);
      await tester.tap(_folderTile());
      await pumpFor(tester, const Duration(seconds: 1));
      appRouter.pop();
      await waitForAccountList(tester);
      await pumpUntil(tester, () => find.text(_accountA).evaluate().isEmpty,
          timeout: shortTimeout, what: "the list to show the Tests folder only",);
      if (hiddenFlow) {
        expect(find.text(_accountCRenamed), findsNothing, reason: "hidden accounts are not listed");
        step("Show All");
        await openMenuItem(tester, "Show All");
      }
      await pumpUntil(tester, () => find.text(_accountCRenamed).evaluate().isNotEmpty,
          timeout: shortTimeout, what: "Carol listed in her folder",);

      // ── Pay ──────────────────────────────────────────────────────────────
      phase("pay");
      appRouter.go(accountRoute("/", a));
      await waitForAccountView(tester, _accountA);
      await waitForSync(tester, a);
      step("appRouter.go(accountRoute('/send', a));");
      appRouter.go(accountRoute("/send", a));
      await pumpUntil(tester, () => fieldNamed("address").evaluate().isNotEmpty,
          timeout: shortTimeout, what: "the send page",);
      // The recipient with a memo goes first and the transparent one right
      // after it, and it is the transparent one that gets edited: queued after
      // a memo, or loaded for editing, a transparent recipient used to inherit
      // the memo and the app crashed on send (#1355).
      await enterRecipient(tester, bAddresses.oaddr!, "0.05", memo: _memo);
      await tapTooltip(tester, "Queue this recipient to create a multi send");
      // The price arrived at startup from the mock; the amount gets a fiat twin.
      await enterRecipient(tester, bAddresses.taddr!, "0.02");
      await pumpUntil(tester, () => fieldValue(tester, "fiat") == doubleToString(0.02 * usdPrice, decimals: 3),
          timeout: shortTimeout, what: "the fiat amount next to the ZEC amount",);
      expect(fieldNamed("memo").hitTestable(), findsNothing, reason: "no memo for a transparent recipient");
      await tapTooltip(tester, "Queue this recipient to create a multi send");
      await enterRecipient(tester, bAddresses.saddr!, "0.01");
      await tapTooltip(tester, "Queue this recipient to create a multi send");
      expect(find.byType(ListTile), findsNWidgets(3));
      // Edit the transparent recipient's amount, saved by tapping the Sapling
      // tile. The queue sits below the form, so each tile is scrolled into
      // view first.
      await _tapTile(tester, bAddresses.taddr!);
      await enterAmount(tester, "0.05");
      await _tapTile(tester, bAddresses.saddr!);
      step("queue tiles: ${tileTexts().join(" || ")}");
      // The amount is a Text.rich of two spans; match its plain text.
      final taddrTile = find.ancestor(of: find.text(bAddresses.taddr!), matching: find.byType(ListTile));
      expect(find.descendant(of: taddrTile, matching: find.text(zatToString(zecToZat("0.05")), findRichText: true)), findsOneWidget,
          reason: "the edited amount shows on the tile",);
      final editedTile = find.ancestor(of: find.text(bAddresses.saddr!), matching: find.byType(ListTile));
      expect(tester.widget<ListTile>(editedTile).selected, isTrue, reason: "the tapped tile is highlighted");
      // Drop the Sapling recipient.
      final saddrTile = find.ancestor(of: find.text(bAddresses.saddr!), matching: find.byType(ListTile));
      await tester.ensureVisible(saddrTile);
      await tester.pump();
      await tester.tap(find.descendant(of: saddrTile, matching: find.byIcon(Icons.delete)));
      await pumpFor(tester, const Duration(seconds: 1));
      expect(find.byType(ListTile), findsNWidgets(2));
      // Clear the form so only the queue is sent, then send.
      await tapTooltip(tester, "Clear Form Inputs");
      step("await tapTooltip(tester, 'Send transaction (including queued recipients)');");
      await tapTooltip(tester, "Send transaction (including queued recipients)");
      await pumpUntil(tester, () => find.text("Extra Options").evaluate().isNotEmpty,
          timeout: shortTimeout, what: "the send options page",);
      await tapTooltip(tester, "Send (Summary and Confirmation)");
      await pumpUntil(tester, () => find.byType(TxPage).evaluate().isNotEmpty && find.text("Output 2").evaluate().isNotEmpty,
          timeout: const Duration(minutes: 2), what: "the transaction page with two outputs",);
      expect(find.text("Sync Warning"), findsNothing, reason: "A was fully synced before sending");
      // The outputs include Alice's change, so count recipients, not outputs.
      expect(find.text("Address: ${bAddresses.oaddr!}"), findsOneWidget);
      expect(find.text("Address: ${bAddresses.taddr!}"), findsOneWidget);
      expect(find.text("Address: ${bAddresses.saddr!}"), findsNothing, reason: "the deleted recipient is not paid");
      expect(find.textContaining("Fee: "), findsOneWidget);
      step("await tapTooltip(tester, 'Confirm, broadcast transaction');");
      await tapTooltip(tester, "Confirm, broadcast transaction");
      await answerDialog(tester, "Are you sure you want to send this transaction?", "Ok");
      final txLine = await awaitStatus(
        tester,
        rendezvous,
        readStatus: () {
          final found = find.byWidgetPredicate((w) => w is CopyableText && w.text.startsWith("Transaction ID: ")).evaluate();
          return found.isEmpty ? "" : (found.first.widget as CopyableText).text;
        },
        isDone: (s) => s.startsWith("Transaction ID: "),
        timeout: const Duration(minutes: 3),
        what: "the transaction id",
      );
      final txid = txLine.substring("Transaction ID: ".length).trim();
      debugPrint("[wallet-ui] txid $txid");
      rendezvous.publish("txid", txid);
      expect(find.text("Transaction Sent"), findsOneWidget);

      // ── Settle ───────────────────────────────────────────────────────────
      phase("settle");
      await pumpUntil(tester, () => rendezvous.readOrchestrator()?["mined"] == true,
          timeout: chainTimeout, what: "the orchestrator to mine the payment",);
      appRouter.go(accountRoute("/", b));
      await waitForAccountView(tester, _accountB);
      await waitForBalance(tester, b, atLeast: zecToZat("0.10"), what: "B to receive the two outputs");
      final bNotes = (await tester.runAsync(() => listNotes(account: b, c: coinContext.coin)))!;
      expect(bNotes.where((n) => n.value == zecToZat("0.05")).length, 2, reason: "two incoming notes of 0.05");
      expect(bNotes.map((n) => n.pool).toSet(), {0, 3}, reason: "one transparent, one Ironwood");
      await tapTab(tester, "Transactions");
      await pumpUntil(tester, () => find.text("Receive").evaluate().isNotEmpty,
          timeout: shortTimeout, what: "the incoming transaction in B's history",);
      await tester.tap(find.text("Receive").first);
      await pumpUntil(tester, () => find.byType(TxViewPage).evaluate().isNotEmpty,
          timeout: shortTimeout, what: "the transaction view",);
      await pumpUntil(tester, () => find.text(_memo).evaluate().isNotEmpty,
          timeout: shortTimeout, what: "the memo in B's transaction view",);

      // ── Pricing ──────────────────────────────────────────────────────────
      phase("pricing");
      appRouter.go("/accounts");
      await waitForAccountList(tester);
      await pumpUntil(tester, () => find.text("1 ZEC = ${usdPrice.toStringAsFixed(2)} USD").evaluate().isNotEmpty,
          timeout: shortTimeout, what: "the exchange rate button with the mock price",);
      appRouter.go(accountRoute("/", a));
      await waitForAccountView(tester, _accountA);
      var aBalance = await tester.runAsync(() => totalBalance(a));
      await pumpUntil(tester, () => find.text(formatFiat(aBalance!.toDouble() / 1e8 * usdPrice, "usd")).evaluate().isNotEmpty,
          timeout: shortTimeout, what: "A's balance in USD",);
      await openMenuItem(tester, "Fetch Tx Prices");
      await answerDialog(tester, "Do you want to retrieve historical ZEC prices for your past transactions?", "Ok");
      await pumpUntil(tester, () => find.textContaining("transaction price").evaluate().isNotEmpty,
          timeout: shortTimeout, what: "the prices updated snackbar",);
      var aTxs = (await tester.runAsync(() => listTxHistory(account: a, c: coinContext.coin)))!;
      expect(aTxs, isNotEmpty);
      for (final t in aTxs) {
        expect(t.price, isNotNull, reason: "every transaction priced");
        expect(t.price!, closeTo(usdPrice, 1e-6), reason: "priced in USD");
      }
      // Switch to EUR: the conversion dialog pre-fills eur/usd from the mock.
      unawaited(appRouter.push("/settings"));
      await pumpUntil(tester, () => find.text("Currency").evaluate().isNotEmpty,
          timeout: shortTimeout, what: "the settings page",);
      await tester.ensureVisible(find.text("Currency"));
      await tester.tap(find.text("Currency"));
      await pumpUntil(tester, () => find.text("EUR").evaluate().isNotEmpty,
          timeout: shortTimeout, what: "the currency list from the mock",);
      await tester.tap(find.text("EUR"));
      await pumpFor(tester, const Duration(seconds: 1));
      // The page converts prices in its PopScope handler, which the back
      // button runs and a router pop skips.
      await tester.pageBack();
      await pumpUntil(tester, () => find.text("Convert Prices").evaluate().isNotEmpty,
          timeout: shortTimeout, what: "the convert prices dialog",);
      await tester.tap(find.text("Convert"));
      await pumpUntil(tester, () => find.text("Settings").evaluate().isNotEmpty && find.text("EUR").evaluate().isNotEmpty,
          timeout: shortTimeout, what: "settings showing EUR",);
      appRouter.pop(); // leaving settings saves them
      await waitForAccountView(tester, _accountA);
      await pumpUntil(tester, () => container.read(appSettingsProvider).value?.currency == "eur",
          timeout: shortTimeout, what: "the currency setting to be saved",);
      aBalance = await tester.runAsync(() => totalBalance(a));
      await pumpUntil(tester, () => find.text(formatFiat(aBalance!.toDouble() / 1e8 * eurPrice, "eur")).evaluate().isNotEmpty,
          timeout: shortTimeout, what: "A's balance in EUR",);
      aTxs = (await tester.runAsync(() => listTxHistory(account: a, c: coinContext.coin)))!;
      for (final t in aTxs) {
        expect(t.price!, closeTo(eurPrice, 1e-6), reason: "transaction prices converted to EUR");
      }

      // ── Delete ───────────────────────────────────────────────────────────
      // The list page was rebuilt since the edit phase: pick the Tests folder
      // and "Show All" again to reach the hidden account.
      phase("delete");
      appRouter.go("/accounts");
      await waitForAccountList(tester);
      step("await openMenuItem(tester, 'Folders');");
      await openMenuItem(tester, "Folders");
      await pumpUntil(tester, () => _folderTile().evaluate().isNotEmpty,
          timeout: shortTimeout, what: "the folders page before the delete",);
      await tester.tap(_folderTile());
      await pumpFor(tester, const Duration(seconds: 1));
      appRouter.pop();
      await waitForAccountList(tester);
      if (hiddenFlow && find.text(_accountCRenamed).evaluate().isEmpty) await openMenuItem(tester, "Show All");
      await pumpUntil(tester, () => find.text(_accountCRenamed).evaluate().isNotEmpty,
          timeout: shortTimeout, what: "Carol listed for deletion",);
      step("select Carol and delete");
      await selectAndTap(tester, c, Icons.delete);
      await answerDialog(tester, "Are you sure you want to delete these accounts?", "Ok");
      await pumpUntil(tester, () => find.text(_accountCRenamed).evaluate().isEmpty,
          timeout: shortTimeout, what: "Carol to leave the list",);
      final remaining = (await tester.runAsync(() => accountIds()))!;
      expect(remaining.keys.toSet(), {_accountA, _accountB});

      phase("done");
      rendezvous.publish("done", "true");
      await pumpFor(tester, const Duration(seconds: 3));
    } finally {
      await savedPrefs.restore(prefs);
      if (savedApiKey == null) {
        await prefs.remove("coingecko");
      } else {
        await prefs.setString("coingecko", savedApiKey);
      }
    }
  }, timeout: const Timeout(Duration(minutes: 12)));
}

/// Creates an account through the new-account page; a [mnemonic] restores one.
Future<void> _createAccount(WidgetTester tester, GoRouter appRouter, {required String name, String? mnemonic}) async {
  unawaited(appRouter.push("/account/new"));
  await pumpUntil(tester, () => find.text("Create a new wallet or restore an existing one").evaluate().isNotEmpty,
      timeout: shortTimeout, what: "the new account page",);
  await tester.enterText(fieldNamed("name"), name);
  await tester.pump();
  if (mnemonic != null) {
    await tester.ensureVisible(switchNamed("restore"));
    await tester.tap(switchNamed("restore"));
    await pumpFor(tester, const Duration(seconds: 1));
    await tester.enterText(fieldNamed("key"), mnemonic);
    await tester.pump();
  }
  await tapTooltip(tester, "Save");
  // The page asks and tells on the way to the account view: no birth height
  // given (restore), the seed phrase to back up (fresh), whether to scan for
  // rotated transparent addresses and whether to sync now (restore). Answer
  // each as it comes; the sync happens later, when the test needs it.
  // Keyed on body text: an AwesomeDialog title is not a plain Text widget.
  const dialogs = {
    "Are you sure you don't want to enter the birth height?": "Ok",
    "SEED PHRASE - SAVE IT OR YOU CAN LOSE YOUR FUNDS": "Ok",
    "Scan for alternative transparent addresses?": "Close",
    "Account imported successfully. Would you like to synchronize it now?": "Cancel",
  };
  final deadline = DateTime.now().add(const Duration(minutes: 2));
  while (DateTime.now().isBefore(deadline)) {
    if (find.text(name).evaluate().isNotEmpty && find.byTooltip("Sync this account").evaluate().isNotEmpty) {
      break;
    }
    for (final entry in dialogs.entries) {
      // The body shows a frame or two before the buttons finish sliding in.
      if (find.text(entry.key).evaluate().isNotEmpty && find.text(entry.value).evaluate().isNotEmpty) {
        debugPrint("[wallet-ui] dialog '${entry.key}' -> ${entry.value}");
        await tester.tap(find.text(entry.value).last);
        await pumpFor(tester, const Duration(seconds: 1));
      }
    }
    await tester.pump(const Duration(milliseconds: 500));
  }
  await waitForAccountView(tester, name);
  // The page may still be finishing (a late "synchronize now?" prompt, then
  // its own navigation to the account): keep answering and going back to
  // the list until the list stays up.
  final listDeadline = DateTime.now().add(shortTimeout);
  while (DateTime.now().isBefore(listDeadline)) {
    for (final entry in dialogs.entries) {
      if (find.text(entry.key).evaluate().isNotEmpty && find.text(entry.value).evaluate().isNotEmpty) {
        debugPrint("[wallet-ui] late dialog '${entry.key}' -> ${entry.value}");
        await tester.tap(find.text(entry.value).last);
        await pumpFor(tester, const Duration(seconds: 1));
      }
    }
    appRouter.go("/accounts");
    await pumpFor(tester, const Duration(seconds: 2));
    if (find.text("Account List").evaluate().isNotEmpty) break;
  }
  await waitForAccountList(tester);
}

Finder _folderTile() => find.widgetWithText(ListTile, _folderName);

/// Taps the queued-recipient tile titled [address], scrolled into view.
Future<void> _tapTile(WidgetTester tester, String address) async {
  final tile = find.ancestor(of: find.text(address), matching: find.byType(ListTile));
  await tester.ensureVisible(tile);
  await tester.pump();
  await tester.tap(tile);
  await pumpFor(tester, const Duration(seconds: 1));
}
