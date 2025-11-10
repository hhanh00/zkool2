import 'dart:async';
import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated_io.dart';
import 'package:gap/gap.dart';
import 'package:go_router/go_router.dart';
import 'package:showcaseview/showcaseview.dart';
import 'package:zkool/main.dart';
import 'package:zkool/pages/account.dart';
import 'package:zkool/router.dart';
import 'package:zkool/src/rust/api/account.dart';
import 'package:zkool/src/rust/api/network.dart';
import 'package:zkool/store.dart';
import 'package:zkool/utils.dart';
import 'package:zkool/widgets/editable_list.dart';

final heightID = GlobalKey();
final settingsID = GlobalKey();
final syncID = GlobalKey();

final accountListID = GlobalKey();
final avatarID = GlobalKey();

class AccountListPage extends ConsumerStatefulWidget {
  const AccountListPage({super.key});

  @override
  ConsumerState<AccountListPage> createState() => AccountListPageState();
}

class AccountListPageState extends ConsumerState<AccountListPage> with RouteAware {
  var includeHidden = false;
  final listKey = GlobalKey<EditableListState<Account>>();
  double? price;

  @override
  void initState() {
    super.initState();
    final settings = ref.read(appSettingsProvider);
    if (!settings.disclaimerAccepted) {
      WidgetsBinding.instance.addPostFrameCallback((_) {
        logger.i("Disclaimer not accepted");
        GoRouter.of(context).push("/disclaimer");
      });
    }
  }

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    routeObserver.subscribe(this, ModalRoute.of(context)!);
  }

  @override
  void dispose() {
    routeObserver.unsubscribe(this);
    super.dispose();
  }

  @override
  void didPopNext() {
    final selectedAccount = ref.read(selectedAccountProvider.notifier);
    selectedAccount.unselect();
    super.didPopNext();
  }

  void refreshHeight(bool fetchPrice) async {
    final settings = ref.read(appSettingsProvider);
    if (settings.offline) return;
    try {
      final height = await getCurrentHeight();
      final currentHeight = ref.read(currentHeightProvider.notifier);
      currentHeight.setHeight(height);
      if (fetchPrice) {
        final p = await getCoingeckoPrice();
        final currentPrice = ref.read(priceProvider.notifier);
        currentPrice.setPrice(p);
        setState(() {
          price = p;
        });
      }
    } on AnyhowException catch (e) {
      if (mounted) await showException(context, e.message);
    }
  }

  void tutorial() async {
    final settings = ref.read(appSettingsProvider);
    if (!settings.disclaimerAccepted) return;
    tutorialHelper(context, "tutMain0", [newAccountId, settingsID, syncID, heightID]);

    final accounts = await ref.read(getAccountsProvider.future);
    if (!mounted) return;
    if (accounts.isNotEmpty) tutorialHelper(context, "tutMain1", [accountListID, avatarID]);
  }

  @override
  Widget build(BuildContext context) {
    Future(tutorial);

    final tt = Theme.of(context).textTheme;
    final t = tt.bodyMedium!.copyWith(fontFamily: "monospace");

    final accounts = ref.read(getAccountsProvider);
    final selectedFolder = ref.read(selectedFolderProvider);

    final as = accounts
        .whenData((accounts) => accounts.where((a) => !a.internal && (includeHidden || !a.hidden) && a.folder.id == (selectedFolder?.id ?? 0)).toList());

    switch (as) {
      case AsyncLoading():
        return LinearProgressIndicator();
      case AsyncError():
        return Text(as.error.toString());
      case AsyncData():
    }

    final accountList = as.requireValue;
    final currentHeight = ref.watch(currentHeightProvider);
    final h = currentHeight != null ? currentHeight.toString() : 'N/A';

    return Showcase(
      key: accountListID,
      description: "List of Accounts. Tap on a row to select. Long tap then drag and drop to reorder",
      child: EditableList<Account>(
        key: listKey,
        items: accountList,
        headerBuilder: (context) => [
          Showcase(
            key: heightID,
            description: "Current Block Height. Refreshed automatically every 15 seconds. Tap to update manually",
            child: ElevatedButton(
              onPressed: () => Future(() => refreshHeight(true)),
              onLongPress: () => Future(() => refreshHeight(false)),
              child: Text("Height: $h"),
            ),
          ),
          const Gap(8),
          if (price != null) ElevatedButton(onPressed: !Platform.isLinux ? onPrice : null, child: Text("Price: $price USD")),
          const Gap(8),
        ],
        builder: (context, index, account, {selected, onSelectChanged}) {
          final avatar = account.avatar(selected: selected ?? false, onTap: onSelectChanged);
          return Material(
            key: ValueKey(account.id),
            child: GestureDetector(
              child: ListTile(
                leading: account.id == 1 ? Showcase(key: avatarID, description: "Tap to select for edit/delete", child: avatar) : avatar,
                title: Text(account.name, style: !account.enabled ? TextStyle(color: Colors.grey) : null),
                subtitle: zatToText(account.balance, selectable: false, style: t.copyWith(fontWeight: FontWeight.w700)),
                trailing: SmallProgressWidget(account),
              ),
              onTap: () => onOpen(context, account),
            ),
          );
        },
        title: "Account List",
        createBuilder: (context) => GoRouter.of(context).push("/account/new"),
        editBuilder: (context, a) => GoRouter.of(context).push("/account/edit", extra: a),
        deleteBuilder: (context, accounts) async {
          final confirmed = await confirmDialog(context, title: "Delete Account(s)", message: "Are you sure you want to delete these accounts?");
          if (confirmed) {
            for (var a in accounts) {
              await deleteAccount(account: a.id);
            }
            ref.invalidate(getAccountsProvider);
          }
        },
        isEqual: (a, b) => a.id == b.id,
        onReorder: onReorder,
        buttons: [
          Showcase(key: settingsID, description: "Open Settings", child: IconButton(onPressed: onSettings, icon: Icon(Icons.settings))),
          Showcase(
            key: syncID,
            description: "Synchronize all enabled accounts or the accounts currently selected",
            child: IconButton(onPressed: onSync, icon: Icon(Icons.sync)),
          ),
          PopupMenuButton<String>(
            onSelected: (String result) {
              switch (result) {
                case "mempool":
                  onMempool();
                case "hide":
                  onHide();
                case "category":
                  onCategory();
                case "folder":
                  onFolder();
              }
            },
            itemBuilder: (BuildContext context) => <PopupMenuEntry<String>>[
              const PopupMenuItem<String>(
                value: "mempool",
                child: Text("Mempool"),
              ),
              const PopupMenuItem<String>(
                value: "folder",
                child: Text("Folders"),
              ),
              const PopupMenuItem<String>(
                value: "category",
                child: Text("Categories"),
              ),
              PopupMenuItem<String>(
                value: 'hide',
                child: Text("Show All"),
              ),
            ],
          ),
        ],
      ),
    );
  }

  onMempool() => GoRouter.of(context).push('/mempool');

  onHide() async {
    final authenticated = await authenticate(reason: "Show/Hide Hidden Accounts");
    if (!authenticated) return;
    setState(() {
      includeHidden = !includeHidden;
    });
  }

  onFolder() async {
    await GoRouter.of(context).push("/folders");
  }

  onCategory() async {
    await GoRouter.of(context).push("/categories");
  }

  onSync() async {
    try {
      final listState = listKey.currentState!;
      List<Account> accountToSync = [];
      final hasSelection = listState.selected.any((s) => s);
      if (hasSelection) {
        // if any selection, use the selection, otherwise use the enabled flag
        for (var i = 0; i < listState.selected.length; i++) {
          if (listState.selected[i]) accountToSync.add(listState.items[i]);
        }
      } else {
        // no selection, use the enabled flag
        final accounts = await ref.read(getAccountsProvider.future);
        for (var a in accounts) {
          if (a.enabled) accountToSync.add(a);
        }
      }
      final synchronizer = ref.read(synchronizerProvider.notifier);
      await synchronizer.startSynchronize(ref, accountToSync);
    } on AnyhowException catch (e) {
      if (mounted) await showException(context, e.message);
    }
  }

  void onOpen(BuildContext context, Account account) async {
    final selectAccount = ref.read(selectedAccountProvider.notifier);
    selectAccount.selectAccount(account);
    if (!context.mounted) return;
    await GoRouter.of(context).push('/account', extra: account);
  }

  void onReorder(int oldIndex, int newIndex) async {
    final listState = listKey.currentState!;
    await reorderAccount(oldPosition: listState.items[oldIndex].position, newPosition: listState.items[newIndex].position);
    ref.invalidate(getAccountsProvider);
  }

  void onSettings() async {
    final authenticated = await onUnlock();
    if (!mounted) return;
    if (authenticated) {
      await GoRouter.of(context).push('/settings');
    }
  }

  void onPrice() {
    GoRouter.of(context).push('/market');
  }
}
