import 'dart:convert';

/// Parsed proof of block inclusion from a 200 GET tx/{hash} response.
class VoteChainTxConfirmation {
  /// JSON-encoded events array from the response (may be `"[]"`).
  final String eventsJson;

  /// Block height at which the transaction was included; always > 0.
  final int height;

  const VoteChainTxConfirmation({
    required this.eventsJson,
    required this.height,
  });
}

/// Parses a 200 body from the vote chain tx confirmation endpoint.
///
/// Returns null when the body does not prove block inclusion: malformed
/// JSON, a non-map body, a missing `height`, or `height <= 0`. Callers treat
/// null as "not confirmed yet" and keep polling.
VoteChainTxConfirmation? parseVoteChainTxConfirmation(String body) {
  final dynamic decoded;
  try {
    decoded = jsonDecode(body);
  } on FormatException {
    return null;
  }
  if (decoded is! Map<String, dynamic>) return null;
  final rawHeight = decoded['height'];
  // The chain reports height as a JSON string ("7163319"), sometimes a number.
  final height = rawHeight is int
      ? rawHeight
      : rawHeight is num
          ? rawHeight.toInt()
          : rawHeight is String
              ? int.tryParse(rawHeight) ?? 0
              : 0;
  if (height <= 0) return null;
  return VoteChainTxConfirmation(
    eventsJson: jsonEncode(decoded['events'] ?? const []),
    height: height,
  );
}
