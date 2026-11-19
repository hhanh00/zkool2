import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:zkool/router.dart' show navigatorKey;
import 'package:zkool/utils.dart';

import 'platform_fakes.dart';

// The dialog that shows a freshly generated seed phrase opens with the phrase
// masked, so that nobody reads it off the screen before the user is ready
// (#1672). A checkbox shows it and hides it again, and the phrase can only be
// copied while it is shown.

const seed = 'abandon ability able about above absent absorb abstract absurd abuse access accident';
const mask = '•••• •••• •••• •••• •••• •••• •••• •••• •••• •••• •••• ••••';
const title = 'SEED PHRASE - SAVE IT OR YOU CAN LOSE YOUR FUNDS';

void main() {
  /// What the app last put on the clipboard.
  String? clipboard;

  // The dialog has a Rive header, which cannot load under flutter test.
  setUpAll(silenceRiveHeaders);

  setUp(() {
    clipboard = null;
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger.setMockMethodCallHandler(SystemChannels.platform,
        (call) async {
      if (call.method == 'Clipboard.setData') clipboard = (call.arguments as Map)['text'] as String;
      return null;
    });
  });

  tearDown(() {
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(SystemChannels.platform, null);
  });

  Future<void> open(WidgetTester tester) async {
    await tester.pumpWidget(MaterialApp(
      navigatorKey: navigatorKey,
      home: Scaffold(
        body: Builder(
          builder: (context) => TextButton(onPressed: () => showSeed(context, seed), child: const Text('open')),
        ),
      ),
    ),);
    await tester.tap(find.text('open'));
    await tester.pumpAndSettle();
    expect(find.text(title), findsOneWidget);
  }

  bool ticked(WidgetTester tester) => tester.widget<CheckboxListTile>(find.byType(CheckboxListTile)).value!;

  testWidgets('the phrase opens masked, one mask per word', (tester) async {
    await open(tester);

    expect(find.text(seed), findsNothing);
    expect(find.text(mask), findsOneWidget);
    expect(ticked(tester), isFalse);
  });

  testWidgets('the checkbox shows the phrase and hides it again', (tester) async {
    await open(tester);

    await tester.tap(find.text('Show seed phrase'));
    await tester.pumpAndSettle();
    expect(find.text(seed), findsOneWidget);
    expect(find.text(mask), findsNothing);

    await tester.tap(find.text('Show seed phrase'));
    await tester.pumpAndSettle();
    expect(find.text(seed), findsNothing);
    expect(find.text(mask), findsOneWidget);
  });

  testWidgets('showing the phrase does not resize the dialog', (tester) async {
    await open(tester);
    final hidden = tester.getRect(find.byType(CheckboxListTile));

    await tester.tap(find.text('Show seed phrase'));
    await tester.pumpAndSettle();

    expect(tester.getRect(find.byType(CheckboxListTile)), hidden);
  });

  testWidgets('the phrase is copied only while it is shown', (tester) async {
    await open(tester);

    await tester.tap(find.text(mask));
    await tester.pumpAndSettle();
    expect(clipboard, isNull, reason: 'a masked phrase must not be copied');

    await tester.tap(find.text('Show seed phrase'));
    await tester.pumpAndSettle();
    await tester.tap(find.text(seed));
    await tester.pumpAndSettle();
    expect(clipboard, seed);
  });

  testWidgets('Ok closes the dialog without the phrase ever being shown', (tester) async {
    await open(tester);

    await tester.tap(find.text('Ok'));
    await tester.pumpAndSettle();

    expect(find.text(title), findsNothing);
    expect(clipboard, isNull);
  });
}
