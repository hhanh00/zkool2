import 'package:flutter_test/flutter_test.dart';
import 'package:zkool/address_resolver.dart';
import 'package:zkool/src/rust/api/contacts.dart';

Contact contact(int id, String name, List<String> addresses) =>
    Contact(id: id, name: name, addresses: addresses, notes: '');

void main() {
  group('names are exact', () {
    final contacts = [
      contact(1, 'Alice', ['uAlice']),
      contact(2, 'alice', ['ualice']),
      contact(3, 'bob', ['ubob']),
      contact(4, 'carol', []),
    ];

    test('@name resolves the contact with exactly that name', () {
      expect(resolveContactName('@alice', contacts), 'ualice');
      expect(resolveContactName('@Alice', contacts), 'uAlice');
      expect(resolveContactName('@bob', contacts), 'ubob');
    });

    test('a different case is not a match', () {
      expect(resolveContactName('@BOB', contacts), isNull);
      expect(resolveContactName('@Bob', contacts), isNull);
    });

    test('no prefix, nothing after it, or no addresses resolve to nothing', () {
      expect(resolveContactName('bob', contacts), isNull);
      expect(resolveContactName('@', contacts), isNull);
      expect(resolveContactName('@ ', contacts), isNull);
      expect(resolveContactName('@carol', contacts), isNull);
    });

    test('surrounding whitespace is ignored', () {
      expect(resolveContactName('@ bob ', contacts), 'ubob');
    });
  });

  group('exact duplicates', () {
    final names = ['main', 'savings', 'main'];

    test('are ambiguous: not resolved, but countable', () {
      expect(uniqueNamed('main', names, (n) => n), isNull);
      expect(allNamed('main', names, (n) => n), hasLength(2));
      expect(uniqueNamed('savings', names, (n) => n), 'savings');
      expect(allNamed('nope', names, (n) => n), isEmpty);
    });
  });
}
