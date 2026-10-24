/// Docs screenshots: the Categories / Reports pages of the docs site, and
/// the market chart of the Guide.
///
/// Not a test of the app but a scripted walkthrough that
/// `tests/tests/test_docs_report_ui.py` drives on regtest: the account page
/// menu with the price fetch, the transaction page with its price and
/// category, the send page with the exchange rate, the extra options with
/// the category of a payment, the budget charts once the account has
/// categorized income and spending, the market chart, and the account
/// manager menu. The market price and the candles come from the
/// orchestrator's CoinGecko and klines mocks, so the pages work offline
/// with fixed data. Every shot is named after the image it replaces under
/// `docs/report/images` (or `docs/guide/images`) in the docs site. See
/// `docs_shots_support.dart` for how the shots are recorded.
library;

import 'package:flutter/material.dart';
import 'package:flutter_form_builder/flutter_form_builder.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:zkool/chart.dart';
import 'package:zkool/main.dart';
import 'package:zkool/pages/category.dart';
import 'package:zkool/pages/tx.dart';
import 'package:zkool/pages/tx_view.dart';
import 'package:zkool/router.dart';
import 'package:zkool/src/rust/api/account.dart';
import 'package:zkool/src/rust/api/transaction.dart';
import 'package:zkool/store.dart';
import 'package:zkool/utils.dart';
import 'package:zkool/widgets/market_chart.dart';
import 'package:zkool/widgets/theme.dart';

import 'docs_shots_support.dart';
import 'support.dart';

const _main = "Main Account";
const _income = "Salary";
const _newCategory = "Hardware";
const _sendTooltip = "Send funds to one or many addresses";
const _fetchPrompt = "Do you want to retrieve historical ZEC prices for your past transactions?";
const _sendPrompt = "Are you sure you want to send this transaction?";
// Long enough for a toast, such as the synchronizer's, to have gone.
const toastGone = Duration(seconds: 6);

/// A payment the app makes, with the category it files it under.
class _Spend {
  final String amount;
  final String memo;
  final String category;
  const _Spend(this.amount, this.memo, this.category);
}

