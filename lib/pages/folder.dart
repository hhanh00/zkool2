import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:zkool/src/rust/api/account.dart';
import 'package:zkool/store.dart';
import 'package:zkool/utils.dart';

class FolderPage extends ConsumerStatefulWidget {
  const FolderPage({super.key});

  @override
  ConsumerState<FolderPage> createState() => FolderPageState();
}

class FolderPageState extends ConsumerState<FolderPage> {
  List<(Folder, bool)> folders = [];
  int? selectedIndex;

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
    final foldrs = await ref.read(getFoldersProvider.future);
    final selectedFolder = ref.read(selectedFolderProvider);
    if (selectedFolder != null) {
      selectedIndex = foldrs.indexWhere((f) => f.id == selectedFolder.id);
      if (selectedIndex == -1) {
        selectedIndex = null;
        (ref.read(selectedFolderProvider.notifier)).unselect();
      }
    }
    folders = foldrs.map((f) => (f, false)).toList();
    if (mounted) setState(() {});
  }

  @override
  Widget build(BuildContext context) {
    final cs = Theme.of(context).colorScheme;
    return Scaffold(
      appBar: AppBar(title: Text("Folders"), actions: [
        IconButton(onPressed: onNew, icon: Icon(Icons.add)),
        if (hasSingleSelection) IconButton(onPressed: onEdit, icon: Icon(Icons.edit)),
        if (hasSelection) IconButton(onPressed: onDelete, icon: Icon(Icons.delete)),
      ],),
      body: ListView.builder(
        itemBuilder: (BuildContext context, int index) {
          final f = folders[index];
          return ListTile(
            leading: Checkbox(value: f.$2, onChanged: (v) => setState(() => folders[index] = (f.$1, v ?? false))),
            title: Text(f.$1.name),
            onTap: () => onSelect(index),
            selected: selectedIndex == index,
            selectedTileColor: cs.primaryContainer,
          );
        },
        itemCount: folders.length,
      ),
    );
  }

  void onSelect(int index) async {
    final selectedFolder = ref.read(selectedFolderProvider.notifier);
    if (selectedIndex == index) {
      selectedIndex = null;
      selectedFolder.unselect();
    } else {
      selectedIndex = index;
      selectedFolder.selectFolder(folders[index].$1);
    }
    setState(() {});
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
      ref.invalidate(getAccountsProvider);
    }
  }

  Iterable<Folder> get selection => folders.where((a) => a.$2).map((a) => a.$1);
  bool get hasSingleSelection => selection.length == 1;
  bool get hasSelection => selection.isNotEmpty;
}
