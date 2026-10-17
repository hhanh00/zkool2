import 'dart:async';
import 'dart:convert';
import 'dart:io';

import 'package:bubble/bubble.dart';
import 'package:convert/convert.dart';
import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_form_builder/flutter_form_builder.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated.dart';
import 'package:gap/gap.dart';
import 'package:go_router/go_router.dart';
import 'package:qr_flutter/qr_flutter.dart';
import 'package:easy_debounce/easy_debounce.dart';
import 'package:searchable_listview/searchable_listview.dart';

import 'package:zkool/main.dart';
import 'package:zkool/pages/tx.dart';
import 'package:zkool/router.dart';
import 'package:zkool/src/rust/api/account.dart';
import 'package:zkool/src/rust/api/frost.dart';
import 'package:zkool/src/rust/api/pay.dart';
import 'package:zkool/src/rust/api/sync.dart';
import 'package:zkool/src/rust/api/transaction.dart';
import 'package:zkool/src/rust/api/zsa.dart';
import 'package:zkool/store.dart';
import 'package:zkool/utils.dart';
import 'package:zkool/widgets/error_display.dart';
import 'package:zkool/widgets/exchange_rate.dart';
import 'package:zkool/widgets/plugin_memo_view.dart';
import 'package:zkool/widgets/pool_select.dart';
import 'package:zkool/widgets/theme.dart';
import 'package:zkool/widgets/transaction_table.dart';

/// Upgrade a legacy FROST group to ZIP 2005: create the sibling account
/// (or find the existing one), then, when the legacy account still holds
/// funds, plan the sweep into it and open the transaction page, where the
/// PCZT is shared with the other members and the signing session starts.
/// Every member must upgrade on their own device; all arrive at the same
/// address.
Future<void> upgradeFrostGroup(BuildContext context, WidgetRef ref, int account) async {
  final c = coinContext.coin;
  final confirmed = await confirmDialog(
    context,
    title: "Upgrade to ZIP 2005",
    message: "This creates the group's ZIP 2005 account: same shares, a new quantum-recoverable address. "
        "Every member must upgrade and will get the same address. "
        "The funds then have to be swept into it as a multisignature transaction.",
  );
  if (!confirmed) return;
  try {
    await upgradeFrostAccount(account: account, c: c);
    ref.invalidate(getAccountsProvider);
    await ref.read(getAccountsProvider.future);
    ref.invalidate(accountProvider(account));
    PcztPackage pczt;
    try {
      pczt = await planFrostSweep(account: account, c: c);
    } on AnyhowException catch (e) {
      if (e.message.contains("nothing to sweep")) {
        showSnackbar("ZIP 2005 account created; no funds to sweep");
        return;
      }
      rethrow;
    }
    if (!context.mounted) return;
    await GoRouter.of(context).push(accountRoute("/tx", account), extra: pczt);
  } on AnyhowException catch (e) {
    if (context.mounted) await showException(context, e.message);
  }
}

class AccountViewPage extends ConsumerStatefulWidget {
  final int account;
  const AccountViewPage({super.key, required this.account});

  @override
  ConsumerState<AccountViewPage> createState() => AccountViewPageState();
}

class AccountViewPageState extends ConsumerState<AccountViewPage> with SingleTickerProviderStateMixin {
  late final tabController = TabController(length: 4, vsync: this);

  final List<String> tabNames = ["Transactions", "Memos", "Notes", "ZSA Holdings"];

  late final c = coinContext.coin;

  // ZSA Holdings inline editing state
  int? _editingIndex;
  late final TextEditingController _nameController = TextEditingController();
  late final FocusNode _focusNode = FocusNode();
  bool _zsaAvailable = false;

  // Tx search
  final _txSearchController = TextEditingController();
  String _txSearchQuery = '';
  bool _showDustNotes = true;
  bool _groupByPool = false;

  @override
  void initState() {
    super.initState();
    _rememberAccount();
    _focusNode.addListener(_onFocusChange);
    _txSearchController.addListener(_onTxSearchChanged);
    isZsaAvailable(c: c).then((v) {
      if (mounted) setState(() => _zsaAvailable = v);
    });
  }

  @override
  void dispose() {
    _focusNode.removeListener(_onFocusChange);
    _nameController.dispose();
    _focusNode.dispose();
    _txSearchController.removeListener(_onTxSearchChanged);
    _txSearchController.dispose();
    super.dispose();
  }

  void _onFocusChange() {
    if (!_focusNode.hasFocus && _editingIndex != null) {
      _commitEditing();
    }
  }

  void _onTxSearchChanged() {
    EasyDebounce.debounce('tx_search', const Duration(milliseconds: 300), () {
      setState(() => _txSearchQuery = _txSearchController.text);
    });
  }

  List<Tx> _filterTx(List<Tx> txs) {
    final q = _txSearchQuery.toLowerCase();
    if (q.isEmpty) return txs;
    return txs.where((tx) {
      if (tx.memo?.toLowerCase().contains(q) == true) return true;
      if (tx.contactName?.toLowerCase().contains(q) == true) return true;
      if (txIdToString(tx.txid).toLowerCase().contains(q)) return true;
      return false;
    }).toList();
  }

  void _startEditing(int index, ZsaHolding holding) {
    if (_editingIndex != null && _editingIndex != index) {
      _commitEditing(); // save then switch
    }
    _editingIndex = index;
    if (holding.assetName.isNotEmpty) {
      _nameController.text = holding.assetName;
    } else {
      _nameController.clear();
    }
    _nameController.selection = TextSelection.fromPosition(
      TextPosition(offset: _nameController.text.length),
    );
    _focusNode.requestFocus();
    setState(() {});
  }

