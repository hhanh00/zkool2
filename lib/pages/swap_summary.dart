import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import 'package:zkool/main.dart';
import 'package:zkool/services/swap_quote.dart';
import 'package:zkool/src/rust/near_intents.dart';
import 'package:zkool/store.dart';
import 'package:zkool/utils.dart';

class SwapSummaryPage extends ConsumerWidget {
  final SavedSwap swap;
  final String symbol;
  final int decimals;

  const SwapSummaryPage({required this.swap, required this.symbol, required this.decimals, super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
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
                leading: const Icon(Icons.currency_exchange),
                title: Text('ZEC → $symbol'),
                subtitle: const Text('Swap created. Awaiting deposit.')),
            ListTile(contentPadding: EdgeInsets.zero, title: const Text('You send'), subtitle: Text('${swapDecimalAmount(swap.amountIn, 8)} ZEC')),
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
