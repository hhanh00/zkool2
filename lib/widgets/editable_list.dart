import 'dart:async';

import 'package:animated_reorderable_list/animated_reorderable_list.dart';
import 'package:collection/collection.dart';
import 'package:flutter/material.dart';
import 'package:mobx/mobx.dart';
import 'package:showcaseview/showcaseview.dart';

final newAccountId = GlobalKey();

class EditableList<T extends Object> extends StatefulWidget {
  final String title;
  final List<T> items;
  final List<Widget> Function(BuildContext) headerBuilder;
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
  final void Function(int a, int b) onReorder;
  final List<Widget>? buttons;

  const EditableList({
    super.key,
    required this.items,
    required this.builder,
    required this.title,
    required this.headerBuilder,
    required this.createBuilder,
    required this.editBuilder,
    required this.deleteBuilder,
    required this.isEqual,
    required this.onReorder,
    this.buttons
  });

  @override
  State<EditableList> createState() => EditableListState<T>();
}

class EditableListState<T extends Object> extends State<EditableList<T>> {
  late List<T> items = widget.items;
  late List<bool> selected = List.generate(widget.items.length, (_) => false);
  ReactionDisposer? reaction;

  @override
  void initState() {
    super.initState();
  }

  @override
  void dispose() {
    reaction?.call();
    super.dispose();
  }

  @override
  void didUpdateWidget(covariant EditableList<T> oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.items != widget.items) {
      items = widget.items;
      selected = List.generate(items.length, (_) => false);
    }
  }

  @override
  Widget build(BuildContext context) {
    final anySelected = selected.any(identity);

    return Scaffold(
        appBar: AppBar(
          centerTitle: false,
          title: Text(widget.title),
          actions: [
            if (!anySelected)
              Showcase(key: newAccountId, description: "Create new account",
                child: IconButton(onPressed: onNew, icon: Icon(Icons.add))),
            if (anySelected)
              IconButton(onPressed: onEdit, icon: Icon(Icons.edit)),
            if (anySelected)
              IconButton(onPressed: onDelete, icon: Icon(Icons.delete)),
            ...?widget.buttons,
          ],
        ),
        body: Padding(
          padding: const EdgeInsets.symmetric(horizontal: 16),
          child: Column(children: [
          ...widget.headerBuilder(context),
          Expanded(child: AnimatedReorderableListView<T>(
          buildDefaultDragHandles: false,
          items: items,
          itemBuilder: (context, index) =>
              widget.builder(context, index, items[index],
                  selected: selected[index],
                  onSelectChanged: (value) => setState(() {
                        selected[index] = value ?? false;
                      })),
          isSameItem: (T a, T b) => widget.isEqual(a, b),
          onReorder: (int oldIndex, int newIndex) {
            widget.onReorder(oldIndex, newIndex);
            setState(() {
              final T v = items.removeAt(oldIndex);
              items.insert(newIndex, v);
            });
          },
        ))])));
  }

  onNew() => widget.createBuilder(context);
  onEdit() => widget.editBuilder(context, selectedItems);

  onDelete() => widget.deleteBuilder(context, selectedItems);

  List<T> get selectedItems => items.whereIndexed((index, _) => selected[index]).toList();
}

T identity<T>(T t) => t;
