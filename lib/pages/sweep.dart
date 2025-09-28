import 'package:flutter/material.dart';
import 'package:flutter_form_builder/flutter_form_builder.dart';
import 'package:flutter_mobx/flutter_mobx.dart';
import 'package:form_builder_validators/form_builder_validators.dart';
import 'package:gap/gap.dart';
import 'package:zkool/store.dart';
import 'package:zkool/utils.dart';

class TransparentSweepPage extends StatefulWidget {
  const TransparentSweepPage({super.key});

  @override
  State<StatefulWidget> createState() => TransparentSweepPageState();
}

class TransparentSweepPageState extends State<TransparentSweepPage> {
  final defaultGapLimit = 40;
  final formKey = GlobalKey<FormBuilderState>();
  bool running = appStore.transparentScanner.running;

  @override
  Widget build(BuildContext context) {
    final t = Theme.of(context).textTheme;
    final scanner = appStore.transparentScanner;

    return Scaffold(
      appBar: AppBar(
        title: Text("Transparent Scanner"),
        actions: [
          if (!running) IconButton(onPressed: onRun, icon: Icon(Icons.play_arrow)),
          if (running) IconButton(onPressed: onStop, icon: Icon(Icons.stop)),
        ],
      ),
      body: FormBuilder(
        key: formKey,
        child: Padding(
          padding: EdgeInsets.symmetric(horizontal: 8),
          child: Observer(
            builder: (context) => Column(
              children: [
                FormBuilderTextField(
                  name: "gap",
                  decoration: InputDecoration(label: Text("Gap Limit")),
                  initialValue: defaultGapLimit.toString(),
                  validator: FormBuilderValidators.compose([
                    FormBuilderValidators.required(),
                    FormBuilderValidators.integer(),
                  ]),
                ),
                Gap(32),
                if (scanner.running)
                  Column(
                    children: [
                      LinearProgressIndicator(),
                      Gap(16),
                      Text(scanner.address, style: t.bodyMedium),
                    ],
                  ),
              ],
            ),
          ),
        ),
      ),
    );
  }

  void onRun() async {
    final form = formKey.currentState!;
    if (form.validate()) {
      final gapLimitStr = form.fields["gap"]!.value as String? ?? "";
      final gapLimit = int.parse(gapLimitStr);
      await appStore.transparentScanner.run(
        context,
        gapLimit,
        onComplete: () => showSnackbar("Scan Completed"),
      );
      setState(() => running = true);
      showSnackbar("Starting Scan");
    }
  }

  void onStop() async {
    showSnackbar("Cancelling Scan");
    await appStore.transparentScanner.cancel();
    setState(() => running = false);
  }
}
