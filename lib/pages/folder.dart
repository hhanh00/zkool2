import 'package:flutter/material.dart';
import 'package:zkool/src/rust/api/account.dart';
import 'package:zkool/utils.dart';

class FolderPage extends StatefulWidget {
  const FolderPage({super.key});

  @override
  State<StatefulWidget> createState() => FolderPageState();
}

class FolderPageState extends State<FolderPage> {
  List<(Folder, bool)> folders = [];

  @override
  void initState() {
    super.initState();
    Future(refresh);
  }

  Future<void> refresh() async {
    folders = (await listFolders()).map((f) => (f, false)).toList();
    setState(() {});
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: Text("Folders"),
      actions: [
        IconButton(onPressed: onNew, icon: Icon(Icons.add)),
        if (hasSingleSelection) IconButton(onPressed: onEdit, icon: Icon(Icons.edit)),
        if (hasSelection) IconButton(onPressed: onDelete, icon: Icon(Icons.delete)),
      ]),
      body: ListView.builder(
        itemBuilder: (BuildContext context, int index) {
          final f = folders[index];
          return ListTile(
            leading: Checkbox(value: f.$2, onChanged: (v) => setState(() => folders[index] = (f.$1, v ?? false))),
            title: Text(f.$1.name),
          );
        },
        itemCount: folders.length,
      ),
    );
  }

  void onNew() async {
    final folderName = await inputText(context, title: "New Folder");
    if (folderName != null) {
      await createNewFolder(name: folderName);
      await refresh();
    }
  }
  void onEdit() async {
    final folderName = await inputText(context, title: "Rename Folder");
    if (folderName != null) {
      await renameFolder(id: selection.first.id, name: folderName);
      await refresh();
    }
  }
  void onDelete() async {
    final confirmed = await confirmDialog(context, title: "Do you want to delete these folders?", message: "Accounts assigned to these folders will be kept.");
    if (confirmed) {
      await deleteFolders(ids: selection.map((f) => f.id).toList());
      await refresh();
    }
  }

  Iterable<Folder> get selection => folders.where((a) => a.$2).map((a) => a.$1);
  bool get hasSingleSelection => selection.length == 1;
  bool get hasSelection => selection.isNotEmpty;
}
