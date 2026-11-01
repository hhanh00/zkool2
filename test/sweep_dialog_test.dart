import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:go_router/go_router.dart';
import 'package:rive/rive.dart' show RiveFile;
import 'package:zkool/pages/sweep.dart';
import 'package:zkool/router.dart' show navigatorKey;
import 'package:zkool/src/rust/api/coin.dart';
import 'package:zkool/src/rust/api/sweep.dart';
import 'package:zkool/src/rust/frb_generated.dart';
import 'package:zkool/store.dart';

// The transparent scan runs in a keepAlive provider, so closing its dialog
// used to leave the scan running against the account while the editor
// became reachable (#1545). Closing the dialog now cancels a running scan,
// the way Stop does.

class _FakeCoin implements Coin {
  @override
  dynamic noSuchMethod(Invocation invocation) => super.noSuchMethod(invocation);
}

/// A scanner whose run reports one address and then stays open. Like the
/// real one, cancel only flags the scan; the stream ends on its own later,
/// by which time the page has let go of its subscription. The async
/// onCancel gives the subscription's cancel a future the widget tester
/// pumps; the SDK's default completed future lives in the root zone.
class _FakeScanner implements TransparentScanner {
  int cancels = 0;
  final progress = StreamController<String>(onCancel: () async {});

  @override
  Future<void> cancel() async {
    cancels += 1;
  }

  @override
  Stream<String> run({required int endHeight, required int gapLimit, required int account, required Coin c}) {
    progress.add('t1scanning');
    return progress.stream;
  }

  @override
  dynamic noSuchMethod(Invocation invocation) => super.noSuchMethod(invocation);
}

class _Api implements RustLibApi {
  final scanners = <_FakeScanner>[];

  @override
  Coin crateApiCoinCoinNew({int? defaultCoin}) => _FakeCoin();

  @override
  Future<TransparentScanner> crateApiSweepTransparentScannerNew() async {
    final scanner = _FakeScanner();
    scanners.add(scanner);
    return scanner;
  }

  @override
  Future<int> crateApiNetworkGetCurrentHeight({required Coin c}) async => 3123456;

  @override
  dynamic noSuchMethod(Invocation invocation) => super.noSuchMethod(invocation);
}

void main() {
  late _Api api;
  setUpAll(() {
    api = _Api();
    RustLib.initMock(api: api);
    coinContext.set(coin: _FakeCoin());
    // The dialog is an AwesomeDialog with a Rive header; take Rive's one
    // initialisation failure in a guarded zone, as the other page tests do.
    runZonedGuarded(() => unawaited(RiveFile.initialize()), (_, __) {});
  });
  setUp(() => api.scanners.clear());
  tearDownAll(RustLib.dispose);

  /// Pumps through dialog animations; pumpAndSettle never settles on them.
  Future<void> settle(WidgetTester tester) async {
    for (var i = 0; i < 10; i++) {
      await tester.pump(const Duration(milliseconds: 200));
    }
  }

  late ProviderContainer container;

  Future<void> openDialog(WidgetTester tester) async {
    final router = GoRouter(navigatorKey: navigatorKey, routes: [
      GoRoute(
        path: '/',
        builder: (context, state) => Consumer(builder: (context, ref, _) {
          return Scaffold(
            body: TextButton(
              onPressed: () => showTransparentScan(ref, context, 1),
              child: const Text('Scan'),
            ),
          );
        }),
      ),
    ]);
    addTearDown(router.dispose);
    await tester.pumpWidget(ProviderScope(child: MaterialApp.router(routerConfig: router)));
    await tester.pumpAndSettle();
    container = ProviderScope.containerOf(tester.element(find.text('Scan')));
    await tester.tap(find.text('Scan'));
    await settle(tester);
    expect(find.text('Run'), findsOneWidget);
  }

  Future<void> tapDialog(WidgetTester tester, String label) async {
    expect(find.text(label), findsOneWidget);
    await tester.tap(find.text(label));
    await settle(tester);
  }

  testWidgets('closing the dialog cancels a running scan', (tester) async {
    await openDialog(tester);
    await tapDialog(tester, 'Run');
    expect(find.text('Stop'), findsOneWidget, reason: 'the scan is running');
    expect(find.text('t1scanning'), findsOneWidget);

    await tapDialog(tester, 'Close');
    expect(find.text('Stop'), findsNothing, reason: 'the dialog is gone');
    expect(api.scanners.single.cancels, 1);
    final scan = container.read(transparentScanProvider.notifier);
    expect(scan.running, isFalse);
    expect(scan.scanner, isNull);
  });

  testWidgets('closing the dialog with no scan running cancels nothing', (tester) async {
    await openDialog(tester);
    await tapDialog(tester, 'Close');
    expect(api.scanners, isEmpty);
    expect(find.text('Run'), findsNothing);
  });
}
