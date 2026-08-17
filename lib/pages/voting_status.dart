import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import 'package:zkool/main.dart';
import 'package:zkool/src/rust/api/voting.dart';
import 'package:zkool/store.dart';
import 'package:zkool/utils.dart';

/// Delegation execution screen: watches the submission job for a round,
/// renders per-stage progress, blocks leaving mid-run, and offers Retry on
/// error / Done on completion.
class VotingStatusPage extends ConsumerStatefulWidget {
  final String roundId;
  final String chainUrl;
  final String pirServerUrl;
  final VotingPirLayout? pirLayout;
  final String? roundParamsJson;
  final String? roundName;
  final int? maxRealNotesPerBundle;
  final String? lightwalletdUrl;
  final String voteNodeUrl;
  final int ceremonyStart;
  final int? voteEnd;
  final List<String> shareServerUrls;
  final bool singleShare;

  const VotingStatusPage({
    super.key,
    required this.roundId,
    required this.chainUrl,
    required this.pirServerUrl,
    this.pirLayout,
    this.roundParamsJson,
    this.roundName,
    this.maxRealNotesPerBundle,
    this.lightwalletdUrl,
    this.voteNodeUrl = "",
    this.ceremonyStart = 0,
    this.voteEnd,
    this.shareServerUrls = const [],
    this.singleShare = false,
  });

  @override
  ConsumerState<VotingStatusPage> createState() => VotingStatusPageState();
}

class VotingStatusPageState extends ConsumerState<VotingStatusPage> {
  bool _handlingLeave = false;

  @override
  void initState() {
    super.initState();
    Future(_start);
  }

  Future<void> _start() async {
    await ref
        .read(votingSubmissionJobProvider(widget.roundId).notifier)
        .start(
          chainUrl: widget.chainUrl,
          pirServerUrl: widget.pirServerUrl,
          pirLayout: widget.pirLayout,
          roundParamsJson: widget.roundParamsJson,
          roundName: widget.roundName,
          maxRealNotesPerBundle: widget.maxRealNotesPerBundle,
          lightwalletdUrl: widget.lightwalletdUrl,
          voteNodeUrl: widget.voteNodeUrl,
          ceremonyStart: widget.ceremonyStart,
          voteEnd: widget.voteEnd,
          shareServerUrls: widget.shareServerUrls,
          singleShare: widget.singleShare,
        );
  }

  String _stageLabel(VotingSubmissionJobState job) {
    switch (job.stage) {
      case "preparing":
        return "Preparing delegation bundle";
      case "proving":
        return "Generating zero-knowledge proof";
      case "submitting":
        return "Submitting delegation to the vote chain";
      case "confirming":
        return "Waiting for confirmation";
      case "voting":
        return "Casting votes";
      case "shares":
        return "Submitting shares";
      case "done":
        return job.doneLabel ?? "Submission complete";
      case "error":
        return "Submission failed";
      default:
        return "Starting";
    }
  }

