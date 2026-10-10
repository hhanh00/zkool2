import 'package:zkool/pages/swap_summary.dart';
import 'package:zkool/src/rust/api/coin.dart';
import 'package:zkool/src/rust/api/near_intents.dart';
import 'package:zkool/src/rust/api/pay.dart';

sealed class ContinuationContext {
  const ContinuationContext();
}

final class SwapContext extends ContinuationContext {
  final SwapSummaryArgs summary;
  const SwapContext(this.summary);

  Future<SwapSummaryArgs> complete(String txHash, Coin coin) async {
    var swap = summary.swap;
    String? notificationError;
    try {
      await nearIntentsRecordDeposit(idSwap: swap.idSwap, txHash: txHash, c: coin);
    } catch (error) {
      notificationError = error.toString();
    }
    try {
      final swaps = await nearIntentsListSwaps(pendingOnly: false, c: coin);
      swap = swaps.firstWhere((item) => item.idSwap == swap.idSwap);
    } catch (error) {
      notificationError ??= error.toString();
    }
    return (
      swap: swap,
      symbol: summary.symbol,
      decimals: summary.decimals,
      originSymbol: summary.originSymbol,
      originDecimals: summary.originDecimals,
      depositNetwork: summary.depositNetwork,
      notificationError: notificationError,
    );
  }
}

class TxPageArgs {
  final PcztPackage pczt;
  final ContinuationContext continuation;
  const TxPageArgs(this.pczt, this.continuation);
}
