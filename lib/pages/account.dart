import 'dart:async';
import 'dart:convert';
import 'dart:io';

import 'package:bubble/bubble.dart';
import 'package:convert/convert.dart';
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
import 'package:zkool/pages/zsa.dart';
import 'package:zkool/router.dart';
import 'package:zkool/src/rust/api/account.dart';
import 'package:zkool/src/rust/api/sync.dart';
import 'package:zkool/src/rust/api/transaction.dart';
import 'package:zkool/src/rust/api/zsa.dart';
import 'package:zkool/store.dart';
import 'package:zkool/utils.dart';
import 'package:zkool/widgets/error_display.dart';
import 'package:zkool/widgets/exchange_rate.dart';
import 'package:zkool/widgets/pool_select.dart';
import 'package:zkool/widgets/theme.dart';
import 'package:zkool/widgets/transaction_table.dart';

class AccountViewPage extends ConsumerStatefulWidget {
  const AccountViewPage({super.key});

  @override
  ConsumerState<AccountViewPage> createState() => AccountViewPageState();
}

class AccountViewPageState extends ConsumerState<AccountViewPage> with SingleTickerProviderStateMixin {
  late final tabController = TabController(length: 4, vsync: this);

  final List<String> tabNames = ["Transactions", "Memos", "Notes", "ZSA Holdings"];

  late final c = coinContext.coin;
  StreamSubscription<SyncProgress>? progressSubscription;

  // ZSA Holdings inline editing state
  int? _editingIndex;
  late final TextEditingController _nameController = TextEditingController();
  late final FocusNode _focusNode = FocusNode();
  bool _zsaAvailable = false;

  // Tx search
  final _txSearchController = TextEditingController();
  String _txSearchQuery = '';