const _spends = [
  _Spend("0.05", "Weekly groceries", "Food & Groceries"),
  _Spend("0.02", "Lunch with the team", "Restaurants & Coffee"),
];

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();
  // ignore: avoid_print
  debugPrint = (String? message, {int? wrapWidth}) => print(message ?? "");

  testWidgets("Docs: report screenshots", (tester) async {
    final docsDir = await appDocumentsDir();
    final rendezvous = Rendezvous.fromEnvironment(docsDir);
    expect(rendezvous.dir.existsSync(), isTrue,
        reason: "rendezvous dir ${rendezvous.dir.path} missing — "
            "start this from test_docs_report_ui.py",);

    final config = rendezvous.readConfig();
    final dbPath = config["db_path"] as String;
    final destination = config["destination"] as String;
    final fund = config["fund"] as String;
    final coingeckoUrl = config["coingecko_url"] as String;
    final klinesUrl = config["klines_url"] as String;
    // What the orchestrator pays the account: [amount, memo, category].
    final funding = (config["funding"] as List).cast<List>();
    expect(dbPath, contains("regtest"));

    final prefs = SharedPreferencesAsync();
    // The market price is fetched at startup, from the mock.
    final savedPrefs = await SavedPrefs.forceTestValues(prefs, getFx: true, defaultTheme: true);
    // The mock takes any key; an empty one keeps the real default untouched.
    final savedApiKey = await prefs.getString("coingecko");
    await prefs.setString("coingecko", "");
    final shots = Shots(tester, docsDir, config);

    Future<void> waitForText(String text, {required String what}) =>
        pumpUntil(tester, () => find.text(text).evaluate().isNotEmpty, timeout: shortTimeout, what: what);

    // The row of the transaction page titled [title].
    Finder row(String title) => find.ancestor(of: find.text(title), matching: find.byType(ListTile)).first;

    // The category of a payment, on the extra options page: its items
    // include "Unknown", so the dropdown's type is not the int one.
    final categoryField = find.byWidgetPredicate((w) => w is FormBuilderDropdown && w.name == "category");

    // A badge just right of the widget [finder] matches, outside it.
    Annot besides(Finder finder, int n) => shots.badgeAt(shots.rectOf(finder).centerRight + const Offset(26, 0), n);

    // A badge right under the icon [finder] matches, for the app bar's
    // icons, whose boxes reach the window's edge.
    Annot under(Finder finder, int n) => shots.badgeAt(shots.rectOf(finder).bottomCenter + const Offset(0, 10), n);

    Future<void> openMenu() async {
      await tester.tap(find.byType(PopupMenuButton<String>).first);
      await pumpFor(tester, const Duration(seconds: 1));
    }

    /// Picks [category] on the extra options page.
    Future<void> pickCategory(String category) async {
      await tester.tap(categoryField);
      await pumpFor(tester, const Duration(seconds: 1));
      await tester.tap(find.text(category).last);
      await pumpFor(tester, const Duration(seconds: 1));
    }

    /// Waits for [count] transactions in the account's history, syncing
    /// along the way: a payment the app made must be mined and seen before
    /// the next one, and before the charts.
    Future<void> waitForHistory(int account, int count) async {
      final deadline = DateTime.now().add(chainTimeout);
      var polls = 0;
      while (DateTime.now().isBefore(deadline)) {
        final history = (await tester.runAsync(() => listTxHistory(account: account, c: coinContext.coin)))!;
        if (history.length >= count) return;
        if (polls++ % 3 == 0) await tapTooltip(tester, "Sync this account");
        await pumpFor(tester, const Duration(seconds: 5));
      }
      fail("timed out waiting for $count transactions in the history");
    }

    try {
      await tester.runAsync(() async {
        await openTestWallet(
          dbPath: dbPath,
          lwd: config["lwd"] as String,
          docsDir: docsDir.path,
          props: {"coingecko_url": coingeckoUrl, "klines_url": klinesUrl},
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

      // ── Funding ──────────────────────────────────────────────────────────
      appRouter.go(accountRoute("/", a));
      await waitForAccountView(tester, _main);
      await waitForBalance(tester, a, atLeast: zecToZat(fund), what: "the funding to arrive");
      await waitForHistory(a, funding.length);
      await waitForSync(tester, a);

      // ── Account page menu: the price fetch ───────────────────────────────
      await pumpFor(tester, toastGone);
      await openMenu();
      await shots.take("report/images/05.fetch", annots: () => [shots.box(find.text("Fetch Tx Prices"))]);
      await tester.tap(find.text("Fetch Tx Prices").last);
      await pumpFor(tester, const Duration(seconds: 1));
      await answerDialog(tester, _fetchPrompt, "Ok");
      final updated = "Updated ${funding.length} transaction price${funding.length == 1 ? '' : 's'}";
      await waitForText(updated, what: "the price fetch to report its count");
      await pumpUntil(tester, () => find.text(updated).evaluate().isEmpty, timeout: const Duration(seconds: 15), what: "the toast to go");

      // ── The received payments, filed under their income categories ──────
      // Set here rather than through the page, one by one: the page shows
      // the pick once, on the newest payment, below.
      await tester.runAsync(() async {
        final categories = await listCategories(c: coinContext.coin);
        final history = await listTxHistory(account: a, c: coinContext.coin);
        for (final tx in history) {
          final entry = funding.firstWhere((f) => zecToZat(f[0] as String) == BigInt.from(tx.value));
          final category = categories.firstWhere((c) => c.name == entry[2] as String);
          await setTxCategory(id: tx.id, category: category.id, c: coinContext.coin);
        }
      });
      container.invalidate(accountProvider(a));
      await pumpFor(tester, const Duration(seconds: 1));

      // ── Transaction page: price and category ─────────────────────────────
      await tester.tap(find.byType(TransactionTile).first);
      await pumpUntil(tester, () => find.byType(TxViewPage).evaluate().isNotEmpty && find.text("Category").evaluate().isNotEmpty,
          timeout: shortTimeout, what: "the transaction page",);
      await shots.take("report/images/11.price", annots: () => [shots.box(row("Price"))]);
      // The dropdown's box spans its hidden menu: tap its field. It also
      // keeps an offstage copy of its entries, to size the field, laid out
      // where the field is: the menu's own entries are the ones that do not
      // overlap the field, and they exist only while the menu is open.
      final dropdown = find.byWidgetPredicate((w) => w is DropdownMenu);
      final categoryMenu = find.descendant(of: dropdown, matching: find.byType(TextField));
      await tester.ensureVisible(categoryMenu);
      await tester.pump();
      final fieldRect = tester.getRect(categoryMenu);
      Rect? rectOfElement(Element e) {
        final box = e.renderObject;
        if (box is! RenderBox || !box.attached || !box.hasSize) return null;
        return box.localToGlobal(Offset.zero) & box.size;
      }
      bool labelled(Element e, String label) =>
          find.descendant(of: find.byElementPredicate((x) => identical(x, e)), matching: find.text(label)).evaluate().isNotEmpty;
      final incomeItem = find.byElementPredicate((e) {
        if (e.widget is! MenuItemButton || !labelled(e, _income)) return false;
        final rect = rectOfElement(e);
        return rect != null && !rect.overlaps(fieldRect);
      });
      await tester.tap(categoryMenu);
      try {
        await pumpUntil(tester, () => incomeItem.evaluate().isNotEmpty, timeout: const Duration(seconds: 10), what: "the category menu");
      } on TestFailure {
        debugPrint("[docs-shots] the field tap did not open the category menu: tapping its arrow");
        await tester.tap(find.descendant(of: dropdown, matching: find.byType(IconButton)).first);
        await pumpUntil(tester, () => incomeItem.evaluate().isNotEmpty, timeout: const Duration(seconds: 10), what: "the category menu");
      }
      await tester.ensureVisible(incomeItem);
      await tester.pump();
      await tester.tap(incomeItem);
      await pumpUntil(tester, () => incomeItem.evaluate().isEmpty, timeout: shortTimeout, what: "the category menu to close");
      await pumpFor(tester, const Duration(seconds: 1));
      // The pick left the row's title under the app bar: show the whole row.
      await tester.ensureVisible(row("Category"));
      await tester.pump();
      await shots.take("report/images/03.category", annots: () => [
        shots.boxAround([find.byIcon(Icons.chevron_left), find.byIcon(Icons.chevron_right)]),
        shots.boxAround([find.text("Category"), categoryMenu]),
      ]);
      await tester.pageBack();
      await waitForAccountView(tester, _main);

      // ── Two payments, each under a spending category ─────────────────────
      // The first one is shot on its way: the exchange rate on the send page
      // and the category on the extra options page.
      for (var i = 0; i < _spends.length; i++) {
        final spend = _spends[i];
        await waitForSync(tester, a);
        await pumpFor(tester, const Duration(seconds: 1));
        await tapTooltip(tester, _sendTooltip);
        await pumpUntil(tester, () => fieldNamed("address").evaluate().isNotEmpty, timeout: shortTimeout, what: "the send page");
        await enterRecipient(tester, destination, spend.amount, memo: spend.memo);
        if (i == 0) {
          await tester.ensureVisible(fieldNamed("fx").first);
          await tester.pump();
          await shots.take("report/images/09.fx", annots: () => [
            shots.boxAround([fieldNamed("fx"), find.byIcon(Icons.refresh)]),
          ]);
        }
        await tapTooltip(tester, "Send transaction (including queued recipients)");
        await waitForText("Extra Options", what: "the send options page");
        await pickCategory(spend.category);
        if (i == 0) await shots.take("report/images/04.send", annots: () => [shots.box(categoryField)]);
        await tapTooltip(tester, "Send (Summary and Confirmation)");
        await pumpUntil(tester, () => find.byType(TxPage).evaluate().isNotEmpty && find.byTooltip("Confirm, broadcast transaction").evaluate().isNotEmpty,
            timeout: const Duration(minutes: 2), what: "the transaction summary",);
        await tapTooltip(tester, "Confirm, broadcast transaction");
        await answerDialog(tester, _sendPrompt, "Ok");
        await pumpUntil(
          tester,
          () => find.byWidgetPredicate((w) => w is CopyableText && w.text.startsWith("Transaction ID: ")).evaluate().isNotEmpty,
          timeout: const Duration(minutes: 5),
          what: "the transaction id of payment ${i + 1}",
        );
        appRouter.go(accountRoute("/", a));
        await waitForAccountView(tester, _main);
        // Mined by the orchestrator on demand, then seen by a sync.
        await waitForHistory(a, funding.length + i + 1);
      }
      await waitForSync(tester, a);

      // ── The budget charts ────────────────────────────────────────────────
      await pumpFor(tester, toastGone);
      await openMenuItem(tester, "Charts");
      await pumpUntil(tester, () => find.byType(ChartPage).evaluate().isNotEmpty && find.byType(DonutLegendEntry).evaluate().isNotEmpty,
          timeout: shortTimeout, what: "the spending donut",);
      final kind = find.byType(DropdownButton<bool>);
      await shots.take("report/images/12.spending", annots: () => [shots.box(kind)]);
      await tester.tap(kind);
      await pumpFor(tester, const Duration(seconds: 1));
      await tester.tap(find.text("Income").last);
      await pumpUntil(tester, () => find.widgetWithText(DonutLegendEntry, _income).evaluate().isNotEmpty,
          timeout: shortTimeout, what: "the income donut",);
      await shots.take("report/images/01.report", annots: () => [shots.box(kind)]);
      await tester.tap(find.widgetWithText(Tab, "Category"));
      await pumpFor(tester, const Duration(seconds: 1));
      // The tab loads its categories once shown; the dropdown stays shut
      // while it has no items.
      final categoryPick = find.byType(DropdownButton<int>);
      await pumpUntil(
        tester,
        () => categoryPick.evaluate().isNotEmpty && (tester.widget<DropdownButton<int>>(categoryPick).items?.isNotEmpty ?? false),
        timeout: shortTimeout,
        what: "the category chart's dropdown with its categories",
      );
      await tester.tap(categoryPick);
      await pumpFor(tester, const Duration(seconds: 1));
      // The menu builds its entries lazily and the income categories come
      // last: scroll it until the entry exists.
      await tester.scrollUntilVisible(find.text(_income), 200, scrollable: find.byType(Scrollable).last);
      await tester.pump();
      await tester.tap(find.text(_income).last);
      await pumpUntil(tester, () => find.byType(CategoryTimeChart).evaluate().isNotEmpty, timeout: shortTimeout, what: "the category chart");
      // The menu's closing route still takes the taps for a moment.
      await pumpFor(tester, const Duration(seconds: 1));
      for (var attempt = 1; !(tester.widget<Checkbox>(find.byType(Checkbox)).value ?? false); attempt++) {
        if (attempt > 3) fail("the cumulative box did not check");
        await tester.tap(find.byType(Checkbox));
        await pumpFor(tester, const Duration(seconds: 1));
      }
      await shots.take("report/images/13.category", annots: () => [
        shots.boxAround([categoryPick, find.text("Cumulative"), find.byType(Checkbox)]),
      ]);
      await tester.pageBack();
      await waitForAccountView(tester, _main);

      // ── The market chart (a Guide image) ─────────────────────────────────
      await openMenuItem(tester, "Market Price");
      await tester.pump();
      await pumpUntil(
        tester,
        () => find.byType(MarketChart).evaluate().isNotEmpty && find.byType(CircularProgressIndicator).evaluate().isEmpty,
        timeout: shortTimeout,
        what: "the market chart",
      );
      await pumpFor(tester, const Duration(seconds: 1));
      await shots.take("guide/images/15.other", annots: () => [
        besides(find.byType(SegmentedButton<String>), 1),
        under(find.byTooltip("Chart style"), 2),
        under(find.byTooltip("Volume"), 3),
        under(find.byTooltip("Logarithmic scale"), 4),
      ]);
      await tester.pageBack();
      await waitForAccountView(tester, _main);

      // ── Account manager menu ─────────────────────────────────────────────
      appRouter.go("/accounts");
      await waitForAccountList(tester);
      await openMenu();
      await shots.take("report/images/06.categories", annots: () => [shots.box(find.text("Categories"))]);
      await tester.tap(find.text("Categories").last);
      await pumpUntil(tester, () => find.byType(CategoryPage).evaluate().isNotEmpty && find.text(_income).evaluate().isNotEmpty,
          timeout: shortTimeout, what: "the category list",);
      await shots.take("report/images/07.categories", annots: () => [shots.box(find.byIcon(Icons.add))]);

      // ── New category ─────────────────────────────────────────────────────
      await tester.tap(find.byIcon(Icons.add));
      await pumpUntil(tester, () => find.text("New Category").evaluate().isNotEmpty && fieldNamed("name").evaluate().isNotEmpty,
          timeout: shortTimeout, what: "the new category form",);
      await tester.enterText(fieldNamed("name"), _newCategory);
      await tester.pump();
      await shots.take("report/images/10.category_form", annots: () => [
        shots.badge(fieldNamed("name"), 1),
        shots.badgeAfterText(find.text("Income?"), 2),
      ]);
      await answerDialog(tester, "New Category", "Ok");
      await waitForText(_newCategory, what: "the new category in the list");

      // ── A selected category: edit and delete ─────────────────────────────
      final newRow = find.ancestor(of: find.text(_newCategory), matching: find.byType(ListTile)).first;
      await tester.ensureVisible(newRow);
      await tester.pump();
      await tester.tap(find.descendant(of: newRow, matching: find.byType(Checkbox)));
      await pumpUntil(tester, () => find.byIcon(Icons.edit).evaluate().isNotEmpty, timeout: shortTimeout, what: "the selection actions");
      await shots.take("report/images/08.edit_category", annots: () => [
        shots.boxAround([find.byIcon(Icons.edit), find.byIcon(Icons.delete)]),
      ]);

      shots.finish();
      rendezvous.publish("done", "true");
      await pumpFor(tester, const Duration(seconds: 1));
    } finally {
      await savedPrefs.restore(prefs);
      if (savedApiKey == null) {
        await prefs.remove("coingecko");
      } else {
        await prefs.setString("coingecko", savedApiKey);
      }
    }
  }, timeout: const Timeout(Duration(minutes: 40)));
}
