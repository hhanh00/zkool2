/// Delay before the next automatic retry of a transiently failed voting run.
///
/// Deterministic (unlike the synchronizer's randomized backoff) so tests and
/// the status page can predict the retry schedule: 30 s, 60 s, then 120 s for
/// any later attempt.
Duration voteChainRetryDelay(int attempt) {
  switch (attempt) {
    case 1:
      return const Duration(seconds: 30);
    case 2:
      return const Duration(seconds: 60);
    default:
      return const Duration(seconds: 120);
  }
}

/// Maximum automatic retries for a transiently failed voting run.
const voteChainMaxAutoRetries = 3;
