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
  late final c = coinContext.coin;
  late final ProviderContainer container;
  late final SelectedFolder selectedFolderNotifier;
  List<(Folder, bool)> folders = [];
  int? selectedIndex;

  @override
  void initState() {
    super.initState();
    // Capture these now: `ref` is unsafe to use once the widget is unmounted,
    // so the disposal path must not touch it. `containerOf(listen: false)`
    // avoids the inherited-widget dependency that `ref.container` creates
    // (not allowed in initState).
    container = ProviderScope.containerOf(context, listen: false);
    selectedFolderNotifier = ref.read(selectedFolderProvider.notifier);
    _load();
  }

  @override
  void dispose() {
    // Unselect the folder if it was deleted while the page was open. Runs on
    // a later tick and never uses `ref`.
    Future(() async {
      final selected = container.exists(selectedFolderProvider)
          ? container.read(selectedFolderProvider)
          : null;
      if (selected == null) return;
      final foldrs = await listFolders(c: c);
      refresh(foldrs, selected);
    });
    super.dispose();
  }

  // Fetches the latest folders and applies them to the page. `ref` is only
  // used before the await, so this stays safe if the widget unmounts mid-load.
  Future<void> _load() async {
    final foldrs = await ref.read(getFoldersProvider.future);
    refresh(foldrs, container.read(selectedFolderProvider));
  }

  // Applies a folder list to the page. Takes everything it needs as
  // parameters and never touches `ref`, so it is safe to call after unmount.
  void refresh(List<Folder> foldrs, Folder? selectedFolder) {
    if (selectedFolder != null) {
      selectedIndex = foldrs.indexWhere((f) => f.id == selectedFolder.id);
      if (selectedIndex == -1) {
        selectedIndex = null;
        if (container.exists(selectedFolderProvider)) selectedFolderNotifier.unselect();
      }
    }
    folders = foldrs.map((f) => (f, false)).toList();
    if (mounted) setState(() {});
  }

  @override
  Widget build(BuildContext context) {
    final cs = Theme.of(context).colorScheme;
    return Scaffold(
      appBar: AppBar(
        title: Text("Folders"),
        actions: [
          IconButton(onPressed: onNew, icon: Icon(Icons.add)),
          if (hasSingleSelection) IconButton(onPressed: onEdit, icon: Icon(Icons.edit)),
          if (hasSelection) IconButton(onPressed: onDelete, icon: Icon(Icons.delete)),
        ],
      ),
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
      await createNewFolder(name: folderName, c: c);
      await _load();
    }
  }

  void onEdit() async {
    final folderName = await inputText(context, title: "Rename Folder");
    if (folderName != null) {
      await renameFolder(id: selection.first.id, name: folderName, c: c);
      await _load();
    }
  }

  void onDelete() async {
    final confirmed = await confirmDialog(context, title: "Do you want to delete these folders?", message: "Accounts assigned to these folders will be kept.");
    if (confirmed) {
      await deleteFolders(ids: selection.map((f) => f.id).toList(), c: c);
      await _load();
      ref.invalidate(getAccountsProvider);
    }
  }

  Iterable<Folder> get selection => folders.where((a) => a.$2).map((a) => a.$1);
  bool get hasSingleSelection => selection.length == 1;
  bool get hasSelection => selection.isNotEmpty;
}
