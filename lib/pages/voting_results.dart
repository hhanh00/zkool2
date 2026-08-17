import 'dart:async';
import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge.dart';
import 'package:zkool/main.dart';
import 'package:zkool/src/rust/api/voting.dart';
import 'package:zkool/store.dart';

/// Results screen: fetches the round tally from the vote chain and renders
/// per-proposal option bars with the winning option highlighted. While the
/// chain reports the round as tallying, it polls every 10 seconds.
class VotingResultsPage extends ConsumerStatefulWidget {
  final String roundId;
  final String chainUrl;

  const VotingResultsPage({
    super.key,
    required this.roundId,
    required this.chainUrl,
  });

  @override
  ConsumerState<VotingResultsPage> createState() => VotingResultsPageState();
}

class VotingResultsPageState extends ConsumerState<VotingResultsPage> {
  Timer? _pollTimer;
  String? _error;
  bool _tallying = false;
  Map<int, Map<int, num>> _tallies = {}; // proposal id -> option id -> amount

  @override
  void initState() {
    super.initState();
    Future(_load);
  }

  @override
  void dispose() {
    _pollTimer?.cancel();
    super.dispose();
  }

  Future<void> _load() async {
    try {
      final c = coinContext.coin;
      final session = await ref.read(votingSessionProvider(widget.roundId).future);
      final intents = session.intents
          .map((i) => i.proposalId)
          .toSet()
          .toList()
        ..sort();
      final drafts = await votingDraftsLoad(roundId: widget.roundId, c: c);
      if (drafts != null && drafts.isNotEmpty) {
        for (final d in jsonDecode(drafts) as List<dynamic>) {
          final pid = (d as Map<String, dynamic>)['proposal_id'] as int?;
          if (pid != null && !intents.contains(pid)) intents.add(pid);
        }
      }
      final res = await votechainRoundTally(
        baseUrl: widget.chainUrl,
        roundId: widget.roundId,
        c: c,
      );
      if (res.statusCode == 404) {
        // Round still tallying or tally not published yet.
        if (mounted) setState(() => _tallying = true);
        _schedulePoll();
        return;
      }
      if (res.statusCode < 200 || res.statusCode >= 300) {
        throw AnyhowException(
          "Tally fetch failed (HTTP ${res.statusCode}): ${res.body}",
        );
      }
      final body = jsonDecode(res.body) as Map<String, dynamic>;
      final status = ((body['status'] ?? body['phase'] ?? "") as String)
          .toLowerCase();
      _tallying = status == "2" || status == "tallying" || status == "pending";
      _tallies = _parseTally(body, intents);
      if (mounted) setState(() {});
      if (_tallying) _schedulePoll();
    } on AnyhowException catch (e) {
      if (mounted) setState(() => _error = e.message);
    }
  }

  /// Lenient tally parse mirroring vizor's precedence: direct entry,
  /// `tallies`/`results`/`proposals` map or list, then nested
  /// `entries`/`options`/`tally`. Decision keys and amount keys are tried in
  /// the same order as vizor's client.
  Map<int, Map<int, num>> _parseTally(Map<String, dynamic> json, List<int> proposalIds) {
    final result = <int, Map<int, num>>{};

    Object? value(List<String> keys) {
      for (final k in keys) {
        if (json.containsKey(k)) return json[k];
      }
      return null;
    }

    int? toInt(Object? v) => v is int
        ? v
        : v is num
            ? v.toInt()
            : int.tryParse(v?.toString() ?? "");

    num? toNum(Object? v) => v is num
        ? v
        : num.tryParse(v?.toString() ?? "");

    const decisionKeys = [
      "vote_decision",
      "voteDecision",
      "decision",
      "choice",
      "index",
      "option",
      "option_id",
      "optionId",
    ];
    const amountKeys = ["total_value", "totalValue", "amount", "votes", "value"];

    int decisionOf(Object? v) {
      if (v is! Map) return 0;
      for (final k in decisionKeys) {
        if (v.containsKey(k)) return toInt(v[k]) ?? 0;
      }
      return 0;
    }

    num? amountOf(Object? v) {
      if (v is! Map) return null;
      for (final k in amountKeys) {
        if (v.containsKey(k)) return toNum(v[k]);
      }
      return null;
    }

    void addDirect(Object? object, int proposalId) {
      if (object is! Map) return;
      // A tally entry without a decision key is the aggregate total row —
      // skip it so per-option amounts don't double-count.
      if (!object.keys.any((k) => decisionKeys.contains(k))) return;
      final d = decisionOf(object);
      final a = amountOf(object);
      if (a != null) {
        result.putIfAbsent(proposalId, () => {})[d] = a;
      }
    }

    void addEntries(Object? object, int proposalId) {
      if (object is Map) {
        for (final e in object.entries) {
          final d = toInt(e.key) ?? 0;
          final a = toNum(e.value);
          if (a != null) result.putIfAbsent(proposalId, () => {})[d] = a;
        }
      } else if (object is List) {
        for (final entry in object) {
          if (entry is Map) addDirect(entry, proposalId);
        }
      }
    }

    for (final pid in proposalIds) {
      final tallies = value(["tallies", "results", "proposals"]);
      if (tallies is Map) {
        addEntries(tallies[pid.toString()], pid);
      }
      if (tallies is List) {
        for (final item in tallies) {
          if (item is! Map) continue;
          final id = toInt(item["proposal_id"] ?? item["proposalId"] ?? item["id"]);
          if (id == pid) {
            addDirect(item, pid);
            addEntries(item["entries"] ?? item["options"] ?? item["tally"], pid);
          }
        }
      }
      addEntries(value(["entries", "tally"]), pid);
    }
    return result;
  }

