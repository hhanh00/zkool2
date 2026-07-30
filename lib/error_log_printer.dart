import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated.dart';
import 'package:logger/logger.dart';

final _unknownRustFrame = RegExp(r'^[ \t]*\d+:[ \t]+<unknown>[ \t]*$');

String omitUnknownRustFrames(String message) {
  return message.split('\n').where((line) => !_unknownRustFrame.hasMatch(line)).join('\n');
}

class ErrorLogPrinter extends PrettyPrinter {
  @override
  List<String> log(LogEvent event) {
    final error = event.error;
    if (error is! AnyhowException) return super.log(event);

    return super.log(
      LogEvent(
        event.level,
        event.message,
        time: event.time,
        error: AnyhowException(omitUnknownRustFrames(error.message)),
        stackTrace: event.stackTrace,
      ),
    );
  }
}
