import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import 'package:zkool/main.dart';
import 'package:zkool/src/rust/api/voting.dart';
import 'package:zkool/store.dart';
import 'package:zkool/utils.dart';

/// Receipt screen shown after the submission job completes.
class VotingConfirmationPage extends ConsumerStatefulWidget {
  final String roundId;
  final String? roundName;
  final String chainUrl;

  const VotingConfirmationPage({
    super.key,
    required this.roundId,
    this.roundName,
    this.chainUrl = "",
  });

  @override
  ConsumerState<VotingConfirmationPage> createState() =>
      VotingConfirmationPageState();
}

class VotingConfirmationPageState extends ConsumerState<VotingConfirmationPage> {
  @override
  Widget build(BuildContext context) {
    final pinlock = ref.watch(lifecycleProvider);
    if (pinlock.value ?? false) return PinLock();

    final job = ref.watch(votingSubmissionMonitorProvider(widget.roundId));
    // Confirmed vote txs (per proposal), shown as on-chain evidence.
    final confirmedVotes = (ref
                .watch(votingSessionProvider(widget.roundId))
                .value
                ?.recovery
                ?.votes ??
            const <VotingVoteRecovery>[])
        .where((v) => v.phase == "confirmed" && (v.txHash ?? "").isNotEmpty)
        .toList();
    // Human-readable ballot evidence: proposal titles and option labels,
    // keyed by proposal id (not list position).
    final proposals = {
      for (final p in (ref
                  .watch(votingRoundProposalsProvider(
                    widget.roundId,
                    widget.chainUrl,
                  ))
                  .value ??
              const <VotingProposalInfo>[]))
        p.id: p,
    };

    String voteLabel(VotingVoteRecovery v) {
      final proposal = proposals[v.proposalId];
      final title = proposal?.title ?? "Proposal ${v.proposalId}";
      // Option ids are the vote-sdk `index` values (omitted = 0 for the
      // first); never render 1-based list positions.
      final choice = proposal?.optionLabels[v.choice] ?? "Option ${v.choice}";
      return "$title — $choice";
    }

    return Scaffold(
      appBar: AppBar(title: const Text("Vote submitted")),
      body: Center(
        child: Padding(
          padding: const EdgeInsets.all(24),
          child: Column(
            mainAxisAlignment: MainAxisAlignment.center,
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              Icon(
                Icons.check_circle_outline,
                size: 72,
                color: Theme.of(context).colorScheme.primary,
              ),
              const SizedBox(height: 16),
              Text(
                widget.roundName != null &&
                        widget.roundName!.isNotEmpty &&
                        widget.roundName != widget.roundId
                    ? "Your vote for ${widget.roundName} has been submitted."
                    : "Your vote for ${widget.roundId} has been submitted.",
                style: Theme.of(context).textTheme.titleMedium,
                textAlign: TextAlign.center,
              ),
              if (job.eligibleWeightZatoshi != null)
                Padding(
                  padding: const EdgeInsets.only(top: 8),
                  child: Text(
                    "Voting power: ${formatVotingPower(job.eligibleWeightZatoshi!)}",
                    style: Theme.of(context).textTheme.bodyMedium,
                    textAlign: TextAlign.center,
                  ),
                ),
              for (final v in confirmedVotes)
                Padding(
                  padding: const EdgeInsets.only(top: 8),
                  child: SelectableText(
                    "${voteLabel(v)}\n${v.txHash} · tree ${v.vcTreePosition}",
                    textAlign: TextAlign.center,
                    style: Theme.of(context).textTheme.bodySmall,
                  ),
                ),
              const SizedBox(height: 24),
              FilledButton(
                onPressed: () => GoRouter.of(context).go("/voting"),
                child: const Text("Done"),
              ),
            ],
          ),
        ),
      ),
    );
  }
}
