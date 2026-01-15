import 'package:decimal/decimal.dart';
import 'package:flutter/material.dart';
import 'package:flutter_form_builder/flutter_form_builder.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:form_builder_validators/form_builder_validators.dart';
import 'package:gap/gap.dart';
import 'package:zkool/src/rust/api/network.dart';
import 'package:zkool/store.dart';
import 'package:zkool/utils.dart';
import 'package:zkool/validators.dart';

class InputAmount extends ConsumerStatefulWidget {
  final String name;
  final String? initialValue;
  final void Function()? onMax;
  final void Function(String?)? onChanged;
  const InputAmount({required this.name, this.initialValue, this.onMax, this.onChanged, super.key});

  @override
  ConsumerState<InputAmount> createState() => InputAmountState();
}

class InputAmountState extends ConsumerState<InputAmount> {
  final formFieldKey = GlobalKey<FormBuilderFieldState>();
  final formKey = GlobalKey<FormBuilderState>();

  @override
  Widget build(BuildContext context) {
    final coingecko = ref.watch(appSettingsProvider).whenData((s) => s.coingecko);
    if (coingecko.value == null) return blank(context);

    final price = ref.watch(priceProvider);
    return FormBuilderField<String>(
      key: formFieldKey,
      name: widget.name,
      initialValue: widget.initialValue,
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
                      initialValue: displayPrice(price),
                      onChanged: onPriceChanged,
                    ),
                  ),
                  Gap(8),
                  IconButton(
                    onPressed: () => onUpdateFx(coingecko.requireValue),
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

  void onUpdateFx(String coingecko) async {
    final p = await getCoingeckoPrice(api: coingecko);
    setState(() {
      final price = ref.read(priceProvider.notifier);
      price.setPrice(p);
      formKey.currentState!.fields["fx"]!.didChange(displayPrice(p));
    });
  }

  String? fx() => formKey.currentState!.fields["fx"]!.value as String?;
  String? displayPrice(double? p) => p?.let((p) => doubleToString(p, decimals: 3));

  bool disableChangeHandlers = false;

  void onPriceChanged(String? v) {
    if (v == null) return;
    final p = stringToDecimal(v, scale: 3);
    setState(() {
      double pp = p.toDecimal().toDouble();
      final price = ref.read(priceProvider.notifier);
      price.setPrice(pp);
    });
    WidgetsBinding.instance.addPostFrameCallback((_) {
      disableChangeHandlers = true;
      final form = formKey.currentState!;
      final v = form.fields["zat"]!.value;
      if (v != null) {
        final usd = stringToZat(v).toDecimal() * p.toDecimal() / Decimal.fromInt(zatsPerZec);
        form.fields["fiat"]!.didChange(displayPrice(usd.toDecimal().toDouble()));
      }
      disableChangeHandlers = false;
    });
  }

  void onChanged(String? v, {bool interactive = false}) {
    if (disableChangeHandlers || v == null) return;
    final price = ref.read(priceProvider);
    formFieldKey.currentState!.setValue(v);
    final form = formKey.currentState!;
    if (!interactive) form.fields["zat"]!.didChange(v);
    WidgetsBinding.instance.addPostFrameCallback((_) {
      disableChangeHandlers = true;
      if (v.isEmpty) {
        onReset(zat: false);
        formFieldKey.currentState!.reset();
      } else if (price != null) {
        final usd = stringToZat(v).toDouble() * price / 1e8;
        form.fields["fiat"]!.didChange(displayPrice(usd));
      }
      disableChangeHandlers = false;
    });
    widget.onChanged?.call(v);
  }

  void onFiatChanged(String? v, {bool interactive = false}) {
    if (disableChangeHandlers || v == null) return;
    final price = ref.read(priceProvider);
    final form = formKey.currentState!;
    if (!interactive) form.fields["fiat"]!.didChange(v);
    WidgetsBinding.instance.addPostFrameCallback((_) {
      disableChangeHandlers = true;
      if (v.isEmpty) {
        onReset(fiat: false);
        formFieldKey.currentState!.reset();
      } else if (price != null) {
        final zat = double.parse(v) / price * 1e8;
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
