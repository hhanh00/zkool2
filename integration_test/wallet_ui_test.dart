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
import 'package:flutter_form_builder/flutter_form_builder.dart';
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
import 'package:zkool/src/rust/api/network.dart';
import 'package:zkool/src/rust/api/sync.dart';
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

const _shortTimeout = Duration(seconds: 30);
const _chainTimeout = Duration(minutes: 4);

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
      final ids = await tester.runAsync(() => _accountIds());
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
          timeout: _shortTimeout, what: "the receive page",);
      await pumpUntil(tester, () => _copyable(aAddresses.ua!).evaluate().isNotEmpty && _copyable(aAddresses.taddr!).evaluate().isNotEmpty,
          timeout: _shortTimeout, what: "A's addresses on the receive page",);
      rendezvous.publish("a_ua", aAddresses.ua!);
      rendezvous.publish("a_taddr", aAddresses.taddr!);

      appRouter.go(accountRoute("/", a));
      await _waitForAccountView(tester, _accountA);
      await _waitForBalance(tester, a, atLeast: _zat(fundUa) + _zat(fundTaddr),
          what: "A to receive the funding",);
      // pytest pays both addresses in one transaction: one entry in the
      // history, two notes (Ironwood from the unified address, transparent).
      await _tapTab(tester, "Transactions");
      await pumpUntil(tester, () => find.text("Receive").evaluate().isNotEmpty,
          timeout: _shortTimeout, what: "the incoming transaction in A's history",);
      final aNotes = (await tester.runAsync(() => listNotes(account: a, c: coinContext.coin)))!;
      expect(aNotes.map((n) => (n.pool, n.value)).toSet(), {(3, _zat(fundUa)), (0, _zat(fundTaddr))},
          reason: "one Ironwood note and one transparent note",);

      // ── Edit ─────────────────────────────────────────────────────────────
      // The folders page is pushed over the account list and left with pop:
      // the selected folder is an auto-dispose provider the list page keeps
      // alive, and `go` would rebuild that page and lose the selection.
      phase("edit");
      appRouter.go("/accounts");
      await _waitForAccountList(tester);
      step("await _openMenuItem(tester, 'Folders');");
      await _openMenuItem(tester, "Folders");
      await pumpUntil(tester, () => find.text("Folders").evaluate().isNotEmpty,
          timeout: _shortTimeout, what: "the folders page",);
      step('await tester.tap(find.byIcon(Icons.add));');
      await tester.tap(find.byIcon(Icons.add));
      await pumpUntil(tester, () => find.byType(TextField).evaluate().isNotEmpty && find.text("Ok").evaluate().isNotEmpty,
          timeout: _shortTimeout, what: "the new folder dialog",);
      await tester.enterText(find.byType(TextField).last, _folderName);
      await tester.pump();
      step("await tester.tap(find.text('Ok'));");
      await tester.tap(find.text("Ok"));
      // The typed name also matches as text while the dialog is open: wait
      // for the dialog to close and the folder to show as a list tile before
      // leaving, or the pop would close the dialog with no result.
      await pumpUntil(tester, () => find.byType(TextField).evaluate().isEmpty && _folderTile().evaluate().isNotEmpty,
          timeout: _shortTimeout, what: "the new folder in the list",);
      step('appRouter.pop();');
      appRouter.pop();
      await _waitForAccountList(tester);
      step("select Carol and edit");
      await _selectAndTap(tester, c, Icons.edit);
      await pumpUntil(tester, () => fieldNamed("name").evaluate().isNotEmpty,
          timeout: _shortTimeout, what: "the account edit page",);
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
          timeout: _shortTimeout, what: "the unsaved changes dialog",);
      step("await tester.tap(find.text('Save and leave'));");
      await tester.tap(find.text("Save and leave"));
      await _waitForAccountList(tester);
      final carol = (await tester.runAsync(() => listAccounts(c: coinContext.coin)))!
          .firstWhere((x) => x.id == c);
      expect(carol.name, _accountCRenamed, reason: "Carol renamed");
      expect(carol.hidden, hiddenFlow);
      expect(carol.folder.name, _folderName);
      // Not in the root folder any more, and hidden inside hers until "Show All".
      // The list refreshes asynchronously after the save.
      await pumpUntil(tester, () => find.text(_accountCRenamed).evaluate().isEmpty && find.text(_accountC).evaluate().isEmpty,
          timeout: _shortTimeout, what: "Carol to leave the root folder's list",);
      step("await _openMenuItem(tester, 'Folders');");
      await _openMenuItem(tester, "Folders");
      await pumpUntil(tester, () => _folderTile().evaluate().isNotEmpty,
          timeout: _shortTimeout, what: "the folders page to pick Tests",);
      await tester.tap(_folderTile());
      await pumpFor(tester, const Duration(seconds: 1));
      appRouter.pop();
      await _waitForAccountList(tester);
      await pumpUntil(tester, () => find.text(_accountA).evaluate().isEmpty,
          timeout: _shortTimeout, what: "the list to show the Tests folder only",);
      if (hiddenFlow) {
        expect(find.text(_accountCRenamed), findsNothing, reason: "hidden accounts are not listed");
        step("Show All");
        await _openMenuItem(tester, "Show All");
      }
      await pumpUntil(tester, () => find.text(_accountCRenamed).evaluate().isNotEmpty,
          timeout: _shortTimeout, what: "Carol listed in her folder",);

      // ── Pay ──────────────────────────────────────────────────────────────
      phase("pay");
      appRouter.go(accountRoute("/", a));
      await _waitForAccountView(tester, _accountA);
      await _waitForSync(tester, a);
      step("appRouter.go(accountRoute('/send', a));");
      appRouter.go(accountRoute("/send", a));
      await pumpUntil(tester, () => fieldNamed("address").evaluate().isNotEmpty,
          timeout: _shortTimeout, what: "the send page",);
      // The price arrived at startup from the mock; the amount gets a fiat twin.
      // The transparent recipient goes first: queued after a memo it would
      // inherit that memo and the app crashes on send (#1355).
      await _enterRecipient(tester, bAddresses.taddr!, "0.05");
      await pumpUntil(tester, () => _fieldValue(tester, "fiat") == doubleToString(0.05 * usdPrice, decimals: 3),
          timeout: _shortTimeout, what: "the fiat amount next to the ZEC amount",);
      expect(fieldNamed("memo").hitTestable(), findsNothing, reason: "no memo for a transparent recipient");
      await tapTooltip(tester, "Queue this recipient to create a multi send");
      await _enterRecipient(tester, bAddresses.oaddr!, "0.02", memo: _memo);
      await tapTooltip(tester, "Queue this recipient to create a multi send");
      await _enterRecipient(tester, bAddresses.saddr!, "0.01");
      await tapTooltip(tester, "Queue this recipient to create a multi send");
      expect(find.byType(ListTile), findsNWidgets(3));
      // Edit the Orchard recipient's amount, saved by tapping the Sapling
      // tile. A transparent recipient must never be loaded for editing: on
      // send it is saved back with a memo and the app crashes (#1355). Switch
      // to editing the transparent one once that is fixed. The queue sits
      // below the form, so each tile is scrolled into view first.
      await _tapTile(tester, bAddresses.oaddr!);
      await _enterAmount(tester, "0.05");
      await _tapTile(tester, bAddresses.saddr!);
      step("queue tiles: ${_tileTexts().join(" || ")}");
      // The amount is a Text.rich of two spans; match its plain text.
      final oaddrTile = find.ancestor(of: find.text(bAddresses.oaddr!), matching: find.byType(ListTile));
      expect(find.descendant(of: oaddrTile, matching: find.text(zatToString(_zat("0.05")), findRichText: true)), findsOneWidget,
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
          timeout: _shortTimeout, what: "the send options page",);
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
      await _answerDialog(tester, "Are you sure you want to send this transaction?", "Ok");
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
          timeout: _chainTimeout, what: "the orchestrator to mine the payment",);
      appRouter.go(accountRoute("/", b));
      await _waitForAccountView(tester, _accountB);
      await _waitForBalance(tester, b, atLeast: _zat("0.10"), what: "B to receive the two outputs");
      final bNotes = (await tester.runAsync(() => listNotes(account: b, c: coinContext.coin)))!;
      expect(bNotes.where((n) => n.value == _zat("0.05")).length, 2, reason: "two incoming notes of 0.05");
      expect(bNotes.map((n) => n.pool).toSet(), {0, 3}, reason: "one transparent, one Ironwood");
      await _tapTab(tester, "Transactions");
      await pumpUntil(tester, () => find.text("Receive").evaluate().isNotEmpty,
          timeout: _shortTimeout, what: "the incoming transaction in B's history",);
      await tester.tap(find.text("Receive").first);
      await pumpUntil(tester, () => find.byType(TxViewPage).evaluate().isNotEmpty,
          timeout: _shortTimeout, what: "the transaction view",);
      await pumpUntil(tester, () => find.text(_memo).evaluate().isNotEmpty,
          timeout: _shortTimeout, what: "the memo in B's transaction view",);

      // ── Pricing ──────────────────────────────────────────────────────────
      phase("pricing");
      appRouter.go("/accounts");
      await _waitForAccountList(tester);
      await pumpUntil(tester, () => find.text("1 ZEC = ${usdPrice.toStringAsFixed(2)} USD").evaluate().isNotEmpty,
          timeout: _shortTimeout, what: "the exchange rate button with the mock price",);
      appRouter.go(accountRoute("/", a));
      await _waitForAccountView(tester, _accountA);
      var aBalance = await tester.runAsync(() => _total(a));
      await pumpUntil(tester, () => find.text(formatFiat(aBalance!.toDouble() / 1e8 * usdPrice, "usd")).evaluate().isNotEmpty,
          timeout: _shortTimeout, what: "A's balance in USD",);
      await _openMenuItem(tester, "Fetch Tx Prices");
      await _answerDialog(tester, "Do you want to retrieve historical ZEC prices for your past transactions?", "Ok");
      await pumpUntil(tester, () => find.textContaining("transaction price").evaluate().isNotEmpty,
          timeout: _shortTimeout, what: "the prices updated snackbar",);
      var aTxs = (await tester.runAsync(() => listTxHistory(account: a, c: coinContext.coin)))!;
      expect(aTxs, isNotEmpty);
      for (final t in aTxs) {
        expect(t.price, isNotNull, reason: "every transaction priced");
        expect(t.price!, closeTo(usdPrice, 1e-6), reason: "priced in USD");
      }
      // Switch to EUR: the conversion dialog pre-fills eur/usd from the mock.
      unawaited(appRouter.push("/settings"));
      await pumpUntil(tester, () => find.text("Currency").evaluate().isNotEmpty,
          timeout: _shortTimeout, what: "the settings page",);
      await tester.ensureVisible(find.text("Currency"));
      await tester.tap(find.text("Currency"));
      await pumpUntil(tester, () => find.text("EUR").evaluate().isNotEmpty,
          timeout: _shortTimeout, what: "the currency list from the mock",);
      await tester.tap(find.text("EUR"));
      await pumpFor(tester, const Duration(seconds: 1));
      // The page converts prices in its PopScope handler, which the back
      // button runs and a router pop skips.
      await tester.pageBack();
      await pumpUntil(tester, () => find.text("Convert Prices").evaluate().isNotEmpty,
          timeout: _shortTimeout, what: "the convert prices dialog",);
      await tester.tap(find.text("Convert"));
      await pumpUntil(tester, () => find.text("Settings").evaluate().isNotEmpty && find.text("EUR").evaluate().isNotEmpty,
          timeout: _shortTimeout, what: "settings showing EUR",);
      appRouter.pop(); // leaving settings saves them
      await _waitForAccountView(tester, _accountA);
      await pumpUntil(tester, () => container.read(appSettingsProvider).value?.currency == "eur",
          timeout: _shortTimeout, what: "the currency setting to be saved",);
      aBalance = await tester.runAsync(() => _total(a));
      await pumpUntil(tester, () => find.text(formatFiat(aBalance!.toDouble() / 1e8 * eurPrice, "eur")).evaluate().isNotEmpty,
          timeout: _shortTimeout, what: "A's balance in EUR",);
      aTxs = (await tester.runAsync(() => listTxHistory(account: a, c: coinContext.coin)))!;
      for (final t in aTxs) {
        expect(t.price!, closeTo(eurPrice, 1e-6), reason: "transaction prices converted to EUR");
      }

      // ── Delete ───────────────────────────────────────────────────────────
      // The list page was rebuilt since the edit phase: pick the Tests folder
      // and "Show All" again to reach the hidden account.
      phase("delete");
      appRouter.go("/accounts");
      await _waitForAccountList(tester);
      step("await _openMenuItem(tester, 'Folders');");
      await _openMenuItem(tester, "Folders");
      await pumpUntil(tester, () => _folderTile().evaluate().isNotEmpty,
          timeout: _shortTimeout, what: "the folders page before the delete",);
      await tester.tap(_folderTile());
      await pumpFor(tester, const Duration(seconds: 1));
      appRouter.pop();
      await _waitForAccountList(tester);
      if (hiddenFlow && find.text(_accountCRenamed).evaluate().isEmpty) await _openMenuItem(tester, "Show All");
      await pumpUntil(tester, () => find.text(_accountCRenamed).evaluate().isNotEmpty,
          timeout: _shortTimeout, what: "Carol listed for deletion",);
      step("select Carol and delete");
      await _selectAndTap(tester, c, Icons.delete);
      await _answerDialog(tester, "Are you sure you want to delete these accounts?", "Ok");
      await pumpUntil(tester, () => find.text(_accountCRenamed).evaluate().isEmpty,
          timeout: _shortTimeout, what: "Carol to leave the list",);
      final remaining = (await tester.runAsync(() => _accountIds()))!;
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

