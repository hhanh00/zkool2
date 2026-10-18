import 'package:flutter_test/flutter_test.dart';
import 'package:intl/intl.dart';
import 'package:zkool/utils.dart';

void main() {
  group('formatLocale', () {
    test('keeps a locale intl has data for, narrowing a region it lacks to the language', () {
      expect(formatLocale('en_US'), 'en_US');
      expect(formatLocale('de'), 'de');
      expect(formatLocale('fr_FR'), 'fr');
    });

    test('falls back to en_US for the C and POSIX locales of a bare session', () {
      expect(formatLocale('C'), 'en_US');
      expect(formatLocale('POSIX'), 'en_US');
      expect(formatLocale('C.UTF-8'), 'en_US');
    });

    test('the fallback formats amounts the way the app does', () {
      final formatter = NumberFormat.decimalPatternDigits(locale: formatLocale('C'), decimalDigits: 8);
      expect(formatter.format(1.5), '1.50000000');
    });
  });
}
