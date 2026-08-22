import 'dart:convert';

/// How a 422 (or 4xx rejection body) from the vote chain should be handled.
enum ChainRejectionKind {
  /// The chain rejected the submission because a nullifier it carries was
  /// already spent — for a resubmission of identical wire bytes this means
  /// the original transaction committed successfully (the client just never
  /// learned its hash).
  duplicateNullifier,

  /// Any other deterministic rejection; permanent, do not retry blindly.
  other,
}

/// Classifies a vote-chain rejection body.
///
/// The vote-sdk surfaces duplicate nullifiers through CheckTx logs like
/// `nullifier already spent`; every other rejection is surfaced as-is.
ChainRejectionKind classifyVoteChainRejection(String body) {
  final lower = body.toLowerCase();
  if (lower.contains("nullifier") &&
      (lower.contains("spent") ||
          lower.contains("spend") ||
          lower.contains("already"))) {
    return ChainRejectionKind.duplicateNullifier;
  }
  return ChainRejectionKind.other;
}

/// Extracts a tx hash from a vote-chain response body.
///
/// Accepted forms: the `tx_hash` field of a JSON `{tx_hash, code, log}`
/// envelope, and the `tx_hash=...` fragment of the 502
/// "broadcast outcome unknown after retries" message.
String? txHashFromVoteChainBody(String body) {
  try {
    final decoded = jsonDecode(body);
    if (decoded is Map<String, dynamic>) {
      final hash = decoded['tx_hash'];
      if (hash is String && hash.isNotEmpty) return hash;
    }
  } on FormatException {
    // Fall through to the substring form.
  }
  final match = RegExp(r"tx_hash=([0-9A-Fa-f]+)").firstMatch(body);
  return match?.group(1);
}
