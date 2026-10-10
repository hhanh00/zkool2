import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import 'package:zkool/main.dart';
import 'package:zkool/pages/swaps.dart';
import 'package:zkool/src/rust/api/near_intents.dart';
import 'package:zkool/widgets/error_display.dart';
import 'package:zkool/services/swap_quote.dart';
import 'package:zkool/src/rust/near_intents.dart';
import 'package:zkool/store.dart';
import 'package:zkool/utils.dart';

typedef SwapSummaryArgs = ({
  SavedSwap swap,
  String symbol,
  int decimals,
  String originSymbol,
  int originDecimals,
  String? depositNetwork,
  String? notificationError,
});

class SwapSummaryPage extends ConsumerStatefulWidget {
  final SavedSwap swap;
  final String symbol;
  final int decimals;
  final String originSymbol;
  final int originDecimals;
  final String? depositNetwork;
  final String? notificationError;

  const SwapSummaryPage(
      {required this.swap,
      required this.symbol,
      required this.decimals,
      this.originSymbol = 'ZEC',
      this.originDecimals = 8,
      this.depositNetwork,
      this.notificationError,
      super.key});

  @override
  ConsumerState<SwapSummaryPage> createState() => _SwapSummaryPageState();
}

class _SwapSummaryPageState extends ConsumerState<SwapSummaryPage> {
  late SavedSwap _swap = widget.swap;
  late String? _notificationError = widget.notificationError;
  final _coin = coinContext.coin;
  Timer? _timer;
  bool _refreshing = false;
  Object? _error;

  bool get _completed => const ['SUCCESS', 'REFUNDED', 'FAILED'].contains(_swap.status);

  @override
  void initState() {
    super.initState();
    unawaited(_refresh());
  }

  @override
  void dispose() {
    _timer?.cancel();
    super.dispose();
  }

  Future<void> _refresh() async {
    if (!mounted || _refreshing || _completed) return;
    _timer?.cancel();
    setState(() => _refreshing = true);
    Object? error;
    try {
      if (_swap.depositTxHash != null && _swap.depositSubmittedAt == null) {
        try {
          await nearIntentsRecordDeposit(idSwap: _swap.idSwap, txHash: _swap.depositTxHash!, c: _coin);
          _notificationError = null;
        } catch (e) {
          _notificationError = e.toString();
        }
      }
      if (!mounted) return;
      await nearIntentsRefreshSwapStatus(idSwap: _swap.idSwap, c: _coin);
      if (!mounted) return;
      final swaps = await nearIntentsListSwaps(pendingOnly: false, c: _coin);
      if (!mounted) return;
      _swap = swaps.firstWhere((swap) => swap.idSwap == _swap.idSwap);
      ref.invalidate(savedSwapsProvider);
    } catch (e) {
      error = e;
    } finally {
      if (mounted) {
        setState(() {
          _refreshing = false;
          _error = error;
        });
        if (!_completed) {
          _timer = Timer(const Duration(seconds: 5), () => unawaited(_refresh()));
        }
      }
    }
  }

  String get _status => switch (_swap.status) {
        'SUCCESS' => 'Swap completed.',
        'REFUNDED' => 'Swap refunded to your refund address.',
        'FAILED' => 'Swap failed.',
        'PROCESSING' => 'Deposit received. Processing swap…',
        'KNOWN_DEPOSIT_TX' => 'Deposit reported. Waiting for confirmation…',
        'INCOMPLETE_DEPOSIT' => 'Incomplete deposit received.',
        'PENDING_DEPOSIT' || null => _swap.depositTxHash != null
            ? 'Deposit sent. Waiting for 1Click to confirm it…'
            : widget.depositNetwork == null
                ? 'Waiting for deposit…'
                : 'Send the deposit from your ${widget.depositNetwork} wallet.',
        final status => status,
      };

