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
    if (folderName != null)
      await createNewFolder(name: folderName);
  }
  void onEdit() async {}
  void onDelete() async {}

  Iterable<Folder> get selection => folders.where((a) => a.$2).map((a) => a.$1);
  bool get hasSingleSelection => selection.length == 1;
  bool get hasSelection => selection.isNotEmpty;
}

class Folder {
  int id = 0;
  String name = "";
}
