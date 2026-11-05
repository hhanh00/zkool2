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
List<List<(BalanceChip, Rect)>> _lines(WidgetTester tester) {
  final byLine = <double, List<(BalanceChip, Rect)>>{};
  for (final element in find.byType(BalanceChip).evaluate()) {
    final rect = tester.getRect(find.byWidget(element.widget));
    byLine.putIfAbsent(rect.center.dy, () => []).add((element.widget as BalanceChip, rect));
  }
  final tops = byLine.keys.toList()..sort();
  return [for (final top in tops) byLine[top]!..sort((a, b) => a.$2.left.compareTo(b.$2.left))];
}

/// The pool names on each line.
List<List<String>> _rows(WidgetTester tester) => [
      for (final line in _lines(tester)) [for (final (chip, _) in line) chip.pool.name],
    ];

/// Each line of chips, as a whole, sits on the screen's vertical center line.
void _expectRowsCentered(WidgetTester tester) {
  final screenCenter = tester.getCenter(find.byType(Scaffold)).dx;
  for (final line in _lines(tester)) {
    expect((line.first.$2.left + line.last.$2.right) / 2, moreOrLessEquals(screenCenter));
  }
}

void main() {
  test('the chip formatter rounds 49,999 zatoshis down and 50,000 up', () {
    expect(zatToShortString(BigInt.from(_dust)), '0.000');
    expect(zatToShortString(BigInt.from(_cutoff)), '0.001');
  });

  test('the cutoff derived from the short format is 50,000 zatoshis', () {
    expect(minNonZeroShortZat, BigInt.from(50000));
    expect(zatToShortString(minNonZeroShortZat - BigInt.one), '0.000');
    expect(zatToShortString(minNonZeroShortZat), '0.001');
  });

  group('PoolBalanceControl', () {
    Widget control(PoolBalance balance, {required bool collapsed, bool incomingOnly = false, VoidCallback? onToggle}) => PoolBalanceControl(
          balance,
          collapsed: collapsed,
          incomingOnly: incomingOnly,
          onToggleCollapsed: onToggle ?? () {},
        );

    testWidgets('49,999 zatoshis is hidden and 50,000 is shown', (tester) async {
      await _pump(tester, control(_balance(0, _dust, _cutoff, 0), collapsed: false));

      expect(_chips(tester), ['orchard 0.001']);
    });

    testWidgets('three chips split 2 + 1, in pool order, each row centered', (tester) async {
      await _pump(tester, control(_balance(_cutoff, 0, _cutoff, _cutoff), collapsed: false));

      expect(_chips(tester), ['transparent 0.001', 'orchard 0.001', 'ironwood 0.001']);
      expect(_rows(tester), [
        ['transparent', 'orchard'],
        ['ironwood'],
      ]);
      _expectRowsCentered(tester);
    });

    testWidgets('four chips split 2 + 2 and two chips share one row', (tester) async {
      await _pump(tester, control(_balance(_cutoff, _cutoff, _cutoff, _cutoff), collapsed: false));
      expect(_rows(tester), [
        ['transparent', 'sapling'],
        ['orchard', 'ironwood'],
      ]);

      await _pump(tester, control(_balance(0, _cutoff, 0, _cutoff), collapsed: false));
      expect(_rows(tester), [
        ['sapling', 'ironwood'],
      ]);
      _expectRowsCentered(tester);
    });

    testWidgets('each chip shows its own pool balance', (tester) async {
      await _pump(tester, control(_balance(100000, 200000, 300000, 400000), collapsed: false));

      expect(_chips(tester), ['transparent 0.001', 'sapling 0.002', 'orchard 0.003', 'ironwood 0.004']);
    });

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

    testWidgets('tapping a chip reaches the collapse toggle', (tester) async {
      var toggles = 0;
      await _pump(tester, control(_balance(0, 0, _cutoff, 0), collapsed: false, onToggle: () => toggles++));

      await tester.tap(find.byType(BalanceChip));

      expect(toggles, 1);
    });

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

    testWidgets('tapping a chip reports its pool, even an empty one', (tester) async {
      final selected = <String, int>{};
      late BalanceChip tapped;
      await _pump(tester, BalanceWidget(_balance(0, 0, 0, 0), onPoolSelected: (index) => selected[tapped.pool.name] = index));

      for (final chip in tester.widgetList<BalanceChip>(find.byType(BalanceChip)).toList()) {
        tapped = chip;
        await tester.tap(find.byWidget(chip));
      }

      expect(selected, {'transparent': 0, 'sapling': 1, 'orchard': 2, 'ironwood': 3});
    });
  });
}
