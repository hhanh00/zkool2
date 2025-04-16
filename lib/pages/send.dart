import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_form_builder/flutter_form_builder.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge.dart';
import 'package:form_builder_validators/form_builder_validators.dart';
import 'package:go_router/go_router.dart';
import 'package:zkool/main.dart';
import 'package:zkool/pages/account.dart';
import 'package:zkool/router.dart';
import 'package:zkool/src/rust/api/key.dart';
import 'package:zkool/src/rust/api/pay.dart';
import 'package:zkool/src/rust/pay.dart';
import 'package:zkool/utils.dart';
import 'package:zkool/validators.dart';
import 'package:zkool/widgets/pool_select.dart';
import 'package:zkool/widgets/scanner.dart';

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
  List<Recipient> recipients = [];

  @override
  Widget build(BuildContext context) {
    final recipientTiles = recipients
        .map((r) => ListTile(
              title: Text(r.address),
              subtitle: Text(zatToString(r.amount)),
              trailing: IconButton(
                  icon: Icon(Icons.delete),
                  onPressed: () {
                    setState(() {
                      recipients.remove(r);
                    });
                  }),
            ))
        .toList();

    return Scaffold(
        appBar: AppBar(
          title: Text("Recipient"),
          actions: [
            IconButton(
                tooltip: "Open Log",
                onPressed: () => onOpenLog(context),
                icon: Icon(Icons.description)),
            IconButton(
                tooltip: "Add to Multi Tx",
                onPressed: onAdd,
                icon: Icon(Icons.add)),
            IconButton(
                tooltip: "Send (Next Step)",
                onPressed: onSend,
                icon: Icon(Icons.send)),
          ],
        ),
        body: SingleChildScrollView(
            child: Padding(
                padding: EdgeInsets.symmetric(horizontal: 8),
                child: FormBuilder(
                    key: formKey,
                    child: Column(children: [
                      ...recipientTiles,
                      Row(children: [
                        Expanded(
                            child: FormBuilderTextField(
                          name: "address",
                          decoration:
                              const InputDecoration(labelText: "Address"),
                          validator: FormBuilderValidators.compose(
                              [FormBuilderValidators.required(), validAddress]),
                          initialValue: address,
                          onChanged: (v) => setState(() => address = v!),
                        )),
                        IconButton(
                            tooltip: "Scan",
                            onPressed: onScan,
                            icon: Icon(Icons.qr_code_scanner)),
                      ]),
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
                          onChanged: (v) => setState(() => memo = v),
                          maxLines: 8,
                        ),
                    ])))));
  }

  void onAdd() async {
    final recipient = await validateAndGetRecipient();
    if (recipient != null) {
      setState(() {
        recipients.add(recipient);
      });
    }
  }

  void onSend() async {
    final recipient = await validateAndGetRecipient();
    if (recipient != null && mounted)
      await GoRouter.of(context).push("/send2", extra: [recipient]);
  }

  void onScan() async {
    final address2 = await showScanner(context, validator: validAddress);
    logger.i(address2);
    if (address2 != null) {
      setState(() {
        address = address2;
        formKey.currentState!.fields['address']!.didChange(address2);
      });
    }
  }

  Future<Recipient?> validateAndGetRecipient() async {
    final form = formKey.currentState!;
    if (form.saveAndValidate()) {
      final address = form.fields['address']?.value as String;
      final amount = form.fields['amount']?.value as String;
      final memo = form.fields['memo']?.value as String?;
      logger.i("Send $amount to $address");

      final recipient = Recipient(
          address: address, amount: stringToZat(amount), userMemo: memo);
      return recipient;
    }
    return null;
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
          IconButton(
              tooltip: "Open Log",
              onPressed: () => onOpenLog(context),
              icon: Icon(Icons.description)),
          IconButton(
              tooltip: "Send (Compute Tx)",
              onPressed: onSend,
              icon: Icon(Icons.send)),
        ],
      ),
      body: SingleChildScrollView(
        child: Padding(
          padding: EdgeInsets.symmetric(horizontal: 8),
          child: FormBuilder(
              key: formKey,
              child: Column(children: [
                InputDecorator(
                    decoration: InputDecoration(labelText: "Source Pools"),
                    child: Align(
                        alignment: Alignment.centerRight,
                        child: FormBuilderField<int>(
                          name: "source pools",
                          builder: (field) =>
                              PoolSelect(onChanged: (v) => field.didChange(v)),
                        ))),
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
      return;
    }

    final srcPools2 = int.parse(srcPools);

    try {
      final tx = await prepare(
          srcPools: srcPools2,
          recipients: widget.recipients,
          recipientPaysFee: recipientPaysFee);

      GoRouter.of(navigatorKey.currentContext!).go("/tx", extra: tx);
    } on AnyhowException catch (e) {
      if (mounted) await showException(context, e.message);
    }
  }
}
