import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import 'package:zkool/main.dart' show logger;
import 'package:zkool/src/rust/api/voting.dart';
import 'package:zkool/src/rust/api/voting_drive.dart';
import 'package:zkool/src/rust/api/voting_share_tracking.dart';
import 'package:zkool/store.dart'
    show
        VotingDriveJobState,
        appSettingsProvider,
        coinContext,
        votingDriveJobProvider,
        votingQuiescenceLabel;
import 'package:zkool/utils.dart' show zatToString;

class VotingRoundPage extends ConsumerStatefulWidget {
  final VotingRoundListItem round;

  const VotingRoundPage({super.key, required this.round});

  @override
  ConsumerState<VotingRoundPage> createState() => _VotingRoundPageState();
}

class _VotingRoundPageState extends ConsumerState<VotingRoundPage> {
  int _currentStep = 0;
  late final _coin = coinContext.coin;
  bool _loading = true;
  bool _saving = false;
  String? _loadError;
  final Map<int, int> _selections = {};
  final Set<int> _skipped = {};

  @override
  void initState() {
    super.initState();
    unawaited(_resumeShareTracking());
    unawaited(_loadSelections());
  }

  Future<void> _loadSelections() async {
    setState(() {
      _loading = true;
      _loadError = null;
    });
    try {
      final selections = await votingLoadSelections(roundId: widget.round.roundId, c: _coin);
      if (!mounted) return;
      setState(() {
        _selections.clear();
        _skipped.clear();
        for (final selection in selections) {
          final proposals = widget.round.proposals.where((p) => p.proposalId == selection.proposalId);
          if (proposals.isEmpty) continue;
          switch (selection.decision) {
            case Decision_Choice(:final choice):
              if (choice >= 0 && choice < proposals.first.options.length) {
                _selections[selection.proposalId] = choice;
              }
            case Decision_Skipped():
              _skipped.add(selection.proposalId);
          }
        }
      });
    } catch (e) {
      if (mounted) setState(() => _loadError = 'Could not load saved selections: $e');
    } finally {
      if (mounted) setState(() => _loading = false);
    }
  }

