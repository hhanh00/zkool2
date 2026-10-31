import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:go_router/go_router.dart';
import 'package:zkool/main.dart' show PinLock;
import 'package:zkool/pages/migrate.dart';
import 'package:zkool/pages/voting.dart';
import 'package:zkool/pages/voting_round.dart';
import 'package:zkool/pages/zsa.dart';
import 'package:zkool/router.dart' show navigatorKey;
import 'package:zkool/src/rust/api/coin.dart';
import 'package:zkool/src/rust/api/voting.dart';
import 'package:zkool/src/rust/api/voting_drive.dart';
import 'package:zkool/src/rust/frb_generated.dart';
import 'package:zkool/store.dart';

// Every wallet page enforces the PIN lock itself: it watches the lifecycle
// and shows PinLock while locked. These pages used to skip that (#1417,
// V12 F-154), so a locked wallet left on one of them stayed usable.

class _FakeCoin implements Coin {
  @override
  dynamic noSuchMethod(Invocation invocation) => super.noSuchMethod(invocation);
}

class _FakeDrive implements VotingDrive {
  @override
  dynamic noSuchMethod(Invocation invocation) => super.noSuchMethod(invocation);
}

class _Api implements RustLibApi {
  @override
  Coin crateApiCoinCoinNew({int? defaultCoin}) => _FakeCoin();

  @override
  VotingDrive crateApiVotingDriveVotingDriveNew() => _FakeDrive();

  // The voting pages load their data in initState, lock or not.
  @override
  Future<List<VotingRoundListItem>> crateApiVotingVotingRoundList({required int account, required Coin c}) async => [];

  @override
  Future<List<VotingSelection>> crateApiVotingVotingLoadSelections(
          {required String roundId, required int account, required Coin c}) async =>
      [];

  @override
  dynamic noSuchMethod(Invocation invocation) => super.noSuchMethod(invocation);
}

class _Locked extends Lifecycle {
  @override
  Future<bool> build() async => true;
}

/// A voting job that does nothing: the round page resumes one on open.
class _IdleJob extends VotingDriveJob {
  @override
  VotingDriveJobState build(String roundId, int account) => const VotingDriveJobState(stage: 'idle');

  @override
  Future<void> resume({
    required String lightwalletdUrl,
    required String roundName,
    required List<int> proposalIds,
    bool offline = false,
  }) async {}
}

class _Settings extends AppSettingsNotifier {
  @override
  Future<AppSettings> build() async => AppSettings(
        dbName: '',
        net: '',
        isLightNode: false,
        lwd: '',
        blockExplorer: '',
        syncInterval: '',
        actionsPerSync: '',
        transport: 0,
        proxy: '',
        coingecko: '',
        recovery: false,
        needPin: true,
        pinUnlockedAt: DateTime(2026),
        offline: true,
        getFx: false,
        qrSettings: QRSettings(enabled: false, size: 100, ecLevel: 0, delay: 0, repair: 0),
        vault: false,
        expertMode: false,
        paletteName: '',
        darkMode: false,
        transactionTableMode: false,
        collapsePoolBalances: false,
        currency: '',
        votingConfigUrl: '',
        marketInterval: '1d',
        marketStyle: 'candles',
      );
}

const _round = VotingRoundListItem(
  roundId: 'r1',
  title: 'Round 1',
  status: 'open',
  snapshotHeight: 100,
  bundleCount: 0,
  action: 'vote',
  proposals: [],
  helperUrls: [],
);

void main() {
  setUpAll(() => RustLib.initMock(api: _Api()));
  tearDownAll(RustLib.dispose);

  Future<void> openLocked(WidgetTester tester, Widget page) async {
    final router = GoRouter(navigatorKey: navigatorKey, initialLocation: '/page', routes: [
      GoRoute(path: '/', builder: (context, state) => const Scaffold(body: Text('Home'))),
      GoRoute(path: '/page', builder: (context, state) => page),
    ]);
    addTearDown(router.dispose);
    await tester.pumpWidget(ProviderScope(
      overrides: [
        lifecycleProvider.overrideWith(_Locked.new),
        appSettingsProvider.overrideWith(_Settings.new),
        votingDriveJobProvider.overrideWith(_IdleJob.new),
        getAccountsProvider.overrideWith((ref) async => []),
      ],
      child: MaterialApp.router(routerConfig: router),
    ));
    await tester.pumpAndSettle();
  }

  final pages = <String, Widget>{
    'ZSA holdings': const ZsaHoldingsPage(account: 1),
    'Issue asset': const IssueAssetPage(account: 1),
    'Migrate': const MigratePage(account: 1),
    'Voting': const VotingPage(account: 1),
    'Voting round': const VotingRoundPage(round: _round, account: 1),
  };

  for (final MapEntry(key: name, value: page) in pages.entries) {
    testWidgets('$name shows the PIN lock while the wallet is locked', (tester) async {
      await openLocked(tester, page);
      expect(find.byType(PinLock), findsOneWidget);
      expect(find.text('Locked'), findsOneWidget);
      expect(find.byType(TextField), findsNothing, reason: 'none of the page is reachable');
      expect(find.byTooltip('Issue'), findsNothing);
    });
  }
}
