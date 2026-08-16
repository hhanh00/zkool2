import 'package:flutter_test/flutter_test.dart';
import 'package:zkool/services/votechain_confirmation.dart';

void main() {
  group('parseVoteChainTxConfirmation', () {
    test('parses a valid body with events', () {
      final conf = parseVoteChainTxConfirmation(
        '{"height": 12345, "code": 0, "events": [{"type": "delegate-vote"}]}',
      );
      expect(conf, isNotNull);
      expect(conf!.height, 12345);
      expect(conf.eventsJson, contains('delegate-vote'));
    });

    test('height 0 means not yet included', () {
      expect(
        parseVoteChainTxConfirmation('{"height": 0, "events": []}'),
        isNull,
      );
    });

    test('missing height is not a confirmation', () {
      expect(
        parseVoteChainTxConfirmation('{"code": 0, "events": []}'),
        isNull,
      );
    });

    test('null height is not a confirmation', () {
      expect(
        parseVoteChainTxConfirmation('{"height": null, "events": []}'),
        isNull,
      );
    });

    test('negative height is not a confirmation', () {
      expect(
        parseVoteChainTxConfirmation('{"height": -5, "events": []}'),
        isNull,
      );
    });

    test('numeric height is accepted', () {
      final conf =
          parseVoteChainTxConfirmation('{"height": 12.0, "events": []}');
      expect(conf, isNotNull);
      expect(conf!.height, 12);
    });

    test('malformed JSON is not a confirmation', () {
      expect(parseVoteChainTxConfirmation('not json'), isNull);
    });

    test('non-map body is not a confirmation', () {
      expect(parseVoteChainTxConfirmation('[1, 2, 3]'), isNull);
    });

    test('missing events defaults to an empty array', () {
      final conf =
          parseVoteChainTxConfirmation('{"height": 7, "code": 0}');
      expect(conf, isNotNull);
      expect(conf!.eventsJson, '[]');
    });
  });
}