  @override
  void initState() {
    super.initState();
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

  Future<void> _commitEditing() async {
    final index = _editingIndex;
    if (index == null) return;

    final fullData = ref.read(fullAccountPageDataProvider).value;
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

    final selectedAccountAV = ref.watch(selectedAccountProvider);
    ref.watch(appSettingsProvider); // rebuild when settings change (e.g. list/table toggle)
    return selectedAccountAV.when(
      loading: () => blank(context),
      error: (error, stack) => showError(error),
      data: (selectedAccount) {
        if (selectedAccount == null) {
          WidgetsBinding.instance.addPostFrameCallback((_) {
            if (GoRouter.of(context).state.matchedLocation == '/accounts') return;
            GoRouter.of(context).go("/accounts");
          });
        }

        final fullDataAV = ref.watch(fullAccountPageDataProvider);

        return Scaffold(
          appBar: AppBar(
            title: Text(fullDataAV.value?.currentAccount?.account.name ?? "Loading"),
            actions: [
              Tooltip(
                message: "Synchronize only this account",
                child: IconButton(
                  tooltip: "Sync this account",
                  onPressed: fullDataAV.value?.currentAccount != null ? () => onSync(fullDataAV.value!.currentAccount!) : null,
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
                      GoRouter.of(context).push("/accounts");
                    case "backup":
                      final account = fullDataAV.value?.currentAccount?.account;
                      if (account != null) {
                        GoRouter.of(context).push("/viewing_keys", extra: account.id);
                      }
                    case "edit_account":
                      final account = fullDataAV.value?.currentAccount?.account;
                      if (account != null) {
                        GoRouter.of(context).push("/account/edit", extra: [account]);
                      }
                    case "market_price":
                      GoRouter.of(context).push("/market");
                    case "update_fx":
                      onUpdateAllTxPrices();
                    case "toggle_tx_view":
                      ref.read(appSettingsProvider.notifier)
                          .setTransactionViewMode(
                              !(ref.read(appSettingsProvider).value?.transactionTableMode ?? false));
                    case "charts":
                      GoRouter.of(context).push("/chart");
                    case "settings":
                      GoRouter.of(context).push("/settings");
                    default:
                      onExport(int.parse(result));
                  }
                },
                itemBuilder: (BuildContext context) => <PopupMenuEntry<String>>[
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
                    child: Text((ref.read(appSettingsProvider).value?.transactionTableMode ?? false)
                        ? "View: List"
                        : "View: Table"),
                  ),
                  if (!Platform.isLinux)
                    const PopupMenuItem<String>(
                      value: "charts",
                      child: Text("Charts"),
                    ),
                ],
              ),
            ],
            bottom: TabBar(
              controller: tabController,
              tabs: tabNames.map((n) => Tab(text: n)).toList(),
            ),
          ),
          body: fullDataAV.when(
            skipLoadingOnReload: true,
            skipLoadingOnRefresh: true,
            loading: () => blank(context),
            error: (error, stack) => showError(error),
            data: (fullData) {
              final account = fullData.currentAccount!;

              final b = account.balance.field0;
              final settings = ref.read(appSettingsProvider).requireValue;
              final currency = settings.currency;
              final fiat = fullData.price?.let((p) {
                final f = (b[0] + b[1] + b[2]).toDouble() * p / zatsPerZec.toDouble();
                return formatFiat(f, currency);
              });

              final t = Theme.of(context);
              final tt = t.textTheme;

              final unconfirmedAmount = fullData.mempool.unconfirmedFunds[account.account.id];

              return Builder(
                builder: (context) {
                  final ss = fullData.syncState;
                  if (ss == null) return const SizedBox.shrink();

                  final syncing = ss.start != ss.end;
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
                                      if (syncing) ...[
                                        HeroProgressWidget(account.account),
                                        Gap(8),
                                      ],
                                      DisplayPanel(
                                        child: Column(children: [
                                          Tooltip(
                                            message: "Balance across all pools",
                                            child: Column(children: [
                                              zatToText(
                                                b[0] + b[1] + b[2],
                                                selectable: true,
                                                style: tt.displaySmall!,
                                              ),
                                            ]),
                                          ),
                                          if (fiat != null) Text(fiat),
                                          const Gap(4),
                                          const ExchangeRateButton(),
                                          Gap(8),
                                          BalanceWidget(account.balance),
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
                              if (settings.transactionTableMode)
                                SliverToBoxAdapter(
                                  child: TransactionTable(
                                    transactions: account.transactions,
                                    currency: currency,
                                    onTap: (id) => gotoTransaction(context, id),
                                    onMemoChanged: (txId, newMemo) async {
                                      await setUserMemo(
                                        idTx: txId,
                                        memo: newMemo,
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
                                      _filterTx(account.transactions),
                                      _txSearchController,
                                    ),
                                  ),
                                ),
                            ],
                          ),
                          showMemos(context, account.memos, () {
                            final selectedAccount =
                                ref.read(selectedAccountProvider).requireValue!;
                            ref.invalidate(
                                accountProvider(selectedAccount.id));
                          }),
                          showNotes(ref, account.notes),
                          _showZsaHoldings(context, account.zsas),
                        ],
                      ));
                },
              );
            },
          ),
        );
      },
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
    await GoRouter.of(context).push("/receive");
  }

  void onSend() async {
    await GoRouter.of(context).push("/send");
  }

  void onUpdateAllTxPrices() async {
    final settings = await ref.read(appSettingsProvider.future);
    final confirmed =
        await confirmDialog(context, title: "Fetch Tx Market Price", message: "Do you want to retrieve historical ZEC prices for your past transactions?");
    if (confirmed) {
      try {
        await fillMissingTxPrices(c: c, api: settings.coingecko, currency: settings.currency);
      } on AnyhowException catch (e) {
        if (mounted) await showException(context, e.message);
      }
    }
  }

  void onExport(int index) async {
    final data = await getExportedData(type: index, c: c);
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
                onPressed: _zsaAvailable ? () => GoRouter.of(context).push("/zsa/issue") : null,
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
                      onTap: () => GoRouter.of(context).push(
                        "/zsa/issue",
                        extra: IssuanceArgs(
                          assetName: displayName,
                          isReissuance: true,
                          assetDescHash: h.assetDescHash,
                        ),
                      ),
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
  late List<Account> accounts = widget.accounts;
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
      setState(() => this.folders = folders);
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
  void didUpdateWidget(covariant AccountEditPage oldWidget) {
    accounts = widget.accounts;
    super.didUpdateWidget(oldWidget);
  }

  @override
  Widget build(BuildContext context) {
    final pinlock = ref.watch(lifecycleProvider);
    if (pinlock.value ?? false) return PinLock();

    if (folders == null) return SizedBox.expand();

    final account = accounts.length == 1 ? accounts.first : null;
    final folder = accounts.first.folder;
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
                        initialValue: account?.name ?? "(Multiple)",
                        readOnly: account == null,
                        onChanged: (account != null) ? onEditName : null,
                      ),
                    ),
                  ),
                  if (account != null) Tooltip(message: "Edit Account Icon", child: account.avatar(onTap: (_) => onEditIcon())),
                ],
              ),
              Tooltip(
                message: "Edit Height at the creation of the account",
                child: FormBuilderTextField(
                  name: 'birth',
                  decoration: InputDecoration(labelText: 'Birth Height'),
                  initialValue: account?.birth.toString() ?? "",
                  keyboardType: TextInputType.number,
                  readOnly: account == null,
                  inputFormatters: [FilteringTextInputFormatter.digitsOnly],
                  onChanged: (account != null) ? onEditBirth : null,
                ),
              ),
              Tooltip(
                message: "Enable or disable. Only enabled accounts participate in the global sync",
                child: FormBuilderCheckbox(
                  name: "enabled",
                  title: Text("Enabled"),
                  initialValue: accounts.every((a) => a.enabled == accounts[0].enabled) ? accounts[0].enabled : null,
                  tristate: account == null,
                  onChanged: onEditEnabled,
                ),
              ),
              Tooltip(
                message: "Hide this account from the account list",
                child: FormBuilderCheckbox(
                  name: "hidden",
                  title: Text("Hidden"),
                  initialValue: accounts.every((a) => a.hidden == accounts[0].hidden) ? accounts[0].hidden : null,
                  tristate: account == null,
                  onChanged: onEditHidden,
                ),
              ),
              Tooltip(
                message: "Assign Account to Folder",
                child: FormBuilderDropdown<int>(
                  name: "folder",
                  initialValue: accounts.every((a) => a.folder.id == folder.id) ? folder.id : null,
                  items: folderOptions,
                  onChanged: onEditFolder,
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }

  void onEditName(String? name) async {
    assert(accounts.length == 1);
    if (name != null) {
      accounts[0] = accounts[0].copyWith(name: name);
      await updateAccount(
        update: AccountUpdate(
          coin: accounts[0].coin,
          id: accounts[0].id,
          name: name,
          folder: accounts[0].folder.id,
        ),
        c: c,
      );
      final a = accounts[0];
      final seed = a.seed;
      if (seed != null) {
        final settings = await ref.read(appSettingsProvider.future);
        if (settings.vault)
          await ref.read(vaultProvider.notifier).storeAccount(
                name: name,
                seed: seed,
                aindex: a.aindex,
                useInternal: a.useInternal,
                birthHeight: a.birth,
              );
      }
      ref.invalidate(getAccountsProvider);
      ref.invalidate(accountProvider(accounts[0].id));
      setState(() {});
    }
  }

  void onEditIcon() async {
    final icon = await pickImage();
    var changed = false;
    Uint8List? bytes;
    if (icon != null) {
      bytes = await icon.readAsBytes();
      changed = true;
    } else {
      final remove = await confirmDialog(context, title: "Reset Icon", message: "Do you want to remove the current icon?");
      if (remove) {
        bytes = Uint8List(0);
        changed = true;
      }
    }
    if (changed) {
      accounts[0] = accounts[0].copyWith(icon: bytes?.isNotEmpty == true ? bytes : null);
      await updateAccount(
        update: AccountUpdate(
          coin: accounts[0].coin,
          id: accounts[0].id,
          icon: bytes,
          folder: accounts[0].folder.id,
        ),
        c: c,
      );
      ref.invalidate(accountProvider(accounts[0].id));
      setState(() {});
    }
  }

  void onEditBirth(String? birth) async {
    if (birth != null && birth.isNotEmpty) {
      final bh = int.parse(birth);
      accounts[0] = accounts[0].copyWith(birth: bh);
      await updateAccount(
        update: AccountUpdate(
          coin: accounts[0].coin,
          id: accounts[0].id,
          birth: bh,
          folder: accounts[0].folder.id,
        ),
        c: c,
      );
      final a = accounts[0];
      final seed = await getAccountSeed(account: a.id, c: c);
      if (seed != null) {
        final settings = await ref.read(appSettingsProvider.future);
        if (settings.vault)
          await ref.read(vaultProvider.notifier).storeAccount(
                name: a.name,
                seed: seed.mnemonic,
                aindex: a.aindex,
                useInternal: a.useInternal,
                birthHeight: bh,
              );
      }
      ref.invalidate(accountProvider(accounts[0].id));
      setState(() {});
    }
  }

  void onEditEnabled(bool? v) async {
    if (v == null) return;
    for (var i = 0; i < accounts.length; i++) {
      accounts[i] = accounts[i].copyWith(enabled: v);
      await updateAccount(
        update: AccountUpdate(
          coin: accounts[i].coin,
          id: accounts[i].id,
          enabled: v,
          folder: accounts[i].folder.id,
        ),
        c: c,
      );
      ref.invalidate(accountProvider(accounts[i].id));
    }
    setState(() {});
  }

  void onEditHidden(bool? v) async {
    if (v == null) return;
    for (var i = 0; i < accounts.length; i++) {
      accounts[i] = accounts[i].copyWith(hidden: v);
      await updateAccount(
        update: AccountUpdate(
          coin: accounts[i].coin,
          id: accounts[i].id,
          hidden: v,
          folder: accounts[i].folder.id,
        ),
        c: c,
      );
      ref.invalidate(accountProvider(accounts[i].id));
    }
    setState(() {});
  }

  void onEditFolder(int? v) async {
    if (v == null) return;
    final folders = ref.read(getFoldersProvider).requireValue;
    for (var i = 0; i < accounts.length; i++) {
      accounts[i] = accounts[i].copyWith(folder: folders.firstWhere((f) => f.id == v, orElse: () => Folder(id: 0, name: "")));
      await updateAccount(
        update: AccountUpdate(
          coin: accounts[i].coin,
          id: accounts[i].id,
          folder: v,
        ),
        c: c,
      );
      ref.invalidate(accountProvider(accounts[i].id));
    }
    setState(() {});
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
    final dbHeight = await getDbHeight(c: c);
    await rewindSync(height: dbHeight.height - 60, account: account.id, c: c);
    final h = await getDbHeight(c: c);
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
    return Row(
      mainAxisAlignment: MainAxisAlignment.center,
      children: [
        GestureDetector(onTap: () => onPoolSelected?.call(0), child: BalanceChip(PoolType.transparent, zatToShortString(balance.field0[0]))),
        Gap(8),
        GestureDetector(onTap: () => onPoolSelected?.call(1), child: BalanceChip(PoolType.sapling, zatToShortString(balance.field0[1]))),
        Gap(8),
        GestureDetector(onTap: () => onPoolSelected?.call(2), child: BalanceChip(PoolType.orchard, zatToShortString(balance.field0[2]))),
      ],
    );
  }
}