  Future<bool> _saveSelection(VotingProposalListItem proposal, Decision? decision) async {
    if (_saving) return false;
    setState(() => _saving = true);
    try {
      if (decision == null) {
        await votingClearSelection(roundId: widget.round.roundId, proposalId: proposal.proposalId, c: _coin);
      } else {
        await votingSaveSelection(
          roundId: widget.round.roundId,
          proposalId: proposal.proposalId,
          decision: decision,
          numOptions: proposal.options.length,
          c: _coin,
        );
      }
      if (!mounted) return false;
      setState(() {
        _selections.remove(proposal.proposalId);
        _skipped.remove(proposal.proposalId);
        switch (decision) {
          case Decision_Choice(:final choice):
            _selections[proposal.proposalId] = choice;
          case Decision_Skipped():
            _skipped.add(proposal.proposalId);
          case null:
            break;
        }
      });
      return true;
    } catch (e) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(SnackBar(content: Text('Could not save selection: $e')));
      }
      return false;
    } finally {
      if (mounted) setState(() => _saving = false);
    }
  }

  /// Restarts delivery of this round's persisted helper shares, if it has any.
  ///
  /// Opening a round adds no network of its own: the helper fleet and
  /// vote-end boundary were resolved when the user loaded the round list and
  /// travel on [VotingRoundListItem], and the pending-share check is a local
  /// sidecar query. The only traffic is the tracker's own helper delivery,
  /// which is the work being resumed. Offline mode suppresses even that, and
  /// it is deliberately not re-run on app resume — returning to the
  /// foreground is not a request to contact vote servers.
  ///
  /// Best-effort: a round the user is about to vote in must not fail to open
  /// because recovery for an earlier ballot could not reach the helpers.
  Future<void> _resumeShareTracking() async {
    final round = widget.round;
    // Opening a round is a deliberate navigation, so every outcome is worth a
    // line: when delivery does not resume, the reason is the thing to know.
    void skipped(String reason) => logger.i(
          "[Voting] share daemon not resumed for round ${round.roundId}: $reason",
        );

    if (round.helperUrls.isEmpty) {
      skipped("the round list carried no helper servers");
      return;
    }
    // Read before the first await: a widget ref dies with its page and throws
    // if it is touched after the user has navigated away.
    final settings = ref.read(appSettingsProvider.future);
    try {
      if ((await settings).offline) {
        skipped("offline mode is on");
        return;
      }
      final pending = await votingPendingShareRounds(c: coinContext.coin);
      if (!pending.any((r) => r.roundId == round.roundId)) {
        skipped("no shares are awaiting confirmation");
        return;
      }
      final started = await votingStartShareTracking(
        roundId: round.roundId,
        helperUrls: round.helperUrls,
        voteEndTimeSeconds: round.voteEndTime,
        c: coinContext.coin,
      );
      logger.i(
        "[Voting] share daemon resume for round ${round.roundId}: "
        "${started ? "started" : "already running"}, "
        "${round.helperUrls.length} helpers, "
        "vote end ${round.voteEndTime ?? "unknown"}",
      );
    } on Exception catch (e) {
      logger.e("[Voting] share daemon resume failed: $e");
    }
  }

  String? _selectionLabel(VotingProposalListItem proposal) {
    final index = _selections[proposal.proposalId];
    return index == null || index >= proposal.options.length ? null : proposal.options[index];
  }

  /// Ballot complete → explicit prepare (eligibility preview) → confirm
  /// dialog → start the driver run. The durable work happens in Rust; this
  /// page only observes the polled status afterwards.
  Future<void> _onBallotComplete() async {
    final settings = await ref.read(appSettingsProvider.future);
    if (settings.offline) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          const SnackBar(content: Text('Voting submission is disabled in offline mode')),
        );
      }
      return;
    }
    final job = ref.read(votingDriveJobProvider(widget.round.roundId).notifier);
    final prepared = await job.prepare(lightwalletdUrl: settings.lwd, roundName: widget.round.title);
    if (!prepared) {
      if (mounted) {
        final error = ref.read(votingDriveJobProvider(widget.round.roundId)).error;
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text('Could not prepare the round: ${error ?? "unknown error"}')),
        );
      }
      return;
    }
    if (!mounted) return;
    final confirmed = await _confirmSubmission(
      ref.read(votingDriveJobProvider(widget.round.roundId)).eligibility,
    );
    if (!confirmed || !mounted) return;
    await job.start(lightwalletdUrl: settings.lwd, roundName: widget.round.title);
  }

  /// Eligibility preview plus the finality warning; the last gate before the
  /// driver is allowed to submit.
  Future<bool> _confirmSubmission(VotingEligibilityPreview? preview) async {
    if (preview == null) return false;
    final trim = preview.privacyTrimDroppedValueZatoshi;
    final result = await showDialog<bool>(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('Submit votes?'),
        content: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text('${preview.noteCount} notes'),
            const SizedBox(height: 4),
            Text('${zatToString(preview.eligibleWeightZatoshi)} ZEC voting weight'),
            if (!preview.isEligible) ...[
              const SizedBox(height: 8),
              const Text(
                'This weight is below the round\'s minimum voting rule.',
                style: TextStyle(fontWeight: FontWeight.bold),
              ),
            ],
            if (trim > BigInt.zero) ...[
              const SizedBox(height: 8),
              Text(
                'The privacy trim withholds ${zatToString(trim)} ZEC '
                '(${preview.skippedSuffixBundles} bundles, ${preview.skippedSuffixNotes} notes).',
              ),
            ],
            const SizedBox(height: 12),
            const Text(
              'Votes are final once submission starts and cannot be changed after confirmation.',
              style: TextStyle(fontWeight: FontWeight.bold),
            ),
          ],
        ),
        actions: [
          TextButton(onPressed: () => Navigator.of(context).pop(false), child: const Text('Cancel')),
          FilledButton(onPressed: () => Navigator.of(context).pop(true), child: const Text('Submit votes')),
        ],
      ),
    );
    return result ?? false;
  }

  Future<String> _lwdUrl() async => (await ref.read(appSettingsProvider.future)).lwd;

  @override
  Widget build(BuildContext context) {
    final job = ref.watch(votingDriveJobProvider(widget.round.roundId));
    return Scaffold(
      appBar: AppBar(title: Text(widget.round.title)),
      body: job.stage == "driving"
          ? _DriveStatusView(
              job: job,
              onDone: () => context.pop(),
              onCancel: () => ref
                  .read(votingDriveJobProvider(widget.round.roundId).notifier)
                  .cancel(),
              onResume: () async {
                final lwd = await _lwdUrl();
                await ref
                    .read(votingDriveJobProvider(widget.round.roundId).notifier)
                    .start(lightwalletdUrl: lwd, roundName: widget.round.title);
              },
            )
          : _buildBallot(),
    );
  }

  Widget _buildBallot() {
    final proposals = widget.round.proposals;
    final ballotComplete = proposals.every(
      (proposal) => _selections.containsKey(proposal.proposalId) || _skipped.contains(proposal.proposalId),
    );
    final secondaryColor = Theme.of(context).colorScheme.secondary;
    return _loading
        ? const Center(child: CircularProgressIndicator())
          : _loadError != null
              ? Center(
                  child: Column(mainAxisSize: MainAxisSize.min, children: [
                  Text(_loadError!),
                  TextButton(onPressed: _loadSelections, child: const Text('Retry')),
                ]))
              : proposals.isEmpty
                  ? const Center(child: Text('No proposals found for this round'))
                  : AbsorbPointer(
                      absorbing: _saving,
                      child: Stepper(
                        currentStep: _currentStep,
                        stepIconBuilder: (index, state) {
                          if (state == StepState.complete) {
                            return Container(
                              decoration: BoxDecoration(
                                color: secondaryColor,
                                shape: BoxShape.circle,
                              ),
                              child: Icon(
                                Icons.check,
                                color: Theme.of(context).colorScheme.onSecondary,
                                size: 18,
                              ),
                            );
                          }
                          return null;
                        },
                        onStepTapped: (step) => setState(() => _currentStep = step),
                        onStepContinue: () {
                          if (_currentStep < proposals.length - 1) {
                            setState(() => _currentStep++);
                          }
                        },
                        onStepCancel: _currentStep == 0 ? null : () => setState(() => _currentStep--),
                        controlsBuilder: (context, details) => Padding(
                          padding: const EdgeInsets.only(top: 16),
                          child: Row(
                            children: [
                              FilledButton(
                                onPressed: _currentStep < proposals.length - 1
                                    ? () async {
                                        final proposal = proposals[_currentStep];
                                        if (!_selections.containsKey(proposal.proposalId) && !_skipped.contains(proposal.proposalId)) {
                                          if (!await _saveSelection(proposal, const Decision.skipped())) return;
                                        }
                                        if (mounted) details.onStepContinue?.call();
                                      }
                                    : ballotComplete
                                        ? () => unawaited(_onBallotComplete())
                                        : null,
                                child: Text(
                                  _currentStep == proposals.length - 1
                                      ? 'Submit'
                                      : _selections.containsKey(proposals[_currentStep].proposalId)
                                          ? 'Next'
                                          : 'Skip',
                                ),
                              ),
                              if (_currentStep > 0) ...[
                                const SizedBox(width: 8),
                                TextButton(onPressed: details.onStepCancel, child: const Text('Back')),
                              ],
                              if (!_selections.containsKey(proposals[_currentStep].proposalId) &&
                                  !_skipped.contains(proposals[_currentStep].proposalId) &&
                                  _currentStep == proposals.length - 1)
                                TextButton(
                                  onPressed: () => _saveSelection(proposals[_currentStep], const Decision.skipped()),
                                  child: const Text('Skip'),
                                ),
                              if (_selections.containsKey(proposals[_currentStep].proposalId) || _skipped.contains(proposals[_currentStep].proposalId)) ...[
                                const SizedBox(width: 8),
                                TextButton.icon(
                                  onPressed: () => _saveSelection(proposals[_currentStep], null),
                                  icon: const Icon(Icons.clear),
                                  label: const Text('Clear selection'),
                                ),
                              ],
                            ],
                          ),
                        ),
                        steps: [
                          for (final proposal in proposals)
                            Step(
                              title: Text(proposal.title),
                              subtitle: _skipped.contains(proposal.proposalId)
                                  ? const Text('Skipped')
                                  : switch (_selectionLabel(proposal)) {
                                      final label? => Text('Selected: $label'),
                                      _ => null,
                                    },
                              isActive: proposals.indexOf(proposal) <= _currentStep,
                              state: _selections.containsKey(proposal.proposalId) || _skipped.contains(proposal.proposalId)
                                  ? StepState.complete
                                  : StepState.indexed,
                              content: Column(
                                children: [
                                  for (var i = 0; i < proposal.options.length; i++)
                                    RadioListTile<int>(
                                      contentPadding: EdgeInsets.zero,
                                      title: Text(proposal.options[i]),
                                      value: i,
                                      groupValue: _selections[proposal.proposalId],
                                      onChanged: (value) {
                                        if (value != null) {
                                          unawaited(_saveSelection(proposal, Decision.choice(choice: value)));
                                        }
                                      },
                                    ),
                                ],
                              ),
                            ),
                        ],
                      ),
                    );
  }
}

