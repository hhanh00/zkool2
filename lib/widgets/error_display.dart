import 'package:awesome_dialog/awesome_dialog.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:gap/gap.dart';

/// A widget that displays errors with expandable stack trace
/// and copy functionality for debugging
class ErrorDisplay extends StatefulWidget {
  final Object error;
  final StackTrace? stackTrace;
  final String? customMessage;

  const ErrorDisplay({
    super.key,
    required this.error,
    this.stackTrace,
    this.customMessage,
  });

  @override
  State<ErrorDisplay> createState() => _ErrorDisplayState();
}

class _ErrorDisplayState extends State<ErrorDisplay> {
  bool _isExpanded = false;

  String get _errorMessage {
    if (widget.customMessage != null) return widget.customMessage!;
    return widget.error.toString();
  }

  String get _fullError {
    final buffer = StringBuffer();
    buffer.writeln('Error: $_errorMessage');
    if (widget.stackTrace != null) {
      buffer.writeln('\nStackTrace:');
      buffer.writeln(widget.stackTrace.toString());
    }
    return buffer.toString();
  }

  void _copyToClipboard() {
    Clipboard.setData(ClipboardData(text: _fullError));
    ScaffoldMessenger.of(context).showSnackBar(
      const SnackBar(
        content: Text('Error details copied to clipboard'),
        duration: Duration(seconds: 2),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final colorScheme = theme.colorScheme;
    final textTheme = theme.textTheme;

    return Container(
      margin: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        borderRadius: BorderRadius.circular(16),
        gradient: LinearGradient(
          begin: Alignment.topCenter,
          end: Alignment.bottomCenter,
          colors: [
            colorScheme.errorContainer.withAlpha(51),
            colorScheme.errorContainer.withAlpha(26),
          ],
          stops: const [0.0, 0.6],
        ),
        border: Border.all(
          color: colorScheme.error.withAlpha(51),
          width: 1,
        ),
        boxShadow: [
          BoxShadow(
            color: colorScheme.error.withAlpha(20),
            blurRadius: 12,
            offset: const Offset(0, 6),
          ),
        ],
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        mainAxisSize: MainAxisSize.min,
        children: [
          // Header section
          Container(
            padding: const EdgeInsets.all(16),
            decoration: BoxDecoration(
              color: colorScheme.error.withAlpha(13),
              borderRadius: const BorderRadius.only(
                topLeft: Radius.circular(16),
                topRight: Radius.circular(16),
              ),
            ),
            child: Row(
              children: [
                Icon(
                  Icons.error_outline,
                  color: colorScheme.error,
                  size: 28,
                ),
                const Gap(12),
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      Text(
                        'Error',
                        style: textTheme.titleMedium?.copyWith(
                          color: colorScheme.error,
                          fontWeight: FontWeight.bold,
                        ),
                      ),
                      const Gap(4),
                      Text(
                        _errorMessage,
                        style: textTheme.bodyMedium?.copyWith(
                          color: colorScheme.onErrorContainer,
                        ),
                        maxLines: _isExpanded ? null : 2,
                        overflow: _isExpanded ? null : TextOverflow.ellipsis,
                      ),
                    ],
                  ),
                ),
              ],
            ),
          ),

          // Stack trace section (expandable)
          if (widget.stackTrace != null) ...[
            const Gap(8),
            InkWell(
              onTap: () => setState(() => _isExpanded = !_isExpanded),
              borderRadius: const BorderRadius.only(
                bottomLeft: Radius.circular(16),
                bottomRight: Radius.circular(16),
              ),
              child: Container(
                padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Row(
                      children: [
                        Icon(
                          _isExpanded ? Icons.expand_less : Icons.expand_more,
                          color: colorScheme.error,
                          size: 20,
                        ),
                        const Gap(8),
                        Text(
                          _isExpanded ? 'Hide Stack Trace' : 'Show Stack Trace',
                          style: textTheme.labelMedium?.copyWith(
                            color: colorScheme.error,
                            fontWeight: FontWeight.w600,
                          ),
                        ),
                      ],
                    ),
                    if (_isExpanded) ...[
                      const Gap(12),
                      Container(
                        padding: const EdgeInsets.all(12),
                        decoration: BoxDecoration(
                          color: colorScheme.surface.withAlpha(230),
                          borderRadius: BorderRadius.circular(8),
                          border: Border.all(
                            color: colorScheme.error.withAlpha(38),
                            width: 1,
                          ),
                        ),
                        child: SingleChildScrollView(
                          child: SelectableText(
                            widget.stackTrace.toString(),
                            style: textTheme.bodySmall?.copyWith(
                              fontFamily: 'monospace',
                              color: colorScheme.onSurface,
                            ),
                          ),
                        ),
                      ),
                    ],
                  ],
                ),
              ),
            ),
          ],