  @override
  Widget build(BuildContext context) {
    final swap = _swap;
    final symbol = widget.symbol;
    final decimals = widget.decimals;
    final originSymbol = widget.originSymbol;
    final originDecimals = widget.originDecimals;
    final depositNetwork = widget.depositNetwork;
    if (ref.watch(lifecycleProvider).value ?? false) return PinLock();
    void dismiss() => GoRouter.of(context).go('/');

    return PopScope(
      canPop: false,
      onPopInvokedWithResult: (didPop, result) {
        if (!didPop) dismiss();
      },
      child: Scaffold(
        appBar: AppBar(title: const Text('Swap summary'), leading: BackButton(onPressed: dismiss)),
        body: ListView(
          padding: const EdgeInsets.all(16),
          children: [
            ListTile(
                contentPadding: EdgeInsets.zero,
                leading: Icon(switch (swap.status) {
                  'SUCCESS' => Icons.check_circle_outline,
                  'REFUNDED' => Icons.undo,
                  'FAILED' => Icons.error_outline,
                  _ => Icons.currency_exchange,
                }),
                title: Text('$originSymbol → $symbol'),
                subtitle: Text(_status)),
            if (!_completed) const LinearProgressIndicator(),
            if (_error != null) ErrorCard(error: _error!, onRetry: _refreshing ? null : _refresh),
            if (swap.depositTxHash != null)
              ListTile(contentPadding: EdgeInsets.zero, title: const Text('Deposit transaction'), subtitle: SelectableText(swap.depositTxHash!)),
            if (_notificationError != null && !_completed)
              const ListTile(
                  contentPadding: EdgeInsets.zero, title: Text('Deposit sent; 1Click notification pending.'), subtitle: Text('Retrying automatically…')),
            ListTile(
                contentPadding: EdgeInsets.zero,
                title: const Text('You send'),
                subtitle: Text('${swapDecimalAmount(swap.amountIn, originDecimals)} $originSymbol')),
            ListTile(
                contentPadding: EdgeInsets.zero, title: const Text('You receive'), subtitle: Text('${swapDecimalAmount(swap.amountOut, decimals)} $symbol')),
            ListTile(contentPadding: EdgeInsets.zero, title: const Text('Receiving address'), subtitle: SelectableText(swap.recipient)),
            ListTile(
                contentPadding: EdgeInsets.zero,
                title: const Text('Minimum received'),
                subtitle: Text(swap.minAmountOut == null ? 'Unavailable' : '${swapDecimalAmount(swap.minAmountOut!, decimals)} $symbol')),
            ListTile(
                contentPadding: EdgeInsets.zero,
                title: const Text('Slippage tolerance'),
                subtitle: Text('${swapDecimalAmount(swap.slippageTolerance.toString(), 2)}%')),
            ListTile(
                contentPadding: EdgeInsets.zero,
                title: const Text('Swap deadline'),
                subtitle: Text(exactTimeToString(swapDeadline(swap).millisecondsSinceEpoch ~/ 1000))),
            ListTile(contentPadding: EdgeInsets.zero, title: const Text('Deposit address'), subtitle: SelectableText(swap.depositAddress)),
            if (depositNetwork != null) ListTile(contentPadding: EdgeInsets.zero, title: const Text('Deposit network'), subtitle: Text(depositNetwork)),
            ListTile(contentPadding: EdgeInsets.zero, title: const Text('Refund address'), subtitle: SelectableText(swap.refundTo)),
            if (swap.depositMemo != null)
              ListTile(contentPadding: EdgeInsets.zero, title: const Text('Deposit memo'), subtitle: SelectableText(swap.depositMemo!)),
          ],
        ),
        bottomNavigationBar: SafeArea(
          child: Padding(
            padding: const EdgeInsets.all(16),
            child: SizedBox(
              width: double.infinity,
              child: ElevatedButton(onPressed: dismiss, child: const Text('Dismiss')),
            ),
          ),
        ),
      ),
    );
  }
}
