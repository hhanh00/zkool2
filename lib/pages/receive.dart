import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge.dart';
import 'package:gap/gap.dart';
import 'package:go_router/go_router.dart';

import 'package:zkool/main.dart';
import 'package:zkool/pages/sweep.dart';
import 'package:zkool/src/rust/api/account.dart';
import 'package:zkool/store.dart';
import 'package:zkool/utils.dart';
import 'package:zkool/widgets/error_display.dart';
import 'package:zkool/widgets/pool_select.dart';

class ReceivePage extends ConsumerStatefulWidget {
  const ReceivePage({super.key});

  @override
  ConsumerState<ReceivePage> createState() => ReceivePageState();
}

class ReceivePageState extends ConsumerState<ReceivePage> {
  late final c = coinContext.coin;
  Account? account;
  Addresses? addresses;
  int uaPools = 0;
  int availablePools = 0;

  @override
  void initState() {
    super.initState();

    Future(() async {
      final selectedAccount = ref.read(selectedAccountProvider).requireValue!;
      final a = await ref.read(accountProvider(selectedAccount.id).future);
      final pools = a.pool; // All pools including transparent
      final defaultPools = pools & 6; // Default to shielded pools only
      final addrs = await getAddresses(uaPools: defaultPools, c: c);
      setState(() {
        account = a.account;
        addresses = addrs;
        availablePools = pools;
        uaPools = defaultPools;
      });
    });
  }

  @override
  Widget build(BuildContext context) {
    if (this.account == null) return blank(context);
    final pinlock = ref.watch(lifecycleProvider);
    if (pinlock.value ?? false) return PinLock();

    final account = this.account!;
    final addresses = this.addresses!;

    return Scaffold(
      appBar: AppBar(
        title: Text("Receive Funds"),
        actions: [
          IconButton(
            tooltip: "View All Addresses",
            onPressed: onViewAddresses,
            icon: Icon(Icons.visibility),
          ),
          IconButton(
            tooltip:
                "Find other transparent addresses. If you restored from a wallet that has address rotation (such as Ledger, Exodus, etc), Tap, then Reset and Sync",
            onPressed: onSweep,
            icon: Icon(Icons.search),
          ),
          IconButton(
            tooltip: "Generate a new set of addresses (transparent/sapling and orchard). Previous addresses can still receive funds",
            onPressed: onGenerateAddress,
            icon: Icon(Icons.skip_next),
          ),
        ],
      ),
      body: SingleChildScrollView(
        child: Padding(
          padding: EdgeInsets.symmetric(horizontal: 8),
          child: Column(
            children: [
              if (addresses.saddr != null || addresses.oaddr != null)
                PoolSelect(
                  enabled: availablePools,
                  initialValue: uaPools,
                  onChanged: onChangedUAPools,
                ),
              if (addresses.ua != null) ...[
                Gap(8),
                ListTile(
                  title: Text("Unified Address"),
                  subtitle: CopyableText(addresses.ua!),
                  trailing: IconButton(
                    tooltip: "Show address as a QR Code",
                    icon: Icon(Icons.qr_code),
                    onPressed: () => onShowQR("Unified Address", addresses.ua!),
                  ),
                ),
              ],
              if (addresses.oaddr != null)
                ListTile(
                  title: Text("Orchard only Address"),
                  subtitle: CopyableText(addresses.oaddr!),
                  trailing: IconButton(
                    icon: Icon(Icons.qr_code),
                    onPressed: () => onShowQR("Orchard", addresses.oaddr!),
                  ),
                ),
              if (addresses.saddr != null)
                ListTile(
                  title: Text("Sapling Address"),
                  subtitle: CopyableText(addresses.saddr!),
                  leading: account.hw != 0 ? IconButton(onPressed: onCheckSapling, icon: Icon(Icons.check)) : null,
                  trailing: IconButton(
                    icon: Icon(Icons.qr_code),
                    onPressed: () => onShowQR("Sapling", addresses.saddr!),
                  ),
                ),
              if (addresses.taddr != null)
                ListTile(
                  title: Text("Transparent Address"),
                  subtitle: CopyableText(addresses.taddr!),
                  leading: account.hw != 0 ? IconButton(onPressed: onCheckTransparent, icon: Icon(Icons.check)) : null,
                  trailing: IconButton(
                    icon: Icon(Icons.qr_code),
                    onPressed: () => onShowQR("Transparent", addresses.taddr!),
                  ),
                ),
            ],
          ),
        ),
      ),
    );
  }

  void onCheckSapling() async {
    showSnackbar("Check address on the device");
    try {
      await showLedgerSaplingAddress(c: c);
    } on AnyhowException catch (e) {
      await showException(context, e.message);
    }
  }

  void onCheckTransparent() async {
    showSnackbar("Check address on the device");
    try {
      await showLedgerTransparentAddress(c: c);
    } on AnyhowException catch (e) {
      await showException(context, e.message);
    }
  }

  void onChangedUAPools(int pools) async {
    uaPools = pools;
    addresses = await getAddresses(uaPools: uaPools, c: c);
    setState(() {});
  }

