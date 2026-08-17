import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge.dart';
import 'package:go_router/go_router.dart';
import 'package:zkool/main.dart';
import 'package:zkool/src/rust/api/voting.dart';
import 'package:zkool/store.dart';
import 'package:zkool/utils.dart';
import 'package:zkool/widgets/error_display.dart';

/// One parsed proposal option.
class _Option {
  final int id;
  final String label;

  const _Option({required this.id, required this.label});
}

/// One parsed proposal from the round status body (lenient).
class _Proposal {
  final int id;
  final String title;
  final List<_Option> options;

  const _Proposal({
    required this.id,
    required this.title,
    required this.options,
  });
}

/// Ballot screen: shows the round's proposals, lets the voter choose or skip
/// each one, persists the draft (props) and the durable ballot intent (voting
/// DB) on every change, then hands off to the review screen.
class VotingProposalPage extends ConsumerStatefulWidget {
  final String roundId;
  final String chainUrl;

  const VotingProposalPage({
    super.key,
    required this.roundId,
    required this.chainUrl,
  });

  @override
  ConsumerState<VotingProposalPage> createState() => VotingProposalPageState();
}

class VotingProposalPageState extends ConsumerState<VotingProposalPage> {
  List<_Proposal> _proposals = [];
  Map<int, int> _choices = {}; // proposal id -> option id
  final Set<int> _skipped = {};
  String? _error;
  String? _roundParamsJson;
  String? _roundName;
  int? _snapshotHeight;
  BigInt? _votingPower;

  @override
  void initState() {
    super.initState();
    Future(_load);
  }

  /// Lenient nested field lookup (mirrors vizor's round-status parsing).
  Object? _find(Map<String, dynamic> json, String key) {
    if (json.containsKey(key)) return json[key];
    for (final entry in json.entries) {
      if (entry.value is Map) {
        final match = _find((entry.value as Map).cast<String, dynamic>(), key);
        if (match != null) return match;
      }
    }
    return null;
  }

  Future<void> _load() async {
    try {
      final c = coinContext.coin;
      final res = await votechainRoundStatus(
        baseUrl: widget.chainUrl,
        roundId: widget.roundId,
        c: c,
      );
      if (res.statusCode < 200 || res.statusCode >= 300) {
        throw AnyhowException(
          "Round status failed (HTTP ${res.statusCode}): ${res.body}",
        );
      }
      final body = jsonDecode(res.body) as Map<String, dynamic>;
      final round = body['round'] as Map<String, dynamic>? ?? {};
      final proposalsJson = round['proposals'] as List<dynamic>? ?? [];
      _proposals = proposalsJson
          .map(_parseProposal)
          .whereType<_Proposal>()
          .toList();

      // Derive the authenticated round params for delegation_prepare from the
      // cached config + the chain-reported snapshot fields.
      final snapshotHeight = _find(round, "snapshot_height");
      final ncRoot = _find(round, "nc_root");
      final nullifierImtRoot = _find(round, "nullifier_imt_root");
      if (snapshotHeight is int && ncRoot is String && nullifierImtRoot is String) {
        _snapshotHeight = snapshotHeight;
        _votingPower =
            await votingEligibleWeight(snapshotHeight: snapshotHeight, c: c);
        _roundName = (_find(round, "title") ??
                    _find(round, "round_name") ??
                    _find(round, "name"))
                ?.toString() ??
            widget.roundId;
        final settings = await ref.read(appSettingsProvider.future);
        if (settings.votingConfigUrl.isNotEmpty) {
          _roundParamsJson = await votingRoundParamsJson(
            source: settings.votingConfigUrl,
            roundId: widget.roundId,
            snapshotHeight: BigInt.from(snapshotHeight),
            ncRoot: base64Decode(ncRoot),
            nullifierImtRoot: base64Decode(nullifierImtRoot),
            c: c,
          );
        }
      }

      // Best-effort vote-tree pre-sync so the commit step doesn't wait on it.
      final settings = await ref.read(appSettingsProvider.future);
      if (settings.voteNodeUrl.isNotEmpty) {
        try {
          await votingSyncTree(
            roundId: widget.roundId,
            voteNodeUrl: settings.voteNodeUrl,
            c: c,
          );
        } on AnyhowException catch (_) {
          // The round may not exist locally yet; the commit step syncs anyway.
        }
      }

      final drafts = await votingDraftsLoad(roundId: widget.roundId, c: c);
      if (drafts != null && drafts.isNotEmpty) {
        final list = jsonDecode(drafts) as List<dynamic>;
        for (final d in list) {
          final map = d as Map<String, dynamic>;
          final pid = map['proposal_id'] as int? ?? 0;
          final choice = map['choice'] as int? ?? 0;
          final numOptions = map['num_options'] as int? ?? 2;
          if (choice == numOptions) {
            _skipped.add(pid);
          } else {
            _choices[pid] = choice;
          }
        }
      }
      if (mounted) setState(() {});
    } on AnyhowException catch (e) {
      if (mounted) setState(() => _error = e.message);
    }
  }

