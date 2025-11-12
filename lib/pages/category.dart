import 'package:flutter/material.dart';
import 'package:flutter_form_builder/flutter_form_builder.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:gap/gap.dart';
import 'package:zkool/src/rust/api/account.dart';
import 'package:zkool/store.dart';
import 'package:zkool/utils.dart';

class CategoryPage extends ConsumerStatefulWidget {
  const CategoryPage({super.key});

  @override
  ConsumerState<CategoryPage> createState() => CategoryPageState();
}

class CategoryPageState extends ConsumerState<CategoryPage> {
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
    final categoryList = await ref.read(getCategoriesProvider.future);
    categories = categoryList.map((f) => (f, false)).toList();
    if (mounted) setState(() {});
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: Text("Categories"),
        actions: [
          IconButton(onPressed: onNew, icon: Icon(Icons.add)),
          if (hasSingleSelection) IconButton(onPressed: onEdit, icon: Icon(Icons.edit)),
          if (hasSelection) IconButton(onPressed: onDelete, icon: Icon(Icons.delete)),
        ],
      ),
      body: ListView.separated(
        itemBuilder: (BuildContext context, int index) {
          final f = categories[index];
          return ListTile(
            leading: Checkbox(value: f.$2, onChanged: (v) => setState(() => categories[index] = (f.$1, v ?? false))),
            title: Text(f.$1.name),
          );
        },
        itemCount: categories.length,
        separatorBuilder: (BuildContext context, int index) =>
            (index < categories.length - 1 && categories[index].$1.isIncome != categories[index + 1].$1.isIncome) ? Divider() : SizedBox.expand(),
      ),
    );
  }

  void onNew() async {
    final category = await categoryForm(Category(id: 0, name: "", isIncome: false), title: "New Category");
    if (category != null) {
      await createNewCategory(category: category);
      await refresh();
    }
  }

  void onEdit() async {
    final category = await categoryForm(selection.first, title: "Edit Category");
    if (category != null) {
      await renameCategory(category: category);
      await refresh();
    }
  }

  void onDelete() async {
    final confirmed =
        await confirmDialog(context, title: "Do you want to delete these categories?", message: "Transactions assigned to these categories will be kept.");
    if (confirmed) {
      await deleteCategories(ids: selection.map((f) => f.id).toList());
      await refresh();
      ref.invalidate(getCategoriesProvider);
    }
  }

  Future<Category?> categoryForm(Category initialValue, {required String title}) async {
    final t = Theme.of(context).textTheme;
    final formKey = GlobalKey<FormBuilderState>();
    return await inputData(
      context,
      builder: (context) => FormBuilder(
        key: formKey,
        child: Column(
          children: [
            Text(title, style: t.headlineSmall),
            Gap(8),
            FormBuilderTextField(
              name: "name",
              decoration: InputDecoration(label: Text("Name")),
              initialValue: initialValue.name,
              autofocus: true,
            ),
            FormBuilderCheckbox(
              name: "income",
              title: Text("Income?"),
              initialValue: initialValue.isIncome,
            ),
          ],
        ),
      ),
      onConfirmed: () {
        final fields = formKey.currentState!.fields;
        final name = fields["name"]!.value as String;
        final income = fields["income"]!.value as bool;
        return Category(id: 0, name: name, isIncome: income);
      },
    );
  }

  Iterable<Category> get selection => categories.where((a) => a.$2).map((a) => a.$1);
  bool get hasSingleSelection => selection.length == 1;
  bool get hasSelection => selection.isNotEmpty;
}
