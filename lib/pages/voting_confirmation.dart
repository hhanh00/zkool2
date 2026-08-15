import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import 'package:zkool/main.dart';
import 'package:zkool/store.dart';

/// Receipt screen shown after the submission job completes.
class VotingConfirmationPage extends ConsumerStatefulWidget {
  final String roundId;

  const VotingConfirmationPage({super.key, required this.roundId});

  @override
  ConsumerState<VotingConfirmationPage> createState() =>
      VotingConfirmationPageState();
}

class VotingConfirmationPageState extends ConsumerState<VotingConfirmationPage> {
  @override
  Widget build(BuildContext context) {
    final pinlock = ref.watch(lifecycleProvider);
    if (pinlock.value ?? false) return PinLock();

    return Scaffold(
      appBar: AppBar(title: const Text("Vote submitted")),
      body: Center(
        child: Padding(
          padding: const EdgeInsets.all(24),
          child: Column(
            mainAxisAlignment: MainAxisAlignment.center,
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              Icon(
                Icons.check_circle_outline,
                size: 72,
                color: Theme.of(context).colorScheme.primary,
              ),
              const SizedBox(height: 16),
              Text(
                "Your vote for ${widget.roundId} has been submitted.",
                style: Theme.of(context).textTheme.titleMedium,
                textAlign: TextAlign.center,
              ),
              const SizedBox(height: 24),
              FilledButton(
                onPressed: () => GoRouter.of(context).go("/voting"),
                child: const Text("Done"),
              ),
            ],
          ),
        ),
      ),
    );
  }
}