  void _schedulePoll() {
    _pollTimer?.cancel();
    _pollTimer = Timer(const Duration(seconds: 10), () {
      if (mounted) Future(_load);
    });
  }

  @override
  Widget build(BuildContext context) {
    final pinlock = ref.watch(lifecycleProvider);
    if (pinlock.value ?? false) return PinLock();

    // Proposal titles and option labels, keyed by proposal id (not list
    // position) — decision ids are the vote-sdk option ids.
    final proposalsAsync = ref.watch(
      votingRoundProposalsProvider(widget.roundId, widget.chainUrl),
    );
    final proposals = {
      for (final p in (proposalsAsync.value ?? const <VotingProposalInfo>[]))
        p.id: p,
    };

    return Scaffold(
      appBar: AppBar(title: Text("${widget.roundId} results")),
      body: _error != null
          ? Center(child: Text(_error!))
          : _tallies.isEmpty
              ? Center(
                  child: Text(
                    _tallying
                        ? "Results pending..."
                        : "No tally data for this round",
                  ),
                )
              : ListView.builder(
                  itemCount: _tallies.length,
                  itemBuilder: (context, i) {
                    final pid = _tallies.keys.elementAt(i);
                    final tally = _tallies[pid]!;
                    final total = tally.values.fold<num>(0, (a, b) => a + b);
                    final winner = tally.entries.reduce(
                      (a, b) => a.value >= b.value ? a : b,
                    );
                    final proposal = proposals[pid];
                    return Card(
                      child: Padding(
                        padding: const EdgeInsets.all(12),
                        child: Column(
                          crossAxisAlignment: CrossAxisAlignment.stretch,
                          children: [
                            Text(proposal?.title ?? "Proposal $pid",
                                style: Theme.of(context).textTheme.titleMedium),
                            const SizedBox(height: 8),
                            ...tally.entries.map((e) {
                              final fraction = total == 0
                                  ? 0.0
                                  : e.value / total;
                              final winning = e.key == winner.key;
                              return Padding(
                                padding: const EdgeInsets.symmetric(vertical: 2),
                                child: Row(
                                  children: [
                                    SizedBox(
                                      width: 80,
                                      child: Text(
                                        proposal?.optionLabels[e.key] ??
                                            "Option ${e.key}",
                                        style: TextStyle(
                                          fontWeight: winning
                                              ? FontWeight.bold
                                              : FontWeight.normal,
                                        ),
                                      ),
                                    ),
                                    Expanded(
                                      child: LinearProgressIndicator(
                                        value: fraction.clamp(0.0, 1.0),
                                        minHeight: 8,
                                      ),
                                    ),
                                    SizedBox(
                                      width: 90,
                                      child: Text(
                                        "${(fraction * 100).toStringAsFixed(1)}%",
                                        textAlign: TextAlign.end,
                                      ),
                                    ),
                                  ],
                                ),
                              );
                            }),
                          ],
                        ),
                      ),
                    );
                  },
                ),
    );
  }
}
