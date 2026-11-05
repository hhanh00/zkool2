import 'package:flutter/material.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:zkool/pages/account.dart';
import 'package:zkool/src/rust/api/sync.dart';
import 'package:zkool/utils.dart';
import 'package:zkool/widgets/theme.dart';

/// Below the 50,000 zatoshi cutoff: shows as 0.000.
const _dust = 49999;

/// At the cutoff: shows as 0.001.
const _cutoff = 50000;

PoolBalance _balance(int transparent, int sapling, int orchard, int ironwood) => PoolBalance(
      field0: Uint64List.fromList([
        BigInt.from(transparent),
        BigInt.from(sapling),
        BigInt.from(orchard),
        BigInt.from(ironwood),
      ]),
    );

Future<void> _pump(WidgetTester tester, Widget child) => tester.pumpWidget(
      MaterialApp(home: Scaffold(body: Center(child: child))),
    );

/// The chips on screen as "pool value", in widget order.
List<String> _chips(WidgetTester tester) => [
      for (final chip in tester.widgetList<BalanceChip>(find.byType(BalanceChip))) '${chip.pool.name} ${chip.value}',
    ];

/// The chips grouped by the line they sit on, top to bottom, left to right.
List<List<String>> _rows(WidgetTester tester) {
  final finder = find.byType(BalanceChip);
  final byLine = <double, List<(double, String)>>{};
  for (final element in finder.evaluate()) {
    final center = tester.getCenter(find.byWidget(element.widget));
    byLine.putIfAbsent(center.dy, () => []).add((center.dx, (element.widget as BalanceChip).pool.name));
  }
  final lines = byLine.keys.toList()..sort();
  return [
    for (final line in lines) [for (final (_, name) in byLine[line]!..sort((a, b) => a.$1.compareTo(b.$1))) name],
  ];
}

/// Each line of chips, as a whole, sits on the screen's vertical center line.
void _expectRowsCentered(WidgetTester tester) {
  final screenCenter = tester.getCenter(find.byType(Scaffold)).dx;
  final byLine = <double, List<Rect>>{};
  for (final element in find.byType(BalanceChip).evaluate()) {
    final rect = tester.getRect(find.byWidget(element.widget));
    byLine.putIfAbsent(rect.center.dy, () => []).add(rect);
  }
  for (final rects in byLine.values) {
    final left = rects.map((r) => r.left).reduce((a, b) => a < b ? a : b);
    final right = rects.map((r) => r.right).reduce((a, b) => a > b ? a : b);
    expect((left + right) / 2, moreOrLessEquals(screenCenter));
  }
}

