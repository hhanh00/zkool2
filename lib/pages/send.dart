import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_form_builder/flutter_form_builder.dart';
import 'package:form_builder_validators/form_builder_validators.dart';
import 'package:go_router/go_router.dart';
import 'package:zkool/main.dart';
import 'package:zkool/src/rust/api/key.dart';
import 'package:zkool/src/rust/api/pay.dart';
import 'package:zkool/src/rust/pay.dart';
import 'package:zkool/utils.dart';
import 'package:zkool/validators.dart';

class SendPage extends StatefulWidget {
  const SendPage({super.key});

  @override
  State<SendPage> createState() => SendPageState();
}

class SendPageState extends State<SendPage> {
  final formKey = GlobalKey<FormBuilderState>();
  var address = "";
  var amount = "";
  String? memo;

  @override
  Widget build(BuildContext context) {
    return Scaffold(
        appBar: AppBar(
          title: Text("Recipient"),
          actions: [
            IconButton(onPressed: onAdd, icon: Icon(Icons.add)),
            IconButton(onPressed: onSend, icon: Icon(Icons.send)),
          ],
        ),
        body: SingleChildScrollView(
            child: Padding(
                padding: EdgeInsets.symmetric(horizontal: 8),
                child: FormBuilder(
                    key: formKey,
                    child: Column(children: [
                      FormBuilderTextField(
                        name: "address",
                        decoration: const InputDecoration(labelText: "Address"),
                        validator: FormBuilderValidators.compose(
                            [FormBuilderValidators.required(), validAddress]),
                        initialValue: address,
                        onChanged: (v) => setState(() => address = v!),
                      ),
                      FormBuilderTextField(
                        name: "amount",
                        decoration: const InputDecoration(labelText: "Amount"),
                        validator: FormBuilderValidators.compose(
                            [FormBuilderValidators.required(), validAmount]),
                        keyboardType: TextInputType.number,
                        initialValue: amount,
                        onChanged: (v) => setState(() => amount = v!),
                      ),
                      if (validAddress(address) == null &&
                          !isValidTransparentAddress(address: address))
                        FormBuilderTextField(
                          name: "memo",
                          decoration: const InputDecoration(labelText: "Memo"),
                          initialValue: memo,
                          onChanged: (v) => setState(() => memo = v!),
                          maxLines: 8,
                        ),
                    ])))));
  }

  void onAdd() {}
  void onSend() async {
    final form = formKey.currentState!;
    if (form.saveAndValidate()) {
      final address = form.fields['address']?.value as String;
      final amount = form.fields['amount']?.value as String;
      final memo = form.fields['memo']?.value as String?;
      logger.i("Send $amount to $address");

      final recipient = Recipient(
          address: address, amount: stringToZat(amount), userMemo: memo);
      if (mounted)
        await GoRouter.of(context).push("/send2", extra: [recipient]);
    } else {
      print("Invalid form");
    }
  }
}

class Send2Page extends StatefulWidget {
  final List<Recipient> recipients;
  const Send2Page(this.recipients, {super.key});

  @override
  State<Send2Page> createState() => Send2PageState();
}

class Send2PageState extends State<Send2Page> {
  String? txId;
  var recipientPaysFee = false;
  var srcPools = "7";
  final formKey = GlobalKey<FormBuilderState>();

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: Text("Extra Options"),
        actions: [
          IconButton(onPressed: onSend, icon: Icon(Icons.send)),
        ],
      ),
      body: SingleChildScrollView(
        child: Padding(
          padding: EdgeInsets.symmetric(horizontal: 8),
          child: FormBuilder(
              key: formKey,
              child: Column(children: [
                FormBuilderTextField(
                  name: "source pools",
                  decoration: const InputDecoration(labelText: "Source Pools"),
                  validator: FormBuilderValidators.compose([
                    FormBuilderValidators.required(),
                    FormBuilderValidators.integer(),
                    FormBuilderValidators.min(1),
                    FormBuilderValidators.max(7),
                  ]),
                  keyboardType: TextInputType.number,
                  inputFormatters: [FilteringTextInputFormatter.digitsOnly],
                  initialValue: srcPools,
                  onChanged: (v) => setState(() => srcPools = v!),
                ),
                FormBuilderSwitch(
                  name: "recipientPaysFee",
                  title: Text("Recipient Pays Fee"),
                  initialValue: false,
                  onChanged: (v) => setState(() => recipientPaysFee = v!),
                ),
              ])),
        ),
      ),
    );
  }

  void onSend() async {
    final form = formKey.currentState!;
    if (!form.saveAndValidate()) {
      print("Invalid form");
      return;
    }
    
    final srcPools2 = int.parse(srcPools);

    final tx = await prepare(
        srcPools: srcPools2,
        recipients: widget.recipients,
        recipientPaysFee: recipientPaysFee);

    if (mounted)
      await GoRouter.of(context).push("/tx", extra: tx);
  }
}
