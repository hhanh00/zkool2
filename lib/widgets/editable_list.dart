import 'dart:async';

import 'package:animated_reorderable_list/animated_reorderable_list.dart';
import 'package:flutter/material.dart';

class EditableList<T extends Object> extends StatefulWidget {
  final String title;
  final List<T> items;
  final List<Widget> Function(BuildContext)? headerBuilder;
  final FutureOr<void> Function(BuildContext) createBuilder;
  final FutureOr<void> Function(BuildContext, List<T>) editBuilder;
  final FutureOr<void> Function(BuildContext, List<T>) deleteBuilder;
  final Widget Function(
    BuildContext,
    int,
    T, {
    bool? selected,
    void Function(bool?)? onSelectChanged,
  }) builder;
  final bool Function(T a, T b) isEqual;
  final void Function(int a, int b)? onReorder;
  final List<Widget>? buttons;
  final WidgetBuilder emptyBuilder;

  const EditableList({
    super.key,
    required this.items,
    required this.builder,
    required this.title,
    this.headerBuilder,
    required this.createBuilder,
    required this.editBuilder,
    required this.deleteBuilder,
    required this.isEqual,
    this.onReorder,
    this.buttons,
    required this.emptyBuilder,
  });

  @override
  State<EditableList> createState() => EditableListState<T>();
}

class EditableListState<T extends Object> extends State<EditableList<T>> {
  late List<T> items = widget.items;
  /// The selection, as items compared with `isEqual`: it follows them
  /// through a reorder and a reload, where a row index would not.
  List<T> _selected = [];

  bool _isSelected(T item) => _selected.any((s) => widget.isEqual(s, item));

  void _select(T item, bool value) {
    setState(() {
      _selected.removeWhere((s) => widget.isEqual(s, item));
      if (value) _selected.add(item);
    });
  }

  @override
  void didUpdateWidget(covariant EditableList<T> oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.items != widget.items) {
      items = widget.items;
      // Keep what is still there, as the reloaded objects.
      _selected = items.where(_isSelected).toList();
    }
  }

  @override
  Widget build(BuildContext context) {
    final anySelected = _selected.isNotEmpty;

    return Scaffold(
      appBar: AppBar(
        centerTitle: false,
        title: Text(widget.title),
        actions: [
          if (!anySelected) IconButton(onPressed: onNew, tooltip: "Create new account", icon: Icon(Icons.add)),
          if (anySelected) IconButton(onPressed: onEdit, icon: Icon(Icons.edit)),
          if (anySelected) IconButton(onPressed: onDelete, icon: Icon(Icons.delete)),
          ...?widget.buttons,
        ],
      ),
      body: Padding(
        padding: const EdgeInsets.symmetric(horizontal: 16),
        child: Column(
          children: [
            ...?widget.headerBuilder?.call(context),
            if (items.isEmpty)
              Expanded(child: widget.emptyBuilder(context))
            else
              Expanded(
                child: AnimatedReorderableListView<T>(
                  buildDefaultDragHandles: false,
                  items: items,
                  itemBuilder: (context, index) => widget.builder(
                    context,
                    index,
                    items[index],
                    selected: _isSelected(items[index]),
                    onSelectChanged: (value) => _select(items[index], value ?? false),
                  ),
                  isSameItem: (T a, T b) => widget.isEqual(a, b),
                  onReorder: (int oldIndex, int newIndex) {
                    widget.onReorder?.call(oldIndex, newIndex);
                    setState(() {
                      final T v = items.removeAt(oldIndex);
                      items.insert(newIndex, v);
                    });
                  },
                ),
              ),
          ],
        ),
      ),
    );
  }

  onNew() => widget.createBuilder(context);
  onEdit() => widget.editBuilder(context, selectedItems);

  onDelete() => widget.deleteBuilder(context, selectedItems);

  /// The selected items, in list order.
  List<T> get selectedItems => items.where(_isSelected).toList();
}
