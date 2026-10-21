import 'package:flutter/material.dart';

/// The last failed pass of a protocol driver (DKG or FROST signing): what
/// went wrong, how many passes in a row have failed, and when the last one
/// did. A pass runs once per block, so the count is about how many blocks
/// the protocol has been stuck for.
class ProtocolFailure {
  final String message;
  final int count;
  final DateTime last;

  const ProtocolFailure(this.message, {this.count = 1, required this.last});

  /// This failure followed by another one.
  ProtocolFailure again(String message) => ProtocolFailure(message, count: count + 1, last: DateTime.now());

  static ProtocolFailure first(String message) => ProtocolFailure(message, last: DateTime.now());
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
                    "Last attempt failed $ago$times. It retries on the next block.",
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
