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
import 'package:zkool/widgets/error_display.dart';
import 'package:zkool/widgets/editable_list.dart';
import 'package:zkool/widgets/theme.dart';

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
    WidgetsBinding.instance.addPostFrameCallback((_) {
      coinContext.setAccount(account: 0);
    });
    super.didPopNext();
  }

  void refreshHeight(bool fetchPrice) async {
    final settings = ref.read(appSettingsProvider).requireValue;
    if (settings.offline) return;
    try {
      final height = await getCurrentHeight(c: coinContext.coin);
      final currentHeight = ref.read(currentHeightProvider.notifier);
      currentHeight.setHeight(height);
      if (fetchPrice) {
        final currentPrice = ref.read(priceProvider.notifier);
        await currentPrice.fetch(settings.coingecko);
      }
    } on AnyhowException catch (e) {
      if (mounted) await showException(context, e.message);
    }
  }

  void tutorial() async {
    tutorialHelper(context, "tutMain0", [newAccountId, settingsID, syncID, heightID]);

    final accounts = await ref.read(getAccountsProvider.future);
    if (!mounted) return;
    if (accounts.isNotEmpty) tutorialHelper(context, "tutMain1", [accountListID, avatarID]);
  }

  @override
  Widget build(BuildContext context) {
    final tt = Theme.of(context).textTheme;

    Future(tutorial);

    final pinlock = ref.watch(lifecycleProvider);
    if (pinlock.value ?? false) return PinLock();

    final pageDataAV = ref.watch(accountsPageDataProvider);

    return pageDataAV.when(
      loading: () => blank(context),
      error: (error, stack) => showError(error),
      data: (pageData) {
        final accountList =
            pageData.accounts.where((a) => !a.internal && (includeHidden || !a.hidden) && a.folder.id == (pageData.selectedFolder?.id ?? 0)).toList();

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
              if (pageData.price != null) ElevatedButton(onPressed: !Platform.isLinux ? onPrice : null, child: Text("Price: ${pageData.price} USD")),
              const Gap(8),
              if (pageData.settings.offline) ...[
                Text("Wallet is in offline mode", style: tt.labelSmall),
                const Gap(8),
              ],
            ],
            builder: (context, index, account, {selected, onSelectChanged}) {
              final avatar = account.avatar(selected: selected ?? false, onTap: onSelectChanged);
              final fiat = pageData.price?.let((p) {
                final f = account.balance.toDouble() * p / zatsPerZec.toDouble();
                return fiatFormatter.format(f);
              });
              return Material(
                key: ValueKey(account.id),
                child: GestureDetector(
                  child: AccountCard(
                    leading: account.id == 1 ? Showcase(key: avatarID, description: "Tap to select for edit/delete", child: avatar) : avatar,
                    name: account.name,
                    balance: zatToText(account.balance, selectable: false, style: tt.titleLarge!.copyWith(fontWeight: FontWeight.w700)),
                    fiat: fiat != null ? Text("\$$fiat", style: tt.titleSmall!.copyWith(color: Colors.green)) : null,
                    height: SmallProgressWidget(account, style: tt.labelSmall),
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
                  await deleteAccount(account: a.id, c: coinContext.coin);
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
      },
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
      await synchronizer.startSynchronize(accountToSync);
    } on AnyhowException catch (e) {
      if (mounted) await showException(context, e.message);
    }
  }

  void onOpen(BuildContext context, Account account) async {
    // Invalidate cache to ensure fresh data
    ref.invalidate(getCurrentAccountProvider);
    // Update both the coin context and selected account ID
    await coinContext.setAccount(account: account.id);
    ref.read(selectedAccountIdProvider.notifier).set(account.id);
    // Wait for getCurrentAccountProvider (what the page watches) to complete
    await ref.read(getCurrentAccountProvider.future);
    if (!context.mounted) return;
    await GoRouter.of(context).push('/account', extra: account);
  }

  void onReorder(int oldIndex, int newIndex) async {
    final listState = listKey.currentState!;
    await reorderAccount(
      oldPosition: listState.items[oldIndex].position,
      newPosition: listState.items[newIndex].position,
      c: coinContext.coin,
    );
    ref.invalidate(getAccountsProvider);
  }

  void onSettings() async {
    final authenticated = await onUnlock(ref);
    if (!mounted) return;
    if (authenticated) {
      await GoRouter.of(context).push('/settings');
    }
  }

  void onPrice() {
    GoRouter.of(context).push('/market');
  }
}
