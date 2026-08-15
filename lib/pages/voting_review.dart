import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge.dart';
import 'package:go_router/go_router.dart';
import 'package:zkool/main.dart';
import 'package:zkool/src/rust/api/voting.dart';
import 'package:zkool/store.dart';

/// Read-only summary of the persisted draft ballot. "Confirm & submit" hands
/// off to the execution screen, which runs the submission job.
class VotingReviewPage extends ConsumerStatefulWidget {
  final String roundId;
  final String chainUrl;
  final String? roundParamsJson;
  final String? roundName;
  final int? snapshotHeight;

  const VotingReviewPage({
    super.key,
    required this.roundId,
    required this.chainUrl,
    this.roundParamsJson,
    this.roundName,
    this.snapshotHeight,
  });

  @override
  ConsumerState<VotingReviewPage> createState() => VotingReviewPageState();
}

class VotingReviewPageState extends ConsumerState<VotingReviewPage> {
  List<Map<String, dynamic>> _drafts = [];
  String? _error;

  @override
  void initState() {
    super.initState();
    Future(_load);
  }

  Future<void> _load() async {
    try {
      final c = coinContext.coin;
      final drafts = await votingDraftsLoad(roundId: widget.roundId, c: c);
      if (drafts != null && drafts.isNotEmpty) {
        _drafts = (jsonDecode(drafts) as List<dynamic>)
            .map((d) => d as Map<String, dynamic>)
            .toList();
      }
      if (mounted) setState(() {});
    } on AnyhowException catch (e) {
      if (mounted) setState(() => _error = e.message);
    }
  }

  String _answerLabel(Map<String, dynamic> draft) {
    final choice = draft['choice'] as int? ?? 0;
    final numOptions = draft['num_options'] as int? ?? 2;
    if (choice == numOptions) return "Skipped";
    return "Option ${choice + 1}";
  }

  @override
  Widget build(BuildContext context) {
    final pinlock = ref.watch(lifecycleProvider);
    if (pinlock.value ?? false) return PinLock();

    return Scaffold(
      appBar: AppBar(title: const Text("Review your answers")),
      body: _error != null
          ? Center(child: Text(_error!))
          : _drafts.isEmpty
              ? const Center(child: Text("No ballot saved for this round"))
              : ListView.builder(
                  itemCount: _drafts.length,
                  itemBuilder: (context, i) {
                    final draft = _drafts[i];
                    return ListTile(
                      title: Text("Proposal ${draft['proposal_id']}"),
                      trailing: Text(
                        _answerLabel(draft),
                        style: TextStyle(
                          color: draft['choice'] == draft['num_options']
                              ? Theme.of(context).colorScheme.outline
                              : Theme.of(context).colorScheme.primary,
                        ),
                      ),
                    );
                  },
                ),
      bottomNavigationBar: SafeArea(
        child: Padding(
          padding: const EdgeInsets.all(12),
          child: FilledButton(
            onPressed: _drafts.isEmpty
                ? null
                : () async {
                    final settings = await ref.read(appSettingsProvider.future);
                    if (!context.mounted) return;
                    await GoRouter.of(context).push("/voting/status", extra: {
                      "roundId": widget.roundId,
                      "chainUrl": widget.chainUrl,
                      "pirServerUrl": "",
                      "voteNodeUrl": settings.voteNodeUrl,
                      "roundParamsJson": widget.roundParamsJson,
                      "roundName": widget.roundName,
                      "snapshotHeight": widget.snapshotHeight,
                    });
                  },
            child: const Text("Confirm & submit"),
          ),
        ),
      ),
    );
  }
}
