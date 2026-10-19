// The error dialog shows an anyhow error's context chain, not its Debug
// output: with RUST_BACKTRACE set the Debug text runs to dozens of frames,
// grows the dialog past the window and pushes the Ok button off screen
// (#1368). The backtrace stays reachable in a collapsed "Details" section.
import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated.dart' show AnyhowException;
import 'package:flutter_test/flutter_test.dart';
import 'package:rive/rive.dart' show RiveFile;
import 'package:zkool/utils.dart';
import 'package:zkool/widgets/error_display.dart';

/// An anyhow Debug output of the shape the send page reports: a context
/// chain and a long backtrace, far taller than an 800x600 window.
final anyhowDebug = '''
plan_transaction

Caused by:
    No feasible note selection found

Stack backtrace:
   0: <unknown>
${List.generate(70, (i) => '  ${i + 1}: zkool::frame_$i').join('\n')}
''';

/// Pumps an app whose body opens the error dialog with [show] on a button tap.
Future<void> openDialog(
  WidgetTester tester,
  Future<void> Function(BuildContext context) show,
) async {
  await tester.pumpWidget(
    MaterialApp(
      home: Scaffold(
        body: Builder(
          builder: (context) => Center(
            child: ElevatedButton(onPressed: () => show(context), child: const Text('fail')),
          ),
        ),
      ),
    ),
  );
  await tester.tap(find.text('fail'));
  await tester.pumpAndSettle();
}

void main() {
  setUpAll(() {
    // AwesomeDialog's header is a Rive animation. Rive's text engine loads a
    // native library the test VM does not have: its first initialisation
    // fails with an uncaught async error and later ones wait forever. Take
    // that one failure here, in a guarded zone, so it does not fail the
    // first test that opens a dialog.
    runZonedGuarded(() => unawaited(RiveFile.initialize()), (_, __) {});
  });

  /// The size the UI tests run at, where the issue was found.
  Future<void> setWindow(WidgetTester tester) async {
    tester.view.physicalSize = const Size(800, 600);
    tester.view.devicePixelRatio = 1.0;
    addTearDown(tester.view.reset);
  }

  testWidgets('shows the compact message with the Ok button on screen', (tester) async {
    await setWindow(tester);
    await openDialog(tester, (context) => showException(context, anyhowDebug));

    expect(find.text('plan_transaction: No feasible note selection found'), findsOneWidget);
    expect(find.textContaining('Stack backtrace'), findsNothing, reason: 'the backtrace is not in the message');

    final ok = tester.getRect(find.text('Ok'));
    expect(ok.bottom, lessThanOrEqualTo(600), reason: 'the Ok button is inside the window');

    await tester.tap(find.text('Ok'));
    await tester.pumpAndSettle();
    expect(find.text('Ok'), findsNothing, reason: 'Ok closes the dialog');
  });

  testWidgets('keeps the backtrace in collapsed details', (tester) async {
    await setWindow(tester);
    await openDialog(tester, (context) => showException(context, anyhowDebug));

    expect(find.text('Details'), findsOneWidget);
    expect(find.textContaining('zkool::frame_69'), findsNothing, reason: 'details start collapsed');

    await tester.tap(find.text('Details'));
    await tester.pumpAndSettle();
    final details = tester.widget<SelectableText>(find.byWidgetPredicate(
      (w) => w is SelectableText && (w.data ?? '').startsWith('Stack backtrace'),
    ));
    expect(details.data, contains('zkool::frame_69'));
    expect(details.data, isNot(contains('<unknown>')), reason: 'unknown frames are dropped');

    // Expanded details scroll inside the body; the Ok button stays on screen.
    final ok = tester.getRect(find.text('Ok'));
    expect(ok.bottom, lessThanOrEqualTo(600));
  });

  testWidgets('a plain message has no details section', (tester) async {
    await setWindow(tester);
    await openDialog(tester, (context) => showException(context, 'Unknown contact: nobody'));

    expect(find.text('Unknown contact: nobody'), findsOneWidget);
    expect(find.text('Details'), findsNothing);
  });

  testWidgets('an AnyhowException passed as the error shows its message', (tester) async {
    await setWindow(tester);
    await openDialog(tester, (context) => ErrorDialog.show(context, error: AnyhowException(anyhowDebug)));

    expect(find.text('plan_transaction: No feasible note selection found'), findsOneWidget);
    expect(find.textContaining('AnyhowException('), findsNothing);
  });

  group('errorDisplayText', () {
    test('compacts an AnyhowException', () {
      expect(errorDisplayText(AnyhowException(anyhowDebug)), 'plan_transaction: No feasible note selection found');
    });

    test('leaves other errors to toString', () {
      expect(errorDisplayText(Exception('boom')), 'Exception: boom');
      expect(errorDisplayText('plain'), 'plain');
    });
  });

  group('anyhowBacktrace', () {
    test('returns the backtrace without unknown frames', () {
      final bt = anyhowBacktrace(anyhowDebug)!;
      expect(bt, startsWith('Stack backtrace:'));
      expect(bt, isNot(contains('<unknown>')));
      expect(bt, contains('70: zkool::frame_69'));
    });

    test('is null without a backtrace', () {
      expect(anyhowBacktrace('plain'), isNull);
      expect(anyhowBacktrace('a\n\nCaused by:\n    b\n'), isNull);
    });
  });
}
