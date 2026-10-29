import 'package:animated_reorderable_list/animated_reorderable_list.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:zkool/widgets/editable_list.dart';

class _Row {
  final int id;
  final String name;
  const _Row(this.id, this.name);
}

Widget _list(List<_Row> items) => MaterialApp(
      home: EditableList<_Row>(
        items: items,
        title: 'Rows',
        builder: (context, index, row, {selected, onSelectChanged}) => CheckboxListTile(
          key: ValueKey(row.id),
          value: selected ?? false,
          onChanged: onSelectChanged,
          title: Text(row.name),
        ),
        createBuilder: (_) {},
        editBuilder: (_, __) {},
        deleteBuilder: (_, __) {},
        isEqual: (a, b) => a.id == b.id,
        emptyBuilder: (_) => const Text('empty'),
      ),
    );

List<String> _selectedNames(WidgetTester tester) => tester
    .state<EditableListState<_Row>>(find.byType(EditableList<_Row>))
    .selectedItems
    .map((r) => r.name)
    .toList();

void main() {
  testWidgets('the selection follows the item through a reorder', (tester) async {
    await tester.pumpWidget(_list([const _Row(1, 'a'), const _Row(2, 'b'), const _Row(3, 'c')]));
    await tester.tap(find.text('c'));
    await tester.pump();
    expect(_selectedNames(tester), ['c']);

    // Drag the third row to the top, as the drag handle would.
    tester
        .widget<AnimatedReorderableListView<_Row>>(find.byType(AnimatedReorderableListView<_Row>))
        .onReorder
        ?.call(2, 0);
    await tester.pumpAndSettle();
    expect(_selectedNames(tester), ['c'], reason: 'the selection stays on c, now at index 0');
    expect(tester.widget<CheckboxListTile>(find.byKey(const ValueKey(3))).value, isTrue);
    expect(tester.widget<CheckboxListTile>(find.byKey(const ValueKey(1))).value, isFalse);
  });

  testWidgets('a reload keeps the items still present and drops the rest', (tester) async {
    await tester.pumpWidget(_list([const _Row(1, 'a'), const _Row(2, 'b'), const _Row(3, 'c')]));
    await tester.tap(find.text('b'));
    await tester.tap(find.text('c'));
    await tester.pump();
    expect(_selectedNames(tester), ['b', 'c']);
    expect(find.byIcon(Icons.delete), findsOneWidget);

    // Reloaded: fresh objects, c renamed, b gone.
    await tester.pumpWidget(_list([const _Row(3, 'c2'), const _Row(1, 'a')]));
    await tester.pumpAndSettle();
    expect(_selectedNames(tester), ['c2'], reason: 'c by id, with its reloaded object');
    expect(find.byIcon(Icons.delete), findsOneWidget);

    await tester.pumpWidget(_list([const _Row(1, 'a')]));
    await tester.pumpAndSettle();
    expect(_selectedNames(tester), isEmpty);
    expect(find.byIcon(Icons.delete), findsNothing, reason: 'nothing selected, no delete');
  });
}
