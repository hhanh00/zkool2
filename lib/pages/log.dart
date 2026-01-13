import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:zkool/main.dart';
import 'package:zkool/store.dart';

class LogviewPage extends ConsumerStatefulWidget {
  const LogviewPage({super.key});

  @override
  ConsumerState<LogviewPage> createState() => LogviewPageState();
}

class LogviewPageState extends ConsumerState<LogviewPage> {
  @override
  Widget build(BuildContext context) {
    final pinlock = ref.watch(lifecycleProvider);
    if (pinlock.value ?? false) return PinLock();

    final log = ref.watch(logProvider);
    final fullLog = log.join("\n");

    return Scaffold(
      appBar: AppBar(
        title: const Text("Log"),
      ),
      body: SingleChildScrollView(child: Padding(padding: const EdgeInsets.symmetric(horizontal: 8), child: SelectableText(fullLog))),
    );
  }
}
