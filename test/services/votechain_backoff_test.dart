import 'package:flutter_test/flutter_test.dart';
import 'package:zkool/services/votechain_backoff.dart';

void main() {
  test('voteChainRetryDelay follows the 30/60/120 schedule', () {
    expect(voteChainRetryDelay(1), const Duration(seconds: 30));
    expect(voteChainRetryDelay(2), const Duration(seconds: 60));
    expect(voteChainRetryDelay(3), const Duration(seconds: 120));
    expect(voteChainRetryDelay(10), const Duration(seconds: 120));
  });

  test('max auto retries is bounded', () {
    expect(voteChainMaxAutoRetries, 3);
  });
}
