import 'package:awesome_dialog/awesome_dialog.dart';
import 'package:flutter/material.dart';
import 'package:flutter_form_builder/flutter_form_builder.dart';
import 'package:flutter_mobx/flutter_mobx.dart';
import 'package:form_builder_validators/form_builder_validators.dart';
import 'package:gap/gap.dart';
import 'package:go_router/go_router.dart';
import 'package:zkool/store.dart';
import 'package:zkool/utils.dart';


Future<void> showTransparentScan(BuildContext context) async {
  final t = Theme.of(context).textTheme;
  final formKey = GlobalKey<FormBuilderState>();

  bool validated = false;
  late final AwesomeDialog dialog;
  final scanner = appStore.transparentScanner;
  dialog = AwesomeDialog(
    context: context,
    dialogType: DialogType.question,
    animType: AnimType.rightSlide,
    body: FormBuilder(
      key: formKey,
      child: Observer(
        builder: (context) => Column(
          children: [
            FormBuilderTextField(
              name: "gap",
              decoration: InputDecoration(label: Text("Gap Limit")),
              initialValue: scanner.gapLimit.toString(),
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
    btnCancelOnPress: () {},
    btnOkOnPress: () {},
    btnOk: Observer(
      builder: (context) => scanner.running ?
      AnimatedButton(
        isFixedHeight: false,
        text: "Stop",
        color: Colors.red,
        pressEvent: () async {
          showSnackbar("Cancelling Scan");
          await scanner.cancel();
        }) :
      AnimatedButton(
        isFixedHeight: false,
        text: "Run",
        color: const Color(0xFF00CA71),
        pressEvent: () async {
          final form = formKey.currentState!;
          validated = form.validate();
          if (validated) {
            final gapLimitStr = form.fields["gap"]!.value as String? ?? "";
            final gapLimit = int.parse(gapLimitStr);
            await appStore.transparentScanner.run(
              context,
              gapLimit,
              onComplete: () => showSnackbar("Scan Completed"),
            );
          }
        },
      ),
    ),
    btnCancelText: "Close",
    onDismissCallback: (type) {
      GoRouter.of(context).pop();
    },
    dismissOnTouchOutside: false,
    autoDismiss: false,
  );
  await dialog.show();
}