  @override
  void didUpdateWidget(AccountViewPage oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.account != widget.account) _rememberAccount();
  }

  /// Record the account as the one to reopen at the next launch. Deferred
  /// past the current frame: this runs from initState, and a provider must
  /// not change while the tree is building (riverpod asserts in debug).
  void _rememberAccount() {
    final account = widget.account;
    Future(() {
      if (!mounted) return;
      unawaited(ref.read(selectedAccountIdProvider.notifier).set(account));
    });
  }

  Future<void> _commitEditing() async {
    final index = _editingIndex;
    if (index == null) return;

    final fullData = ref.read(fullAccountPageDataProvider(widget.account)).value;
    final account = fullData?.currentAccount;
    if (account == null) return;

    final h = account.zsas[index];
    final newName = _nameController.text;

    // Close editor immediately (optimistic)
    _editingIndex = null;
    if (mounted) setState(() {});

    // Noop if unchanged
    if (newName == h.assetName) return;

    try {
      await setAssetName(
        idAsset: h.idAsset,
        name: newName,
        c: c,
      );
      ref.invalidate(accountProvider);
    } on AnyhowException catch (e) {
      if (mounted) {
        await showException(context, e.message);
      }
    }
  }

  @override
  Widget build(BuildContext context) {
    final pinlock = ref.watch(lifecycleProvider);
    if (pinlock.value ?? false) return PinLock();

    final accountsAV = ref.watch(getAccountsProvider);
    ref.watch(appSettingsProvider); // rebuild when settings change (e.g. list/table toggle)
    return accountsAV.when(
      skipLoadingOnReload: true,
      loading: () => blank(context),
      error: (error, stack) => showError(error),
      data: (accounts) {
        if (!accounts.any((a) => a.id == widget.account)) {
          // The account is gone (deleted): fall back to the account list.
          WidgetsBinding.instance.addPostFrameCallback((_) {
            if (GoRouter.of(context).state.matchedLocation == '/accounts') return;
            GoRouter.of(context).go("/accounts");
          });
          return blank(context);
        }

        final fullDataAV = ref.watch(fullAccountPageDataProvider(widget.account));

        return fullDataAV.when(
          skipLoadingOnReload: true,
          loading: () => blank(context),
          error: (error, stack) => showError(error),
          data: (fullData) => _buildAccountScaffold(context, fullData),
        );
      },
    );
  }

  /// What needs the user's attention before the funds are safe: a legacy
  /// FROST group holding funds (upgrade, then sweep), else Orchard-pool
  /// notes once Ironwood is active (migrate). The legacy case comes first
  /// because its sweep lands everything in Ironwood anyway.
  List<Widget> _attentionBanners(BuildContext context, AccountData account, List<BigInt> b, bool ironwoodActive) {
    final fp = account.frostParams;
    final t = Theme.of(context);
    Widget banner(String text, String action, VoidCallback onPressed) => Card(
          color: t.colorScheme.errorContainer,
          margin: const EdgeInsets.fromLTRB(0, 8, 0, 8),
          child: Padding(
            padding: const EdgeInsets.fromLTRB(16, 12, 16, 12),
            child: Row(
              children: [
                Icon(Icons.warning_amber, color: t.colorScheme.onErrorContainer),
                Gap(12),
                Expanded(child: Text(text, style: t.textTheme.bodyMedium?.copyWith(color: t.colorScheme.onErrorContainer))),
                Gap(12),
                FilledButton(onPressed: onPressed, child: Text(action)),
              ],
            ),
          ),
        );
    if (fp != null && fp.dkgVersion < 2 && !fp.quantumRecoverable) {
      final text = fp.sibling == null
          ? "This group's funds are not quantum recoverable. Upgrade creates the group's ZIP 2005 account with a new address, then sweeps the funds into it."
          : "The funds are still at the legacy address and are not quantum recoverable. Sweep them into the group's ZIP 2005 account.";
      return [banner(text, "Upgrade", () => upgradeFrostGroup(context, ref, widget.account))];
    }
    if (b[2] > BigInt.zero && ironwoodActive) {
      return [
        banner(
          "Some notes are in the Orchard pool and are not quantum recoverable. Migrate them to Ironwood.",
          "Migrate to Ironwood",
          () => GoRouter.of(context).push(accountRoute("/migrate", widget.account)),
        )
      ];
    }
    return [];
  }

  Widget _buildAccountScaffold(BuildContext context, FullAccountPageData fullData) {
    final account = fullData.currentAccount!;
    final b = account.balance.field0;
    final settings = ref.read(appSettingsProvider).requireValue;
    final currency = settings.currency;
    final hasOrchardBalance = b[2] > BigInt.zero || b[3] > BigInt.zero;
    final ironwoodActive = ref.watch(ironwoodActiveProvider).value ?? false;

    final fiat = fullData.price?.let((p) {
      final f = (b[0] + b[1] + b[2] + b[3]).toDouble() * p / zatsPerZec.toDouble();
      return formatFiat(f, currency);
    });

    final t = Theme.of(context);
    final tt = t.textTheme;

    return Scaffold(
      appBar: AppBar(
        title: Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            Flexible(child: Text(account.account.name, overflow: TextOverflow.ellipsis)),
            if (account.frostParams != null) ...[
              Gap(8),
              Tooltip(
                message: "FROST group, DKG version ${account.frostParams!.dkgVersion}",
                child: Icon(Icons.ac_unit, size: 20),
              ),
            ],
          ],
        ),
        actions: [
          Tooltip(
            message: "Synchronize only this account",
            child: IconButton(
              tooltip: "Sync this account",
              onPressed: () => onSync(account),
              icon: Icon(Icons.sync),
            ),
          ),
          IconButton(
            tooltip: "Show the account receiving addresses",
            onPressed: onReceive,
            icon: Icon(Icons.download),
          ),
          IconButton(
            tooltip: "Send funds to one or many addresses",
            onPressed: onSend,
            icon: Icon(Icons.send),
          ),
          PopupMenuButton<String>(
            onSelected: (String result) async {
              switch (result) {
                case "account_manager":
                  await GoRouter.of(context).push("/accounts");
                case "backup":
                  await GoRouter.of(context).push("/viewing_keys", extra: account.account.id);
                case "edit_account":
                  await GoRouter.of(context).push("/account/edit", extra: [account.account]);
                case "swap":
                  await GoRouter.of(context).push(accountRoute("/swaps", widget.account));
                case "market_price":
                  await GoRouter.of(context).push("/market");
                case "update_fx":
                  onUpdateAllTxPrices();
                case "toggle_tx_view":
                  await ref.read(appSettingsProvider.notifier).setTransactionViewMode(!(ref.read(appSettingsProvider).value?.transactionTableMode ?? false));
                case "charts":
                  await GoRouter.of(context).push(accountRoute("/chart", widget.account));
                case "migration":
                  await GoRouter.of(context).push(accountRoute("/migrate", widget.account));
                case "voting":
                  await GoRouter.of(context).push(accountRoute("/voting", widget.account));
                case "settings":
                  await GoRouter.of(context).push("/settings");
                default:
                  onExport(int.parse(result));
              }
            },
            itemBuilder: (BuildContext context) => <PopupMenuEntry<String>>[
              if (settings.expertMode)
                const PopupMenuItem<String>(
                  value: "swap",
                  child: Text("Swaps"),
                ),
              const PopupMenuItem<String>(
                value: "account_manager",
                child: Text("Account Manager"),
              ),
              const PopupMenuItem<String>(
                value: "settings",
                child: Text("Settings"),
              ),
              const PopupMenuItem<String>(
                value: "edit_account",
                child: Text("Edit Account"),
              ),
              const PopupMenuItem<String>(
                value: "backup",
                child: Text("Backup"),
              ),
              const PopupMenuItem<String>(
                value: "market_price",
                child: Text("Market Price"),
              ),
              const PopupMenuItem<String>(
                value: "update_fx",
                child: Text("Fetch Tx Prices"),
              ),
              PopupMenuItem<String>(
                value: tabIndex(context).toString(),
                child: Text("Export ${tabNames[tabIndex(context)]}"),
              ),
              PopupMenuItem<String>(
                value: "toggle_tx_view",
                child: Text((ref.read(appSettingsProvider).value?.transactionTableMode ?? false) ? "View: List" : "View: Table"),
              ),
              if (!Platform.isLinux)
                const PopupMenuItem<String>(
                  value: "charts",
                  child: Text("Charts"),
                ),
              // Show migration option when Ironwood is active and there are Orchard/IW notes
              if (hasOrchardBalance && ironwoodActive)
                const PopupMenuItem<String>(
                  value: "migration",
                  child: Text("Note Migration"),
                ),
              const PopupMenuItem<String>(
                value: "voting",
                child: Text("Voting"),
              ),
            ],
          ),
        ],
        bottom: TabBar(
          controller: tabController,
          tabs: tabNames.map((n) => Tab(text: n)).toList(),
        ),
      ),
      body: Builder(
        builder: (context) {
          final ss = fullData.syncState;
          if (ss == null) return const SizedBox.shrink();

          final cancelling = ref.watch(synchronizerProvider).cancelling;
          final syncing = ss.start != ss.end || cancelling;
          final unconfirmedAmount = fullData.mempool.unconfirmedFunds[account.account.id];

          return Padding(
              padding: const EdgeInsets.fromLTRB(8, 0, 8, 8),
              child: TabBarView(
                controller: tabController,
                children: [
                  CustomScrollView(
                    slivers: [
                      PinnedHeaderSliver(
                        child: Container(
                            color: Theme.of(context).colorScheme.surface,
                            child: Column(children: [
                              ..._attentionBanners(context, account, b, ironwoodActive),
                              if (syncing) ...[
                                if (cancelling) const Text("Cancelling Synchronization"),
                                HeroProgressWidget(account.account),
                                Gap(8),
                              ],
                              DisplayPanel(
                                child: Column(children: [
                                  Tooltip(
                                    message: "Balance across all pools",
                                    child: Column(children: [
                                      zatToText(
                                        b[0] + b[1] + b[2] + b[3],
                                        selectable: true,
                                        style: tt.displaySmall!,
                                      ),
                                    ]),
                                  ),
                                  if (fiat != null) Text(fiat),
                                  const Gap(4),
                                  const ExchangeRateButton(),
                                  Gap(8),
                                  GestureDetector(
                                    behavior: HitTestBehavior.opaque,
                                    onTap: () => ref
                                        .read(appSettingsProvider.notifier)
                                        .setCollapsePoolBalances(!settings.collapsePoolBalances),
                                    child: SizedBox(
                                      width: double.infinity,
                                      child: settings.collapsePoolBalances
                                          ? Row(
                                              mainAxisAlignment: MainAxisAlignment.center,
                                              children: [
                                                Icon(Icons.expand_more, size: 18),
                                                const Gap(4),
                                                Text("Pool Balances", style: tt.bodySmall),
                                              ],
                                            )
                                          : BalanceWidget(account.balance),
                                    ),
                                  ),
                                ]),
                              ),
                              Gap(8),
                              if (unconfirmedAmount != null) ...[
                                zatToText(
                                  BigInt.from(unconfirmedAmount),
                                  prefix: "Unconfirmed: ",
                                  colored: true,
                                  selectable: true,
                                  style: tt.bodyLarge,
                                ),
                                Gap(8),
                              ],
                            ])),
                      ),
                      SliverToBoxAdapter(
                        child: Padding(
                          padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 4),
                          child: TextField(
                            controller: _txSearchController,
                            decoration: const InputDecoration(
                              labelText: "Search transactions (memo, contact, txid)",
                              fillColor: Colors.white,
                              prefixIcon: Icon(Icons.search),
                            ),
                          ),
                        ),
                      ),
                      SliverToBoxAdapter(
                        child: Padding(
                          padding: const EdgeInsets.symmetric(vertical: 4),
                          child: Center(
                            child: Text(
                              "Transaction History (${_filterTx(account.transactions).length} txs)",
                              style: Theme.of(context).textTheme.bodyLarge,
                            ),
                          ),
                        ),
                      ),
                      if (settings.transactionTableMode)
                        SliverToBoxAdapter(
                          child: TransactionTable(
                            transactions: _filterTx(account.transactions),
                            currency: currency,
                            onTap: (id) => gotoTransaction(context, widget.account, id),
                            onMemoChanged: (txId, newMemo) async {
                              await setUserMemo(
                                idTx: txId,
                                memo: newMemo,
                                account: widget.account,
                                c: c,
                              );
                              ref.invalidate(accountProvider);
                            },
                          ),
                        )
                      else
                        SliverToBoxAdapter(
                          child: SizedBox(
                            height: MediaQuery.of(context).size.height * 0.55,
                            child: showTxHistory(
                              context,
                              widget.account,
                              _filterTx(account.transactions),
                            ),
                          ),
                        ),
                    ],
                  ),
                  showMemos(context, account.memos, widget.account, () {
                    ref.invalidate(accountProvider(widget.account));
                  }),
                  showNotes(ref, widget.account, account.notes, _showDustNotes, () => setState(() => _showDustNotes = !_showDustNotes), _groupByPool,
                      () => setState(() => _groupByPool = !_groupByPool)),
                  _showZsaHoldings(context, account.zsas),
                ],
              ));
        },
      ),
    );
  }

  void onSync(AccountData account) async {
    try {
      final synchronizer = ref.read(synchronizerProvider.notifier);
      await synchronizer.startSynchronize([account.account]);
    } on AnyhowException catch (e) {
      if (mounted) await showException(context, e.message);
    }
  }

  void onReceive() async {
    await GoRouter.of(context).push(accountRoute("/receive", widget.account));
  }

  void onSend() async {
    await GoRouter.of(context).push(accountRoute("/send", widget.account));
  }

  void onUpdateAllTxPrices() async {
    final settings = await ref.read(appSettingsProvider.future);
    if (!mounted) return;
    final confirmed =
        await confirmDialog(context, title: "Fetch Tx Market Price", message: "Do you want to retrieve historical ZEC prices for your past transactions?");
    if (confirmed) {
      if (!mounted) return;
      unawaited(showDialog<void>(
        context: context,
        barrierDismissible: false,
        builder: (_) => const Center(child: CircularProgressIndicator()),
      ));
      try {
        final n = await fillMissingTxPrices(api: settings.coingecko, currency: settings.currency, account: widget.account, c: c);
        if (mounted) {
          GoRouter.of(context).pop();
          if (n > 0) {
            showSnackbar("Updated $n transaction price${n == 1 ? '' : 's'}");
          } else {
            showSnackbar("No transactions needed price updates");
          }
        }
      } on AnyhowException catch (e) {
        if (mounted) {
          GoRouter.of(context).pop();
          await showException(context, e.message);
        }
      }
    }
  }

  void onExport(int index) async {
    final data = await getExportedData(type: index, account: widget.account, c: c);
    final filename = await saveFile(data: utf8.encode(data));
    if (!mounted) return;
    if (filename != null) await showMessage(context, "$filename Saved");
  }

  Widget _showZsaHoldings(BuildContext context, List<ZsaHolding> zsas) {
    final tt = Theme.of(context).textTheme;

    return CustomScrollView(
      slivers: [
        SliverToBoxAdapter(
          child: OverflowBar(
            children: [
              IconButton(
                tooltip: _zsaAvailable ? "Issue new ZSA token" : "ZSA not available on this network",
                onPressed: _zsaAvailable ? () => GoRouter.of(context).push(accountRoute("/zsa/issue", widget.account)) : null,
                icon: const Icon(Icons.add),
              ),
            ],
          ),
        ),
        if (zsas.isEmpty)
          SliverFillRemaining(
            child: Center(
              child: Text("Any ZSA tokens you receive will appear here.", style: tt.bodyMedium),
            ),
          )
        else
          SliverFixedExtentList.builder(
            itemCount: zsas.length,
            itemExtent: 64,
            itemBuilder: (context, index) {
              final h = zsas[index];

              final displayName = h.assetName.isNotEmpty ? h.assetName : hex.encode(h.assetDescHash.sublist(0, 4));

              final isEditing = _editingIndex == index;

              return Column(
                children: [
                  Expanded(
                    child: ListTile(
                      leading: CircleAvatar(
                        backgroundColor: Colors.blue,
                        child: Text(
                          initials(displayName),
                          style: tt.titleMedium?.copyWith(color: Colors.white),
                        ),
                      ),
                      title: isEditing
                          ? TextField(
                              controller: _nameController,
                              focusNode: _focusNode,
                              textInputAction: TextInputAction.done,
                              onEditingComplete: _commitEditing,
                              decoration: InputDecoration(
                                isDense: true,
                                contentPadding: const EdgeInsets.symmetric(vertical: 4),
                                border: const OutlineInputBorder(),
                                hintText: h.assetName.isEmpty ? hex.encode(h.assetDescHash.sublist(0, 4)) : null,
                              ),
                              style: tt.titleMedium,
                            )
                          : GestureDetector(
                              onLongPress: () => _startEditing(index, h),
                              child: Text(displayName),
                            ),
                      subtitle: Text(hex.encode(h.assetDescHash.sublist(0, 4))),
                      trailing: Text(h.balance.toString(), style: tt.titleMedium),
                    ),
                  ),
                  const Divider(height: 1, thickness: 1, indent: 16, endIndent: 16),
                ],
              );
            },
          ),
      ],
    );
  }

  int tabIndex(BuildContext context) => tabController.index;
}

