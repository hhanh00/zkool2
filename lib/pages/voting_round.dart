import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
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
  /// Durable round state polled while this page is open: it routes an
  /// already-voted round to the status view and drives share confirmation.
  VotingDriveStatus? _shareStatus;
  Timer? _shareTimer;
  bool _sharePassRunning = false;

  @override
  void initState() {
    super.initState();
    unawaited(_loadSelections());
    _startShareRefresh();
  }

  @override
  void dispose() {
    _shareTimer?.cancel();
    super.dispose();
  }

  void _startShareRefresh() {
    unawaited(_refreshShares());
    _shareTimer ??=
        Timer.periodic(const Duration(seconds: 5), (_) => unawaited(_refreshShares()));
  }

  /// Polls the round's durable state and, while shares are pending and no drive
  /// run is live, runs one helper-share tracking pass so confirmation advances
  /// without a resident background driver.
  Future<void> _refreshShares() async {
    try {
      final status = await votingDriveStatus(roundId: widget.round.roundId, c: _coin);
      if (!mounted) return;
      final pending = status.sharesTotal > status.sharesConfirmed;
      if (!pending) {
        _shareTimer?.cancel();
        _shareTimer = null;
      } else {
        // The single share mechanism: one pass per poll while the page is open,
        // whether or not a round driver run is on screen.
        unawaited(_trackSharesOnce());
      }
      setState(() => _shareStatus = status);
    } on Exception {
      // Transient status errors: the next tick retries.
    }
  }

  /// Runs one share tracking pass, overlapping with nothing.
  ///
  /// Guarded so a slow pass is not re-entered by the next 5s poll; a pass that
  /// confirms nothing simply runs again on the next tick.
  Future<void> _trackSharesOnce() async {
    if (_sharePassRunning || widget.round.helperUrls.isEmpty) return;
    _sharePassRunning = true;
    try {
      await votingTrackSharesOnce(
        roundId: widget.round.roundId,
        helperUrls: widget.round.helperUrls,
        voteEndTimeSeconds: widget.round.voteEndTime,
        c: _coin,
      );
    } on Exception {
      // A pass that fails leaves the rows durable; the next tick retries.
    } finally {
      _sharePassRunning = false;
    }
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
        // Resume at the last proposal already answered, so reopening a
        // partly-filled ballot continues where the voter left off instead of
        // starting at the first proposal.
        var lastAnswered = -1;
        for (var i = 0; i < widget.round.proposals.length; i++) {
          final id = widget.round.proposals[i].proposalId;
          if (_selections.containsKey(id) || _skipped.contains(id)) lastAnswered = i;
        }
        if (lastAnswered >= 0) _currentStep = lastAnswered;
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
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: SelectableText('Could not save selection: $e')),
        );
      }
      return false;
    } finally {
      if (mounted) setState(() => _saving = false);
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
          const SnackBar(content: SelectableText('Voting submission is disabled in offline mode')),
        );
      }
      return;
    }
    final job = ref.read(votingDriveJobProvider(widget.round.roundId).notifier);
    // A fresh prepare already ran for this ballot: reuse its preview. The
    // eligible note set is snapshot-determined, so it cannot have changed.
    final cached = ref.read(votingDriveJobProvider(widget.round.roundId));
    final prepared = (cached.stage == "ready" && cached.eligibility != null) ||
        await job.prepare(
          lightwalletdUrl: settings.lwd,
          roundName: widget.round.title,
        );
    if (!prepared) {
      if (mounted) {
        final error = ref.read(votingDriveJobProvider(widget.round.roundId)).error;
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
            content: SelectableText('Could not prepare the round: ${error ?? "unknown error"}'),
            duration: const Duration(seconds: 10),
          ),
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
    if (!mounted) return;
    final stage = ref.read(votingDriveJobProvider(widget.round.roundId)).stage;
    if (stage == "error") {
      final error = ref.read(votingDriveJobProvider(widget.round.roundId)).error;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: SelectableText('Could not start the submission: ${error ?? "unknown error"}'),
          duration: const Duration(seconds: 10),
        ),
      );
    }
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
    final status = _effectiveStatus(job);
    final showStatus = job.stage == "starting" ||
        job.stage == "driving" ||
        (status != null && status.sharesTotal > 0);
    Widget body;
    if (job.stage == "preparing") {
      body = const _PreparingView();
    } else if (showStatus) {
      body = _DriveStatusView(
        job: job,
        status: status,
        onDone: () => context.pop(),
        onCancel: () =>
            ref.read(votingDriveJobProvider(widget.round.roundId).notifier).cancel(),
        onResume: () async {
          final lwd = await _lwdUrl();
          await ref
              .read(votingDriveJobProvider(widget.round.roundId).notifier)
              .start(lightwalletdUrl: lwd, roundName: widget.round.title);
        },
      );
    } else {
      body = _buildBallot();
    }
    return Scaffold(
      appBar: AppBar(
        title: Text(widget.round.title),
        actions: [
          IconButton(
            icon: const Icon(Icons.receipt_long_outlined),
            tooltip: 'Vote report',
            onPressed: _showReport,
          ),
        ],
      ),
      body: body,
    );
  }

  /// Human-readable choice recorded for one proposal.
  String _choiceText(VotingProposalListItem proposal) {
    if (_skipped.contains(proposal.proposalId)) return 'Skipped';
    final choice = _selections[proposal.proposalId];
    if (choice == null) return 'Not answered';
    if (choice < 0 || choice >= proposal.options.length) return 'Choice $choice';
    return proposal.options[choice];
  }

  /// Report of the choices made in this round, one row per proposal.
  void _showReport() {
    final proposals = widget.round.proposals;
    final answered = proposals
        .where((p) => _selections.containsKey(p.proposalId) || _skipped.contains(p.proposalId))
        .length;
    showDialog<void>(
      context: context,
      builder: (context) => AlertDialog(
        title: Text('Vote report — ${widget.round.title}'),
        content: SizedBox(
          width: 480,
          child: proposals.isEmpty
              ? const Text('No proposals recorded for this round.')
              : Column(
                  mainAxisSize: MainAxisSize.min,
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text('$answered of ${proposals.length} proposals answered'),
                    const SizedBox(height: 8),
                    Flexible(
                      child: ListView.separated(
                        shrinkWrap: true,
                        itemCount: proposals.length,
                        separatorBuilder: (_, __) => const Divider(height: 12),
                        itemBuilder: (context, index) {
                          final proposal = proposals[index];
                          return Column(
                            crossAxisAlignment: CrossAxisAlignment.start,
                            children: [
                              Text(
                                proposal.title,
                                style: const TextStyle(fontWeight: FontWeight.bold),
                              ),
                              const SizedBox(height: 2),
                              Text('Choice: ${_choiceText(proposal)}'),
                            ],
                          );
                        },
                      ),
                    ),
                  ],
                ),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(context).pop(),
            child: const Text('Close'),
          ),
        ],
      ),
    );
  }

  /// Status shown by the page, synthesized for an already-voted round opened
  /// with no live in-process run (the driver registry is process-local).
  VotingDriveStatus? _effectiveStatus(VotingDriveJobState job) {
    final status = _shareStatus ?? job.status;
    if (status == null || status.running || status.quiescence != null) {
      return status;
    }
    if (status.sharesTotal == 0) return status;
    return status.copyWith(
      quiescence: status.sharesConfirmed >= status.sharesTotal
          ? 'done'
          : 'background_shares(${status.sharesTotal - status.sharesConfirmed})',
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
                  SelectableText(_loadError!),
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

/// Immediate feedback while the explicit prepare runs: config, round, and
/// lightwalletd fetches plus bundle setup take several seconds.
class _PreparingView extends StatelessWidget {
  const _PreparingView();

  @override
  Widget build(BuildContext context) {
    return Center(
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          const CircularProgressIndicator(),
          const SizedBox(height: 16),
          Text('Preparing vote…', style: Theme.of(context).textTheme.titleMedium),
          const SizedBox(height: 4),
          Text(
            'Fetching the round configuration and building the bundle plan',
            style: Theme.of(context).textTheme.bodySmall,
            textAlign: TextAlign.center,
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
  /// Effective status for this page. Prefers the durable poll over the job's
  /// process-local snapshot, so an already-voted round reopened with no live
  /// run still reads correctly.
  final VotingDriveStatus? status;
  final VoidCallback onDone;
  final VoidCallback onCancel;
  final Future<void> Function() onResume;

  const _DriveStatusView({
    required this.job,
    required this.status,
    required this.onDone,
    required this.onCancel,
    required this.onResume,
  });

  @override
  Widget build(BuildContext context) {
    final status = this.status ?? job.status;
    final running = status?.running ?? true;
    final theme = Theme.of(context);
    final failures = status?.failures ?? const <String>[];
    final progress = switch (status) {
      null => null,
      final s when s.totalProposals > 0 => s.completedProposals / s.totalProposals,
      final s when s.sharesTotal > 0 => s.sharesConfirmed / s.sharesTotal,
      _ => null,
    };
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
              job.stage == "starting"
                  ? "Starting submission"
                  : votingQuiescenceLabel(running ? null : status?.quiescence),
              textAlign: TextAlign.center,
              style: theme.textTheme.titleMedium,
            ),
            const SizedBox(height: 16),
            LinearProgressIndicator(value: progress, minHeight: 8),
            const SizedBox(height: 8),
            if (status != null) ...[
              if (status.totalProposals > 0)
                Text(
                  '${status.completedProposals} of ${status.totalProposals} proposals',
                  textAlign: TextAlign.center,
                ),
              if (status.sharesTotal > 0 &&
                  status.sharesConfirmed < status.sharesTotal) ...[
                const SizedBox(height: 4),
                Text(
                  '${status.sharesTotal - status.sharesConfirmed} of '
                  '${status.sharesTotal} helper shares left to confirm',
                  textAlign: TextAlign.center,
                  style: theme.textTheme.bodySmall,
                ),
                const SizedBox(height: 4),
                Text(
                  'The wallet confirms shares as helpers reveal them. Keep the '
                  'app open to make progress.',
                  textAlign: TextAlign.center,
                  style: theme.textTheme.bodySmall,
                ),
              ] else if (running) ...[
                const SizedBox(height: 4),
                Text(
                  '${status.dispatches} dispatches • '
                  '${status.remainingObligations} obligations left',
                  textAlign: TextAlign.center,
                  style: theme.textTheme.bodySmall,
                ),
              ],
            ],
            if (job.error != null) ...[
              const SizedBox(height: 12),
              SelectableText(job.error!, style: TextStyle(color: theme.colorScheme.error)),
            ],
            if (failures.isNotEmpty) ...[
              const SizedBox(height: 12),
              for (final failure in failures)
                Padding(
                  padding: const EdgeInsets.only(bottom: 4),
                  child: SelectableText(failure, style: TextStyle(color: theme.colorScheme.error)),
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
                'done' => FilledButton(onPressed: onDone, child: const Text('Done')),
                // Anything else (including share delivery still in progress) is
                // not done: label the exit honestly.
                _ => FilledButton(onPressed: onDone, child: const Text('Close')),
              },
            ],
          ],
        ),
      ),
    );
  }
}

