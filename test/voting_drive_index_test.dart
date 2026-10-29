import 'dart:async';

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:zkool/src/rust/api/coin.dart';
import 'package:zkool/src/rust/api/voting_drive.dart';
import 'package:zkool/src/rust/frb_generated.dart';
import 'package:zkool/store.dart';

class _FakeCoin implements Coin {
  @override
  dynamic noSuchMethod(Invocation invocation) => super.noSuchMethod(invocation);
}

/// A run whose status stream the test feeds, and that records cancels.
class _FakeDrive implements VotingDrive {
  final statuses = StreamController<VotingDriveStatus>();
  int cancels = 0;
  bool running = false;
  int? account;

  @override
  Stream<VotingDriveStatus> run({
    required String roundId,
    required String lightwalletdUrl,
    required String roundName,
    required int account,
    required Coin c,
  }) {
    running = true;
    this.account = account;
    return statuses.stream;
  }

  @override
  void cancel() => cancels++;

  @override
  bool isRunning() => running;

  @override
  VotingDriveStatus? status() => null;

  @override
  dynamic noSuchMethod(Invocation invocation) => super.noSuchMethod(invocation);
}

class _Api implements RustLibApi {
  final drives = <_FakeDrive>[];

  @override
  Coin crateApiCoinCoinNew({int? defaultCoin}) => _FakeCoin();

  @override
  VotingDrive crateApiVotingDriveVotingDriveNew() {
    final drive = _FakeDrive();
    drives.add(drive);
    return drive;
  }

  @override
  dynamic noSuchMethod(Invocation invocation) => super.noSuchMethod(invocation);
}

VotingDriveStatus _stopped(String roundId) => VotingDriveStatus(
      roundId: roundId,
      phase: 'idle',
      running: false,
      dispatches: BigInt.zero,
      completedProposals: 0,
      totalProposals: 0,
      remainingObligations: 0,
      sharesConfirmed: 0,
      sharesTotal: 0,
      quiescence: 'cancelled',
      failures: const [],
    );

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();
  final api = _Api();
  setUpAll(() => RustLib.initMock(api: api));
  tearDownAll(RustLib.dispose);

  test('two accounts in one round: each run is tracked on its own', () async {
    api.drives.clear();
    final container = ProviderContainer();
    addTearDown(container.dispose);
    final guard = container.read(votingSubmissionGuardProvider.notifier);

    final a = container.read(votingDriveJobProvider('round-1', 1).notifier);
    await a.start(lightwalletdUrl: 'http://lwd', roundName: 'Round 1');
    final b = container.read(votingDriveJobProvider('round-1', 2).notifier);
    await b.start(lightwalletdUrl: 'http://lwd', roundName: 'Round 1');
    expect(api.drives, hasLength(2));
    final driveA = api.drives.firstWhere((d) => d.account == 1);
    final driveB = api.drives.firstWhere((d) => d.account == 2);
    expect(container.read(votingSubmissionGuardProvider), isTrue);

    // Account 1's run ends while account 2's is still live.
    driveA.statuses.add(_stopped('round-1'));
    await pumpEventQueue();
    expect(container.read(votingSubmissionGuardProvider), isTrue,
        reason: "account 2's run is still live, so the submission guard must hold");

    // A wallet-level stop must reach the run that is still live.
    container.read(votingDriveStopAllProvider.notifier).run();
    expect(driveB.cancels, 1, reason: "stop-all must cancel account 2's run");
    expect(driveA.cancels, 0, reason: 'a run that already stopped is not cancelled');

    guard.setActive(false); // leave the container clean
  });
}