class AccountEditPage extends ConsumerStatefulWidget {
  final List<Account> accounts;
  const AccountEditPage(this.accounts, {super.key});

  @override
  ConsumerState<AccountEditPage> createState() => AccountEditPageState();
}

class AccountEditPageState extends ConsumerState<AccountEditPage> with RouteAware {
  late final c = coinContext.coin;
  late final List<Account> _savedAccounts = List.of(widget.accounts);
  late final List<Account> accounts = _savedAccounts;
  FrostParams? frostParams;
  late final Map<String, dynamic> _initialValues = {
    "name": accounts.length == 1 ? accounts.first.name : "(Multiple)",
    "birth": accounts.length == 1 ? accounts.first.birth : null,
    "icon": accounts.length == 1 ? accounts.first.icon : null,
    "enabled": _commonValue((a) => a.enabled),
    "hidden": _commonValue((a) => a.hidden),
    "useInternal": _commonValue((a) => a.useInternal),
    "folder": _commonValue((a) => a.folder.id),
  };

  T? _commonValue<T>(T Function(Account) value) {
    final first = value(accounts.first);
    return accounts.every((a) => value(a) == first) ? first : null;
  }
  Future<bool>? _pendingExit;
  bool _allowExit = false;
  late final _birthController = TextEditingController(
    text: _savedAccounts.length == 1 ? _savedAccounts.first.birth.toString() : "",
  );
  final formKey = GlobalKey<FormBuilderState>(debugLabel: "formKey");
  List<Folder>? folders;

