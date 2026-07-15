import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:gap/gap.dart';
import 'package:go_router/go_router.dart';

import 'package:zkool/main.dart';
import 'package:zkool/src/rust/api/migrate.dart';
import 'package:zkool/store.dart';
import 'package:zkool/utils.dart';

class MigratePage extends ConsumerStatefulWidget {
  const MigratePage({super.key});

  @override
  ConsumerState<MigratePage> createState() => _MigratePageState();
}

class _MigratePageState extends ConsumerState<MigratePage>
    with WidgetsBindingObserver {
  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addObserver(this);
    // Start the migration poller
    Future.microtask(() => ref.read(migrationProvider.notifier).start());
  }

  @override
  void dispose() {
    WidgetsBinding.instance.removeObserver(this);
    ref.read(migrationProvider.notifier).stop();
    super.dispose();
  }

  @override
  void didChangeAppLifecycleState(AppLifecycleState state) {
    if (state == AppLifecycleState.resumed) {
      ref.read(migrationProvider.notifier).onResume();
    }
  }

  @override
  Widget build(BuildContext context) {
    final pinlock = ref.watch(lifecycleProvider);
    if (pinlock.value ?? false) return PinLock();

    final migrationAV = ref.watch(migrationProvider);
    final fees = ref.watch(migrationFeeTrackerProvider);
    final settings = ref.watch(appSettingsProvider).value;
    final currency = settings?.currency ?? 'usd';

    return Scaffold(
      appBar: AppBar(
        title: const Text("Note Migration"),
        actions: [
          IconButton(
            tooltip: "Close",
            onPressed: () => GoRouter.of(context).pop(),
            icon: const Icon(Icons.close),
          ),
        ],
      ),
      body: migrationAV.when(
        loading: () => const Center(child: CircularProgressIndicator()),
        error: (error, stack) => Center(
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              Text("Error loading migration state",
                  style: Theme.of(context).textTheme.bodyLarge),
              const Gap(16),
              FilledButton(
                onPressed: () => ref.invalidate(migrationProvider),
                child: const Text("Retry"),
              ),
            ],
          ),
        ),
        data: (status) {
          if (status == null) {
            return _buildNoMigration(context);
          }
          return _buildMigrationView(context, status, fees, currency);
        },
      ),
    );
  }

  Widget _buildNoMigration(BuildContext context) {
    return Center(
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          const Icon(Icons.check_circle_outline, size: 64,
              color: Colors.green),
          const Gap(16),
          Text("No Orchard notes to migrate",
              style: Theme.of(context).textTheme.headlineSmall),
          const Gap(8),
          Text("Your wallet is ready for the Ironwood era.",
              style: Theme.of(context).textTheme.bodyLarge),
          const Gap(24),
          FilledButton(
            onPressed: () => GoRouter.of(context).pop(),
            child: const Text("Done"),
          ),
        ],
      ),
    );
  }

  Widget _buildMigrationView(BuildContext context, MigrationStatus status,
      MigrationFeeTracker fees, String currency) {
    final t = Theme.of(context);
    final tt = t.textTheme;
    final phase = status.phase;
    final isComplete = phase == 'complete';
    final isActive = phase == 'splitting' || phase == 'migrating';

    return ListView(
      padding: const EdgeInsets.all(16),
      children: [
        // Phase indicator
        Card(
          child: Padding(
            padding: const EdgeInsets.all(16),
            child: Column(
              children: [
                Icon(
                  isComplete
                      ? Icons.check_circle
                      : isActive
                          ? Icons.sync
                          : Icons.schedule,
                  size: 48,
                  color: isComplete
                      ? Colors.green
                      : isActive
                          ? t.colorScheme.primary
                          : Colors.grey,
                ),
                const Gap(8),
                Text(
                  isComplete
                      ? "Migration Complete"
                      : isActive
                          ? "Migrating to Ironwood"
                          : "Ready to Migrate",
                  style: tt.headlineSmall,
                ),
                const Gap(4),
                Text(
                  phase == 'splitting'
                      ? "Phase 1: Splitting into standard denominations"
                      : phase == 'migrating'
                          ? "Phase 2: Migrating notes to Ironwood"
                          : phase == 'complete'
                              ? "All notes have been migrated"
                              : "Start the migration to move your funds",
                  style: tt.bodyMedium?.copyWith(
                      color: t.colorScheme.onSurfaceVariant),
                ),
              ],
            ),
          ),
        ),
        const Gap(16),

        // Progress
        if (isActive || isComplete) ...[
          Card(
            child: Padding(
              padding: const EdgeInsets.all(16),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text("Progress", style: tt.titleMedium),
                  const Gap(8),
                  LinearProgressIndicator(value: status.progress),
                  const Gap(8),
                  Text(status.workSummary, style: tt.bodyMedium),
                ],
              ),
            ),
          ),
          const Gap(16),
        ],

        // Next action
        if (isActive) ...[
          Card(
            child: Padding(
              padding: const EdgeInsets.all(16),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text("Next Action", style: tt.titleMedium),
                  const Gap(8),
                  Text(status.nextAction, style: tt.bodyLarge),
                ],
              ),
            ),
          ),
          const Gap(16),
        ],

        // Fee summary
        Card(
          child: Padding(
            padding: const EdgeInsets.all(16),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text("Fees", style: tt.titleMedium),
                const Gap(8),
                _feeRow("Split fees", fees.splitFees, currency),
                _feeRow("Migration fees", fees.migrateFees, currency),
                const Divider(),
                _feeRow("Total", fees.totalFees, currency,
                    bold: true),
              ],
            ),
          ),
        ),
        const Gap(16),

        // Note counts
        if (isActive || isComplete) ...[
          Card(
            child: Padding(
              padding: const EdgeInsets.all(16),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text("Orchard Notes", style: tt.titleMedium),
                  const Gap(8),
                  Text(
                      "SD notes: ${status.sdNotesCount}  |  Non-SD: ${status.nonSdNotesCount}",
                      style: tt.bodyLarge),
                ],
              ),
            ),
          ),
          const Gap(24),
        ],

        // Actions
        if (isComplete)
          FilledButton(
            onPressed: () => GoRouter.of(context).pop(),
            child: const Text("Done"),
          ),
      ],
    );
  }

  Widget _feeRow(String label, BigInt amount, String currency,
      {bool bold = false}) {
    final zecAmount = (amount.toDouble()) / zatsPerZec;
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 2),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.spaceBetween,
        children: [
          Text(label,
              style: TextStyle(
                  fontWeight: bold ? FontWeight.bold : FontWeight.normal)),
          Text(
            zecAmount.toStringAsFixed(8),
            style: TextStyle(
              fontWeight: bold ? FontWeight.bold : FontWeight.normal,
              fontFamily: 'monospace',
            ),
          ),
        ],
      ),
    );
  }
}
