import 'dart:async';

import 'package:flutter/material.dart';
import 'package:gap/gap.dart';
import 'package:go_router/go_router.dart';

import 'package:zkool/main.dart';
import 'package:zkool/src/rust/api/migrate.dart';
import 'package:zkool/store.dart';
import 'package:zkool/utils.dart';

class MigratePage extends StatefulWidget {
  const MigratePage({super.key});

  @override
  State<MigratePage> createState() => _MigratePageState();
}

class _MigratePageState extends State<MigratePage>
    with WidgetsBindingObserver {
  StreamSubscription<MigrationStatus>? _sub;
  MigrationStatus? _status;
  bool _started = false;
  bool _didShowCompleteDialog = false;

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addObserver(this);
  }

  void _startMigration() {
    _sub?.cancel();
    final c = coinContext.coin;
    final stream = runMigration(c: c);
    _sub = stream.listen(
      (status) => setState(() => _status = status),
      onError: (e) => logger.w('Migration stream error: $e'),
    );
  }

  @override
  void dispose() {
    WidgetsBinding.instance.removeObserver(this);
    _sub?.cancel();
    _sub = null;
    super.dispose();
  }

  @override
  void didChangeAppLifecycleState(AppLifecycleState state) {
    if (state == AppLifecycleState.resumed) _startMigration();
  }

  @override
  Widget build(BuildContext context) {
    final status = _status;

    // Show success dialog once when migration completes
    if (!_didShowCompleteDialog && status?.phase == 'complete') {
      _didShowCompleteDialog = true;
      WidgetsBinding.instance.addPostFrameCallback((_) {
        showDialog(
          context: context,
          builder: (_) => AlertDialog(
            title: const Text("Migration Complete"),
            content: const Text(
                "All notes have been migrated to Ironwood."),
            actions: [
              TextButton(
                onPressed: () {
                  Navigator.of(context).pop();
                  if (context.mounted) GoRouter.of(context).pop();
                },
                child: const Text("Done"),
              ),
            ],
          ),
        );
      });
    }

    if (status == null) {
      return Scaffold(
        appBar: AppBar(title: const Text("Note Migration")),
        body: _started
            ? const Center(child: CircularProgressIndicator())
            : ListView(
                padding: const EdgeInsets.all(16),
                children: [
                  Card(
                    child: Padding(
                      padding: const EdgeInsets.all(16),
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          Text("Orchard to Ironwood Migration",
                              style: Theme.of(context).textTheme.headlineSmall),
                          const Gap(12),
                          const Text(
                            "This process migrates your Orchard notes to Ironwood "
                            "in two phases:\n\n"
                            "1. Splitting — non-standard notes are split into "
                            "standard denominations that can be migrated "
                            "efficiently.\n\n"
                            "2. Migrating — standard-denomination notes are "
                            "moved one-by-one to Ironwood.\n\n"
                            "The migration runs automatically in the background "
                            "with random delays between steps. You can close "
                            "this page at any time and resume later.\n\n"
                            "Fees apply to each transaction.",
                          ),
                          const Gap(16),
                          FilledButton.icon(
                            onPressed: () {
                              setState(() => _started = true);
                              _startMigration();
                            },
                            icon: const Icon(Icons.play_arrow),
                            label: const Text("Start Migration"),
                          ),
                        ],
                      ),
                    ),
                  ),
                ],
              ),
      );
    }

    final t = Theme.of(context);
    final tt = t.textTheme;
    final phase = status.phase;
    final isComplete = phase == 'complete';
    final isActive = phase == 'splitting' || phase == 'migrating';

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
      body: ListView(
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
                        "SD: ${status.sdNotesCount}  |  Non-SD: ${status.nonSdNotesCount}",
                        style: tt.bodyLarge),
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
                  _feeRow("Split fees", status.splitFees),
                  _feeRow("Migration fees", status.migrateFees),
                  const Divider(),
                  _feeRow("Total", status.totalFees, bold: true),
                ],
              ),
            ),
          ),
          const Gap(16),

          // Actions
          if (isComplete)
            FilledButton(
              onPressed: () => GoRouter.of(context).pop(),
              child: const Text("Done"),
            ),
        ],
      ),
    );
  }

  Widget _feeRow(String label, BigInt zats, {bool bold = false}) {
    final zec = zats.toDouble() / zatsPerZec;
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 2),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.spaceBetween,
        children: [
          Text(label,
              style: TextStyle(
                  fontWeight: bold ? FontWeight.bold : FontWeight.normal)),
          Text(
            zec.toStringAsFixed(8),
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