  @override
  void didPop() {
    super.didPop();
    ref.invalidate(getAccountsProvider);
  }

  @override
  void initState() {
    super.initState();
    Future(() async {
      final folders = await ref.read(getFoldersProvider.future);
      if (mounted) setState(() => this.folders = folders);
      if (accounts.length == 1) {
        final fp = await getAccountFrostParams(account: accounts.first.id, c: c);
        if (mounted) setState(() => frostParams = fp);
      }
    });
  }

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    final route = ModalRoute.of(context);
    if (route is PageRoute) {
      routeObserver.subscribe(this, route);
    }
  }

  @override
  void dispose() {
    routeObserver.unsubscribe(this);
    _birthController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final pinlock = ref.watch(lifecycleProvider);
    if (pinlock.value ?? false) return PinLock();

    if (folders == null) return SizedBox.expand();

    final account = accounts.length == 1 ? accounts.first : null;
    final folderOptions = [DropdownMenuItem(value: 0, child: Text("No Folder"))] +
        folders!
            .map(
              (f) => DropdownMenuItem(value: f.id, child: Text(f.name)),
            )
            .toList();

    return Scaffold(
      appBar: AppBar(
        title: Text('Account Edit'),
        actions: [
          if (account != null) ...[
            if (frostParams != null && frostParams!.dkgVersion < 2)
              IconButton(
                tooltip: "Upgrade the FROST group to ZIP 2005",
                onPressed: () => upgradeFrostGroup(context, ref, account.id),
                icon: Icon(Icons.upgrade),
              ),
            IconButton(
              tooltip: "Export an encrypted file of this account",
              onPressed: onExport,
              icon: Icon(Icons.save),
            ),
            IconButton(
              tooltip: "Rewind back a few blocks",
              onPressed: onRewind,
              icon: Icon(Icons.fast_rewind),
            ),
          ],
          IconButton(
            tooltip: "Clear and reset account to birth height",
            onPressed: onReset,
            icon: Icon(Icons.delete_sweep),
          ),
        ],
      ),
      body: Padding(
        padding: const EdgeInsets.symmetric(horizontal: 16),
        child: FormBuilder(
          key: formKey,
          initialValue: _initialValues,
          child: Column(
            children: [
              Row(
                children: [
                  Expanded(
                    child: Tooltip(
                      message: "Edit Name of the account",
                      child: FormBuilderTextField(
                        name: 'name',
                        decoration: InputDecoration(labelText: 'Name'),
                        readOnly: account == null,
                      ),
                    ),
                  ),
                  if (account != null)
                    FormBuilderField<Uint8List>(
                      name: "icon",
                      builder: (field) => Tooltip(
                        message: "Edit Account Icon",
                        child: account.copyWith(icon: field.value).avatar(onTap: (_) => onEditIcon()),
                      ),
                    ),
                ],
              ),
              Tooltip(
                message: "Edit Height at the creation of the account",
                child: FormBuilderField<int>(
                  name: 'birth',
                  validator: account == null ? null : (height) =>
                      height == null || height < 0 || height > 0xffffffff
                          ? "Enter a valid birth height"
                          : null,
                  builder: (field) => TextField(
                    controller: _birthController,
                    decoration: InputDecoration(labelText: 'Birth Height', errorText: field.errorText),
                    keyboardType: TextInputType.number,
                    readOnly: account == null,
                    inputFormatters: [FilteringTextInputFormatter.digitsOnly],
                    onChanged: account == null ? null : (value) => field.didChange(int.tryParse(value)),
                  ),
                ),
              ),
              Tooltip(
                message: "Enable or disable. Only enabled accounts participate in the global sync",
                child: FormBuilderCheckbox(
                  name: "enabled",
                  title: Text("Enabled"),
                  tristate: account == null,
                ),
              ),
              Tooltip(
                message: "Hide this account from the account list",
                child: FormBuilderCheckbox(
                  name: "hidden",
                  title: Text("Hidden"),
                  tristate: account == null,
                ),
              ),
              Tooltip(
                message: "Use an internal address for change (ZIP 316). Ledger accounts have a fixed change setting",
                child: FormBuilderCheckbox(
                  name: "useInternal",
                  title: Text("Use Internal Change"),
                  tristate: account == null,
                  enabled: accounts.every((a) => a.hw != 1 && a.hw != 2),
                ),
              ),
              Tooltip(
                message: "Assign Account to Folder",
                child: FormBuilderDropdown<int>(
                  name: "folder",
                  items: folderOptions,
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }

  Future<void> _updateVaultAccount(Account account) async {
    final settings = await ref.read(appSettingsProvider.future);
    if (!settings.vault) return;
    final seed = await getAccountSeed(account: account.id, c: c);
    if (seed == null) return;
    await ref.read(vaultProvider.notifier).storeAccount(
          name: account.name,
          seed: seed.mnemonic,
          aindex: account.aindex,
          useInternal: account.useInternal,
          birthHeight: account.birth,
        );
  }

  Set<String> get _changedFields {
    final fields = formKey.currentState?.fields;
    if (fields == null) return {};
    return fields.entries.where((entry) {
      final value = entry.value.value;
      final initial = _initialValues[entry.key];
      return value is Uint8List || initial is Uint8List
          ? !listEquals(value as Uint8List?, initial as Uint8List?)
          : value != initial;
    }).map((entry) => entry.key).toSet();
  }

  Future<bool> confirmLeave() {
    return _pendingExit ??= _confirmLeave().whenComplete(() => _pendingExit = null);
  }

  Future<bool> _confirmLeave() async {
    final changed = _changedFields;
    if (_allowExit || changed.isEmpty) return true;
    final resetsSync = changed.contains("birth") || changed.contains("useInternal");
    final save = await showDialog<bool>(
      context: context,
      builder: (dialogContext) => AlertDialog(
        title: Text("Unsaved account changes"),
        content: Text(resetsSync
            ? "Save your changes before leaving? Changing birth height or internal change will clear the affected accounts' sync data and reset them to their birth height. You will need to resync. Your funds will not be lost."
            : "Save your account changes before leaving?"),
        actions: [
          TextButton(onPressed: () => Navigator.of(dialogContext).pop(), child: Text("Stay")),
          TextButton(onPressed: () => Navigator.of(dialogContext).pop(false), child: Text("Leave without saving")),
          FilledButton(onPressed: () => Navigator.of(dialogContext).pop(true), child: Text("Save and leave")),
        ],
      ),
    );
    if (save == null || !mounted) return false;
    if (!save) {
      _allowExit = true;
      return true;
    }
    if (!(formKey.currentState?.saveAndValidate() ?? false)) return false;
    final values = formKey.currentState!.value;
    try {
      for (var i = 0; i < accounts.length; i++) {
        final saved = accounts[i];
        final folderId = values["folder"] as int?;
        final a = saved.copyWith(
          name: accounts.length == 1 ? values["name"] as String : saved.name,
          birth: accounts.length == 1 ? values["birth"] as int : saved.birth,
          icon: accounts.length == 1 ? values["icon"] as Uint8List? : saved.icon,
          enabled: values["enabled"] as bool? ?? saved.enabled,
          hidden: values["hidden"] as bool? ?? saved.hidden,
          useInternal: values["useInternal"] as bool? ?? saved.useInternal,
          folder: folderId == null ? saved.folder : folders!.firstWhere(
            (f) => f.id == folderId, orElse: () => Folder(id: 0, name: "")),
        );
        if (a == saved) continue;
        final birthChanged = a.birth != saved.birth;
        final internalChanged = a.useInternal != saved.useInternal;
        await updateAccount(
          update: AccountUpdate(
            coin: a.coin,
            id: a.id,
            name: a.name != saved.name ? a.name : null,
            icon: !listEquals(a.icon, saved.icon) ? a.icon ?? Uint8List(0) : null,
            birth: birthChanged ? a.birth : null,
            folder: a.folder.id,
            enabled: a.enabled != saved.enabled ? a.enabled : null,
            hidden: a.hidden != saved.hidden ? a.hidden : null,
            useInternal: internalChanged ? a.useInternal : null,
          ),
          c: c,
        );
        if (a.name != saved.name || birthChanged || internalChanged) await _updateVaultAccount(a);
        if (birthChanged || internalChanged) {
          await resetSync(id: a.id, c: c);
          ref.invalidate(syncStateAccountProvider(a.id));
        }
        ref.invalidate(accountProvider(a.id));
      }
      ref.invalidate(getAccountsProvider);
      _allowExit = true;
      return true;
    } catch (error, stack) {
      if (mounted) await showException(context, error.toString(), stackTrace: stack);
      return false;
    }
  }

  void onEditIcon() async {
    final icon = await pickImage();
    Uint8List? bytes;
    if (icon != null) {
      bytes = await icon.readAsBytes();
    } else {
      if (!mounted) return;
      final remove = await confirmDialog(context, title: "Reset Icon", message: "Do you want to remove the current icon?");
      if (!remove) return;
    }
    if (mounted) formKey.currentState?.fields["icon"]?.didChange(bytes);
  }

  void onExport() async {
    final account = accounts.first;
    final password = await inputPassword(context, title: "Export Account", message: "File Password", repeated: true, required: true);
    if (password != null) {
      final res = await exportAccount(id: account.id, passphrase: password, c: c);
      await saveFile(title: "Please select an output file for the encrypted account:", fileName: "${account.name}.bin", data: res);
    }
  }

  void onRewind() async {
    final account = accounts.first;
    final confirmed = await confirmDialog(
      context,
      title: "Rewind",
      message:
          "Are you sure you want to rewind this account? This will rollback the account to a previous height. You will not lose any funds, but you will need to resync the account",
    );
    if (!confirmed) return;
    final dbHeight = await getDbHeight(account: account.id, c: c);
    await rewindSync(height: dbHeight.height - 60, account: account.id, c: c);
    final h = await getDbHeight(account: account.id, c: c);
    final ss = ref.read(syncStateAccountProvider(account.id).notifier);
    ss.updateHeight(h.height, h.time);
  }

  void onReset() async {
    final confirmed = await confirmDialog(
      context,
      title: "Reset Account",
      message:
          "Are you sure you want to reset this account? This will clear all sync data and reset the account to the birth height. You will not lose any funds, but you will need to resync the account",
    );
    if (!confirmed) return;
    for (var account in accounts) {
      await resetSync(id: account.id, c: c);
      ref.invalidate(accountProvider(account.id));
    }
  }
}

extension AccountExtension on Account {
  Widget avatar({bool? selected, void Function(bool?)? onTap}) {
    final t = Theme.of(navigatorKey.currentContext!).colorScheme;
    final i = initials(name);
    final s = selected ?? false;
    return GestureDetector(
      onTap: () => onTap?.call(!s),
      child: CircleAvatar(
        backgroundColor: s ? Colors.blue.shade700 : t.primaryContainer,
        child: s
            ? Icon(Icons.check, color: Colors.white)
            : icon != null
                ? ClipOval(child: Image.memory(icon!))
                : Text(i, style: TextStyle(color: t.onPrimaryContainer)),
      ),
    );
  }
}

class BalanceWidget extends StatelessWidget {
  final PoolBalance balance;
  final void Function(int)? onPoolSelected;
  const BalanceWidget(this.balance, {super.key, this.onPoolSelected});

  @override
  Widget build(BuildContext context) {
    final b = balance.field0;
    return Column(
      mainAxisSize: MainAxisSize.min,
      children: [
        Row(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            GestureDetector(onTap: () => onPoolSelected?.call(0), child: BalanceChip(PoolType.transparent, zatToShortString(b[0]))),
            const Gap(8),
            GestureDetector(onTap: () => onPoolSelected?.call(1), child: BalanceChip(PoolType.sapling, zatToShortString(b[1]))),
          ],
        ),
        const Gap(8),
        Row(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            GestureDetector(onTap: () => onPoolSelected?.call(2), child: BalanceChip(PoolType.orchard, zatToShortString(b[2]))),
            const Gap(8),
            GestureDetector(onTap: () => onPoolSelected?.call(3), child: BalanceChip(PoolType.ironwood, zatToShortString(b[3]))),
          ],
        ),
      ],
    );
  }
}

Widget showTxHistory(BuildContext context, int account, List<Tx> transactions) {
  return ListView.builder(
    itemExtent: 65,
    itemCount: transactions.length,
    itemBuilder: (context, index) {
      final tx = transactions[index];
      final (color, icon, label) = getTransactionType(tx.tpe);
      final tile = TransactionTile(
        icon: icon,
        color: color,
        label: label,
        amount: BigInt.from(tx.value),
        date: tx.time,
        id: tx.id,
        onTap: () => gotoTransaction(context, account, tx.id),
        zsaValue: tx.zsaValue != 0 ? BigInt.from(tx.zsaValue) : null,
        zsaLabel: tx.zsaValue != 0 ? tx.assetDisplay : null,
        contactName: tx.contactName,
      );
      return Column(children: [
        SizedBox(height: 64, child: tile),
        const Divider(height: 1, thickness: 1, indent: 16, endIndent: 16),
      ]);
    },
  );
}

void gotoTransaction(BuildContext context, int account, int idTx) async {
  await GoRouter.of(context).push(accountRoute("/tx_view", account), extra: idTx);
}

Uint8List trimTrailingZeros(Uint8List bytes) {
  int end = bytes.length;
  while (end > 0 && bytes[end - 1] == 0x00) {
    end--;
  }
  return bytes.sublist(0, end);
}

Widget showMemos(BuildContext context, List<Memo> memos, int account, VoidCallback onMemoChanged) {
  // Use a key derived from the memo list content to force rebuild when memos update,
  // since SearchableList only reads initialList once.
  final memoKey = memos.isEmpty ? 'empty' : '${memos.first.id}_${memos.last.id}_${memos.length}';
  return SearchableList(
    key: ValueKey(memoKey),
    initialList: memos,
    itemBuilder: (memo) => MemoWidget(memo, account: account, onMemoChanged: onMemoChanged),
    filter: (query) => memos.where((m) => query.isEmpty || (m.memo?.contains(query) == true)).toList(),
    inputDecoration: InputDecoration(
      labelText: "Search Memos",
      fillColor: Colors.white,
    ),
  );
}

Widget showNotes(WidgetRef ref, int account, List<TxNote> notes, bool showDust, VoidCallback onToggleDust, bool groupByPool, VoidCallback onToggleGroup) {
  final t = Theme.of(navigatorKey.currentContext!);
  final currentHeight = ref.read(currentHeightProvider).value;
  final dustThreshold = BigInt.from(5000);
  final filtered = showDust ? notes : notes.where((n) => n.idAsset != null || n.value > dustThreshold).toList();

  // Build grouped items: header + its notes
  List<({int pool, List<TxNote> poolNotes})> groups = [];
  if (groupByPool) {
    final grouped = <int, List<TxNote>>{};
    for (final n in filtered) {
      grouped.putIfAbsent(n.pool, () => []).add(n);
    }
    // Sort pools in display order: Transparent, Sapling, Orchard, Ironwood
    final poolOrder = [0, 1, 2, 3];
    groups = poolOrder.where((p) => grouped.containsKey(p)).map((p) => (pool: p, poolNotes: grouped[p]!)).toList();
  }

  // Flattened item count: toolbar + (header + notes per group) or all notes flat
  final totalItems = groupByPool ? 1 + groups.fold<int>(0, (sum, g) => sum + 1 + g.poolNotes.length) : filtered.length + 1;

  return ListView.builder(
    itemCount: totalItems,
    itemBuilder: (context, index) {
      if (index == 0)
        return OverflowBar(
          children: [
            IconButton(onPressed: () => onLockRecent(ref, context, account, currentHeight), tooltip: "Lock recently mined notes", icon: Icon(Icons.table_rows)),
            IconButton(onPressed: () => onToggleAll(ref, context, account), tooltip: "Toggle all notes", icon: Icon(Icons.sync_alt)),
            IconButton(onPressed: () => onUnlockAll(ref, context, account), tooltip: "Unlock all notes", icon: Icon(Icons.select_all)),
            IconButton(
              onPressed: onToggleDust,
              tooltip: showDust ? "Hide dust notes" : "Show dust notes",
              icon: Icon(showDust ? Icons.filter_alt_off : Icons.filter_alt),
            ),
            IconButton(
              onPressed: onToggleGroup,
              tooltip: groupByPool ? "Ungroup notes" : "Group by pool",
              icon: Icon(groupByPool ? Icons.dashboard : Icons.view_list),
            ),
          ],
        );

      if (groupByPool) {
        // Walk the grouped structure
        int offset = 1;
        for (final g in groups) {
          // Header
          if (offset == index) {
            final allLocked = g.poolNotes.every((n) => n.locked);
            return Padding(
              padding: const EdgeInsets.fromLTRB(16, 8, 8, 0),
              child: Row(
                children: [
                  Expanded(
                    child: Text(
                      poolToString(g.pool),
                      style: t.textTheme.titleSmall?.copyWith(
                        fontWeight: FontWeight.bold,
                        color: t.colorScheme.primary,
                      ),
                    ),
                  ),
                  IconButton(
                    onPressed: () => onTogglePoolNotes(ref, context, account, g.poolNotes),
                    tooltip: allLocked ? "Unlock all in pool" : "Lock all in pool",
                    icon: Icon(allLocked ? Icons.lock_open : Icons.lock_outline),
                    visualDensity: VisualDensity.compact,
                  ),
                ],
              ),
            );
          }
          offset++;
          // Notes in this group
          if (index < offset + g.poolNotes.length) {
            final note = g.poolNotes[index - offset];
            return ListTile(
              key: ValueKey(note.id),
              dense: true,
              onTap: () => toggleLock(ref, context, account, note.id, !note.locked),
              leading: Text("${note.height}"),
              title: Text(poolToString(note.pool)),
              trailing: note.idAsset != null ? Text("${note.value} ${note.assetDisplay}", softWrap: false) : zatToText(note.value, selectable: false),
              textColor: note.locked ? t.disabledColor : null,
            );
          }
          offset += g.poolNotes.length;
        }
        // Shouldn't reach here
        return const SizedBox.shrink();
      }

      final noteIndex = index - 1;
      final note = filtered[noteIndex];
      return ListTile(
        key: ValueKey(note.id),
        onTap: () => toggleLock(ref, context, account, note.id, !note.locked),
        leading: Text("${note.height}"),
        title: Text(poolToString(note.pool)),
        trailing: note.idAsset != null ? Text("${note.value} ${note.assetDisplay}", softWrap: false) : zatToText(note.value, selectable: false),
        textColor: note.locked ? t.disabledColor : null,
      );
    },
  );
}

void onLockRecent(WidgetRef ref, BuildContext context, int account, int? currentHeight) async {
  if (currentHeight == null) return;
  final c = coinContext.coin;
  final s = await inputText(context, title: "Enter confirmation threshold");
  final threshold = s?.let((v) => int.tryParse(v));
  if (threshold != null) {
    await lockRecentNotes(
      height: currentHeight,
      threshold: threshold,
      account: account,
      c: c,
    );
    ref.invalidate(accountProvider(account));
  }
}

void onUnlockAll(WidgetRef ref, BuildContext context, int account) async {
  final c = coinContext.coin;
  final confirmed = await confirmDialog(context, title: "Unlock All", message: "Do you want to unlock every note?");
  if (confirmed) {
    await unlockAllNotes(account: account, c: c);
    ref.invalidate(accountProvider(account));
  }
}

void onToggleAll(WidgetRef ref, BuildContext context, int account) async {
  final c = coinContext.coin;
  final confirmed = await confirmDialog(context, title: "Toggle All", message: "Do you want to toggle the lock state of every note?");
  if (confirmed) {
    await toggleAllNotes(account: account, c: c);
    ref.invalidate(accountProvider(account));
  }
}

void onTogglePoolNotes(WidgetRef ref, BuildContext context, int account, List<TxNote> poolNotes) async {
  final allLocked = poolNotes.every((n) => n.locked);
  final target = !allLocked;
  final c = coinContext.coin;
  for (final note in poolNotes) {
    await lockNote(id: note.id, locked: target, account: account, c: c);
  }
  ref.invalidate(accountProvider(account));
}

void toggleLock(WidgetRef ref, BuildContext context, int account, int id, bool locked) async {
  final c = coinContext.coin;
  await lockNote(id: id, locked: locked, account: account, c: c);
  ref.invalidate(accountProvider(account));
}

class MemoWidget extends StatefulWidget {
  final Memo memo;
  final int account;
  final void Function() onMemoChanged;
  const MemoWidget(this.memo, {super.key, required this.account, required this.onMemoChanged});

  @override
  State<MemoWidget> createState() => _MemoWidgetState();
}

class _MemoWidgetState extends State<MemoWidget> {
  bool _editing = false;
  late final TextEditingController _controller = TextEditingController();
  late final FocusNode _focusNode = FocusNode();
  late final c = coinContext.coin;

  @override
  void initState() {
    super.initState();
    _focusNode.addListener(_onFocusChange);
  }

  @override
  void dispose() {
    _focusNode.removeListener(_onFocusChange);
    _controller.dispose();
    _focusNode.dispose();
    super.dispose();
  }

  void _onFocusChange() {
    if (!_focusNode.hasFocus && _editing) {
      _commitEditing();
    }
  }

  void _startEditing() {
    // Pre-populate with the text memo (not binary)
    _controller.text = widget.memo.memo ?? '';
    _controller.selection = TextSelection.fromPosition(
      TextPosition(offset: _controller.text.length),
    );
    _editing = true;
    _focusNode.requestFocus();
    setState(() {});
  }

  Future<void> _commitEditing() async {
    _editing = false;
    if (!mounted) return;
    final newText = _controller.text.trim();
    final oldText = widget.memo.memo ?? '';
    setState(() {});

    if (newText == oldText) return;

    if (newText.isEmpty) {
      await setUserMemo(idTx: widget.memo.idTx, memo: null, account: widget.account, c: c);
    } else {
      await setUserMemo(idTx: widget.memo.idTx, memo: newText, account: widget.account, c: c);
    }
    widget.onMemoChanged();
  }

  @override
  Widget build(BuildContext context) {
    final t = Theme.of(context);
    final cs = t.colorScheme;
    final incoming = widget.memo.idNote != null;
    final memoText = widget.memo.memo;

    return GestureDetector(
      onTap: () => gotoTransaction(context, widget.account, widget.memo.idTx),
      onLongPress: _startEditing,
      child: Padding(
        padding: const EdgeInsetsGeometry.symmetric(vertical: 4),
        child: Bubble(
          nip: incoming ? BubbleNip.leftTop : BubbleNip.rightTop,
          color: incoming ? Colors.transparent : cs.secondaryContainer,
          borderColor: incoming ? cs.outline : Colors.transparent,
          borderWidth: incoming ? 1 : 0,
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Align(alignment: Alignment.centerRight, child: Text(timeToString(widget.memo.time), style: t.textTheme.labelMedium)),
              const Gap(8),
              if (_editing)
                TextField(
                  controller: _controller,
                  focusNode: _focusNode,
                  maxLines: null,
                  minLines: 2,
                  textInputAction: TextInputAction.newline,
                  onEditingComplete: _commitEditing,
                  style: widget.memo.isUserMemo
                      ? TextStyle(
                          color: Colors.orange.shade700,
                          fontWeight: FontWeight.w500,
                        )
                      : null,
                  decoration: const InputDecoration(
                    isDense: true,
                    border: OutlineInputBorder(),
                  ),
                )
              else
                CopyableText(
                  memoText ?? hex.encode(trimTrailingZeros(widget.memo.memoBytes)),
                  style: widget.memo.isUserMemo
                      ? TextStyle(
                          color: Colors.orange.shade700,
                          fontWeight: FontWeight.w500,
                        )
                      : null,
                ),
              // Plugin-parsed memo sections (for 0xFF arbitrary memos)
              PluginMemoView(widget.memo.memoBytes),
            ],
          ),
        ),
      ),
    );
  }
}

class ViewingKeysPage extends ConsumerStatefulWidget {
  // the viewing keys page is opened from the edit account page
  // and the account is passed as an argument
  // because the selected account may be different
  final int account;
  const ViewingKeysPage(this.account, {super.key});

  @override
  ConsumerState<ViewingKeysPage> createState() => ViewingKeysPageState();
}

class ViewingKeysPageState extends ConsumerState<ViewingKeysPage> {
  late final c = coinContext.coin;
  int pools = 7;
  String? uvk;
  Seed? seed;
  int accountPools = 7; // default to all pools
  bool showSeed = false;
  FrostParams? frostParams;
  String? frostBackup;

  @override
  void initState() {
    super.initState();
    Future(() async {
      seed = await getAccountSeed(account: widget.account, c: c);
      accountPools = await getAccountPools(account: widget.account, c: c);
      frostParams = await getAccountFrostParams(account: widget.account, c: c);
      setState(() {});
    });
    Future(() => onPoolChanged(pools));
  }

  @override
  Widget build(BuildContext context) {
    final pinlock = ref.watch(lifecycleProvider);
    if (pinlock.value ?? false) return PinLock();

    return Scaffold(
      appBar: AppBar(
        title: Text('Viewing Keys'),
        actions: [
          if (seed != null) IconButton(tooltip: "Show Seed Phrase", onPressed: onShowSeed, icon: Icon(Icons.key)),
          if (seed == null && frostParams != null)
            IconButton(tooltip: "Show Backup String", onPressed: onShowFrostBackup, icon: Icon(Icons.key)),
        ],
      ),
      body: CustomScrollView(
        slivers: [
          if (showSeed && seed != null) ...[
            SliverToBoxAdapter(
              child: Padding(
                padding: EdgeInsets.symmetric(horizontal: 16),
                child: Column(children: [
                  ListTile(title: Text("Mnemonic"), subtitle: CopyableText(seed!.mnemonic)),
                  ListTile(title: Text("Passphrase"), subtitle: CopyableText(seed!.phrase)),
                  ListTile(title: Text("Index"), subtitle: CopyableText(seed!.aindex.toString())),
                  Divider(),
                  Gap(8),
                ]),
              ),
            ),
          ],
          if (frostBackup != null) ...[
            SliverToBoxAdapter(
              child: Padding(
                padding: EdgeInsets.symmetric(horizontal: 16),
                child: Column(children: [
                  ListTile(
                    leading: Icon(Icons.warning_amber),
                    title: Text("Backup String"),
                    subtitle: Text("Treat this string like a seed phrase: anyone holding it has your share of the group. "
                        "Restore it with the group's viewing key below."),
                  ),
                  CopyableText(frostBackup!),
                  Gap(16),
                  QrImageView(data: frostBackup!, size: 200, backgroundColor: Colors.white),
                  if (frostParams!.dkgVersion < 2) ...[
                    Gap(16),
                    Text(
                      "This backs up your membership against device loss. It does not make the group's existing funds quantum recoverable; upgrade and sweep for that.",
                      style: Theme.of(context).textTheme.bodySmall,
                    ),
                  ],
                  Divider(),
                  Gap(8),
                ]),
              ),
            ),
          ],
          SliverPersistentHeader(
            pinned: true,
            delegate: _PoolSelectHeaderDelegate(
              child: Padding(
                padding: EdgeInsets.fromLTRB(16, 0, 16, 8),
                child: Center(child: PoolSelect(enabled: accountPools, initialValue: accountPools, onChanged: onPoolChanged)),
              ),
            ),
          ),
          SliverToBoxAdapter(child: Gap(32)),
          if (uvk != null)
            SliverToBoxAdapter(
              child: Padding(
                padding: EdgeInsets.symmetric(horizontal: 16),
                child: CopyableText(uvk!),
              ),
            ),
          SliverToBoxAdapter(child: Gap(32)),
          if (uvk != null)
            SliverToBoxAdapter(
              child: Center(
                child: QrImageView(data: uvk!, size: 200, backgroundColor: Colors.white),
              ),
            ),
          SliverToBoxAdapter(child: Gap(16)),
          SliverToBoxAdapter(
            child: Padding(
              padding: EdgeInsets.symmetric(horizontal: 16),
              child: Text("If the account does not include a pool, its receiver will be absent"),
            ),
          ),
        ],
      ),
    );
  }

  onPoolChanged(int? v) async {
    if (v == null) return;
    try {
      final uuvk = await getAccountUfvk(account: widget.account, pools: v, c: c);
      setState(() {
        pools = v;
        uvk = uuvk;
      });
    } on AnyhowException catch (e) {
      if (!mounted) return;
      await showException(context, e.message);
      setState(() {
        uvk = null;
      });
    }
  }

  void onShowSeed() async {
    final authenticated = await authenticate(reason: "Show Seed Phrase");
    if (!authenticated) return;
    setState(() {
      showSeed = true;
    });
  }

  void onShowFrostBackup() async {
    final authenticated = await authenticate(reason: "Show Backup String");
    if (!authenticated) return;
    try {
      final backup = await exportFrostBackup(account: widget.account, c: c);
      setState(() => frostBackup = backup);
    } on AnyhowException catch (e) {
      if (mounted) await showException(context, e.message);
    }
  }
}

class _PoolSelectHeaderDelegate extends SliverPersistentHeaderDelegate {
  final Widget child;

  _PoolSelectHeaderDelegate({required this.child});

  @override
  Widget build(BuildContext context, double shrinkOffset, bool overlapsContent) {
    final t = Theme.of(context);
    return Container(
      color: t.colorScheme.surface,
      child: child,
    );
  }

  @override
  double get maxExtent => 56;

  @override
  double get minExtent => 56;

  @override
  bool shouldRebuild(covariant _PoolSelectHeaderDelegate oldDelegate) => child != oldDelegate.child;
}