  void onGenerateAddress() async {
    try {
      final confirmed = await confirmDialog(context,
          title: "New Addresses", message: "Do you want to generate a new set of addresses? Previous addresses can still receive funds");
      if (!confirmed) return;
      if (!mounted) return;
      final dialog = await showMessage(context, "Please wait for the address generation\nCheck your Ledger", dismissable: false);
      await generateNextDindex(c: c); // This takes a while on the Ledger
      addresses = await getAddresses(uaPools: uaPools, c: c);
      dialog.dismiss();
      setState(() {});
    } on AnyhowException catch (e) {
      await showException(context, e.message);
    }
  }

  void onShowQR(String title, String text) {
    GoRouter.of(context).push("/qr", extra: {"title": title, "text": text});
  }

  void onViewAddresses() async {
    final txCounts = await fetchAddressTxCount(c: c);
    final availablePools = await getAccountPools(account: c.account, c: c);
    if (!mounted) return;
    await GoRouter.of(context).push("/addresses", extra: {
      'txCounts': txCounts,
      'availablePools': availablePools,
    });
  }

  void onSweep() async {
    await showTransparentScan(ref, context);
  }
}

class AddressesPage extends ConsumerStatefulWidget {
  final List<TAddressTxCount> txCounts;
  final int availablePools;

  const AddressesPage({super.key, required this.txCounts, required this.availablePools});

  @override
  ConsumerState<AddressesPage> createState() => _AddressesPageState();
}

class _AddressesPageState extends ConsumerState<AddressesPage> {
  int _usageFilter = 0; // 0=all, 1=used, 2=unused
  late Set<int> _selectedPools;

  static const _poolNames = {0: "Transparent", 1: "Sapling", 2: "Orchard"};
  static const _poolBits = {0: 1, 1: 2, 2: 4};

  @override
  void initState() {
    super.initState();
    _selectedPools = {};
    if (widget.availablePools & 1 != 0) _selectedPools.add(0);
    if (widget.availablePools & 2 != 0) _selectedPools.add(1);
    if (widget.availablePools & 4 != 0) _selectedPools.add(2);
  }

  bool _isPoolAvailable(int pool) => widget.availablePools & _poolBits[pool]! != 0;

  void _togglePool(int pool) {
    setState(() {
      if (_selectedPools.contains(pool)) {
        if (_selectedPools.length > 1) _selectedPools.remove(pool);
      } else {
        _selectedPools.add(pool);
      }
    });
  }

  List<TAddressTxCount> _filtered() => widget.txCounts.where((tx) {
    if (!_selectedPools.contains(tx.pool)) return false;
    switch (_usageFilter) {
      case 1: return tx.txCount > 0;
      case 2: return tx.txCount == 0;
      default: return true;
    }
  }).toList();

  @override
  Widget build(BuildContext context) {
    final pinlock = ref.watch(lifecycleProvider);
    if (pinlock.value ?? false) return PinLock();

    final filtered = _filtered();

    return Scaffold(
      appBar: AppBar(title: Text("Addresses")),
      body: Column(
        children: [
          Container(
            padding: EdgeInsets.symmetric(horizontal: 12, vertical: 8),
            color: Theme.of(context).colorScheme.secondaryContainer.withValues(alpha: 0.3),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Row(
                  children: [
                    Text("Show: ", style: TextStyle(fontWeight: FontWeight.w500)),
                    SizedBox(width: 4),
                    ChoiceChip(label: Text("All"), selected: _usageFilter == 0, onSelected: (_) => setState(() => _usageFilter = 0)),
                    SizedBox(width: 4),
                    ChoiceChip(label: Text("Used"), selected: _usageFilter == 1, onSelected: (_) => setState(() => _usageFilter = 1)),
                    SizedBox(width: 4),
                    ChoiceChip(label: Text("Unused"), selected: _usageFilter == 2, onSelected: (_) => setState(() => _usageFilter = 2)),
                  ],
                ),
                SizedBox(height: 6),
                Row(
                  children: [
                    Text("Pools: ", style: TextStyle(fontWeight: FontWeight.w500)),
                    for (final e in _poolNames.entries)
                      Padding(
                        padding: EdgeInsets.only(left: 4),
                        child: FilterChip(
                          label: Text(e.value),
                          selected: _selectedPools.contains(e.key),
                          onSelected: _isPoolAvailable(e.key) ? (_) => _togglePool(e.key) : null,
                        ),
                      ),
                  ],
                ),
              ],
            ),
          ),
          Expanded(
            child: ListView.builder(
              itemCount: filtered.length,
              itemBuilder: (context, index) {
                final tx = filtered[index];
                final scope = tx.scope == 0 ? "External" : "Change";
                final pool = _poolNames[tx.pool] ?? "Unknown";
                final lastUsed = tx.time > 0 ? timeToString(tx.time) : "Never";
                return ListTile(
                  title: CopyableText(tx.address),
                  subtitle: Text("$pool · $scope · Index ${tx.dindex} · ${tx.txCount} txs · $lastUsed"),
                  trailing: Text(zatToString(tx.amount)),
                );
              },
            ),
          ),
        ],
      ),
    );
  }
}