BigInt _zat(String zec) => BigInt.from((double.parse(zec) * 1e8).round());

Future<Map<String, int>> _accountIds() async {
  final accounts = await listAccounts(c: coinContext.coin);
  return {for (final a in accounts) a.name: a.id};
}

Future<BigInt> _total(int account) async {
  final b = await balance(account: account, c: coinContext.coin);
  return b.field0.fold<BigInt>(BigInt.zero, (sum, v) => sum + v);
}

String? _fieldValue(WidgetTester tester, String name) {
  final found = fieldNamed(name).evaluate();
  if (found.isEmpty) return null;
  final state = (found.first as StatefulElement).state as FormBuilderFieldState;
  return state.value as String?;
}

/// Creates an account through the new-account page; a [mnemonic] restores one.
Future<void> _createAccount(WidgetTester tester, GoRouter appRouter, {required String name, String? mnemonic}) async {
  unawaited(appRouter.push("/account/new"));
  await pumpUntil(tester, () => find.text("Create a new wallet or restore an existing one").evaluate().isNotEmpty,
      timeout: _shortTimeout, what: "the new account page",);
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
  await _waitForAccountView(tester, name);
  appRouter.go("/accounts");
  await _waitForAccountList(tester);
}

Future<void> _waitForAccountList(WidgetTester tester) => pumpUntil(
      tester,
      () => find.text("Account List").evaluate().isNotEmpty,
      timeout: _shortTimeout,
      what: "the account list",
    );

