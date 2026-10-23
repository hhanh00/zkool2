/// Docs screenshots: the Categories / Reports pages of the docs site.
///
/// Not a test of the app but a scripted walkthrough that
/// `tests/tests/test_docs_report_ui.py` drives on regtest: the account page
/// menu with the price fetch, the transaction page with its price and
/// category, the send page with the exchange rate, the extra options with
/// the category of a payment and the account manager menu. The
/// market price comes from the orchestrator's CoinGecko mock, so the fetch
/// works offline and the prices are fixed. The charts are web views, which
/// the frame capture cannot rasterize: their images are left as they are.
/// Every shot is named after the image it replaces under
/// `docs/report/images` in the docs site. See `docs_shots_support.dart` for
/// how the shots are recorded.
library;


import 'package:flutter/material.dart';
import 'package:flutter_form_builder/flutter_form_builder.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:zkool/main.dart';
import 'package:zkool/pages/tx_view.dart';
import 'package:zkool/router.dart';
import 'package:zkool/src/rust/api/account.dart';
import 'package:zkool/store.dart';
import 'package:zkool/widgets/theme.dart';

import 'docs_shots_support.dart';
import 'support.dart';

const _main = "Main Account";
const _amount = "0.05";
const _memo = "Weekly groceries";
const _income = "Salary";
const _expense = "Food & Groceries";
const _sendTooltip = "Send funds to one or many addresses";
const _fetchPrompt = "Do you want to retrieve historical ZEC prices for your past transactions?";
// Long enough for a toast, such as the synchronizer's, to have gone.
const toastGone = Duration(seconds: 6);

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
    final funded = config["funded"] as int;
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

    Future<void> openMenu() async {
      await tester.tap(find.byType(PopupMenuButton<String>).first);
      await pumpFor(tester, const Duration(seconds: 1));
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
      await waitForSync(tester, a);

      // ── Account page menu: the price fetch ───────────────────────────────
      await pumpFor(tester, toastGone);
      await openMenu();
      await shots.take("report/images/05.fetch", annots: () => [shots.box(find.text("Fetch Tx Prices"))]);
      await tester.tap(find.text("Fetch Tx Prices").last);
      await pumpFor(tester, const Duration(seconds: 1));
      await answerDialog(tester, _fetchPrompt, "Ok");
      final updated = "Updated $funded transaction price${funded == 1 ? '' : 's'}";
      await waitForText(updated, what: "the price fetch to report its count");
      await pumpUntil(tester, () => find.text(updated).evaluate().isEmpty, timeout: const Duration(seconds: 15), what: "the toast to go");

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

      // ── Send page: the exchange rate ─────────────────────────────────────
      await tapTooltip(tester, _sendTooltip);
      await pumpUntil(tester, () => fieldNamed("address").evaluate().isNotEmpty, timeout: shortTimeout, what: "the send page");
      await enterRecipient(tester, destination, _amount, memo: _memo);
      await tester.ensureVisible(fieldNamed("fx").first);
      await tester.pump();
      await shots.take("report/images/09.fx", annots: () => [
        shots.boxAround([fieldNamed("fx"), find.byIcon(Icons.refresh)]),
      ]);
      await tapTooltip(tester, "Send transaction (including queued recipients)");
      await waitForText("Extra Options", what: "the send options page");
      await tester.tap(categoryField);
      await pumpFor(tester, const Duration(seconds: 1));
      await tester.tap(find.text(_expense).last);
      await pumpFor(tester, const Duration(seconds: 1));
      await shots.take("report/images/04.send", annots: () => [shots.box(categoryField)]);
      appRouter.go(accountRoute("/", a));
      await waitForAccountView(tester, _main);

      // ── Account manager menu, the category list ──────────────────────────
      appRouter.go("/accounts");
      await waitForAccountList(tester);
      await openMenu();
      await shots.take("report/images/06.categories", annots: () => [shots.box(find.text("Categories"))]);
      // The Categories page itself, its new category form and the actions on
      // a selected category are not shot: its list separator is an unbounded
      // SizedBox.expand(), which the debug build refuses to lay out (#1392),
      // so the page shows its first row only. The docs keep their previous
      // images 07 and 08; add the shots back once that is fixed.
      debugPrint("[docs-shots] report/images/07.categories, 08.edit_category SKIPPED: #1392");

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
  }, timeout: const Timeout(Duration(minutes: 20)));
}
