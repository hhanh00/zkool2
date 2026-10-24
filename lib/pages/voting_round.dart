import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import 'package:zkool/src/rust/api/voting.dart';
import 'package:zkool/src/rust/api/voting_drive.dart';
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

  final int account;
  const VotingRoundPage({super.key, required this.round, required this.account});

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
    unawaited(_loadSelections());
    unawaited(_resumeSubmission());
  }

  /// Reads where the round stands and resumes a submission started earlier
  /// and not settled: the run is idempotent, so no question is asked. The
  /// standing also routes an already-voted round to the status view.
  Future<void> _resumeSubmission() async {
    final settings = await ref.read(appSettingsProvider.future);
    if (!mounted) return;
    await ref.read(votingDriveJobProvider(widget.round.roundId, widget.account).notifier).resume(
          lightwalletdUrl: settings.lwd,
          roundName: widget.round.title,
          proposalIds: [for (final p in widget.round.proposals) p.proposalId],
          offline: settings.offline,
        );
  }

  Future<void> _loadSelections() async {
    setState(() {
      _loading = true;
      _loadError = null;
    });
    try {
      final selections = await votingLoadSelections(roundId: widget.round.roundId, account: widget.account, c: _coin);
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
        await votingClearSelection(roundId: widget.round.roundId, proposalId: proposal.proposalId, account: widget.account, c: _coin);
      } else {
        await votingSaveSelection(
          roundId: widget.round.roundId,
          proposalId: proposal.proposalId,
          decision: decision,
          numOptions: proposal.options.length,
          account: widget.account,
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
  /// dialog → start the run. The durable work happens in Rust; this page
  /// only displays the streamed status afterwards.
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
    final job = ref.read(votingDriveJobProvider(widget.round.roundId, widget.account).notifier);
    // A fresh prepare already ran for this ballot: reuse its preview. The
    // eligible note set is snapshot-determined, so it cannot have changed.
    final cached = ref.read(votingDriveJobProvider(widget.round.roundId, widget.account));
    final prepared = (cached.stage == "ready" && cached.eligibility != null) ||
        await job.prepare(
          lightwalletdUrl: settings.lwd,
          roundName: widget.round.title,
        );
    if (!prepared) {
      if (mounted) {
        final error = ref.read(votingDriveJobProvider(widget.round.roundId, widget.account)).error;
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
      ref.read(votingDriveJobProvider(widget.round.roundId, widget.account)).eligibility,
    );
    if (!confirmed || !mounted) return;
    await job.start(lightwalletdUrl: settings.lwd, roundName: widget.round.title);
    if (!mounted) return;
    final stage = ref.read(votingDriveJobProvider(widget.round.roundId, widget.account)).stage;
    if (stage == "error") {
      final error = ref.read(votingDriveJobProvider(widget.round.roundId, widget.account)).error;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: SelectableText('Could not start the submission: ${error ?? "unknown error"}'),
          duration: const Duration(seconds: 10),
        ),
      );
    }
  }

  /// Treat proposals the user did not visit as skipped, then submit the ballot.
  /// Skipping is distinct from choosing an abstention option: no proposal
  /// option is recorded for these entries.
  Future<void> _submitBallot() async {
    for (final proposal in widget.round.proposals) {
      if (_selections.containsKey(proposal.proposalId) || _skipped.contains(proposal.proposalId)) {
        continue;
      }
      if (!await _saveSelection(proposal, const Decision.skipped())) return;
    }
    if (mounted) await _onBallotComplete();
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
    final job = ref.watch(votingDriveJobProvider(widget.round.roundId, widget.account));
    final notifier = ref.read(votingDriveJobProvider(widget.round.roundId, widget.account).notifier);
    // The status view shows a run of this process, or a submission the
    // sidecar knows about: started earlier, settled or not.
    final showStatus = job.stage == "starting" ||
        job.stage == "driving" ||
        (job.stage == "error" && job.status != null) ||
        (job.standing?.submissionStarted ?? false);
    Widget body;
    if (job.stage == "preparing") {
      body = const _PreparingView();
    } else if (showStatus) {
      body = _DriveStatusView(
        job: job,
        onDone: () => context.pop(),
        onCancel: notifier.cancel,
        onResume: () async {
          final lwd = await _lwdUrl();
          await notifier.start(lightwalletdUrl: lwd, roundName: widget.round.title);
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

  Widget _buildBallot() {
    final proposals = widget.round.proposals;
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
                                    : () => unawaited(_submitBallot()),
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

/// Display of one round's submission: what the run streams while it is
/// live, its outcome once it stopped, or the sidecar's standing for a round
/// opened with no run in this process. The only actions are cancel while
/// casting, resume after a stop, and leaving the page.
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
    final theme = Theme.of(context);
    final status = job.status;
    final standing = job.standing;
    final starting = job.stage == "starting";
    final running = starting || (status?.running ?? false);
    final confirming = running && status?.phase == "confirming_shares";
    final sharesTotal = status?.sharesTotal ?? standing?.sharesTotal ?? 0;
    final sharesConfirmed = status?.sharesConfirmed ?? standing?.sharesConfirmed ?? 0;
    final failures = status?.failures ?? const <String>[];

    // The headline and the exits follow the run's phase; a round with no run
    // in this process reads from its standing.
    final String headline;
    final Widget? action;
    final List<Widget> exits;
    if (starting || (running && !confirming)) {
      headline = "Submitting votes";
      action = OutlinedButton.icon(
        onPressed: onCancel,
        icon: const Icon(Icons.close),
        label: const Text('Cancel submission'),
      );
      exits = const [];
    } else if (confirming) {
      headline = "Votes cast, confirming helper shares";
      action = null;
      exits = [FilledButton(onPressed: onDone, child: const Text('Close'))];
    } else if (status != null) {
      headline = votingQuiescenceLabel(status.quiescence);
      action = null;
      exits = [
        if (status.quiescence case 'cancelled' || 'chain_recovery_stalled' || 'pass_budget_exhausted')
          FilledButton.icon(
            onPressed: () => onResume(),
            icon: const Icon(Icons.play_arrow),
            label: const Text('Resume submission'),
          ),
        FilledButton(onPressed: onDone, child: Text(status.quiescence == 'done' ? 'Done' : 'Close')),
      ];
    } else if (job.stage == "error") {
      headline = "Submission failed";
      action = null;
      exits = [
        FilledButton.icon(
          onPressed: () => onResume(),
          icon: const Icon(Icons.play_arrow),
          label: const Text('Try again'),
        ),
        FilledButton(onPressed: onDone, child: const Text('Close')),
      ];
    } else if (standing?.workPending ?? false) {
      headline = "Submission interrupted";
      action = null;
      exits = [
        FilledButton.icon(
          onPressed: () => onResume(),
          icon: const Icon(Icons.play_arrow),
          label: const Text('Resume submission'),
        ),
        FilledButton(onPressed: onDone, child: const Text('Close')),
      ];
    } else {
      headline = "Votes submitted";
      action = null;
      exits = [FilledButton(onPressed: onDone, child: const Text('Done'))];
    }
    final done = headline == "Votes submitted";
    final failed = !running && !done && (job.error != null || failures.isNotEmpty || status?.quiescence == 'chain_terminal');

    final progress = switch (status) {
      null => null,
      final s when s.running && s.phase == "confirming_shares" && s.sharesTotal > 0 =>
        s.sharesConfirmed / s.sharesTotal,
      final s when s.running && s.totalProposals > 0 => s.completedProposals / s.totalProposals,
      final s when s.running => null,
      _ => 1.0,
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
                  : done
                      ? Icons.check_circle
                      : failed
                          ? Icons.error_outline
                          : Icons.info_outline,
              size: 48,
              color: running
                  ? theme.colorScheme.secondary
                  : done
                      ? Colors.green
                      : failed
                          ? theme.colorScheme.error
                          : null,
            ),
            const SizedBox(height: 12),
            Text(headline, textAlign: TextAlign.center, style: theme.textTheme.titleMedium),
            const SizedBox(height: 16),
            if (running || done) ...[
              LinearProgressIndicator(value: progress, minHeight: 8),
              const SizedBox(height: 8),
            ],
            if (running && !confirming && status != null && status.totalProposals > 0)
              Text(
                '${status.completedProposals} of ${status.totalProposals} proposals',
                textAlign: TextAlign.center,
              ),
            if (running && !confirming) ...[
              const SizedBox(height: 4),
              Text(
                'Proving may take a few minutes per note. Please keep the app open and stay on this page if possible.',
                textAlign: TextAlign.center,
                style: theme.textTheme.bodySmall,
              ),
            ],
            if (confirming || (!running && sharesTotal > 0)) ...[
              Text(
                '$sharesConfirmed of $sharesTotal helper shares confirmed',
                textAlign: TextAlign.center,
              ),
            ],
            if (confirming) ...[
              const SizedBox(height: 4),
              Text(
                'Stay on this page if possible. The submission continues in the background if you leave it; '
                'if the app is closed, open this round again to resume.',
                textAlign: TextAlign.center,
                style: theme.textTheme.bodySmall,
              ),
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
            if (action != null) action,
            for (final exit in exits) ...[exit, const SizedBox(height: 8)],
          ],
        ),
      ),
    );
  }
}
