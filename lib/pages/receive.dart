import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge.dart';
import 'package:gap/gap.dart';
import 'package:go_router/go_router.dart';

import 'package:zkool/main.dart';
import 'package:zkool/pages/sweep.dart';
import 'package:zkool/src/rust/api/account.dart';
import 'package:zkool/src/rust/api/coin.dart';
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
    final txCounts = await fetchAddressTxCount(c: c, aggregate: false);
    final availablePools = await getAccountPools(account: c.account, c: c);
    if (!mounted) return;
    await GoRouter.of(context).push("/addresses", extra: {
      'txCounts': txCounts,
      'availablePools': availablePools,
      'c': c,
    });
  }

  void onSweep() async {
    await showTransparentScan(ref, context);
  }
}

class AddressesPage extends ConsumerStatefulWidget {
  final List<TAddressTxCount> txCounts;
  final int availablePools;
  final Coin c;

  const AddressesPage({super.key, required this.txCounts, required this.availablePools, required this.c});

  @override
  ConsumerState<AddressesPage> createState() => _AddressesPageState();
}

class _AddressesPageState extends ConsumerState<AddressesPage> {
  int _usageFilter = 0; // 0=all, 1=used, 2=unused
  int _scopeFilter = 0; // 0=all, 1=external, 2=change
  bool _aggregate = false;
  late Set<int> _selectedPools;
  late List<TAddressTxCount> _txCounts;

  static const _poolBits = {0: 1, 1: 2, 2: 4};

  @override
  void initState() {
    super.initState();
    _txCounts = widget.txCounts;
    _selectedPools = {};
    if (widget.availablePools & 1 != 0) _selectedPools.add(0);
    if (widget.availablePools & 2 != 0) _selectedPools.add(1);
    if (widget.availablePools & 4 != 0) _selectedPools.add(2);
  }

  Future<void> _toggleAggregate(bool v) async {
    setState(() => _aggregate = v);
    final txCounts = await fetchAddressTxCount(c: widget.c, aggregate: v);
    if (mounted) setState(() => _txCounts = txCounts);
  }

  static const _poolIcons = {0: Icons.visibility, 1: Icons.eco, 2: Icons.park};

  Widget _poolIcon(int pool, double size) {
    final icon = _poolIcons[pool] ?? Icons.help_outline;
    final color = switch (pool) { 0 => Colors.red, 1 => Colors.orange, 2 => Colors.green, _ => Colors.grey };
    return Icon(icon, size: size, color: color);
  }

  ButtonStyle get _segmentedStyle => SegmentedButton.styleFrom(
    backgroundColor: Colors.grey[200],
    foregroundColor: Colors.red,
    selectedForegroundColor: Colors.white,
    selectedBackgroundColor: Colors.green,
  );

  List<TAddressTxCount> _filtered() => _txCounts.where((tx) {
    if (!_selectedPools.contains(tx.pool)) return false;
    switch (_scopeFilter) {
      case 1: if (tx.scope != 0) return false;
      case 2: if (tx.scope != 1) return false;
    }
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
                    SizedBox(width: 56, child: Text("UA", style: TextStyle(fontWeight: FontWeight.w500, fontSize: 12))),
                    SegmentedButton<bool>(
                      style: _segmentedStyle,
                      showSelectedIcon: false,
                      segments: const [
                        ButtonSegment(value: false, label: Text("Off")),
                        ButtonSegment(value: true, label: Text("On")),
                      ],
                      selected: {_aggregate},
                      onSelectionChanged: (s) => _toggleAggregate(s.first),
                    ),
                  ],
                ),
                SizedBox(height: 6),
                Row(
                  children: [
                    SizedBox(width: 56, child: Text("Show", style: TextStyle(fontWeight: FontWeight.w500, fontSize: 12))),
                    SegmentedButton<int>(
                      style: _segmentedStyle,
                      showSelectedIcon: false,
                      segments: const [
                        ButtonSegment(value: 0, label: Text("All")),
                        ButtonSegment(value: 1, label: Text("Used")),
                        ButtonSegment(value: 2, label: Text("Unused")),
                      ],
                      selected: {_usageFilter},
                      onSelectionChanged: (s) => setState(() => _usageFilter = s.first),
                    ),
                  ],
                ),
                SizedBox(height: 6),
                Row(
                  children: [
                    SizedBox(width: 56, child: Text("Scope", style: TextStyle(fontWeight: FontWeight.w500, fontSize: 12))),
                    SegmentedButton<int>(
                      style: _segmentedStyle,
                      showSelectedIcon: false,
                      segments: const [
                        ButtonSegment(value: 0, label: Text("All")),
                        ButtonSegment(value: 1, label: Text("External")),
                        ButtonSegment(value: 2, label: Text("Change")),
                      ],
                      selected: {_scopeFilter},
                      onSelectionChanged: (s) => setState(() => _scopeFilter = s.first),
                    ),
                  ],
                ),
                if (!_aggregate) ...[
                  SizedBox(height: 6),
                  Row(
                    children: [
                      SizedBox(width: 56, child: Text("Pools", style: TextStyle(fontWeight: FontWeight.w500, fontSize: 12))),
                      PoolSelect(
                        enabled: widget.availablePools,
                        initialValue: _selectedPools.fold(0, (acc, p) => acc | _poolBits[p]!),
                        onChanged: (v) => setState(() {
                        _selectedPools = {};
                        if (v & 1 != 0) _selectedPools.add(0);
                        if (v & 2 != 0) _selectedPools.add(1);
                        if (v & 4 != 0) _selectedPools.add(2);
                      }),
                    ),
                  ],
                ),
                ],
              ],
            ),
          ),
          Expanded(
            child: ListView.builder(
              itemCount: filtered.length,
              itemBuilder: (context, index) {
                final tx = filtered[index];
                final lastUsed = tx.time > 0 ? timeToString(tx.time) : "Never";
                final trimmed = tx.address.length > 20
                    ? '${tx.address.substring(0, 10)}...${tx.address.substring(tx.address.length - 8)}'
                    : tx.address;
                return ListTile(
                  leading: Row(
                    mainAxisSize: MainAxisSize.min,
                    children: _aggregate
                        ? [Icon(tx.scope == 0 ? Icons.call_made : Icons.sync, size: 20, color: Colors.grey[600])]
                        : [
                            _poolIcon(tx.pool, 20),
                            SizedBox(width: 4),
                            Icon(tx.scope == 0 ? Icons.arrow_outward : Icons.sync, size: 20),
                          ],
                  ),
                  title: Text(trimmed, style: TextStyle(fontFamily: "monospace")),
                  subtitle: Text(
                    "${_aggregate ? "Unified · " : ""}Idx ${tx.dindex} · ${tx.txCount} txs${tx.time > 0 ? " · $lastUsed" : ""}",
                    style: TextStyle(fontSize: 13),
                  ),
                  trailing: Text(zatToString(tx.amount)),
                  onTap: () => copyToClipboard(tx.address),
                );
              },
            ),
          ),
        ],
      ),
    );
  }
}
