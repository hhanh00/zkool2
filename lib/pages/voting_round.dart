import 'package:flutter/material.dart';
import 'package:zkool/src/rust/api/voting.dart';

class VotingRoundPage extends StatefulWidget {
  final VotingRoundListItem round;

  const VotingRoundPage({super.key, required this.round});

  @override
  State<VotingRoundPage> createState() => _VotingRoundPageState();
}

class _VotingRoundPageState extends State<VotingRoundPage> {
  int _currentStep = 0;
  final Map<int, int> _selections = {};
  final Set<int> _skipped = {};

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
                                  // TODO: Submit or review the completed ballot.
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