Future<void> _waitForAccountView(WidgetTester tester, String name) => pumpUntil(
      tester,
      () => find.text(name).evaluate().isNotEmpty && find.byTooltip("Sync this account").evaluate().isNotEmpty,
      timeout: _shortTimeout,
      what: "the account view of $name",
    );

/// Taps the sync button and waits until the account's total reaches [atLeast].
/// Logs every poll: a stall here is the first thing to read in the log.
Future<void> _waitForBalance(WidgetTester tester, int account, {required BigInt atLeast, required String what}) async {
  final deadline = DateTime.now().add(_chainTimeout);
  var polls = 0;
  while (DateTime.now().isBefore(deadline)) {
    debugPrint("[wallet-ui] ${DateTime.now().toIso8601String()} poll ${++polls}: reading balance of $account");
    final total = await tester.runAsync(() => _total(account).timeout(const Duration(seconds: 60)));
    debugPrint("[wallet-ui] balance of $account: $total (want $atLeast)");
    if (total! >= atLeast) return;
    if (polls % 3 == 1) await tapTooltip(tester, "Sync this account");
    await pumpFor(tester, const Duration(seconds: 5));
  }
  debugPrint("[ui-test] visible texts: ${visibleTexts().join(" | ")}");
  fail("timed out waiting for $what");
}

