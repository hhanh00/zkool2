import 'package:flutter_test/flutter_test.dart';
import 'package:zkool/services/votechain_failover.dart';
import 'package:zkool/src/rust/api/voting.dart';

VotingChainResponse response(int status, {String body = '{}', int? retryAfter}) {
  return VotingChainResponse(
    statusCode: status,
    body: body,
    retryAfterSecs: retryAfter == null ? null : BigInt.from(retryAfter),
  );
}

void main() {
  test('candidate ordering: caller list first, then config, deduplicated', () {
    final failover = VoteChainFailover(
      allServers: const ['a', 'b', 'c'],
      delay: (_) async {},
    );
    expect(
      failover.orderedCandidates(['c', 'd'], 'r1'),
      ['c', 'd', 'a', 'b'],
    );
  });

  test('remembers the last working URL per round and tries it first', () async {
    final calls = <String>[];
    final failover = VoteChainFailover(
      allServers: const ['a', 'b'],
      delay: (_) async {},
    );
    await failover.run(
      baseUrls: ['a', 'b'],
      roundId: 'r1',
      call: (url) async {
        calls.add(url);
        if (url == 'a') throw Exception('down');
        return response(200);
      },
    );
    expect(calls, ['a', 'b']);

    final again = await failover.run(
      baseUrls: ['a', 'b'],
      roundId: 'r1',
      call: (url) async {
        calls.add(url);
        return response(200);
      },
    );
    expect(again.statusCode, 200);
    // The remembered URL (b) is tried first and answers immediately.
    expect(calls, ['a', 'b', 'b']);
  });

  test('4xx is a final answer', () async {
    final failover = VoteChainFailover(delay: (_) async {});
    final res = await failover.run(
      baseUrls: ['a', 'b'],
      roundId: 'r1',
      call: (url) async => url == 'a' ? response(422, body: 'rejected') : fail('b must not be tried'),
    );
    expect(res.statusCode, 422);
  });

  test('404 is a final answer (not confirmed yet)', () async {
    final failover = VoteChainFailover(delay: (_) async {});
    final res = await failover.run(
      baseUrls: ['a'],
      roundId: 'r1',
      call: (_) async => response(404, body: '{"error":"tx not found"}'),
    );
    expect(res.statusCode, 404);
  });

  test('5xx rotates to the next server', () async {
    final failover = VoteChainFailover(delay: (_) async {});
    final res = await failover.run(
      baseUrls: ['a', 'b'],
      roundId: 'r1',
      call: (url) async => url == 'a' ? response(500) : response(200),
    );
    expect(res.statusCode, 200);
  });

  test('503 honors Retry-After and retries the same URL once', () async {
    final delays = <Duration>[];
    var calls = 0;
    final failover = VoteChainFailover(delay: (d) async => delays.add(d));
    final res = await failover.run(
      baseUrls: ['a'],
      roundId: 'r1',
      call: (_) async {
        calls++;
        return calls == 1
            ? response(503, retryAfter: 5)
            : response(200);
      },
    );
    expect(res.statusCode, 200);
    expect(delays, [const Duration(seconds: 5)]);
  });

  test('502 with a tx hash is returned as an answer', () async {
    final failover = VoteChainFailover(delay: (_) async {});
    final res = await failover.run(
      baseUrls: ['a'],
      roundId: 'r1',
      call: (_) async =>
          response(502, body: 'broadcast outcome unknown after retries; tx_hash=ABCDEF'),
    );
    expect(res.statusCode, 502);
    expect(res.body, contains('ABCDEF'));
  });

  test('502 without a hash rotates to the next server', () async {
    final failover = VoteChainFailover(delay: (_) async {});
    final res = await failover.run(
      baseUrls: ['a', 'b'],
      roundId: 'r1',
      call: (url) async => url == 'a' ? response(502, body: 'boom') : response(200),
    );
    expect(res.statusCode, 200);
  });

  test('all candidates failing raises TransientVoteChainException', () async {
    final failover = VoteChainFailover(delay: (_) async {});
    expect(
      () => failover.run(
        baseUrls: ['a', 'b'],
        roundId: 'r1',
        call: (_) async => throw Exception('down'),
      ),
      throwsA(isA<TransientVoteChainException>()),
    );
  });
}
