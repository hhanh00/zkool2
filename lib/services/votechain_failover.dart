import 'package:zkool/src/rust/api/voting.dart';

import 'votechain_classify.dart';

/// Raised when every candidate vote-chain server failed with a transport
/// error or an unresolved 5xx — a transient, retryable condition from the
/// caller's perspective.
class TransientVoteChainException implements Exception {
  final String message;

  TransientVoteChainException(this.message);

  @override
  String toString() => message;
}

/// Raised when a re-broadcast was rejected because a nullifier it carries was
/// already spent on-chain: the original submission committed successfully but
/// its tx hash was never recorded locally. The caller reconciles the
/// confirmation from the commitment tree instead of failing the run.
class DuplicateNullifierOnResubmitException implements Exception {
  final String message;

  DuplicateNullifierOnResubmitException(this.message);

  @override
  String toString() => message;
}

/// A per-URL vote-chain call: transport failures throw, completed HTTP
/// responses (any status) are returned as `VotingChainResponse`.
typedef VoteChainCall = Future<VotingChainResponse> Function(String baseUrl);

/// Rotates a vote-chain call across the configured server list.
///
/// Policy per candidate URL, in order:
/// - transport error → next candidate;
/// - 2xx / 4xx → final answer (404 "not found" and 422 "rejected" are
///   answers, not failures);
/// - 503 → honor `Retry-After` and retry the same URL up to `max503Retries`
///   times (the vote-sdk warms its verifier cache behind a 503 gate);
/// - 502 → an answer only when the body carries a tx hash (the vote-sdk's
///   "broadcast outcome unknown; tx_hash=..." envelope); otherwise next;
/// - any other 5xx → next candidate.
///
/// Candidate order comes directly from the caller's `baseUrls` (already
/// randomised per-share by the voting library's CSPRNG), followed by any
/// remaining configured servers as fallback. All candidates failing raises
/// [TransientVoteChainException].
class VoteChainFailover {
  final List<String> allServers;

  /// Injectable sleep (tests pass a no-op).
  final Future<void> Function(Duration duration) delay;

  VoteChainFailover({
    this.allServers = const [],
    Future<void> Function(Duration duration)? delay,
  }) : delay = delay ?? Future<void>.delayed;

  /// Runs [call] with failover across `baseUrls` (caller's preferred URL
  /// first) plus [allServers].
  Future<VotingChainResponse> run({
    required List<String> baseUrls,
    required VoteChainCall call,
    int max503Retries = 1,
  }) async {
    Object? lastError;
    for (final url in orderedCandidates(baseUrls)) {
      var retries = 0;
      while (true) {
        final VotingChainResponse res;
        try {
          res = await call(url);
        } catch (e) {
          lastError = e;
          break; // transport failure: this server did not answer
        }
        final status = res.statusCode;
        if (status >= 200 && status < 300) {
          return res;
        }
        if (status >= 400 && status < 500) {
          // A definitive answer (404 not found, 422 rejected, 409 conflict).
          return res;
        }
        if (status == 502) {
          if (txHashFromVoteChainBody(res.body) != null) {
            return res;
          }
          lastError = Exception('vote chain 502 from $url: ${res.body}');
          break;
        }
        if (status == 503 && retries < max503Retries) {
          final wait = (res.retryAfterSecs?.toInt() ?? 2).clamp(1, 120);
          await delay(Duration(seconds: wait));
          retries++;
          continue;
        }
        lastError = Exception('vote chain HTTP $status from $url: ${res.body}');
        break;
      }
    }
    throw TransientVoteChainException(
      'all vote chain servers failed (${orderedCandidates(baseUrls).length} tried): $lastError',
    );
  }

  /// Candidate order: the caller's list (CSPRNG-randomised by the voting
  /// library per share), then the remaining configured servers; deduplicated.
  List<String> orderedCandidates(List<String> baseUrls) {
    final ordered = <String>[];
    void add(String? url) {
      if (url == null || url.isEmpty || ordered.contains(url)) return;
      ordered.add(url);
    }

    for (final url in baseUrls) {
      add(url);
    }
    for (final url in allServers) {
      add(url);
    }
    return ordered;
  }
}
