import 'package:flutter_test/flutter_test.dart';
import 'package:zkool/services/swap_quote.dart';

void main() {
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