/// Syncs until the account is at the chain tip, so the send page needs no
/// sync warning.
Future<void> _waitForSync(WidgetTester tester, int account) async {
  final deadline = DateTime.now().add(_chainTimeout);
  var polls = 0;
  while (DateTime.now().isBefore(deadline)) {
    final tip = await tester.runAsync(() => getCurrentHeight(c: coinContext.coin).timeout(const Duration(seconds: 60)));
    final accounts = await tester.runAsync(() => listAccounts(c: coinContext.coin).timeout(const Duration(seconds: 60)));
    final height = accounts!.firstWhere((x) => x.id == account).height;
    debugPrint("[wallet-ui] account $account at $height, tip $tip");
    if (height >= tip!) return;
    if (polls++ % 3 == 0) await tapTooltip(tester, "Sync this account");
    await pumpFor(tester, const Duration(seconds: 5));
  }
  fail("timed out waiting for account $account to reach the chain tip");
}

Future<void> _tapTab(WidgetTester tester, String name) async {
  await tester.tap(find.widgetWithText(Tab, name));
  await pumpFor(tester, const Duration(seconds: 1));
}

Future<void> _openMenuItem(WidgetTester tester, String item) async {
  await tester.tap(find.byType(PopupMenuButton<String>).first);
  await pumpFor(tester, const Duration(seconds: 1));
  await tester.tap(find.text(item).last);
  await pumpFor(tester, const Duration(seconds: 1));
}

