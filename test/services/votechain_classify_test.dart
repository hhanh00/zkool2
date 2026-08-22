import 'package:flutter_test/flutter_test.dart';
import 'package:zkool/services/votechain_classify.dart';

void main() {
  group('classifyVoteChainRejection', () {
    test('detects duplicate nullifier rejections', () {
      expect(
        classifyVoteChainRejection('nullifier already spent'),
        ChainRejectionKind.duplicateNullifier,
      );
      expect(
        classifyVoteChainRejection(
          '{"code":1,"log":"nullifier already spent"}',
        ),
        ChainRejectionKind.duplicateNullifier,
      );
      expect(
        classifyVoteChainRejection(
          'failed to execute message; nullifier spend check failed',
        ),
        ChainRejectionKind.duplicateNullifier,
      );
    });

    test('classifies other rejections as permanent', () {
      expect(
        classifyVoteChainRejection('vote round is not active'),
        ChainRejectionKind.other,
      );
      expect(
        classifyVoteChainRejection('{"code":2,"log":"bad proof"}'),
        ChainRejectionKind.other,
      );
      expect(classifyVoteChainRejection(''), ChainRejectionKind.other);
    });
  });

  group('txHashFromVoteChainBody', () {
    test('extracts tx_hash from the JSON envelope', () {
      expect(
        txHashFromVoteChainBody(
          '{"tx_hash":"ABCDEF123456","code":0,"log":""}',
        ),
        'ABCDEF123456',
      );
    });

    test('extracts tx_hash from the 502 unknown-outcome message', () {
      expect(
        txHashFromVoteChainBody(
          'broadcast outcome unknown after retries; tx_hash=ABCDEF123456',
        ),
        'ABCDEF123456',
      );
    });

    test('returns null when no hash is present', () {
      expect(txHashFromVoteChainBody('{}'), isNull);
      expect(txHashFromVoteChainBody('not json at all'), isNull);
      expect(txHashFromVoteChainBody(''), isNull);
    });
  });
}
