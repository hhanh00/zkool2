import 'package:flutter_test/flutter_test.dart';
import 'package:zkool/utils.dart';

void main() {
  group('compactAnyhowError', () {
    test('joins the context chain and drops the stack backtrace', () {
      const error = '''
transport error

Caused by:
    0: connection error
    1: peer closed connection

Stack backtrace:
   0: <unknown>
  12: resolved_function
''';
      expect(
        compactAnyhowError(error),
        'transport error: connection error: peer closed connection',
      );
    });

    test('handles a single unnumbered cause (the DKG locked-note case)', () {
      const error = '''
plan_transaction in DKG publish

Caused by:
    No feasible note selection found

Stack backtrace:
   0: <unknown>
''';
      expect(
        compactAnyhowError(error),
        'plan_transaction in DKG publish: No feasible note selection found',
      );
    });

    test('returns a single-line message unchanged', () {
      expect(compactAnyhowError('No feasible note selection found'),
          'No feasible note selection found');
    });

    test('ignores stray backtrace frames when there is no Caused by block', () {
      const error = '''
no rows returned by a query that expected to return at least one row
   1: anyhow::error::from
   2: rlz::frost::do_dkg
''';
      expect(
        compactAnyhowError(error),
        'no rows returned by a query that expected to return at least one row',
      );
    });
  });
}
