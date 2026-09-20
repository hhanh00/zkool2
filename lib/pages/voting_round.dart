import 'package:flutter/material.dart';
import 'package:zkool/src/rust/api/voting.dart';

class VotingRoundPage extends StatelessWidget {
  final VotingRoundListItem round;

  const VotingRoundPage({super.key, required this.round});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: Text(round.title)),
      body: round.proposals.isEmpty
          ? const Center(child: Text('No proposals found for this round'))
          : ListView.separated(
              padding: const EdgeInsets.symmetric(vertical: 8),
              itemCount: round.proposals.length,
              separatorBuilder: (_, __) => const Divider(height: 1),
              itemBuilder: (context, index) {
                final proposal = round.proposals[index];
                return ListTile(
                  leading: CircleAvatar(
                    child: Text(proposal.proposalId.toString()),
                  ),
                  title: Text(proposal.title),
                );
              },
            ),
    );
  }
}
