import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge.dart';
import 'package:gap/gap.dart';
import 'package:go_router/go_router.dart';

import 'package:zkool/main.dart';
import 'package:zkool/src/rust/api/migrate.dart';
import 'package:zkool/store.dart';
import 'package:zkool/utils.dart';
import 'package:zkool/widgets/error_display.dart';

class MigratePage extends StatefulWidget {
  const MigratePage({super.key});

  @override
  State<MigratePage> createState() => _MigratePageState();
}

class _MigratePageState extends State<MigratePage>
    with WidgetsBindingObserver {
  StreamSubscription<MigrationStatus>? _sub;
  MigrationStatus? _status;
  Timer? _countdown;
  int _countdownSecs = 0;
  bool _started = false;
  bool _didShowCompleteDialog = false;
  double _speedIndex = 1; // default: Fast (60s)

  static const _speedLabels = ["Very Fast", "Fast", "Medium", "Slow"];
  static const _speedMeanMs = [15000, 60000, 300000, 3600000];
  static const _speedDescriptions = [
    "~15s between steps",
    "~1m between steps",
    "~5m between steps",
    "~1h between steps",
  ];

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addObserver(this);
  }

  void _startMigration() {
    try {
      _sub?.cancel();
      final c = coinContext.coin;
      final meanDelayMs =
          BigInt.from(_speedMeanMs[_speedIndex.round()]);
      final stream = runMigration(c: c, meanDelayMs: meanDelayMs);
      _sub = stream.listen(
        (status) {
          setState(() => _status = status);

          if (status.nextAction.startsWith('Waiting')) {
            // Parse total seconds and start countdown
            final m =
                RegExp(r'Waiting (\d+)s').firstMatch(status.nextAction);
            if (m != null) {
              _countdownSecs = int.parse(m.group(1)!);
              _countdown?.cancel();
              _countdown = Timer.periodic(
                const Duration(seconds: 1),
                (_) {
                  if (_countdownSecs > 0) {
                    setState(() => _countdownSecs--);
                  }
                },
              );
            }

            ScaffoldMessenger.of(context).showSnackBar(
              SnackBar(
                content: Text(status.nextAction),
                duration: const Duration(seconds: 4),
              ),
            );
          } else {
            _countdown?.cancel();
            _countdown = null;
          }
        },
        onError: (e) async {
          final exc = e as AnyhowException;
          if (!context.mounted) return;
          await showException(context, exc.message);
        },
      );
    } on AnyhowException catch (e) {
      if (!context.mounted) return;
      showException(context, e.message);
    }
  }

  @override
  void dispose() {
    WidgetsBinding.instance.removeObserver(this);
    _sub?.cancel();
    _sub = null;
    _countdown?.cancel();
    _countdown = null;
    super.dispose();
  }

  @override
  void didChangeAppLifecycleState(AppLifecycleState state) {
    // Don't restart on resume — the stream keeps running in background.
    // Restarting creates a new Rust task whose first step runs immediately.
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
                          // Speed selector
                          Text("Migration Speed",
                              style: Theme.of(context).textTheme.titleMedium),
                          const Gap(8),
                          Row(
                            children: [
                              Expanded(
                                child: Slider(
                                  value: _speedIndex,
                                  min: 0,
                                  max: 3,
                                  divisions: 3,
                                  label: _speedLabels[_speedIndex.round()],
                                  onChanged: (v) =>
                                      setState(() => _speedIndex = v),
                                ),
                              ),
                              SizedBox(
                                width: 80,
                                child: Text(
                                  _speedLabels[_speedIndex.round()],
                                  style: Theme.of(context)
                                      .textTheme
                                      .bodyMedium
                                      ?.copyWith(fontWeight: FontWeight.bold),
                                ),
                              ),
                            ],
                          ),
                          Text(
                            _speedDescriptions[_speedIndex.round()],
                            style: Theme.of(context).textTheme.bodySmall?.copyWith(
                                color: Theme.of(context).colorScheme.onSurfaceVariant),
                          ),
                          const Gap(12),
                          // Privacy note
                          Card(
                            color: Theme.of(context)
                                .colorScheme
                                .surfaceContainerHighest,
                            child: Padding(
                              padding: const EdgeInsets.all(12),
                              child: Row(
                                crossAxisAlignment: CrossAxisAlignment.start,
                                children: [
                                  Icon(Icons.shield_outlined,
                                      size: 20,
                                      color: Theme.of(context)
                                          .colorScheme
                                          .onSurfaceVariant),
                                  const Gap(8),
                                  Expanded(
                                    child: Text(
                                      "Faster migration creates transactions closer "
                                      "together, making it easier for an observer "
                                      "to correlate them as part of the same "
                                      "migration. Slower speeds spread "
                                      "transactions out over time, improving "
                                      "privacy.",
                                      style: Theme.of(context)
                                          .textTheme
                                          .bodySmall
                                          ?.copyWith(
                                              color: Theme.of(context)
                                                  .colorScheme
                                                  .onSurfaceVariant),
                                    ),
                                  ),
                                ],
                              ),
                            ),
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
    final waiting = status.nextAction.startsWith('Waiting');
    final isComplete = phase == 'complete';
    final isActive = phase == 'splitting' || phase == 'migrating' || waiting;

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
                        : waiting
                            ? "Waiting..."
                            : isActive
                                ? "Migrating to Ironwood"
                                : "Ready to Migrate",
                    style: tt.headlineSmall,
                  ),
                  const Gap(4),
                  Text(
                    waiting
                        ? "Waiting ${_countdownSecs}s..."
                        : phase == 'splitting'
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
                  ],
                ),
              ),
            ),
            const Gap(16),
          ],

          // Note counts — phase-dependent
          if (isActive || isComplete) ...[
            Card(
              child: Padding(
                padding: const EdgeInsets.all(16),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                        phase == 'migrating'
                            ? "Migration Progress"
                            : "Orchard Notes",
                        style: tt.titleMedium),
                    const Gap(8),
                    Text(
                        phase == 'migrating'
                            ? "Orchard: ${status.sdNotesCount} SD  |  Ironwood: ${status.ironwoodSdCount} SD"
                            : "SD: ${status.sdNotesCount}  |  Non-SD: ${status.nonSdNotesCount}",
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
