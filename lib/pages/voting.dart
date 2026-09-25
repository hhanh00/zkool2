import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import 'package:zkool/store.dart' show coinContext, votingDriveJobProvider, votingQuiescenceLabel;
import 'package:zkool/src/rust/api/voting.dart';
import 'package:zkool/widgets/error_display.dart';
import 'package:zkool/widgets/loading_steps.dart';

class VotingPage extends ConsumerStatefulWidget {
  const VotingPage({super.key});

  @override
  ConsumerState<VotingPage> createState() => _VotingPageState();
}

class _VotingPageState extends ConsumerState<VotingPage> {
  late Future<List<VotingRoundListItem>> _rounds;
  bool _loading = true;
  int _loadGeneration = 0;

  @override
  void initState() {
    super.initState();
    _rounds = _loadRounds();
  }

  Future<List<VotingRoundListItem>> _loadRounds() async {
    final generation = ++_loadGeneration;
    try {
      final c = coinContext.coin;
      final rounds = await votingRoundList(c: c);
      return rounds;
    } finally {
      if (mounted && generation == _loadGeneration) {
        setState(() => _loading = false);
      }
    }
  }

  void _refresh() {
    setState(() {
      _loading = true;
      _rounds = _loadRounds();
    });
  }

  @override
  Widget build(BuildContext context) {
    // TODO: Show pending helper-share delivery status.
    return Scaffold(
      appBar: AppBar(
        title: const Text('Voting'),
        actions: [
          IconButton(
            onPressed: _loading ? null : _refresh,
            icon: const Icon(Icons.refresh),
          ),
        ],
      ),
      body: FutureBuilder<List<VotingRoundListItem>>(
        key: ObjectKey(_rounds),
        future: _rounds,
        builder: (context, snapshot) {
          if (_loading || snapshot.connectionState != ConnectionState.done) {
            return LoadingSteps(
              title: 'Loading voting rounds',
              icon: Icons.how_to_vote_outlined,
              activeStep: 0,
              steps: const [
                LoadingStep(
                  title: 'Load voting rounds',
                  description: 'Verifying the configuration, fetching rounds, and checking your progress.',
                ),
              ],
            );
          }
          if (snapshot.hasError) {
            return ErrorCard(error: snapshot.error!, onRetry: _refresh);
          }
          final rounds = snapshot.data ?? const <VotingRoundListItem>[];
          if (rounds.isEmpty) {
            return const Center(
              child: Column(
                mainAxisSize: MainAxisSize.min,
                children: [
                  Icon(Icons.how_to_vote_outlined, size: 48),
                  SizedBox(height: 12),
                  Text('No voting rounds'),
                ],
              ),
            );
          }
          return ListView.separated(
            padding: const EdgeInsets.symmetric(vertical: 8),
            itemCount: rounds.length,
            separatorBuilder: (_, __) => const Divider(height: 1),
            itemBuilder: (context, index) => _VotingRoundTile(
              round: rounds[index],
              onPressed: () => context.push(
                '/voting/round',
                extra: rounds[index],
              ),
            ),
          );
        },
      ),
    );
  }
}

class _VotingRoundTile extends ConsumerWidget {
  final VotingRoundListItem round;
  final VoidCallback onPressed;

  const _VotingRoundTile({required this.round, required this.onPressed});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final details = <String>[
      if (round.snapshotHeight case final height?) 'Snapshot height $height',
      '${round.bundleCount} bundle${round.bundleCount == 1 ? '' : 's'}',
    ];
    // Submission truth comes from the drive registry, not the round list:
    // a live run (or its last outcome) overrides the tile's default action.
    final job = ref.watch(votingDriveJobProvider(round.roundId));
    final driving = job.stage == "driving";
    final running = driving && (job.status?.running ?? true);

    return ListTile(
      contentPadding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
      title: Text(round.title),
      subtitle: Padding(
        padding: const EdgeInsets.only(top: 6),
        child: Wrap(
          spacing: 8,
          runSpacing: 6,
          crossAxisAlignment: WrapCrossAlignment.center,
          children: [
            _VotingStatus(status: round.status),
            Text(details.join(' • ')),
            if (driving)
              _VotingDriveChip(
                label: votingQuiescenceLabel(running ? null : job.status?.quiescence),
                running: running,
              ),
          ],
        ),
      ),
      trailing: _VotingProgressButton(
        action: round.action,
        driving: driving,
        onPressed: onPressed,
      ),
      onTap: onPressed,
    );
  }
}

/// Live submission state on a round tile: spinner while the driver runs,
/// the quiescence outcome once it stops.
class _VotingDriveChip extends StatelessWidget {
  final String label;
  final bool running;

  const _VotingDriveChip({required this.label, required this.running});

  @override
  Widget build(BuildContext context) {
    final colors = Theme.of(context).colorScheme;
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 3),
      decoration: BoxDecoration(
        color: colors.secondaryContainer,
        borderRadius: BorderRadius.circular(12),
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          if (running) ...[
            SizedBox(
              width: 10,
              height: 10,
              child: CircularProgressIndicator(strokeWidth: 2, color: colors.onSecondaryContainer),
            ),
            const SizedBox(width: 6),
          ],
          Text(
            label,
            style: Theme.of(context).textTheme.labelSmall?.copyWith(color: colors.onSecondaryContainer),
          ),
        ],
      ),
    );
  }
}

class _VotingStatus extends StatelessWidget {
  final String status;

  const _VotingStatus({required this.status});

  @override
  Widget build(BuildContext context) {
    final colors = Theme.of(context).colorScheme;
    final (label, foreground, background) = switch (status) {
      'active' => ('Active', colors.onPrimaryContainer, colors.primaryContainer),
      'tallying' => ('Tallying', colors.onSecondaryContainer, colors.secondaryContainer),
      'finalized' => ('Finalized', colors.onTertiaryContainer, colors.tertiaryContainer),
      'pending' => ('Pending', colors.onSurfaceVariant, colors.surfaceContainerHighest),
      'ceremony_failed' => ('Ceremony failed', colors.onErrorContainer, colors.errorContainer),
      _ => (status.replaceAll('_', ' '), colors.onSurfaceVariant, colors.surfaceContainerHighest),
    };

    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 3),
      decoration: BoxDecoration(
        color: background,
        borderRadius: BorderRadius.circular(12),
      ),
      child: Text(
        label,
        style: Theme.of(context).textTheme.labelSmall?.copyWith(color: foreground),
      ),
    );
  }
}

class _VotingProgressButton extends StatelessWidget {
  final String action;
  final bool driving;
  final VoidCallback? onPressed;

  const _VotingProgressButton({required this.action, required this.driving, required this.onPressed});

  @override
  Widget build(BuildContext context) {
    final label = driving
        ? 'View status'
        : switch (action) {
            'view_results' => 'View results',
            'review' => 'Review',
            'resume' => 'Resume',
            _ => 'Start voting',
          };
    return FilledButton.tonal(
      onPressed: onPressed,
      child: Text(label),
    );
  }
}