Widget showTxHistory(BuildContext context, List<Tx> transactions, TextEditingController searchController) {
  return Column(
    mainAxisSize: MainAxisSize.min,
    children: [
      Padding(
        padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
        child: TextField(
          controller: searchController,
          decoration: const InputDecoration(
            labelText: "Search transactions (memo, contact)",
            fillColor: Colors.white,
            prefixIcon: Icon(Icons.search),
          ),
        ),
      ),
      Padding(
        padding: const EdgeInsets.symmetric(vertical: 4),
        child: Center(
          child: Text(
            "Transaction History (${transactions.length} txs)",
            style: Theme.of(context).textTheme.bodyLarge,
          ),
        ),
      ),
      Expanded(
        child: ListView.builder(
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
              onTap: () => gotoTransaction(context, tx.id),
              zsaValue: tx.zsaValue != 0 ? BigInt.from(tx.zsaValue) : null,
              zsaLabel: tx.zsaValue != 0 ? tx.assetDisplay : null,
              contactName: tx.contactName,
            );
            return Column(children: [
              SizedBox(height: 64, child: tile),
              const Divider(height: 1, thickness: 1, indent: 16, endIndent: 16),
            ]);
          },
        ),
      ),
    ],
  );
}

void gotoTransaction(BuildContext context, int idTx) async {
  await GoRouter.of(context).push("/tx_view", extra: idTx);
}

