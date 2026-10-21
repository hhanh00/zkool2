// The DKG and FROST signing pages render whatever the protocol driver
// reports. These tests feed the pages a scripted status stream through the
// mock bridge and check what they show, so a page that stops reacting to
// the driver fails here without a chain or peers.
import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:go_router/go_router.dart';
import 'package:zkool/pages/dkg.dart';
import 'package:zkool/pages/frost.dart';
import 'package:zkool/router.dart' show navigatorKey;
import 'package:zkool/src/rust/api/coin.dart';
import 'package:zkool/src/rust/api/frost.dart';
import 'package:zkool/src/rust/frb_generated.dart';
import 'package:zkool/store.dart';

class _FakeCoin implements Coin {
  @override
  dynamic noSuchMethod(Invocation invocation) => super.noSuchMethod(invocation);
}

/// The driver as the page sees it: `run` hands out the scripted stream and
/// `cancel` is recorded.
class _FakeDriver implements ProtocolDriver {
  // Synchronous so a status reaches the page inside `add`, before the pump.
  final statuses = StreamController<ProtocolStatus>.broadcast(sync: true);
  int cancels = 0;

  @override
  Stream<ProtocolStatus> run({required Coin c}) => statuses.stream;

  @override
  void cancel() => cancels++;

  @override
  dynamic noSuchMethod(Invocation invocation) => super.noSuchMethod(invocation);
}

class _Api implements RustLibApi {
  _FakeDriver driver = _FakeDriver();

  @override
  Coin crateApiCoinCoinNew({int? defaultCoin}) => _FakeCoin();

  @override
  ProtocolDriver crateApiFrostProtocolDriverNew() => driver;

  @override
  Future<FundingShortfall?> crateApiFrostFundingShortfall({required Coin c}) async =>
      FundingShortfall(account: 1, balance: BigInt.from(50000), messageCost: BigInt.from(10000));

  @override
  dynamic noSuchMethod(Invocation invocation) => super.noSuchMethod(invocation);
}

class _Unlocked extends Lifecycle {
  @override
  Future<bool> build() async => false;
}