  _Proposal? _parseProposal(dynamic value) {
    if (value is! Map) return null;
    final id = value['id'];
    if (id is! int || id < 1 || id > 15) return null;
    final title = (value['title'] ?? "Proposal $id").toString();
    var options = (value['options'] as List<dynamic>? ?? [])
        .asMap()
        .entries
        .map((entry) {
          final o = entry.value;
          if (o is! Map) return null;
          return _Option(
            // vote-sdk option ids come from `index` (omitted = 0 for the
            // first option); fall back to the list position.
            id: (o['index'] is int) ? o['index'] as int : entry.key,
            label: (o['label'] ?? o['short_title'] ?? o['title'] ?? "Option")
                .toString(),
          );
        })
        .whereType<_Option>()
        .toList();
    if (options.isEmpty) {
      // Vote-sdk default: Yes/No when options are missing.
      options = const [
        _Option(id: 0, label: "Yes"),
        _Option(id: 1, label: "No"),
      ];
    }
    return _Proposal(id: id, title: title, options: options);
  }

  Future<void> _persistSafe() async {
    try {
      await _persist();
    } on AnyhowException catch (e) {
      if (mounted) await showException(context, e.message);
    }
  }

  /// Persists the draft ballot only; durable ballot intents are written by
  /// the submission job from these drafts before the cast loop (mirrors
  /// vizor), when the round row already exists in the voting DB.
  Future<void> _persist() async {
    final c = coinContext.coin;
    // Draft votes mirror the fork's DraftVote JSON: skipped = choice == num_options.
    final drafts = _proposals
        .where((p) => _skipped.contains(p.id) || _choices.containsKey(p.id))
        .map((p) {
      final skipped = _skipped.contains(p.id);
      return {
        "proposal_id": p.id,
        "choice": skipped ? p.options.length : _choices[p.id],
        "num_options": p.options.length,
        "vc_tree_position": 0,
        "single_share": false,
      };
    }).toList();
    await votingDraftsSave(
      roundId: widget.roundId,
      draftsJson: jsonEncode(drafts),
      c: c,
    );
  }

  @override
  Widget build(BuildContext context) {
    final pinlock = ref.watch(lifecycleProvider);
    if (pinlock.value ?? false) return PinLock();

    final allAnswered = _proposals.every(
      (p) => _skipped.contains(p.id) || _choices.containsKey(p.id),
    );

    return Scaffold(
      appBar: AppBar(title: Text(widget.roundId)),
      body: _error != null
          ? Center(child: Text(_error!))
          : _proposals.isEmpty
              ? const Center(child: Text("No proposals found for this round"))
              : ListView.builder(
                  itemCount: _proposals.length,
                  itemBuilder: (context, i) {
                    final p = _proposals[i];
                    final selected = _choices[p.id];
                    final skipped = _skipped.contains(p.id);
                    return Card(
                      child: Padding(
                        padding: const EdgeInsets.all(12),
                        child: Column(
                          crossAxisAlignment: CrossAxisAlignment.stretch,
                          children: [
                            Text(p.title,
                                style: Theme.of(context).textTheme.titleMedium),
                            const SizedBox(height: 8),
                            ...p.options.map((o) => RadioListTile<int>(
                                  title: Text(o.label),
                                  value: o.id,
                                  groupValue: selected,
                                  onChanged: (v) async {
                                    setState(() {
                                      _skipped.remove(p.id);
                                      _choices[p.id] = v!;
                                    });
                                    await _persistSafe();
                                  },
                                )),
                            RadioListTile<int>(
                              title: const Text("Skip"),
                              value: -1,
                              groupValue: skipped ? -1 : null,
                              onChanged: (v) async {
                                setState(() {
                                  _choices.remove(p.id);
                                  _skipped.add(p.id);
                                });
                                await _persistSafe();
                              },
                            ),
                          ],
                        ),
                      ),
                    );
                  },
                ),
      bottomNavigationBar: SafeArea(
        child: Padding(
          padding: const EdgeInsets.all(12),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              if (_snapshotHeight != null)
                Text(
                  "Snapshot height: $_snapshotHeight",
                  textAlign: TextAlign.center,
                ),
              if (_votingPower != null)
                Text(
                  "Voting power: ${formatVotingPower(_votingPower!)}",
                  textAlign: TextAlign.center,
                ),
              if (_snapshotHeight != null || _votingPower != null)
                const SizedBox(height: 8),
              FilledButton(
                onPressed: allAnswered
                    ? () => GoRouter.of(context).push("/voting/review", extra: {
                          "roundId": widget.roundId,
                          "chainUrl": widget.chainUrl,
                          "roundParamsJson": _roundParamsJson,
                          "roundName": _roundName,
                          "snapshotHeight": _snapshotHeight,
                        })
                    : null,
                child: const Text("Review answers"),
              ),
            ],
          ),
        ),
      ),
    );
  }
}
