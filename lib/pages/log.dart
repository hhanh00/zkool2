import 'package:flutter/material.dart';
import 'package:flutter_mobx/flutter_mobx.dart';
import 'package:zkool/store.dart';

class LogviewPage extends StatefulWidget {
  const LogviewPage({super.key});

  @override
  State<LogviewPage> createState() => LogviewPageState();
}

class LogviewPageState extends State<LogviewPage> {
  @override
  Widget build(BuildContext context) {
    return Observer(builder: (context) {
      appStore.log;
      final fullLog = appStore.log.join("\n");

      return Scaffold(
        appBar: AppBar(
          title: const Text("Log"),
        ),
        body: SingleChildScrollView(child: Padding(padding: const EdgeInsets.symmetric(horizontal: 8), child: SelectableText(fullLog))),
      );
    });
  }
}
