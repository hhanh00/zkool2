import 'dart:async';

import 'package:collection/collection.dart';
import 'package:data_table_2/data_table_2.dart';
import 'package:flutter/material.dart';
import 'package:mobx/mobx.dart';

class EditableList<T> extends StatefulWidget {
  final String title;
  final List<T> Function() observable;
  final FutureOr<void> Function()? onCreate;
  final FutureOr<void> Function(BuildContext) createBuilder;
  final FutureOr<void> Function(BuildContext, T) editBuilder;
  final FutureOr<void> Function(BuildContext, List<T>) deleteBuilder;
  final List<DataColumn> columns;
  final DataRow Function(
    BuildContext,
    int,
    T, {
    bool? selected,
    void Function(bool?)? onSelectChanged,
  }) builder;

  const EditableList({
    super.key,
    required this.observable,
    required this.builder,
    required this.title,
    required this.columns,
    this.onCreate,
    required this.createBuilder,
    required this.editBuilder,
    required this.deleteBuilder,
  });

  @override
  State<EditableList> createState() => EditableListState<T>();
}

class EditableListState<T> extends State<EditableList<T>> {
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
        title: Text(widget.title),
        actions: [
          if (!anySelected) IconButton(onPressed: onNew, icon: Icon(Icons.add)),
          if (editEnabled) IconButton(onPressed: onEdit, icon: Icon(Icons.edit)),
          if (anySelected) IconButton(onPressed: onDelete, icon: Icon(Icons.delete)),
          ],
      ),
      body: DataTable2(
          columnSpacing: 8,
          horizontalMargin: 8,
          onSelectAll: (v) {
            if (v != null) {
              setState(() {
                for (var i = 0; i < selected.length; i++) 
                  selected[i] = v;
                if (v && editEnabled) // can only select 1 item
                  selectedValue = items.first;
                else
                  selectedValue = null;
              });
            }
          },
          columns: widget.columns,
          rows: items
              .asMap()
              .entries
              .map((e) => widget.builder(context, e.key, e.value,
                  selected: selected[e.key],
                  onSelectChanged: (value) =>
                      setState(() { 
                        selected[e.key] = value ?? false;
                        selectedValue = items[e.key];
                      })))
              .toList()),
    );
  }

  onNew() => widget.createBuilder(context);
  onEdit() {
    final sv = selectedValue;
    if (sv != null) widget.editBuilder(context, sv);
  }
  onDelete() => widget.deleteBuilder(context, items.whereIndexed((index, _) => selected[index]).toList());
}

T identity<T>(T t) => t;
