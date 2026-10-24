/// The native market page on the real app: opens it on a fresh wallet
/// whose `klines_url` prop points at the orchestrator's klines mock, walks
/// the styles, intervals, volume and log toggles, the crosshair and the
/// go-to-latest button, and waits for the poll of the forming bar. Each
/// screen is rasterized to `<documents>/<frames_dir>`, for the docs.
/// Driven by `tests/tests/test_market_ui.py`; no chain is needed.
library;

import 'package:flutter/gestures.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:zkool/main.dart';
import 'package:zkool/router.dart';
import 'package:zkool/widgets/market_chart.dart';

import 'docs_shots_support.dart';
import 'support.dart';

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();
  // ignore: avoid_print
  debugPrint = (String? message, {int? wrapWidth}) => print(message ?? "");

  testWidgets("Market page", (tester) async {
    final docsDir = await appDocumentsDir();
    final rendezvous = Rendezvous.fromEnvironment(docsDir);
    expect(rendezvous.dir.existsSync(), isTrue,
        reason: "rendezvous dir ${rendezvous.dir.path} missing — start this from test_market_ui.py");
    final config = rendezvous.readConfig();
    final dbPath = config["db_path"] as String;
    final lwd = config["lwd"] as String;
    final klinesUrl = config["klines_url"] as String;

    final prefs = SharedPreferencesAsync();
    final savedPrefs = await SavedPrefs.forceTestValues(prefs, defaultTheme: true);
    final shots = Shots(tester, docsDir, config);

    try {
      await tester.runAsync(() => openTestWallet(
            dbPath: dbPath,
            lwd: lwd,
            docsDir: docsDir.path,
            props: {"klines_url": klinesUrl},
          ));

      final appRouter = router(true, false, initialLocation: "/market");
      await tester.pumpWidget(RepaintBoundary(key: shots.boundary, child: ZkoolApp(router: appRouter)));
      await tester.pump(const Duration(seconds: 1));

      // A pump first: a tap's setState has not rebuilt the tree yet, so the
      // spinner it shows is not in the tree the first check would see.
      Future<void> waitForChart(String what) async {
        await tester.pump();
        await pumpUntil(
          tester,
          () => find.byType(MarketChart).evaluate().isNotEmpty && find.byType(CircularProgressIndicator).evaluate().isEmpty,
          timeout: shortTimeout,
          what: what,
        );
      }

      await waitForChart("the daily chart");
      await shots.take("01.candles_1d");

      // Every style, through the toolbar menu.
      for (final style in MarketChartStyle.values.skip(1)) {
        await tester.tap(find.byTooltip("Chart style"));
        await pumpFor(tester, const Duration(milliseconds: 500));
        await tester.tap(find.text(style.label).last);
        await pumpFor(tester, const Duration(milliseconds: 500));
        await shots.take("02.${style.name}_1d");
      }

      // Back to candles, then the hourly interval.
      await tester.tap(find.byTooltip("Chart style"));
      await pumpFor(tester, const Duration(milliseconds: 500));
      await tester.tap(find.text(MarketChartStyle.candles.label).last);
      await pumpFor(tester, const Duration(milliseconds: 500));
      await tester.tap(find.text("1H"));
      await waitForChart("the hourly chart");
      await shots.take("03.candles_1h");

      // Volume off, log scale on.
      await tester.tap(find.byTooltip("Volume"));
      await tester.tap(find.byTooltip("Logarithmic scale"));
      await pumpFor(tester, const Duration(milliseconds: 500));
      await shots.take("04.no_volume_log");
      await tester.tap(find.byTooltip("Volume"));
      await tester.tap(find.byTooltip("Logarithmic scale"));

      // Scrolled into the past: the legend follows the crosshair on hover,
      // and the go-to-latest button appears.
      await tester.drag(find.byType(MarketChart), const Offset(300, 0));
      await pumpFor(tester, const Duration(milliseconds: 800));
      expect(find.byTooltip("Go to latest"), findsOneWidget);
      final gesture = await tester.createGesture(kind: PointerDeviceKind.mouse);
      await gesture.addPointer(location: Offset.zero);
      addTearDown(gesture.removePointer);
      await gesture.moveTo(tester.getCenter(find.byType(MarketChart)));
      await pumpFor(tester, const Duration(milliseconds: 500));
      await shots.take("05.scrolled_crosshair");
      await gesture.moveTo(Offset.zero);
      await tester.tap(find.byTooltip("Go to latest"));
      await pumpFor(tester, const Duration(seconds: 1));
      expect(find.byTooltip("Go to latest"), findsNothing);
      await shots.take("06.back_to_latest");

      // The poll: after ten seconds the page asks for the last two bars,
      // which the orchestrator checks in the mock's request log.
      await pumpFor(tester, const Duration(seconds: 11));

      shots.finish();
      rendezvous.publish("done", "1");
    } finally {
      await savedPrefs.restore(prefs);
    }
  });
}
