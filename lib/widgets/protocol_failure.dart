import 'package:flutter/material.dart';

/// The last failed pass of a protocol driver (DKG or FROST signing): what
/// went wrong, how many passes in a row have failed, and when the last one
/// did. A pass runs once per block, so the count is about how many blocks
/// the protocol has been stuck for.
class ProtocolFailure {
  final String message;
  final int count;
  final DateTime last;
  /// Replaces the "last attempt failed" headline, for a stall that is not
  /// a failure, such as a funding account that needs topping up.
  final String? headline;

  const ProtocolFailure(this.message, {this.count = 1, required this.last, this.headline});

  /// This failure followed by another one.
  ProtocolFailure again(String message) => ProtocolFailure(message, count: count + 1, last: DateTime.now());

  static ProtocolFailure first(String message) => ProtocolFailure(message, last: DateTime.now());

  /// The transaction being signed is about to expire.
  static ProtocolFailure expiringSoon(int blocksLeft) => ProtocolFailure(
        "The network drops a transaction past its expiry height. If the group cannot finish in "
        "time, prepare a new transaction and sign it again.",
        last: DateTime.now(),
        headline: "This transaction expires in $blocksLeft block${blocksLeft == 1 ? "" : "s"}.",
      );

  /// The funding account cannot pay for the next protocol message.
  static ProtocolFailure shortFunds(String balance, String cost) => ProtocolFailure(
        "The funding account holds $balance ZEC; a protocol message costs about $cost ZEC. "
        "Add funds to it to continue.",
        last: DateTime.now(),
        headline: "Not enough funds for the next message. It retries on the next block.",
      );
}

/// "Commitments: 2 of 3 · waiting for 4 min": a wait's progress and how
/// long the current status has lasted. The pages refresh it with every
/// pass, once a block.
String waitLine(String progress, DateTime since) {
  final d = DateTime.now().difference(since);
  final ago = d.inMinutes < 1
      ? "under a minute"
      : d.inHours < 1
          ? "${d.inMinutes} min"
          : "${d.inHours} h ${d.inMinutes % 60} min";
  return "$progress · waiting for $ago";
}

/// Shows a [ProtocolFailure] under a protocol page's status, until the next
/// pass succeeds. A snackbar flashes by once per block; this stays, so a
/// stall and its cause are visible whenever the page is looked at.
class ProtocolFailureLine extends StatelessWidget {
  final ProtocolFailure failure;

  const ProtocolFailureLine(this.failure, {super.key});

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final t = theme.textTheme;
    final since = DateTime.now().difference(failure.last);
    final ago = since.inMinutes < 1
        ? "just now"
        : since.inHours < 1
            ? "${since.inMinutes} min ago"
            : "${since.inHours} h ago";
    final times = failure.count == 1 ? "" : ", ${failure.count} in a row";
    return Card(
      margin: const EdgeInsets.symmetric(vertical: 12),
      color: theme.colorScheme.errorContainer,
      child: Padding(
        padding: const EdgeInsets.all(12),
        child: Row(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Icon(Icons.warning_amber, color: theme.colorScheme.onErrorContainer),
            const SizedBox(width: 12),
            Expanded(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(
                    failure.headline ?? "Last attempt failed $ago$times. It retries on the next block.",
                    style: t.labelLarge?.copyWith(color: theme.colorScheme.onErrorContainer),
                  ),
                  const SizedBox(height: 4),
                  SelectableText(
                    failure.message,
                    style: t.bodyMedium?.copyWith(color: theme.colorScheme.onErrorContainer),
                  ),
                ],
              ),
            ),
          ],
        ),
      ),
    );
  }
}