void main() {
  late _Api api;
  setUpAll(() {
    api = _Api();
    RustLib.initMock(api: api);
  });
  setUp(() {
    api.driver = _FakeDriver(); // one driver per page, like the app
  });
  tearDownAll(RustLib.dispose);

  Future<GoRouter> open(WidgetTester tester, String path, Widget page) async {
    final router = GoRouter(navigatorKey: navigatorKey, initialLocation: path, routes: [
      GoRoute(path: '/', builder: (context, state) => const Scaffold(body: Text('Home'))),
      GoRoute(path: path, builder: (context, state) => page),
    ]);
    addTearDown(router.dispose);
    await tester.pumpWidget(ProviderScope(
      overrides: [
        lifecycleProvider.overrideWith(_Unlocked.new),
        getAccountsProvider.overrideWith((ref) async => []),
      ],
      child: MaterialApp.router(routerConfig: router),
    ));
    await tester.pump();
    return router;
  }

  Future<void> report(WidgetTester tester, ProtocolStatus status) async {
    api.driver.statuses.add(status);
    await tester.pump();
  }

  group('DKG page', () {
    testWidgets('shows each round as the driver reports it, then the shared address', (tester) async {
      await open(tester, '/dkg3', const DKGPage3());

      await report(tester, const ProtocolStatus.dkg(DKGStatus.publishRound0Pkg()));
      expect(find.text('Broadcasting participant keys'), findsOneWidget);

      await report(tester, const ProtocolStatus.dkg(DKGStatus.waitRound1Pkg()));
      expect(find.text('Waiting for other participants to send their round 1 packages'), findsOneWidget);
      expect(find.byIcon(Icons.cancel), findsOneWidget, reason: 'not finished yet');
      expect(find.text('Dismiss'), findsNothing, reason: 'not finished yet');
      await report(tester, const ProtocolStatus.dkg(DKGStatus.waitProgress(what: 'Round 1 packages', have: 1, of: 2)));
      expect(find.textContaining('Round 1 packages: 1 of 2'), findsOneWidget, reason: 'how far the wait has got');

      await report(tester, const ProtocolStatus.dkg(DKGStatus.sharedAddress('uregtest1abc')));
      expect(find.text('The shared address is: uregtest1abc'), findsOneWidget);
      expect(find.byIcon(Icons.close), findsOneWidget, reason: 'finished');
      expect(find.byIcon(Icons.cancel), findsNothing);
    });

    testWidgets('a finished DKG offers Dismiss, which leaves the page', (tester) async {
      await open(tester, '/dkg3', const DKGPage3());
      await report(tester, const ProtocolStatus.dkg(DKGStatus.sharedAddress('uregtest1abc')));
      expect(find.widgetWithText(ElevatedButton, 'Dismiss'), findsOneWidget);

      await tester.tap(find.widgetWithText(ElevatedButton, 'Dismiss'));
      await tester.pumpAndSettle();
      expect(find.text('Home'), findsOneWidget, reason: 'back at /');
      expect(api.driver.cancels, 1, reason: 'leaving stops the protocol driver');
    });

    testWidgets('shows the wait for our own package to be mined', (tester) async {
      await open(tester, '/dkg3', const DKGPage3());
      await report(tester, const ProtocolStatus.dkg(DKGStatus.waitOwnPublish()));
      expect(find.text('Waiting for our last message to be mined'), findsOneWidget);
    });

    testWidgets('ignores signing statuses and warns on a funding wait', (tester) async {
      await open(tester, '/dkg3', const DKGPage3());

      await report(tester, const ProtocolStatus.sign(SigningStatus.signingCompleted()));
      expect(find.text('Signing completed'), findsNothing);

      await report(tester, const ProtocolStatus.dkg(DKGStatus.waitingForFunds()));
      await tester.pump();
      expect(find.textContaining('Not enough funds'), findsWidgets);
      expect(find.textContaining('0.0005'), findsOneWidget, reason: "the funding account's balance");
      expect(find.textContaining('0.0001'), findsOneWidget, reason: 'what a message costs');
    });

    testWidgets('a failed pass is a warning, and leaving cancels the driver', (tester) async {
      final router = await open(tester, '/dkg3', const DKGPage3());

      // A failed pass arrives as a Warning status, not a stream error: the
      // bridge ends a stream at its first error, which would freeze the
      // page while the driver retries on the next block.
      await report(tester, const ProtocolStatus.warning('publish DKG package'));
      // The message is in the snackbar and on the page's failure line.
      expect(find.textContaining('publish DKG package'), findsWidgets);
      expect(find.textContaining('Last attempt failed'), findsOneWidget, reason: 'the cause stays on the page');
      await report(tester, const ProtocolStatus.warning('publish DKG package'));
      expect(find.textContaining('2 in a row'), findsOneWidget, reason: 'failures in a row are counted');
      await report(tester, const ProtocolStatus.dkg(DKGStatus.waitRound1Pkg()));
      expect(find.textContaining('round 1 packages'), findsOneWidget,
          reason: 'the page keeps reporting after a warning',);
      expect(find.textContaining('Last attempt failed'), findsNothing, reason: 'a pass that got somewhere clears it');

      expect(api.driver.cancels, 0);
      router.go('/');
      await tester.pumpAndSettle();
      expect(api.driver.cancels, 1, reason: 'disposing the page stops the driver');
    });
  });

  group('FROST signing page', () {
    testWidgets('shows each phase as the driver reports it, then the txid', (tester) async {
      await open(tester, '/frost2', const FrostPage2(account: 1));

      await report(tester, const ProtocolStatus.sign(SigningStatus.sendingCommitment()));
      expect(find.text('Sending our commitments to the coordinator'), findsOneWidget);

      await report(tester, const ProtocolStatus.sign(SigningStatus.waitingForSigningPackage()));
      expect(find.text('Waiting for the signing package from the coordinator'), findsOneWidget);
      expect(find.byIcon(Icons.cancel), findsOneWidget, reason: 'not finished yet');
      await report(tester, const ProtocolStatus.sign(SigningStatus.waitProgress(what: 'Signing package parts', have: 1, of: 2)));
      expect(find.textContaining('Signing package parts: 1 of 2'), findsOneWidget);
      expect(find.textContaining('waiting for under a minute'), findsOneWidget);

      await report(tester, const ProtocolStatus.sign(SigningStatus.transactionSent('ab12')));
      expect(find.text('TX ID: ab12'), findsOneWidget);
      expect(find.byIcon(Icons.close), findsOneWidget, reason: 'finished');
    });

    testWidgets('shows the wait for our own message to be mined', (tester) async {
      await open(tester, '/frost2', const FrostPage2(account: 1));
      await report(tester, const ProtocolStatus.sign(SigningStatus.waitOwnPublish()));
      expect(find.text('Waiting for our last message to be mined'), findsOneWidget);
    });

    testWidgets('a participant is finished on SigningCompleted', (tester) async {
      await open(tester, '/frost2', const FrostPage2(account: 1));

      await report(tester, const ProtocolStatus.sign(SigningStatus.signingCompleted()));
      expect(find.text('Signing completed'), findsOneWidget);
      expect(find.byIcon(Icons.close), findsOneWidget);
    });

    testWidgets('a finished signer or coordinator gets Dismiss, back to the account', (tester) async {
      for (final last in [
        const ProtocolStatus.sign(SigningStatus.signingCompleted()),
        const ProtocolStatus.sign(SigningStatus.transactionSent('ab12')),
      ]) {
        api.driver = _FakeDriver();
        await open(tester, '/frost2', const FrostPage2(account: 1));
        await report(tester, const ProtocolStatus.sign(SigningStatus.sendingCommitment()));
        expect(find.text('Dismiss'), findsNothing, reason: 'not finished yet');

        await report(tester, last);
        await tester.tap(find.widgetWithText(ElevatedButton, 'Dismiss'));
        await tester.pumpAndSettle();
        expect(find.text('Home'), findsOneWidget, reason: "the account's page, at / in this router");
        expect(api.driver.cancels, 1, reason: 'leaving stops the protocol driver');
      }
    });

    testWidgets('a funds wait says what the account holds and a message costs', (tester) async {
      await open(tester, '/frost2', const FrostPage2(account: 1));
      await report(tester, const ProtocolStatus.sign(SigningStatus.waitingForFunds()));
      await tester.pump();
      expect(find.textContaining('0.0005'), findsOneWidget);
      await report(tester, const ProtocolStatus.sign(SigningStatus.waitingForSigningPackage()));
      // The snackbar lingers; the card under the status is what goes.
      expect(find.textContaining('The funding account holds'), findsNothing);
    });

    testWidgets('a failed pass stays on the page until one succeeds', (tester) async {
      await open(tester, '/frost2', const FrostPage2(account: 1));
      await report(tester, const ProtocolStatus.warning('publish commitments'));
      expect(find.textContaining('Last attempt failed'), findsOneWidget);
      expect(find.textContaining('publish commitments'), findsWidgets);
      await report(tester, const ProtocolStatus.sign(SigningStatus.waitingForSigningPackage()));
      expect(find.textContaining('Last attempt failed'), findsNothing);
    });

    testWidgets('ignores DKG statuses and cancels the driver on leave', (tester) async {
      final router = await open(tester, '/frost2', const FrostPage2(account: 1));

      await report(tester, const ProtocolStatus.dkg(DKGStatus.finalize()));
      expect(find.text('Deriving the shared key'), findsNothing);

      router.go('/');
      await tester.pumpAndSettle();
      expect(api.driver.cancels, 1);
    });
  });
}
