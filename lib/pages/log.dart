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

      return Scaffold(
        appBar: AppBar(
          title: const Text("Log"),
        ),
        body: Padding(
            padding: const EdgeInsets.symmetric(horizontal: 8),
            child: ListView.builder(
              itemBuilder: (context, index) {
                final logEntry = appStore.log[index];
                return Text(logEntry);
              },
              itemCount: appStore.log.length,
            )),
      );
    });
  }
}
