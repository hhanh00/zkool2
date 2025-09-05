import 'dart:async';
import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter_mobx/flutter_mobx.dart';
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

class AccountListPage extends StatefulWidget {
  const AccountListPage({super.key});

  @override
  State<AccountListPage> createState() => AccountListPageState();
}

class AccountListPageState extends State<AccountListPage> with RouteAware {
  var includeHidden = false;
  final listKey = GlobalKey<EditableListState<Account>>();
  double? price;

  @override
  void initState() {
    super.initState();
    if (!appStore.disclaimerAccepted) {
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
    selectAccount(null);
    super.didPopNext();
  }

  void refreshHeight(bool fetchPrice) async {
    if (appStore.checkOffline()) return;
    try {
      final height = await getCurrentHeight();
      appStore.currentHeight = height;
      if (fetchPrice) {
        final p = await getCoingeckoPrice();
        setState(() => price = p);
      }
    } on AnyhowException catch (e) {
      if (mounted) await showException(context, e.message);
    }
  }

  List<Account> get accounts => appStore.accounts.where((a) => !a.internal && (includeHidden || !a.hidden) && a.folder.id == (appStore.selectedFolder?.id ?? 0)).toList();

  void tutorial() async {
    if (!appStore.disclaimerAccepted) return;
    tutorialHelper(context, "tutMain0", [newAccountId, settingsID, syncID, heightID]);
    if (accounts.isNotEmpty) tutorialHelper(context, "tutMain1", [accountListID, avatarID]);
  }

  @override
  Widget build(BuildContext context) {
    Future(tutorial);

    return Observer(builder: (context) {
      final tt = Theme.of(context).textTheme;
      final t = tt.bodyMedium!.copyWith(fontFamily: "monospace");
      appStore.accounts;
      appStore.selectedFolder;

      return Showcase(
          key: accountListID,
          description: "List of Accounts. Tap on a row to select. Long tap then drag and drop to reorder",
          child: EditableList<Account>(
              key: listKey,
              items: accounts,
              headerBuilder: (context) => [
                    Showcase(
                        key: heightID,
                        description: "Current Block Height. Refreshed automatically every 15 seconds. Tap to update manually",
                        child: Observer(
                            builder: (context) => ElevatedButton(
                                onPressed: () => Future(() => refreshHeight(true)),
                                onLongPress: () => Future(() => refreshHeight(false)),
                                child: Text("Height: ${appStore.currentHeight}")))),
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
                        trailing: Observer(builder: (context) {
                          final h = appStore.heights[account.id];
                          return h!.build(context);
                        }),
                      ),
                      onTap: () => onOpen(context, account),
                    ));
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
                  await appStore.loadAccounts();
                }
              },
              isEqual: (a, b) => a.id == b.id,
              onReorder: onReorder,
              buttons: [
                Showcase(key: settingsID, description: "Open Settings", child: IconButton(onPressed: onSettings, icon: Icon(Icons.settings))),
                Showcase(
                    key: syncID,
                    description: "Synchronize all enabled accounts or the accounts currently selected",
                    child: IconButton(onPressed: onSync, icon: Icon(Icons.sync))),
                PopupMenuButton<String>(
                  onSelected: (String result) {
                    switch (result) {
                      case "mempool": onMempool();
                      case "hide": onHide();
                      case "folder": onFolder();
                    }
                  },
                  itemBuilder: (BuildContext context) => <PopupMenuEntry<String>>[
                    const PopupMenuItem<String>(
                      value: "mempool",
                      child: Text("Mempool")
                    ),
                    const PopupMenuItem<String>(
                      value: "folder",
                      child: Text("Folders")
                    ),
                    PopupMenuItem<String>(
                      value: 'hide',
                      child: Text("Show All"),
                    ),
                  ],
                ),
              ]));
    });
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

  onSync() async {
    try {
      final listState = listKey.currentState!;
      List<int> accountIds = [];
      final hasSelection = listState.selected.any((s) => s);
      if (hasSelection) {
        // if any selection, use the selection, otherwise use the enabled flag
        for (var i = 0; i < listState.selected.length; i++) {
          if (listState.selected[i]) accountIds.add(accounts[i].id);
        }
      } else {
        // no selection, use the enabled flag
        for (var a in appStore.accounts) {
          if (a.enabled) accountIds.add(a.id);
        }
      }
      await appStore.startSynchronize(accountIds, int.parse(appStore.actionsPerSync));
    } on AnyhowException catch (e) {
      if (mounted) await showException(context, e.message);
    }
  }

  void onOpen(BuildContext context, Account account) async {
    await selectAccount(account);
    if (!context.mounted) return;
    await GoRouter.of(context).push('/account', extra: account);
  }

  void onReorder(int oldIndex, int newIndex) async {
    await reorderAccount(oldPosition: accounts[oldIndex].position, newPosition: accounts[newIndex].position);
    await appStore.loadAccounts();
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
