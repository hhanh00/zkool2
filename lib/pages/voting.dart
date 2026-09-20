import 'package:flutter/material.dart';
import 'package:zkool/store.dart' show coinContext;
import 'package:zkool/src/rust/api/voting.dart';
import 'package:zkool/widgets/error_display.dart';
import 'package:zkool/widgets/loading_steps.dart';

class VotingPage extends StatefulWidget {
  const VotingPage({super.key});

  @override
  State<VotingPage> createState() => _VotingPageState();
}

class _VotingPageState extends State<VotingPage> {
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
            return const Center(child: Text('No voting rounds'));
          }
          return ListView.builder(
            itemCount: rounds.length,
            itemBuilder: (context, index) {
              final round = rounds[index];
              return ListTile(
                title: Text(round.title),
                trailing: _VotingProgressButton(
                  action: round.action,
                  onPressed: null,
                ),
              );
            },
          );
        },
      ),
    );
  }
}

class _VotingProgressButton extends StatelessWidget {
  final String action;
  final VoidCallback? onPressed;

  const _VotingProgressButton({required this.action, required this.onPressed});

  @override
  Widget build(BuildContext context) {
    final label = switch (action) {
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