void main() {
  test('the chip formatter rounds 49,999 zatoshis down and 50,000 up', () {
    expect(zatToShortString(BigInt.from(_dust)), '0.000');
    expect(zatToShortString(BigInt.from(_cutoff)), '0.001');
  });

  group('BalanceWidget hiding pools that show as 0.000', () {
    testWidgets('49,999 zatoshis is hidden and 50,000 is shown', (tester) async {
      await _pump(tester, BalanceWidget(_balance(0, _dust, _cutoff, 0), hideZeroPools: true));

      expect(_chips(tester), ['orchard 0.001']);
    });

    testWidgets('three chips split 2 + 1, in pool order, each row centered', (tester) async {
      await _pump(tester, BalanceWidget(_balance(_cutoff, 0, _cutoff, _cutoff), hideZeroPools: true));

      expect(_chips(tester), ['transparent 0.001', 'orchard 0.001', 'ironwood 0.001']);
      expect(_rows(tester), [
        ['transparent', 'orchard'],
        ['ironwood'],
      ]);
      _expectRowsCentered(tester);
    });

    testWidgets('four chips split 2 + 2 and two chips share one row', (tester) async {
      await _pump(tester, BalanceWidget(_balance(_cutoff, _cutoff, _cutoff, _cutoff), hideZeroPools: true));
      expect(_rows(tester), [
        ['transparent', 'sapling'],
        ['orchard', 'ironwood'],
      ]);

      await _pump(tester, BalanceWidget(_balance(0, _cutoff, 0, _cutoff), hideZeroPools: true));
      expect(_rows(tester), [
        ['sapling', 'ironwood'],
      ]);
      _expectRowsCentered(tester);
    });
  });

  group('PoolBalanceControl', () {
    Widget control(PoolBalance balance, {required bool collapsed, bool incomingOnly = false, VoidCallback? onToggle}) => PoolBalanceControl(
          balance,
          collapsed: collapsed,
          incomingOnly: incomingOnly,
          onToggleCollapsed: onToggle ?? () {},
        );

    for (final incomingOnly in [false, true]) {
      final label = incomingOnly ? 'Received by Pool' : 'Pool Balances';

      group(incomingOnly ? 'incoming-only' : 'regular account', () {
        for (final collapsed in [false, true]) {
          testWidgets('is hidden with no visible chips when ${collapsed ? "collapsed" : "expanded"}', (tester) async {
            // 0.0004 ZEC in total: a nonzero account with nothing above the cutoff.
            await _pump(tester, control(_balance(10000, 10000, 10000, 10000), collapsed: collapsed, incomingOnly: incomingOnly));

            expect(find.byType(BalanceWidget), findsNothing);
            expect(find.byType(BalanceChip), findsNothing);
            expect(find.text(label), findsNothing);
            expect(find.byIcon(Icons.expand_more), findsNothing);
            expect(tester.getSize(find.byType(PoolBalanceControl)), Size.zero);
          });
        }

        testWidgets('collapsed, one visible chip shows the "$label" row', (tester) async {
          await _pump(tester, control(_balance(0, 0, _cutoff, 0), collapsed: true, incomingOnly: incomingOnly));

          expect(find.text(label), findsOneWidget);
          expect(find.byType(BalanceChip), findsNothing);
        });

        testWidgets('expanded, only the visible chips show', (tester) async {
          await _pump(tester, control(_balance(0, _dust, _cutoff, 0), collapsed: false, incomingOnly: incomingOnly));

          expect(_chips(tester), ['orchard 0.001']);
          expect(find.text(label), findsNothing);
        });
      });
    }

    testWidgets('tapping toggles the collapse setting; hiding leaves it alone', (tester) async {
      var toggles = 0;
      await _pump(tester, control(_balance(0, 0, _cutoff, 0), collapsed: true, onToggle: () => toggles++));
      await tester.tap(find.text('Pool Balances'));
      expect(toggles, 1);

      // Hidden, then a chip returns: still collapsed, with no toggle in between.
      await _pump(tester, control(_balance(0, 0, 0, 0), collapsed: true, onToggle: () => toggles++));
      await _pump(tester, control(_balance(0, 0, _cutoff, 0), collapsed: true, onToggle: () => toggles++));
      expect(find.text('Pool Balances'), findsOneWidget);
      expect(toggles, 1);
    });
  });

  group('BalanceWidget on the Send page', () {
    testWidgets('shows all four chips, including 0.000, split 2 + 2', (tester) async {
      await _pump(tester, BalanceWidget(_balance(0, _dust, _cutoff, 0), onPoolSelected: (_) {}));

      expect(_chips(tester), ['transparent 0.000', 'sapling 0.000', 'orchard 0.001', 'ironwood 0.000']);
      expect(_rows(tester), [
        ['transparent', 'sapling'],
        ['orchard', 'ironwood'],
      ]);
    });

    testWidgets('tapping an empty pool still selects it', (tester) async {
      final selected = <int>[];
      await _pump(tester, BalanceWidget(_balance(0, 0, 0, 0), onPoolSelected: selected.add));

      for (final chip in tester.widgetList<BalanceChip>(find.byType(BalanceChip)).toList()) {
        await tester.tap(find.byWidget(chip));
      }

      expect(selected, [0, 1, 2, 3]);
    });
  });
}
