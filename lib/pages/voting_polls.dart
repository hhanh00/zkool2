import 'package:collection/collection.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge.dart';
import 'package:go_router/go_router.dart';
import 'package:zkool/main.dart';
import 'package:zkool/src/rust/api/db.dart';
import 'package:zkool/src/rust/api/voting.dart';
import 'package:zkool/store.dart';
import 'package:zkool/utils.dart';
import 'package:zkool/widgets/error_display.dart';

/// Round list for shielded voting (ZIP 262). Each row derives its action
/// label from the fork's resume plan (`primary_action`) plus the chain round
/// status, so a restart shows the correct affordance without any Dart session
/// state:
///
/// - chain tallying/closed → "View results" (a tally exists).
/// - chain active, plan has recovery work (incl. shares sent but unconfirmed)
///   → "Resume" → status page, which re-arms share tracking.
/// - chain active, wallet done → "Review" → vote receipt (no tally yet).
/// - otherwise → "Start voting".
///
/// An unresolved chain status falls back to the plan-only rule.
class VotingPollsPage extends ConsumerStatefulWidget {
  const VotingPollsPage({super.key});

  @override
  ConsumerState<VotingPollsPage> createState() => VotingPollsPageState();
}

class VotingPollsPageState extends ConsumerState<VotingPollsPage> {
  late final c = coinContext.coin;

  @override
  void initState() {
    super.initState();
    Future(_guardHardwareAccount);
    Future(() async {
      try {
        final settings = await ref.read(appSettingsProvider.future);
        if (settings.votingConfigUrl.isNotEmpty) {
          await ref.read(votingConfigProvider.notifier).resolve();
        }
      } on AnyhowException catch (e) {
        if (mounted) await showException(context, e.message);
      }
    });
    // Opening the voting page re-arms helper-share tracking for rounds with
    // pending share work, so a restart resumes delivery without a manual
    // status-page visit.
    Future(() => armShareTrackingForPendingRounds(ref));
  }

  /// Voting v1 supports software accounts only; the fork signs with the
  /// wallet seed, which hardware accounts cannot expose.
  Future<void> _guardHardwareAccount() async {
    try {
      final accounts = await ref.read(getAccountsProvider.future);
      final selected = ref.read(selectedAccountIdProvider);
      final account = accounts.where((a) => a.id == selected).firstOrNull;
      if (account != null && account.hw != 0) {
        if (mounted) {
          await showMessage(
            context,
            "Voting is not supported for hardware accounts yet. "
            "Switch to a software account to vote.",
          );
        }
      }
    } on AnyhowException catch (e) {
      if (mounted) await showException(context, e.message);
    }
  }

  @override
  Widget build(BuildContext context) {
    final pinlock = ref.watch(lifecycleProvider);
    if (pinlock.value ?? false) return PinLock();

    final config = ref.watch(votingConfigProvider);
    final rounds = ref.watch(votingRoundListProvider);
    final shareAttention = ref.watch(votingShareTrackerProvider);
    return Scaffold(
      appBar: AppBar(
        title: Text("Voting"),
        actions: [
          IconButton(
            icon: Icon(Icons.refresh),
            onPressed: () async {
              ref.invalidate(votingRoundListProvider);
              try {
                await ref.read(votingConfigProvider.notifier).resolve();
              } on AnyhowException catch (e) {
                if (context.mounted) await showException(context, e.message);
              }
            },
          ),
        ],
      ),
      body: rounds.when(
        loading: () => blank(context),
        error: (e, _) => showError(e),
        data: (list) {
          final configRounds = config.value?.rounds ?? const <VotingConfigRound>[];
          final localIds = list.map((r) => r.roundId).toSet();
          final chainUrl = (config.value != null &&
                  config.value!.voteServers.isNotEmpty)
              ? config.value!.voteServers.first.url
              : "";
          // Only rounds the chain reports as active are joinable: a closed
          // or tallying round has no open voting window, so it is not
          // offered. An unresolved status (fetch in flight or failed) keeps
          // the round visible rather than hiding it.
          final joinable = configRounds
              .where((r) => !localIds.contains(r.roundId))
              .where((r) {
                final status = ref
                    .watch(votingRoundStatusProvider(r.roundId, chainUrl))
                    .value;
                return status == null || status == "active";
              })
              .toList();
          if (list.isEmpty && joinable.isEmpty) {
            return const Center(child: Text("No voting rounds"));
          }
          return RefreshIndicator(
            onRefresh: () async =>
                ref.invalidate(votingRoundListProvider),
            child: ListView(
              children: [
                if (shareAttention)
                  const Padding(
                    padding: EdgeInsets.fromLTRB(12, 8, 12, 0),
                    child: Card(
                      child: Padding(
                        padding: EdgeInsets.all(12),
                        child: Text(
                          "Helper-share delivery is pending — it will retry "
                          "in the background while the round is open.",
                        ),
                      ),
                    ),
                  ),
                if (joinable.isNotEmpty) ...[
                  const Padding(
                    padding: EdgeInsets.all(12),
                    child: Text("Open rounds",
                        style: TextStyle(fontWeight: FontWeight.bold)),
                  ),
                  ...joinable.map(
                    (r) {
                      final title = ref.watch(
                        votingRoundTitleProvider(r.roundId, chainUrl),
                      );
                      return ListTile(
                        title: Text(title.value ?? r.roundId),
                        trailing: FilledButton.tonal(
                          onPressed: chainUrl.isEmpty
                              ? null
                              : () => GoRouter.of(context)
                                  .push("/voting/proposal", extra: {
                                      "roundId": r.roundId,
                                      "chainUrl": chainUrl,
                                    }),
                          child: const Text("Join"),
                        ),
                      );
                    },
                  ),
                  const Divider(),
                ],
                ...list.map((r) => _RoundTile(round: r, chainUrl: chainUrl)),
              ],
            ),
          );
        },
      ),
    );
  }
}