/// Selects an account in the list by tapping its avatar (rows are keyed by
/// id) and taps the [action] icon the selection reveals. The list clears its
/// selection whenever the page rebuilds with a new account list, which a
/// height or price update does, so the two taps are retried as a unit.
Future<void> _selectAndTap(WidgetTester tester, int id, IconData action) async {
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
    debugPrint("[wallet-ui] selection of $id lost before the action, attempt $attempt");
    await pumpFor(tester, const Duration(seconds: 1));
  }
  fail("could not select account $id and tap its action");
}

Finder _copyable(String text) => find.byWidgetPredicate((w) => w is CopyableText && w.text == text);

Finder _folderTile() => find.widgetWithText(ListTile, _folderName);

/// Every ListTile on screen as its plain texts (Text and RichText), for the log.
List<String> _tileTexts() => [
      for (final tile in find.byType(ListTile).evaluate())
        [
          for (final e in find.descendant(of: find.byWidget(tile.widget), matching: find.byType(RichText)).evaluate())
            (e.widget as RichText).text.toPlainText(),
        ].join(" / "),
    ];

/// Waits for an AwesomeDialog showing [message] (its title is not a Text
/// widget) and taps the [button] once it has slid in.
Future<void> _answerDialog(WidgetTester tester, String message, String button) async {
  await pumpUntil(
    tester,
    () => find.text(message).evaluate().isNotEmpty && find.text(button).evaluate().isNotEmpty,
    timeout: _shortTimeout,
    what: "the dialog '$message'",
  );
  await tester.tap(find.text(button).last);
  await pumpFor(tester, const Duration(seconds: 1));
}

/// Taps the queued-recipient tile titled [address], scrolled into view.
Future<void> _tapTile(WidgetTester tester, String address) async {
  final tile = find.ancestor(of: find.text(address), matching: find.byType(ListTile));
  await tester.ensureVisible(tile);
  await tester.pump();
  await tester.tap(tile);
  await pumpFor(tester, const Duration(seconds: 1));
}

Future<void> _enterAmount(WidgetTester tester, String amount) async {
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
  await pumpUntil(tester, () => _fieldValue(tester, "zat") == amount,
      timeout: const Duration(seconds: 5), what: "the ZEC amount to read $amount",);
}

Future<void> _enterRecipient(WidgetTester tester, String address, String amount, {String? memo}) async {
  await tester.enterText(fieldNamed("address"), address);
  await tester.pump();
  await pumpFor(tester, const Duration(seconds: 1)); // address validation and memo visibility
  await _enterAmount(tester, amount);
  if (memo != null) {
    await tester.enterText(fieldNamed("memo"), memo);
    await tester.pump();
  }
}