          // Action buttons
          Container(
            padding: const EdgeInsets.all(12),
            decoration: BoxDecoration(
              color: colorScheme.error.withAlpha(13),
              borderRadius: const BorderRadius.only(
                bottomLeft: Radius.circular(16),
                bottomRight: Radius.circular(16),
              ),
            ),
            child: Row(
              mainAxisAlignment: MainAxisAlignment.end,
              children: [
                TextButton.icon(
                  onPressed: _copyToClipboard,
                  icon: const Icon(Icons.copy, size: 18),
                  label: const Text('Copy'),
                  style: TextButton.styleFrom(
                    foregroundColor: colorScheme.error,
                  ),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }
}

/// A dialog wrapper for displaying errors using AwesomeDialog
class ErrorDialog extends StatelessWidget {
  final Object error;
  final StackTrace? stackTrace;
  final String? title;
  final String? customMessage;

  const ErrorDialog({
    super.key,
    required this.error,
    this.stackTrace,
    this.title,
    this.customMessage,
  });

  @override
  Widget build(BuildContext context) {
    return Dialog(
      child: ConstrainedBox(
        constraints: const BoxConstraints(maxWidth: 600, maxHeight: 600),
        child: ErrorDisplay(
          error: error,
          stackTrace: stackTrace,
          customMessage: customMessage,
        ),
      ),
    );
  }

  /// Show the error dialog using AwesomeDialog
  static Future<void> show(
    BuildContext context, {
    required Object error,
    StackTrace? stackTrace,
    String? title,
    String? customMessage,
  }) {
    final errorMessage = customMessage ?? error.toString();

    return AwesomeDialog(
      context: context,
      dialogType: DialogType.error,
      animType: AnimType.rightSlide,
      title: title ?? 'Error',
      desc: errorMessage,
      body: _buildErrorBody(context, error, stackTrace, errorMessage),
      btnOkOnPress: () {},
      autoDismiss: true,
    ).show();
  }

  static Widget _buildErrorBody(
    BuildContext context,
    Object error,
    StackTrace? stackTrace,
    String errorMessage,
  ) {
    final theme = Theme.of(context);
    final colorScheme = theme.colorScheme;
    final textTheme = theme.textTheme;

    return Padding(
      padding: const EdgeInsets.all(16),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 16),
            child: Text(
              errorMessage,
              style: textTheme.bodyMedium?.copyWith(
                color: colorScheme.onSurface,
              ),
              textAlign: TextAlign.center,
            ),
          ),
          if (stackTrace != null) ...[
            const Gap(16),
            Container(
              margin: const EdgeInsets.symmetric(horizontal: 16),
              padding: const EdgeInsets.all(12),
              decoration: BoxDecoration(
                color: colorScheme.errorContainer.withAlpha(26),
                borderRadius: BorderRadius.circular(8),
                border: Border.all(
                  color: colorScheme.error.withAlpha(38),
                  width: 1,
                ),
              ),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Row(
                    children: [
                      Icon(
                        Icons.error_outline,
                        color: colorScheme.error,
                        size: 16,
                      ),
                      const Gap(8),
                      Text(
                        'Stack Trace',
                        style: textTheme.labelSmall?.copyWith(
                          color: colorScheme.error,
                          fontWeight: FontWeight.w600,
                        ),
                      ),
                      const Spacer(),
                      InkWell(
                        onTap: () {
                          Clipboard.setData(ClipboardData(
                            text: '$error\n\n$stackTrace',
                          ));
                          ScaffoldMessenger.of(context).showSnackBar(
                            const SnackBar(
                              content: Text('Error details copied to clipboard'),
                              duration: Duration(seconds: 2),
                            ),
                          );
                        },
                        child: Row(
                          mainAxisSize: MainAxisSize.min,
                          children: [
                            Icon(
                              Icons.copy,
                              color: colorScheme.error,
                              size: 16,
                            ),
                            const Gap(4),
                            Text(
                              'Copy',
                              style: textTheme.labelSmall?.copyWith(
                                color: colorScheme.error,
                              ),
                            ),
                          ],
                        ),
                      ),
                    ],
                  ),
                  const Gap(8),
                  Container(
                    padding: const EdgeInsets.all(8),
                    decoration: BoxDecoration(
                      color: colorScheme.surface.withAlpha(230),
                      borderRadius: BorderRadius.circular(4),
                    ),
                    child: SingleChildScrollView(
                      child: SelectableText(
                        stackTrace.toString(),
                        style: textTheme.bodySmall?.copyWith(
                          fontFamily: 'monospace',
                          fontSize: 10,
                          color: colorScheme.onSurface,
                        ),
                      ),
                    ),
                  ),
                ],
              ),
            ),
          ],
        ],
      ),
    );
  }
}

/// A card-style widget for inline error display
class ErrorCard extends StatelessWidget {
  final Object error;
  final StackTrace? stackTrace;
  final String? customMessage;
  final VoidCallback? onRetry;

  const ErrorCard({
    super.key,
    required this.error,
    this.stackTrace,
    this.customMessage,
    this.onRetry,
  });

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final colorScheme = theme.colorScheme;

    return Container(
      margin: const EdgeInsets.all(16),
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        color: colorScheme.errorContainer.withAlpha(51),
        borderRadius: BorderRadius.circular(12),
        border: Border.all(
          color: colorScheme.error.withAlpha(77),
          width: 1,
        ),
      ),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          Row(
            children: [
              Icon(
                Icons.error_outline,
                color: colorScheme.error,
              ),
              const Gap(12),
              Expanded(
                child: Text(
                  customMessage ?? error.toString(),
                  style: theme.textTheme.bodyMedium?.copyWith(
                    color: colorScheme.onErrorContainer,
                  ),
                ),
              ),
            ],
          ),
          if (onRetry != null) ...[
            const Gap(12),
            Align(
              alignment: Alignment.centerRight,
              child: ElevatedButton(
                onPressed: onRetry,
                child: const Text('Retry'),
              ),
            ),
          ],
        ],
      ),
    );
  }
}

/// Show an exception in a dialog using ErrorDialog
Future<void> showException(
  BuildContext context,
  String message, {
  StackTrace? stackTrace,
  String? title,
}) {
  return ErrorDialog.show(
    context,
    error: Exception(message),
    stackTrace: stackTrace,
    title: title,
    customMessage: message,
  );
}
