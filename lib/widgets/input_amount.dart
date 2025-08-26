import 'package:flutter/material.dart';
import 'package:flutter_form_builder/flutter_form_builder.dart';
import 'package:form_builder_validators/form_builder_validators.dart';
import 'package:gap/gap.dart';
import 'package:zkool/src/rust/api/network.dart';
import 'package:zkool/utils.dart';
import 'package:zkool/validators.dart';

class InputAmount extends StatefulWidget {
  final String name;
  final String? amount;
  final void Function(int amount)? onChanged;
  final void Function()? onMax;
  const InputAmount({required this.name, this.amount, this.onChanged, this.onMax, super.key});

  @override
  State<StatefulWidget> createState() => InputAmountState();
}

class InputAmountState extends State<InputAmount> {
  final formKey = GlobalKey<FormBuilderState>();
  double? price;

  @override
  Widget build(BuildContext context) {
    return FormBuilderField<String>(
        name: widget.name,
        initialValue: widget.amount,
        onChanged: onChanged,
        builder: (state) {
          return FormBuilder(
              key: formKey,
              child: Column(children: [
                Row(children: [
                  Expanded(
                      child: FormBuilderTextField(
                    name: "zat",
                    decoration: InputDecoration(label: Text("Amount in ZEC")),
                    validator: FormBuilderValidators.compose([FormBuilderValidators.required(), validAmount]),
                    keyboardType: TextInputType.numberWithOptions(decimal: true),
                    onChanged: (v) => onChanged(v, interactive: true),
                  )),
                  Gap(8),
                  IconButton(onPressed: widget.onMax, icon: Icon(Icons.vertical_align_top))
                ]),
                Row(
                  children: [
                    Expanded(
                        child: FormBuilderTextField(
                      name: "fiat",
                      decoration: InputDecoration(label: Text("Amount in USD")),
                      validator: validAmount,
                      keyboardType: TextInputType.numberWithOptions(decimal: true),
                      onChanged: (v) => onFiatChanged(v, interactive: true),
                    )),
                    Gap(8),
                    ElevatedButton.icon(
                      onPressed: onUpdateFx,
                      label: Text(price != null ? "$price USD" : "? USD"),
                      icon: Icon(Icons.refresh),
                    ),
                  ],
                )
              ]));
        });
  }

  void onUpdateFx() async {
    final p = await getCoingeckoPrice();
    setState(() => price = p);
  }

  bool disableChangeHandlers = false;

  void onChanged(String? v, {bool interactive = false}) {
    if (disableChangeHandlers || v == null) return;
    final form = formKey.currentState!;
    if (!interactive) form.fields["zat"]!.didChange(v);
    WidgetsBinding.instance.addPostFrameCallback((_) {
      disableChangeHandlers = true;
      if (v.isEmpty)
        form.fields["fiat"]!.reset();
      else if (price != null) {
        final usd = stringToZat(v).toDouble() * price! / 1e8;
        form.fields["fiat"]!.didChange(usd.toStringAsFixed(2));
      }
      disableChangeHandlers = false;
    });
  }

  void onFiatChanged(String? v, {bool interactive = false}) {
    if (disableChangeHandlers || v == null) return;
    final form = formKey.currentState!;
    if (!interactive) form.fields["fiat"]!.didChange(v);
    WidgetsBinding.instance.addPostFrameCallback((_) {
      disableChangeHandlers = true;
      if (v.isEmpty)
        form.fields["zat"]!.reset();
      else if (price != null) {
        final zat = double.parse(v) / price! * 1e8;
        form.fields["zat"]!.didChange(zatToString(BigInt.from(zat)));
      }
      disableChangeHandlers = false;
    });
  }
}
