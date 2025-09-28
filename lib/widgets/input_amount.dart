import 'package:decimal/decimal.dart';
import 'package:flutter/material.dart';
import 'package:flutter_form_builder/flutter_form_builder.dart';
import 'package:form_builder_validators/form_builder_validators.dart';
import 'package:gap/gap.dart';
import 'package:zkool/src/rust/api/network.dart';
import 'package:zkool/store.dart';
import 'package:zkool/utils.dart';
import 'package:zkool/validators.dart';

class InputAmount extends StatefulWidget {
  final String name;
  final String? amount;
  final void Function()? onMax;
  const InputAmount({required this.name, this.amount, this.onMax, super.key});

  @override
  State<StatefulWidget> createState() => InputAmountState();
}

class InputAmountState extends State<InputAmount> {
  final formFieldKey = GlobalKey<FormBuilderFieldState>();
  final formKey = GlobalKey<FormBuilderState>();
  double? price = appStore.price;

  @override
  Widget build(BuildContext context) {
    return FormBuilderField<String>(
      key: formFieldKey,
      name: widget.name,
      initialValue: widget.amount,
      onReset: onReset,
      onChanged: onChanged,
      builder: (state) {
        return FormBuilder(
          key: formKey,
          child: Column(
            children: [
              Row(
                children: [
                  Expanded(
                    child: FormBuilderTextField(
                      name: "zat",
                      decoration: InputDecoration(label: Text("Amount in ZEC")),
                      validator: FormBuilderValidators.compose([FormBuilderValidators.required(), validAmount]),
                      keyboardType: TextInputType.numberWithOptions(decimal: true),
                      onChanged: (v) => onChanged(v, interactive: true),
                    ),
                  ),
                  Gap(8),
                  IconButton(onPressed: widget.onMax, icon: Icon(Icons.vertical_align_top)),
                ],
              ),
              Row(
                children: [
                  Expanded(
                    child: FormBuilderTextField(
                      name: "fiat",
                      decoration: InputDecoration(label: Text("Amount in USD")),
                      validator: validAmount,
                      keyboardType: TextInputType.numberWithOptions(decimal: true),
                      onChanged: (v) => onFiatChanged(v, interactive: true),
                    ),
                  ),
                  Gap(8),
                  SizedBox(
                    width: 80,
                    child: FormBuilderTextField(
                      name: "fx",
                      decoration: InputDecoration(label: Text("Fx")),
                      validator: validAmount,
                      keyboardType: TextInputType.numberWithOptions(decimal: true),
                      initialValue: price?.toStringAsFixed(3),
                      onChanged: onPriceChanged,
                    ),
                  ),
                  Gap(8),
                  IconButton(
                    onPressed: onUpdateFx,
                    icon: Icon(Icons.refresh),
                  ),
                ],
              ),
              Gap(16),
              Text("The Amount in USD is indicative. The transaction is always made in ZEC."),
            ],
          ),
        );
      },
    );
  }

  void onUpdateFx() async {
    final p = await getCoingeckoPrice();
    setState(() {
      price = p;
      formKey.currentState!.fields["fx"]!.didChange(price?.toStringAsFixed(3));
    });
  }

  String? fx() => formKey.currentState!.fields["fx"]!.value as String?;

  bool disableChangeHandlers = false;

  void onPriceChanged(String? v) {
    if (v == null) return;
    final p = stringToDecimal(v, scale: 3);
    setState(() {
      price = p.toDecimal().toDouble();
      appStore.price = price;
    });
    WidgetsBinding.instance.addPostFrameCallback((_) {
      disableChangeHandlers = true;
      final form = formKey.currentState!;
      final v = form.fields["zat"]!.value;
      if (v != null) {
        final usd = stringToZat(v).toDecimal() * p.toDecimal() /
          Decimal.fromInt(zatsPerZec);
        form.fields["fiat"]!.didChange(usd.toDecimal().toStringAsFixed(2));
      }
      disableChangeHandlers = false;
    });
  }

  void onChanged(String? v, {bool interactive = false}) {
    if (disableChangeHandlers || v == null) return;
    formFieldKey.currentState!.setValue(v);
    final form = formKey.currentState!;
    if (!interactive) form.fields["zat"]!.didChange(v);
    WidgetsBinding.instance.addPostFrameCallback((_) {
      disableChangeHandlers = true;
      if (v.isEmpty) {
        onReset(zat: false);
        formFieldKey.currentState!.reset();
      } else if (price != null) {
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
      if (v.isEmpty) {
        onReset(fiat: false);
        formFieldKey.currentState!.reset();
      } else if (price != null) {
        final zat = double.parse(v) / price! * 1e8;
        final z = zatToString(BigInt.from(zat));
        form.fields["zat"]!.didChange(z);
        formFieldKey.currentState!.setValue(z);
      }
      disableChangeHandlers = false;
    });
  }

  void onReset({bool zat = true, bool fiat = true}) {
    final form = formKey.currentState!;
    if (zat) form.fields["zat"]!.reset();
    if (fiat) form.fields["fiat"]!.reset();
  }
}