Uint8List trimTrailingZeros(Uint8List bytes) {
  int end = bytes.length;
  while (end > 0 && bytes[end - 1] == 0x00) {
    end--;
  }
  return bytes.sublist(0, end);
}

Widget showMemos(BuildContext context, List<Memo> memos, VoidCallback onMemoChanged) {
  // Use a key derived from the memo list content to force rebuild when memos update,
  // since SearchableList only reads initialList once.
  final memoKey = memos.isEmpty
      ? 'empty'
      : '${memos.first.id}_${memos.last.id}_${memos.length}';
  return SearchableList(
    key: ValueKey(memoKey),
    initialList: memos,
    itemBuilder: (memo) => MemoWidget(memo, onMemoChanged: onMemoChanged),
    filter: (query) => memos.where((m) => query.isEmpty || (m.memo?.contains(query) == true)).toList(),
    inputDecoration: InputDecoration(
      labelText: "Search Memos",
      fillColor: Colors.white,
    ),
  );
}

Widget showNotes(WidgetRef ref, List<TxNote> notes) {
  final t = Theme.of(navigatorKey.currentContext!);
  final currentHeight = ref.read(currentHeightProvider).value;
  return ListView.builder(
    itemCount: notes.length + 1,
    itemBuilder: (context, index) {
      if (index == 0)
        return OverflowBar(
          children: [
            IconButton(onPressed: () => onLockRecent(ref, context, currentHeight), tooltip: "Lock recently mined notes", icon: Icon(Icons.table_rows)),
            IconButton(onPressed: () => onUnlockAll(ref, context), tooltip: "Unlock all notes", icon: Icon(Icons.select_all)),
          ],
        );

      final noteIndex = index - 1;
      final note = notes[noteIndex];
      return ListTile(
        key: ValueKey(note.id),
        onTap: () => toggleLock(ref, context, note.id, !note.locked),
        leading: Text("${note.height}"),
        title: Text(poolToString(note.pool)),
        trailing: zatToText(note.value, selectable: false),
        textColor: note.locked ? t.disabledColor : null,
      );
    },
  );
}

