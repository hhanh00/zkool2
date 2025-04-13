import 'dart:async';

import 'package:animated_reorderable_list/animated_reorderable_list.dart';
import 'package:collection/collection.dart';
import 'package:flutter/material.dart';
import 'package:mobx/mobx.dart';

class EditableList<T extends Object> extends StatefulWidget {
  final String title;
  final List<T> Function() observable;
  final List<Widget> Function(BuildContext) headerBuilder;
  final FutureOr<void> Function()? onCreate;
  final FutureOr<void> Function(BuildContext) createBuilder;
  final FutureOr<void> Function(BuildContext, T) editBuilder;
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
    required this.observable,
    required this.builder,
    required this.title,
    this.onCreate,
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
  List<bool> selected = [];
  List<T> items = [];
  T? selectedValue;
  ReactionDisposer? reaction;

  @override
  void initState() {
    super.initState();

    reaction = autorun((_) {
      final updatedItems = widget.observable();
      setState(() {
        items = updatedItems;
        selected = List.generate(items.length, (index) => false);
      });
    });

    widget.onCreate?.call();
  }

  @override
  void dispose() {
    reaction?.call();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final editEnabled = selected.where(identity).length == 1;
    final anySelected = selected.any(identity);

    return Scaffold(
        appBar: AppBar(
          centerTitle: false,
          title: Text(widget.title),
          actions: [
            if (!anySelected)
              IconButton(onPressed: onNew, icon: Icon(Icons.add)),
            if (editEnabled)
              IconButton(onPressed: onEdit, icon: Icon(Icons.edit)),
            if (anySelected)
              IconButton(onPressed: onDelete, icon: Icon(Icons.delete)),
            ...?widget.buttons,
          ],
        ),
        body: Column(children: [
          ...widget.headerBuilder(context),
          Expanded(child: AnimatedReorderableListView<T>(
          buildDefaultDragHandles: false,
          items: items,
          itemBuilder: (context, index) =>
              widget.builder(context, index, items[index],
                  selected: selected[index],
                  onSelectChanged: (value) => setState(() {
                        selected[index] = value ?? false;
                        selectedValue = items[index];
                      })),
          isSameItem: (T a, T b) => widget.isEqual(a, b),
          onReorder: (int oldIndex, int newIndex) {
            widget.onReorder(oldIndex, newIndex);
            setState(() {
              final T v = items.removeAt(oldIndex);
              items.insert(newIndex, v);
            });
          },
        ))]));
  }

  onNew() => widget.createBuilder(context);
  onEdit() {
    final sv = selectedValue;
    if (sv != null) widget.editBuilder(context, sv);
  }

  onDelete() => widget.deleteBuilder(
      context, items.whereIndexed((index, _) => selected[index]).toList());
}

T identity<T>(T t) => t;
