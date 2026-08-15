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
/// label from the fork's resume plan (`primary_action`), so a restart shows
/// the correct "Resume"/"Start"/"View results" affordance without any Dart
/// session state.
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
      final settings = await ref.read(appSettingsProvider.future);
      if (settings.votingConfigUrl.isNotEmpty) {
        await ref.read(votingConfigProvider.notifier).resolve();
      }
    });
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
    return Scaffold(
      appBar: AppBar(
        title: Text("Voting"),
        actions: [
          IconButton(
            icon: Icon(Icons.refresh),
            onPressed: () {
              ref.invalidate(votingRoundListProvider);
              ref.read(votingConfigProvider.notifier).resolve();
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
          final joinable = configRounds
              .where((r) => !localIds.contains(r.roundId))
              .toList();
          if (list.isEmpty && joinable.isEmpty) {
            return const Center(child: Text("No voting rounds"));
          }
          final chainUrl = (config.value != null &&
                  config.value!.voteServers.isNotEmpty)
              ? config.value!.voteServers.first.url
              : "";
          return RefreshIndicator(
            onRefresh: () async =>
                ref.invalidate(votingRoundListProvider),
            child: ListView(
              children: [
                if (joinable.isNotEmpty) ...[
                  const Padding(
                    padding: EdgeInsets.all(12),
                    child: Text("Open rounds",
                        style: TextStyle(fontWeight: FontWeight.bold)),
                  ),
                  ...joinable.map(
                    (r) => ListTile(
                      title: Text(r.roundId),
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
                    ),
                  ),
                  const Divider(),
                ],
                ...list.map((r) => _RoundTile(round: r)),
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

  const _RoundTile({required this.round});

  String _actionLabel(String primaryAction) {
    switch (primaryAction) {
      case "delegate" || "vote" || "submit_shares":
        return "Resume";
      case "done":
        return "View results";
      default:
        return "Start voting";
    }
  }

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final cs = Theme.of(context).colorScheme;
    final session = ref.watch(votingSessionProvider(round.roundId));
    return session.when(
      loading: () => ListTile(
        title: Text(round.roundId),
        subtitle: Text("Snapshot height ${round.snapshotHeight}"),
        trailing: SizedBox(
          width: 24,
          height: 24,
          child: CircularProgressIndicator(strokeWidth: 2),
        ),
      ),
      error: (e, _) => ListTile(
        title: Text(round.roundId),
        subtitle: Text("Snapshot height ${round.snapshotHeight}"),
        trailing: IconButton(
          icon: Icon(Icons.refresh),
          onPressed: () =>
              ref.invalidate(votingSessionProvider(round.roundId)),
        ),
      ),
      data: (state) {
        final action = state.plan?.primaryAction ?? "idle";
        final label = _actionLabel(action);
        return ListTile(
          title: Text(round.roundId),
          subtitle: Text(
            "Snapshot height ${round.snapshotHeight} • "
            "${round.bundleCount} bundle${round.bundleCount == 1 ? "" : "s"}",
          ),
          trailing: FilledButton.tonal(
            onPressed: () => _openStatus(context, ref, action),
            child: Text(label),
          ),
          selected: action != "idle" && action != "done",
          selectedTileColor: cs.primaryContainer.withValues(alpha: 0.3),
        );
      },
    );
  }

  Future<void> _openStatus(
    BuildContext context,
    WidgetRef ref,
    String action,
  ) async {
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
    if (action == "done") {
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
    final settings = await ref.read(appSettingsProvider.future);
    await GoRouter.of(context).push("/voting/status", extra: {
      "roundId": round.roundId,
      "chainUrl": chainUrl,
      "pirServerUrl": "",
      "voteNodeUrl": settings.voteNodeUrl,
    });
  }
}
