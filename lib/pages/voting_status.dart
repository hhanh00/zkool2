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

  String _stageLabel(String stage) {
    switch (stage) {
      case "preparing":
        return "Preparing delegation bundle";
      case "proving":
        return "Generating zero-knowledge proof";
      case "submitting":
        return "Submitting delegation to the vote chain";
      case "confirming":
        return "Waiting for confirmation";
      case "done":
        return "Delegation confirmed";
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
                Text(
                  _stageLabel(job.stage),
                  style: Theme.of(context).textTheme.titleLarge,
                  textAlign: TextAlign.center,
                ),
                const SizedBox(height: 16),
                LinearProgressIndicator(
                  value: job.stage == "done"
                      ? 1
                      : job.stage == "proving" || job.stage == "confirming"
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
                const SizedBox(height: 24),
                if (job.stage == "error") ...[
                  Text(
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
                      extra: {"roundId": widget.roundId},
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
