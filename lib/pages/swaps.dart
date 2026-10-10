import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import 'package:zkool/main.dart';
import 'package:zkool/src/rust/api/near_intents.dart';
import 'package:zkool/src/rust/near_intents.dart';
import 'package:zkool/store.dart';
import 'package:zkool/widgets/error_display.dart';

final savedSwapsProvider = FutureProvider.autoDispose<List<SavedSwap>>((ref) {
  final account = ref.watch(selectedAccountIdProvider);
  return nearIntentsListSwaps(pendingOnly: false, c: coinContext.coin.copyWith(account: account));
});

final swapAssetsProvider = FutureProvider.autoDispose<List<SwapAsset>>((ref) {
  ref.watch(selectedAccountIdProvider);
  return nearIntentsAssets(c: coinContext.coin);
});

class SwapsPage extends ConsumerStatefulWidget {
  const SwapsPage({super.key});

  @override
  ConsumerState<SwapsPage> createState() => _SwapsPageState();
}

class _SwapsPageState extends ConsumerState<SwapsPage> {
  bool _showCompleted = false;
  bool _refreshing = false;
  Object? _refreshError;

  Future<void> _refresh() async {
    if (_refreshing) return;
    final coin = coinContext.coin.copyWith(account: ref.read(selectedAccountIdProvider));
    setState(() {
      _refreshing = true;
      _refreshError = null;
    });
    Object? error;
    try {
      final swaps = await nearIntentsListSwaps(pendingOnly: true, c: coin);
      for (final swap in swaps) {
        if (!mounted || ref.read(selectedAccountIdProvider) != coin.account) break;
        try {
          await nearIntentsRefreshSwapStatus(idSwap: swap.idSwap, c: coin);
        } catch (e) {
          error ??= e;
        }
      }
    } catch (e) {
      error = e;
    } finally {
      if (mounted) {
        ref.invalidate(savedSwapsProvider);
        setState(() {
          _refreshing = false;
          _refreshError = ref.read(selectedAccountIdProvider) == coin.account ? error : null;
        });
      }
    }
  }

  @override
  Widget build(BuildContext context) {
    if (ref.watch(lifecycleProvider).value ?? false) return PinLock();
    if (!(ref.watch(appSettingsProvider).value?.expertMode ?? false)) {
      return Scaffold(
          appBar: AppBar(title: const Text('Swaps')),
          body: const Padding(padding: EdgeInsets.all(16), child: Text('Enable expert mode in Settings to use swaps.')));
    }
    final swaps = ref.watch(savedSwapsProvider);
    final assets = ref.watch(swapAssetsProvider).value ?? <SwapAsset>[];
    return Scaffold(
      appBar: AppBar(title: const Text('Swaps'), actions: [
        IconButton(tooltip: 'Refresh swap status', onPressed: _refreshing ? null : _refresh, icon: const Icon(Icons.refresh)),
        IconButton(tooltip: 'New swap', onPressed: () => context.push('/swap'), icon: const Icon(Icons.add)),
      ]),
      body: Column(children: [
        if (_refreshing) const LinearProgressIndicator(),
        SwitchListTile(title: const Text('Show completed swaps'), value: _showCompleted, onChanged: (value) => setState(() => _showCompleted = value)),
        if (_refreshError != null) ErrorCard(error: _refreshError!, onRetry: _refreshing ? null : _refresh),
        Expanded(
            child: swaps.when(
          loading: () => const Center(child: CircularProgressIndicator()),
          error: (error, stack) => SingleChildScrollView(child: ErrorCard(error: error, stackTrace: stack, onRetry: () => ref.invalidate(savedSwapsProvider))),
          data: (items) {
            final visible = items.where((s) => _showCompleted || s.completedAt == null).toList();
            return RefreshIndicator(
                onRefresh: _refresh,
                child: ListView(
                  physics: const AlwaysScrollableScrollPhysics(),
                  children: visible.isEmpty
                      ? [
                          Padding(
                              padding: const EdgeInsets.all(24),
                              child: Text(_showCompleted ? 'No swaps yet.' : 'No swaps in progress.', textAlign: TextAlign.center))
                        ]
                      : visible.map((swap) {
                          final matches = assets.where((asset) => asset.assetId == swap.destinationAsset);
                          return _SwapTile(swap: swap, asset: matches.isEmpty ? null : matches.first);
                        }).toList(),
                ));
          },
        )),
      ]),
    );
  }
}

class _SwapTile extends StatelessWidget {
  final SavedSwap swap;
  final SwapAsset? asset;
  const _SwapTile({required this.swap, this.asset});

  @override
  Widget build(BuildContext context) {
    final status = switch (swap.status) {
      'KNOWN_DEPOSIT_TX' => 'Deposit sent',
      'PENDING_DEPOSIT' => 'Waiting for deposit',
      'INCOMPLETE_DEPOSIT' => 'Incomplete deposit',
      'PROCESSING' => 'Processing',
      'SUCCESS' => 'Completed',
      'REFUNDED' => 'Refunded',
      'FAILED' => 'Failed',
      null => 'Status not checked',
      final value => value,
    };
    String? amountIn;
    String? amountOut;
    try {
      final quote = (jsonDecode(swap.quoteResponse) as Map<String, dynamic>)['quote'] as Map<String, dynamic>;
      amountIn = quote['amountInFormatted'] as String?;
      amountOut = quote['amountOutFormatted'] as String?;
    } catch (_) {
      // Older or incomplete snapshots still display their exact base units.
    }
    return ExpansionTile(
      leading: Icon(switch (swap.status) {
        'SUCCESS' => Icons.check_circle_outline,
        'FAILED' => Icons.error_outline,
        'REFUNDED' => Icons.undo,
        _ => Icons.currency_exchange,
      }),
      title: Text('ZEC → ${asset?.symbol ?? 'Receiving asset'}'),
      subtitle: Text(
          '$status\nQuoted: ${amountIn == null ? '${swap.amountIn} base units' : '$amountIn ZEC'} → ${amountOut ?? '${swap.amountOut} base units'}${asset == null ? '' : ' ${asset!.symbol}'}'),
      childrenPadding: const EdgeInsets.fromLTRB(16, 0, 16, 16),
      expandedCrossAxisAlignment: CrossAxisAlignment.start,
      children: [
        _detail('Receive asset', swap.destinationAsset),
        if (asset != null) _detail('Receiving network', asset!.blockchain),
        _detail('Receiving address', swap.recipient),
        _detail('Deposit address', swap.depositAddress),
        if (swap.depositMemo != null) _detail('Deposit memo', swap.depositMemo!),
        if (swap.depositTxHash != null) _detail('Deposit transaction', swap.depositTxHash!),
        _detail('Refund address', swap.refundTo),
        _detail('Deadline', swap.deadline),
      ],
    );
  }

  Widget _detail(String label, String value) => Padding(padding: const EdgeInsets.only(top: 8), child: SelectableText('$label: $value'));
}