class _RoundTile extends ConsumerWidget {
  final VotingRoundInfo round;
  final String chainUrl;

  const _RoundTile({required this.round, required this.chainUrl});

  String _actionLabel(String primaryAction, {bool pendingRecovery = false}) {
    switch (primaryAction) {
      case "delegate" || "vote" || "submit_shares":
        return "Resume";
      case "done":
        // "done" with recovery steps still pending (e.g. helper shares sent
        // but not confirmed) is not finished — keep "Resume" so the status
        // page, which re-arms share tracking, stays reachable.
        return pendingRecovery ? "Resume" : "View results";
      default:
        return "Start voting";
    }
  }

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final cs = Theme.of(context).colorScheme;
    // All tiles share one batch load (a single pool connection); the tile
    // reads its round's session from the shared map.
    final sessions = ref.watch(votingSessionsAllProvider);
    final title = ref.watch(votingRoundTitleProvider(round.roundId, chainUrl));
    final roundTitle = title.value ?? round.roundId;
    return sessions.when(
      loading: () => ListTile(
        title: Text(roundTitle),
        subtitle: Text("Snapshot height ${round.snapshotHeight}"),
        trailing: SizedBox(
          width: 24,
          height: 24,
          child: CircularProgressIndicator(strokeWidth: 2),
        ),
      ),
      error: (e, _) => ListTile(
        title: Text(roundTitle),
        subtitle: Text("Snapshot height ${round.snapshotHeight}"),
        trailing: IconButton(
          icon: Icon(Icons.refresh),
          onPressed: () => ref.invalidate(votingSessionsAllProvider),
        ),
      ),
      data: (map) {
        final state = map[round.roundId];
        if (state == null) {
          return ListTile(
            title: Text(roundTitle),
            subtitle: Text("Snapshot height ${round.snapshotHeight}"),
            trailing: SizedBox(
              width: 24,
              height: 24,
              child: CircularProgressIndicator(strokeWidth: 2),
            ),
          );
        }
        final plan = state.plan;
        final action = plan?.primaryAction ?? "idle";
        final pending = plan?.pendingRecovery ?? false;
        // The chain round status decides whether results exist: an active
        // round has no published tally, so the tile must not offer
        // "View results" even when the wallet's part is done — the plan only
        // picks between "Resume" (work pending) and "Review" (done). An
        // unresolved chain status (fetch failed / no chain URL) falls back
        // to the plan-only rule.
        final chainStatus = ref
            .watch(votingRoundStatusProvider(round.roundId, chainUrl))
            .value;
        final chainDone = chainStatus == "tallying" || chainStatus == "closed";
        final planDone = action == "done" && !pending;
        final done = chainDone || (chainStatus == null && planDone);
        final review = action == "done" && !done && chainStatus != null;
        final label = done
            ? "View results"
            : review
                ? "Review"
                : _actionLabel(action, pendingRecovery: pending);
        return ListTile(
          title: Text(roundTitle),
          subtitle: Text(
            "Snapshot height ${round.snapshotHeight} • "
            "${round.bundleCount} bundle${round.bundleCount == 1 ? "" : "s"}",
          ),
          trailing: FilledButton.tonal(
            onPressed: () => _openStatus(
              context,
              ref,
              action,
              done: done,
              review: review,
              roundName: roundTitle,
            ),
            child: Text(label),
          ),
          selected: pending && !done,
          selectedTileColor: cs.primaryContainer.withValues(alpha: 0.3),
        );
      },
    );
  }

  Future<void> _openStatus(
    BuildContext context,
    WidgetRef ref,
    String action, {
    required bool done,
    required bool review,
    String? roundName,
  }) async {
    final c = coinContext.coin;
    final configValue = ref.read(votingConfigProvider).value;
    final chainUrl = (configValue != null && configValue.voteServers.isNotEmpty)
        ? configValue.voteServers.first.url
        : await getProp(key: "voting_chain_url", c: c) ?? "";
    if (chainUrl.isEmpty) {
      await showMessage(
        context,
        "No vote chain URL configured. Add voting_chain_url in settings or "
        "resolve a voting config source.",
      );
      return;
    }
    if (!context.mounted) return;
    if (done) {
      await GoRouter.of(context).push("/voting/results", extra: {
        "roundId": round.roundId,
        "chainUrl": chainUrl,
      });
      return;
    }
    if (action == "idle") {
      // Fresh round: open the ballot first.
      await GoRouter.of(context).push("/voting/proposal", extra: {
        "roundId": round.roundId,
        "chainUrl": chainUrl,
      });
      return;
    }
    if (review) {
      // Wallet done but the round is still open on the chain: show the
      // vote receipt, not the (not-yet-published) tally.
      await GoRouter.of(context).push("/voting/confirmation", extra: {
        "roundId": round.roundId,
        "roundName": roundName,
        "chainUrl": chainUrl,
      });
      return;
    }
    final settings = await ref.read(appSettingsProvider.future);
    await GoRouter.of(context).push("/voting/status", extra: {
      "roundId": round.roundId,
      "chainUrl": chainUrl,
      "pirServerUrl": "",
      // An unset Vote Node URL defaults to the vote chain server: the same
      // REST API serves the commitment tree the vote syncs from.
      "voteNodeUrl": settings.voteNodeUrl.isNotEmpty
          ? settings.voteNodeUrl
          : chainUrl,
    });
  }
}