  @override
  Widget build(BuildContext context) {
    final pinlock = ref.watch(lifecycleProvider);
    if (pinlock.value ?? false) return PinLock();

    final job = ref.watch(votingSubmissionJobProvider(widget.roundId));
    final running = job.stage != "done" && job.stage != "error";
    // Confirmed vote txs (per proposal), shown as evidence on the done state.
    final confirmedVotes = (ref
                .watch(votingSessionProvider(widget.roundId))
                .value
                ?.recovery
                ?.votes ??
            const <VotingVoteRecovery>[])
        .where((v) => v.phase == "confirmed" && (v.txHash ?? "").isNotEmpty)
        .toList();
    // Human-readable ballot evidence: proposal titles and option labels.
    final proposals = (ref
                .watch(votingRoundProposalsProvider(
                  widget.roundId,
                  widget.chainUrl,
                ))
                .value ??
            const <VotingProposalInfo>[])
        .asMap();

    String voteLabel(VotingVoteRecovery v) {
      final proposal = proposals[v.proposalId];
      final title = proposal?.title ?? "Proposal ${v.proposalId}";
      final choice = proposal?.optionLabels[v.choice] ?? "Option ${v.choice + 1}";
      return "$title — $choice";
    }

    return PopScope(
      canPop: !running,
      onPopInvokedWithResult: (didPop, _) async {
        if (didPop) return;
        if (_handlingLeave) return;
        _handlingLeave = true;
        try {
          final leave = await confirmDialog(
            context,
            title: "Submission in progress",
            message: "Generating zero-knowledge proofs can take a while. "
                "Are you sure you want to leave?",
          );
          if (leave && mounted) GoRouter.of(context).pop();
        } finally {
          _handlingLeave = false;
        }
      },
      child: Scaffold(
        appBar: AppBar(title: const Text("Voting submission")),
        body: Center(
          child: Padding(
            padding: const EdgeInsets.all(24),
            child: Column(
              mainAxisAlignment: MainAxisAlignment.center,
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [
                if (widget.roundName != null &&
                    widget.roundName!.isNotEmpty &&
                    widget.roundName != widget.roundId)
                  Padding(
                    padding: const EdgeInsets.only(bottom: 8),
                    child: Text(
                      widget.roundName!,
                      style: Theme.of(context).textTheme.titleMedium,
                      textAlign: TextAlign.center,
                    ),
                  ),
                Text(
                  _stageLabel(job),
                  style: Theme.of(context).textTheme.titleLarge,
                  textAlign: TextAlign.center,
                ),
                const SizedBox(height: 16),
                LinearProgressIndicator(
                  value: job.stage == "done"
                      ? 1
                      : job.stage == "proving" ||
                              job.stage == "confirming" ||
                              job.stage == "shares"
                          ? null
                          : job.progress,
                  minHeight: 6,
                ),
                if (job.stage == "proving")
                  Padding(
                    padding: const EdgeInsets.only(top: 8),
                    child: Text(
                      "Proof progress ${(job.progress * 100).toStringAsFixed(0)}%",
                      textAlign: TextAlign.center,
                    ),
                  ),
                if (job.eligibleWeightZatoshi != null)
                  Padding(
                    padding: const EdgeInsets.only(top: 8),
                    child: Text(
                      "Voting power: ${formatVotingPower(job.eligibleWeightZatoshi!)}",
                      textAlign: TextAlign.center,
                      style: Theme.of(context).textTheme.bodyMedium,
                    ),
                  ),
                if (job.stage == "done" && job.txHash != null) ...[
                  const SizedBox(height: 12),
                  SelectableText(
                    "Transaction: ${job.txHash}",
                    textAlign: TextAlign.center,
                    style: Theme.of(context).textTheme.bodySmall,
                  ),
                  if (job.confirmHeight != null)
                    Padding(
                      padding: const EdgeInsets.only(top: 4),
                      child: Text(
                        "Included in block ${job.confirmHeight}",
                        textAlign: TextAlign.center,
                        style: Theme.of(context).textTheme.bodySmall,
                      ),
                    ),
                ],
                for (final v in confirmedVotes)
                  Padding(
                    padding: const EdgeInsets.only(top: 4),
                    child: SelectableText(
                      "${voteLabel(v)}\n${v.txHash} · tree ${v.vcTreePosition}",
                      textAlign: TextAlign.center,
                      style: Theme.of(context).textTheme.bodySmall,
                    ),
                  ),
                const SizedBox(height: 24),
                if (job.stage == "error") ...[
                  SelectableText(
                    job.error ?? "Unknown error",
                    style: TextStyle(
                      color: Theme.of(context).colorScheme.error,
                    ),
                    textAlign: TextAlign.center,
                  ),
                  const SizedBox(height: 16),
                  FilledButton(
                    onPressed: () {
                      ref
                          .read(votingSubmissionJobProvider(widget.roundId).notifier)
                          .reset();
                      Future(_start);
                    },
                    child: const Text("Retry"),
                  ),
                ] else if (job.stage == "done")
                  FilledButton(
                    onPressed: () => GoRouter.of(context).pushReplacement(
                      "/voting/confirmation",
                      extra: {
                        "roundId": widget.roundId,
                        "roundName": widget.roundName,
                        "chainUrl": widget.chainUrl,
                      },
                    ),
                    child: const Text("Done"),
                  ),
              ],
            ),
          ),
        ),
      ),
    );
  }
}
