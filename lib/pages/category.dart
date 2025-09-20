import 'package:flutter/material.dart';
import 'package:zkool/src/rust/api/account.dart';
import 'package:zkool/store.dart';
import 'package:zkool/utils.dart';

class CategoryPage extends StatefulWidget {
  const CategoryPage({super.key});

  @override
  State<StatefulWidget> createState() => CategoryPageState();
}

class CategoryPageState extends State<CategoryPage> {
  List<(Category, bool)> categories = [];

  @override
  void initState() {
    super.initState();
    Future(refresh);
  }

  @override
  void dispose() {
    Future(refresh);
    super.dispose();
  }

  Future<void> refresh() async {
    await appStore.loadCategories();
    categories = appStore.categories.map((f) => (f, false)).toList();
    if (mounted) setState(() {});
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: Text("Categories"), actions: [
        IconButton(onPressed: onNew, icon: Icon(Icons.add)),
        if (hasSingleSelection) IconButton(onPressed: onEdit, icon: Icon(Icons.edit)),
        if (hasSelection) IconButton(onPressed: onDelete, icon: Icon(Icons.delete)),
      ],),
      body: ListView.builder(
        itemBuilder: (BuildContext context, int index) {
          final f = categories[index];
          return ListTile(
            leading: Checkbox(value: f.$2, onChanged: (v) => setState(() => categories[index] = (f.$1, v ?? false))),
            title: Text(f.$1.name),
          );
        },
        itemCount: categories.length,
      ),
    );
  }

  void onNew() async {
    final categoryName = await inputText(context, title: "New Category");
    if (categoryName != null) {
      await createNewCategory(name: categoryName);
      await refresh();
    }
  }

  void onEdit() async {
    final categoryName = await inputText(context, title: "Rename Category");
    if (categoryName != null) {
      await renameCategory(id: selection.first.id, name: categoryName);
      await refresh();
    }
  }

  void onDelete() async {
    final confirmed = await confirmDialog(context, title: "Do you want to delete these categories?", message: "Transactions assigned to these categories will be kept.");
    if (confirmed) {
      await deleteCategories(ids: selection.map((f) => f.id).toList());
      await refresh();
      await appStore.loadCategories();
    }
  }

  Iterable<Category> get selection => categories.where((a) => a.$2).map((a) => a.$1);
  bool get hasSingleSelection => selection.length == 1;
  bool get hasSelection => selection.isNotEmpty;
}
