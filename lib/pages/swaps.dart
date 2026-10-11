import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import 'package:zkool/main.dart';
import 'package:zkool/services/swap_quote.dart' show swapDeadline, swapDecimalAmount;
import 'package:zkool/src/rust/api/near_intents.dart';
import 'package:zkool/src/rust/near_intents.dart';
import 'package:zkool/store.dart';
import 'package:zkool/utils.dart';
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
          if (swap.depositTxHash != null && swap.depositSubmittedAt == null) {
            await nearIntentsRecordDeposit(idSwap: swap.idSwap, txHash: swap.depositTxHash!, c: coin);
          }
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
                          final externalAsset = swap.originAsset == 'nep141:zec.omft.near' ? swap.destinationAsset : swap.originAsset;
                          final matches = assets.where((asset) => asset.assetId == externalAsset);
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
    final outgoing = swap.originAsset == 'nep141:zec.omft.near';
    final originSymbol = outgoing ? 'ZEC' : asset?.symbol;
    final destinationSymbol = outgoing ? asset?.symbol : 'ZEC';
    final originDecimals = outgoing ? 8 : asset?.decimals;
    final destinationDecimals = outgoing ? asset?.decimals : 8;
    final status = switch (swap.status) {
      'KNOWN_DEPOSIT_TX' => 'Deposit sent',
      'PENDING_DEPOSIT' => 'Waiting for deposit',
      'INCOMPLETE_DEPOSIT' => 'Incomplete deposit',
      'PROCESSING' => 'Processing',
      'SUCCESS' => 'Completed',
      'REFUNDED' => 'Refunded',
      'FAILED' => 'Failed',
      'EXPIRED' => 'Expired',
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
    if (originDecimals != null) amountIn ??= swapDecimalAmount(swap.amountIn, originDecimals);
    if (destinationDecimals != null) amountOut ??= swapDecimalAmount(swap.amountOut, destinationDecimals);
    return ExpansionTile(
      leading: Icon(switch (swap.status) {
        'SUCCESS' => Icons.check_circle_outline,
        'FAILED' => Icons.error_outline,
        'REFUNDED' => Icons.undo,
        'EXPIRED' => Icons.timer_off_outlined,
        _ => Icons.currency_exchange,
      }),
      title: Text('${originSymbol ?? 'Sending asset'} → ${destinationSymbol ?? 'Receiving asset'}'),
      subtitle: Text(
          '$status\nQuoted: ${amountIn == null ? '${swap.amountIn} base units' : '$amountIn ${originSymbol ?? ''}'} → ${amountOut == null ? '${swap.amountOut} base units' : '$amountOut ${destinationSymbol ?? ''}'}'),
      childrenPadding: const EdgeInsets.fromLTRB(16, 0, 16, 16),
      expandedCrossAxisAlignment: CrossAxisAlignment.start,
      children: [
        _detail(
            'Minimum received',
            swap.minAmountOut == null
                ? 'Unavailable'
                : destinationDecimals == null
                    ? '${swap.minAmountOut} base units'
                    : '${swapDecimalAmount(swap.minAmountOut!, destinationDecimals)} $destinationSymbol'),
        _detail('Slippage tolerance', '${swapDecimalAmount(swap.slippageTolerance.toString(), 2)}%'),
        _detail('Receive asset', swap.destinationAsset),
        if (asset != null) _detail(outgoing ? 'Receiving network' : 'Deposit network', asset!.blockchain),
        _detail('Receiving address', swap.recipient),
        _detail('Deposit address', swap.depositAddress),
        if (swap.depositMemo != null) _detail('Deposit memo', swap.depositMemo!),
        if (swap.depositTxHash != null) _detail('Deposit transaction', swap.depositTxHash!),
        _detail('Refund address', swap.refundTo),
        _detail('Deadline', exactTimeToString(swapDeadline(swap).millisecondsSinceEpoch ~/ 1000)),
      ],
    );
  }

  Widget _detail(String label, String value) => Padding(padding: const EdgeInsets.only(top: 8), child: SelectableText('$label: $value'));
}
