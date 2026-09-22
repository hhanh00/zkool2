import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:zkool/main.dart' show logger;
import 'package:zkool/src/rust/api/voting.dart';
import 'package:zkool/src/rust/api/voting_share_tracking.dart';
import 'package:zkool/store.dart' show appSettingsProvider, coinContext;

class VotingRoundPage extends ConsumerStatefulWidget {
  final VotingRoundListItem round;

  const VotingRoundPage({super.key, required this.round});

  @override
  ConsumerState<VotingRoundPage> createState() => _VotingRoundPageState();
}

class _VotingRoundPageState extends ConsumerState<VotingRoundPage> {
  int _currentStep = 0;
  final Map<int, int> _selections = {};
  final Set<int> _skipped = {};

  @override
  void initState() {
    super.initState();
    unawaited(_resumeShareTracking());
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

  @override
  Widget build(BuildContext context) {
    final proposals = widget.round.proposals;
    final ballotComplete = proposals.every(
      (proposal) => _selections.containsKey(proposal.proposalId) || _skipped.contains(proposal.proposalId),
    );
    final secondaryColor = Theme.of(context).colorScheme.secondary;
    return Scaffold(
      appBar: AppBar(title: Text(widget.round.title)),
      body: proposals.isEmpty
          ? const Center(child: Text('No proposals found for this round'))
          : Stepper(
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
                          ? () {
                              if (!_selections.containsKey(proposals[_currentStep].proposalId)) {
                                _skipped.add(proposals[_currentStep].proposalId);
                              }
                              details.onStepContinue?.call();
                            }
                          : ballotComplete
                              ? () {
                                  // TODO(next): serialize selections/skips as
                                  // DraftVote JSON and hand the completed
                                  // ballot to the existing voting submission job.
                                }
                              : null,
                      child: Text(
                        _currentStep == proposals.length - 1
                            ? 'Done'
                            : _selections.containsKey(proposals[_currentStep].proposalId)
                                ? 'Next'
                                : 'Skip',
                      ),
                    ),
                    if (_currentStep > 0) ...[
                      const SizedBox(width: 8),
                      TextButton(onPressed: details.onStepCancel, child: const Text('Back')),
                    ],
                    if (_selections.containsKey(proposals[_currentStep].proposalId)) ...[
                      const SizedBox(width: 8),
                      TextButton.icon(
                        onPressed: () => setState(
                          () => _selections.remove(proposals[_currentStep].proposalId),
                        ),
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
                    state: _selections.containsKey(proposal.proposalId) || _skipped.contains(proposal.proposalId) ? StepState.complete : StepState.indexed,
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
                                setState(() {
                                  _skipped.remove(proposal.proposalId);
                                  _selections[proposal.proposalId] = value;
                                });
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
