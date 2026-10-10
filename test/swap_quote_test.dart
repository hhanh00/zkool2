import 'package:flutter_test/flutter_test.dart';
import 'package:zkool/services/swap_quote.dart';
import 'package:zkool/pages/swap.dart';
import 'package:zkool/src/rust/near_intents.dart' as ni;

void main() {
  test('swap requests use the correct assets, addresses and amount units in both directions', () {
    for (final direction in SwapDirection.values) {
      for (final mode in SwapAmountMode.values) {
        final outgoing = direction == SwapDirection.sendZec;
        final draft = SwapDraft(
            asset: SwapAsset.usdt,
            network: SwapNetwork.solana,
            direction: direction,
            amountMode: mode,
            amount: '1.25',
            recipient: outgoing ? 'external' : '',
            refundTo: outgoing ? null : 'external',
            slippageTolerance: 25);
        final request = buildSwapRequest(draft, 'usdt', 6, 'wallet', DateTime.utc(2026, 10, 10));
        expect(request.originAsset, outgoing ? 'nep141:zec.omft.near' : 'usdt');
        expect(request.destinationAsset, outgoing ? 'usdt' : 'nep141:zec.omft.near');
        expect(request.recipient, outgoing ? 'external' : 'wallet');
        expect(request.refundTo, outgoing ? 'wallet' : 'external');
        expect(request.amount, (mode == SwapAmountMode.send) == outgoing ? '125000000' : '1250000');
        expect(request.swapType, mode == SwapAmountMode.send ? ni.SwapType.exactInput : ni.SwapType.exactOutput);
        expect(request.slippageTolerance, 25);
        expect(request.dry, isTrue);
      }
    }
  });
  test('slippage converts percentages to basis points without rounding', () {
    expect(swapSlippageBasisPoints('1'), 100);
    expect(swapSlippageBasisPoints('0.25'), 25);
    expect(swapSlippageBasisPoints('0'), 0);
    expect(swapSlippageBasisPoints('100'), 10000);
    for (final value in ['', '-1', '100.01', '0.001', 'NaN']) {
      expect(() => swapSlippageBasisPoints(value), throwsFormatException);
    }
  });
  test('estimated cost uses exact USD subtraction and preserves negative differences', () {
    expect(swapEstimatedCostUsd('100.123456', '99.1'), '1.023456');
    expect(swapEstimatedCostUsd('1', '1.01'), '-0.01');
    expect(swapEstimatedCostUsd('1', '1'), '0');
    expect(swapEstimatedCostUsd(null, '1'), isNull);
    expect(swapEstimatedCostUsd('invalid', '1'), isNull);
    expect(swapEstimatedCostUsd('-1', '1'), isNull);
  });
  test('quote amounts preserve precision beyond floating point integer limits', () {
    expect(swapBaseUnits('123456789.123456789123456789', 18), '123456789123456789123456789');
    expect(swapDecimalAmount('123456789123456789123456789', 18), '123456789.123456789123456789');
    expect(swapBaseUnits('0.00000001', 8), '1');
    expect(swapDecimalAmount('1', 8), '0.00000001');
    expect(swapDecimalAmount('1000000', 6), '1');
  });
  test('amount conversion rejects truncation and invalid inputs', () {
    for (final amount in ['0', '-1', '1e6', '0.0000001']) {
      expect(() => swapBaseUnits(amount, 6), throwsFormatException);
    }
    expect(() => swapDecimalAmount('1.5', 6), throwsFormatException);
  });
}