void onLockRecent(WidgetRef ref, BuildContext context, int? currentHeight) async {
  if (currentHeight == null) return;
  final c = coinContext.coin;
  final s = await inputText(context, title: "Enter confirmation threshold");
  final threshold = s?.let((v) => int.tryParse(v));
  if (threshold != null) {
    await lockRecentNotes(
      height: currentHeight,
      threshold: threshold,
      c: c,
    );
    final selectedAccount = ref.read(selectedAccountProvider).requireValue!;
    ref.invalidate(accountProvider(selectedAccount.id));
  }
}

void onUnlockAll(WidgetRef ref, BuildContext context) async {
  final c = coinContext.coin;
  final confirmed = await confirmDialog(context, title: "Unlock All", message: "Do you want to unlock every note?");
  if (confirmed) {
    await unlockAllNotes(c: c);
    final selectedAccount = ref.read(selectedAccountProvider).requireValue!;
    ref.invalidate(accountProvider(selectedAccount.id));
  }
}

void toggleLock(WidgetRef ref, BuildContext context, int id, bool locked) async {
  final c = coinContext.coin;
  await lockNote(id: id, locked: locked, c: c);
  final selectedAccount = ref.read(selectedAccountProvider).requireValue!;
  ref.invalidate(accountProvider(selectedAccount.id));
}

class MemoWidget extends StatefulWidget {
  final Memo memo;
  final void Function() onMemoChanged;
  const MemoWidget(this.memo, {super.key, required this.onMemoChanged});

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
      await setUserMemo(idTx: widget.memo.idTx, memo: null, c: c);
    } else {
      await setUserMemo(idTx: widget.memo.idTx, memo: newText, c: c);
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
      onTap: () => gotoTransaction(context, widget.memo.idTx),
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
  String? fingerprint;
  Seed? seed;
  int accountPools = 7; // default to all pools
  bool showSeed = false;

  @override
  void initState() {
    super.initState();
    Future(() async {
      fingerprint = await getAccountFingerprint(account: widget.account, c: c);
      seed = await getAccountSeed(account: widget.account, c: c);
      accountPools = await getAccountPools(account: widget.account, c: c);
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
        actions: [if (seed != null) IconButton(tooltip: "Show Seed Phrase", onPressed: onShowSeed, icon: Icon(Icons.key))],
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
              child: Padding(
                padding: EdgeInsets.symmetric(horizontal: 16),
                child: QrImageView(data: uvk!, size: 200, backgroundColor: Colors.white),
              ),
            ),
          SliverToBoxAdapter(child: Gap(8)),
          if (fingerprint != null)
            SliverToBoxAdapter(
              child: Padding(
                padding: EdgeInsets.symmetric(horizontal: 16),
                child: CopyableText(fingerprint!),
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
