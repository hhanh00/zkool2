// The budget charts: slice folding, running totals, and the donut and time
// chart rendering with data, with a single point, and empty, in both themes.
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:zkool/chart.dart';

const _slices = [
  CategorySlice("Food & Groceries", 120.5),
  CategorySlice("Rent", 900),
  CategorySlice("Transport", 45.25),
  CategorySlice("Coffee", 12),
];

Widget _app(Widget child, {bool dark = false}) => MaterialApp(
      theme: ThemeData(brightness: dark ? Brightness.dark : Brightness.light),
      home: Scaffold(body: SizedBox(width: 600, height: 500, child: child)),
    );

void main() {
  group("foldSlices", () {
    test("sorts by value, largest first, and drops empty slices", () {
      final folded = foldSlices([..._slices, const CategorySlice("Nothing", 0)]);
      expect(folded.map((s) => s.name), ["Rent", "Food & Groceries", "Transport", "Coffee"]);
    });

    test("folds the smallest into Other beyond the limit", () {
      final many = [for (var i = 1; i <= 10; i++) CategorySlice("c$i", i.toDouble())];
      final folded = foldSlices(many, max: 4);
      expect(folded.length, 4);
      expect(folded.take(3).map((s) => s.name), ["c10", "c9", "c8"]);
      expect(folded.last, const CategorySlice("Other", 1 + 2 + 3 + 4 + 5 + 6 + 7));
      expect(folded.fold(0.0, (sum, s) => sum + s.value), 55);
    });

    test("keeps exactly the limit without an Other", () {
      final eight = [for (var i = 1; i <= 8; i++) CategorySlice("c$i", i.toDouble())];
      expect(foldSlices(eight).length, 8);
      expect(foldSlices(eight).any((s) => s.name == "Other"), isFalse);
    });
  });

  test("cumulativeSums runs the total over absolute amounts", () {
    expect(cumulativeSums([(1, 1.5), (2, -2.0), (3, 0.5)]), [(1, 1.5), (2, 3.5), (3, 4.0)]);
    expect(cumulativeSums([]), isEmpty);
  });

  for (final dark in [false, true]) {
    final theme = dark ? "dark" : "light";

    testWidgets("donut renders slices and legend, $theme", (tester) async {
      await tester.pumpWidget(_app(const CategoryDonut(slices: _slices), dark: dark));
      await tester.pump();
      expect(tester.takeException(), isNull);
      for (final s in _slices) {
        expect(find.text(s.name), findsOneWidget);
      }
      // Rent is 900 of 1077.75: 84%.
      expect(find.textContaining("(84%)"), findsOneWidget);
      // Tapping a legend entry highlights it.
      await tester.tap(find.text("Rent"));
      await tester.pump();
      final rent = tester.widget<Text>(find.text("Rent"));
      expect(rent.style?.fontWeight, FontWeight.bold);
    });

    testWidgets("donut shows an empty state, $theme", (tester) async {
      await tester.pumpWidget(_app(const CategoryDonut(slices: []), dark: dark));
      await tester.pump();
      expect(find.text("No amounts in this period"), findsOneWidget);
    });

    testWidgets("time chart renders points and the cumulative step line, $theme", (tester) async {
      const points = [(1760000000, 10.0), (1760500000, -25.0), (1761000000, 5.0)];
      await tester.pumpWidget(_app(const CategoryTimeChart(points: points), dark: dark));
      await tester.pump();
      expect(tester.takeException(), isNull);
      expect(find.byType(CategoryTimeChart), findsOneWidget);
      await tester.pumpWidget(_app(const CategoryTimeChart(points: points, cumulative: true), dark: dark));
      await tester.pump();
      expect(tester.takeException(), isNull);
    });

    testWidgets("time chart copes with one point and with none, $theme", (tester) async {
      await tester.pumpWidget(_app(const CategoryTimeChart(points: [(1760000000, 3.0)]), dark: dark));
      await tester.pump();
      expect(tester.takeException(), isNull);
      await tester.pumpWidget(_app(const CategoryTimeChart(points: []), dark: dark));
      await tester.pump();
      expect(find.text("No amounts in this period"), findsOneWidget);
    });
  }
}
