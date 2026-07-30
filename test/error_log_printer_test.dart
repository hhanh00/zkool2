import 'package:flutter_test/flutter_test.dart';
import 'package:zkool/error_log_printer.dart';

void main() {
  test('omits unresolved Rust frames and preserves useful error details', () {
    const error = '''
transport error

Caused by:
    0: connection error
    1: peer closed connection

Stack backtrace:
   0: <unknown>
   1: <unknown>
  12: resolved_function
  13: <unknown>
''';

    expect(
      omitUnknownRustFrames(error),
      '''
transport error

Caused by:
    0: connection error
    1: peer closed connection

Stack backtrace:
  12: resolved_function
''',
    );
  });
}