/// Live view of one round's driver run: progress while running, the
/// quiescence outcome (and failures) once stopped, and resume/cancel/leave
/// actions derived from the polled status.
class _DriveStatusView extends StatelessWidget {
  final VotingDriveJobState job;
  final VoidCallback onDone;
  final VoidCallback onCancel;
  final Future<void> Function() onResume;

  const _DriveStatusView({
    required this.job,
    required this.onDone,
    required this.onCancel,
    required this.onResume,
  });

  @override
  Widget build(BuildContext context) {
    final status = job.status;
    final running = status?.running ?? true;
    final theme = Theme.of(context);
    final failures = status?.failures ?? const <String>[];
    final progress = (status != null && status.totalProposals > 0)
        ? status.completedProposals / status.totalProposals
        : null;
    return Center(
      child: SingleChildScrollView(
        padding: const EdgeInsets.all(24),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            Icon(
              running
                  ? Icons.how_to_vote_outlined
                  : switch (status?.quiescence) {
                      'done' => Icons.check_circle,
                      'cancelled' => Icons.cancel_outlined,
                      'failures' || 'chain_terminal' => Icons.error_outline,
                      _ => Icons.info_outline,
                    },
              size: 48,
              color: running
                  ? theme.colorScheme.secondary
                  : switch (status?.quiescence) {
                      'done' => Colors.green,
                      'failures' || 'chain_terminal' => theme.colorScheme.error,
                      _ => null,
                    },
            ),
            const SizedBox(height: 12),
            Text(
              votingQuiescenceLabel(running ? null : status?.quiescence),
              textAlign: TextAlign.center,
              style: theme.textTheme.titleMedium,
            ),
            const SizedBox(height: 16),
            LinearProgressIndicator(value: progress, minHeight: 8),
            const SizedBox(height: 8),
            if (status != null) ...[
              Text(
                '${status.completedProposals} of ${status.totalProposals} proposals',
                textAlign: TextAlign.center,
              ),
              const SizedBox(height: 4),
              Text(
                '${status.dispatches} dispatches • ${status.remainingObligations} obligations left',
                textAlign: TextAlign.center,
                style: theme.textTheme.bodySmall,
              ),
            ],
            if (job.error != null) ...[
              const SizedBox(height: 12),
              Text(job.error!, style: TextStyle(color: theme.colorScheme.error)),
            ],
            if (failures.isNotEmpty) ...[
              const SizedBox(height: 12),
              for (final failure in failures)
                Padding(
                  padding: const EdgeInsets.only(bottom: 4),
                  child: Text(failure, style: TextStyle(color: theme.colorScheme.error)),
                ),
            ],
            const SizedBox(height: 20),
            if (running)
              OutlinedButton.icon(
                onPressed: onCancel,
                icon: const Icon(Icons.close),
                label: const Text('Cancel submission'),
              )
            else ...[
              switch (status?.quiescence) {
                'cancelled' || 'pass_budget_exhausted' => FilledButton.icon(
                    onPressed: () => onResume(),
                    icon: const Icon(Icons.play_arrow),
                    label: const Text('Resume submission'),
                  ),
                _ => FilledButton(onPressed: onDone, child: const Text('Done')),
              },
            ],
          ],
        ),
      ),
    );
  }
}
