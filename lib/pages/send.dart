import 'dart:io';

import 'package:file_picker/file_picker.dart';
import 'package:flutter/material.dart';
import 'package:flutter_form_builder/flutter_form_builder.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge.dart';
import 'package:form_builder_validators/form_builder_validators.dart';
import 'package:go_router/go_router.dart';
import 'package:showcaseview/showcaseview.dart';
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

final addressID = GlobalKey();
final scanID = GlobalKey();
final amountID = GlobalKey();
final logID2 = GlobalKey();
final openTxID = GlobalKey();
final addTxID = GlobalKey();
final sendID2 = GlobalKey();
final memoID = GlobalKey();

class SendPage extends StatefulWidget {
  const SendPage({super.key});

  @override
  State<SendPage> createState() => SendPageState();
}

class SendPageState extends State<SendPage> {
  final formKey = GlobalKey<FormBuilderState>();
  final addressController = TextEditingController();
  var amount = "";
  String? memo;
  List<Recipient> recipients = [];
  bool supportsMemo = false;

  void tutorial() async {
    tutorialHelper(context, "tutSend0",
        [addressID, scanID, amountID, logID2, openTxID, addTxID, sendID2]);
    if (supportsMemo) tutorialHelper(context, "tutSend1", [memoID]);
  }

  @override
  Widget build(BuildContext context) {
    Future(tutorial);

    final address = formKey.currentState?.fields['address']?.value as String? ?? "";
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

    supportsMemo = validAddress(address) == null &&
        !isValidTransparentAddress(address: address);
    return Scaffold(
        appBar: AppBar(
          title: Text("Recipient"),
          actions: [
            Showcase(
                key: logID2,
                description: "Show App Log",
                child: IconButton(
                    tooltip: "Open Log",
                    onPressed: () => onOpenLog(context),
                    icon: Icon(Icons.description))),
            Showcase(
                key: openTxID,
                description: "Load an unsigned transaction",
                child: IconButton(
                    tooltip: "Load Tx",
                    onPressed: onLoad,
                    icon: Icon(Icons.file_open))),
            Showcase(
                key: addTxID,
                description: "Queue this recipient to create a multi send",
                child: IconButton(
                    tooltip: "Add to Multi Tx",
                    onPressed: onAdd,
                    icon: Icon(Icons.add))),
            Showcase(
                key: sendID2,
                description: "Send transaction (including queued recipients)",
                child: IconButton(
                    tooltip: "Send (Next Step)",
                    onPressed: onSend,
                    icon: Icon(Icons.send))),
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
                            child: Showcase(
                                key: addressID,
                                description:
                                    "Receiver Address (Transparent, Sapling or UA)",
                                child: FormBuilderTextField(
                                  name: "address",
                                  controller: addressController,
                                  decoration: const InputDecoration(
                                      labelText: "Address"),
                                  validator: FormBuilderValidators.compose([
                                    FormBuilderValidators.required(),
                                    validAddressOrPaymentURI
                                  ]),
                                  onChanged: onAddressChanged,
                                ))),
                        Showcase(
                            key: scanID,
                            description: "Open the QR Scanner",
                            child: IconButton(
                                tooltip: "Scan",
                                onPressed: onScan,
                                icon: Icon(Icons.qr_code_scanner))),
                      ]),
                      Showcase(
                          key: amountID,
                          description: "Amount to send",
                          child: FormBuilderTextField(
                            name: "amount",
                            decoration:
                                const InputDecoration(labelText: "Amount"),
                            validator: FormBuilderValidators.compose([
                              FormBuilderValidators.required(),
                              validAmount
                            ]),
                            keyboardType: TextInputType.number,
                            initialValue: amount,
                            onChanged: (v) => setState(() => amount = v!),
                          )),
                      if (supportsMemo)
                        Showcase(
                            key: memoID,
                            description: "Optional memo",
                            child: FormBuilderTextField(
                              name: "memo",
                              decoration:
                                  const InputDecoration(labelText: "Memo"),
                              initialValue: memo,
                              onChanged: (v) => setState(() => memo = v),
                              maxLines: 8,
                            )),
                    ])))));
  }

  void onLoad() async {
    final files = await FilePicker.platform.pickFiles(
      dialogTitle: 'Please select a transaction to sign',
    );
    if (files == null) return;
    final file = File(files.files.first.path!);
    final bytes = await file.readAsBytes();
    final pczt = await unpackTransaction(bytes: bytes);
    if (!mounted) return;
    GoRouter.of(context).go("/tx", extra: pczt.copyWith(canSign: true));
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
        formKey.currentState!.fields['address']!.didChange(address2);
      });
    }
  }

  void onAddressChanged(String? v) {
    if (v == null || v.isEmpty) return;
    final recipients2 = parsePaymentUri(uri: v);
    if (recipients2 != null) {
      setState(() {
        recipients = recipients2;
      });
      WidgetsBinding.instance.addPostFrameCallback((_) {
        addressController.clear();
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

final sourceID = GlobalKey();
final feeSourceID = GlobalKey();
final sendID3 = GlobalKey();

class Send2Page extends StatefulWidget {
  final List<Recipient> recipients;
  const Send2Page(this.recipients, {super.key});

  @override
  State<Send2Page> createState() => Send2PageState();
}

class Send2PageState extends State<Send2Page> {
  String? txId;
  var recipientPaysFee = false;
  final formKey = GlobalKey<FormBuilderState>();

  void tutorial() async {
    tutorialHelper(context, "tutSend2", [sourceID, feeSourceID, sendID3]);
  }

  @override
  Widget build(BuildContext context) {
    Future(tutorial);

    return Scaffold(
      appBar: AppBar(
        title: Text("Extra Options"),
        actions: [
          IconButton(
              tooltip: "Open Log",
              onPressed: () => onOpenLog(context),
              icon: Icon(Icons.description)),
          Showcase(
              key: sendID3,
              description: "Send (Summary and Confirmation)",
              child: IconButton(
                  tooltip: "Send (Compute Tx)",
                  onPressed: onSend,
                  icon: Icon(Icons.send))),
        ],
      ),
      body: SingleChildScrollView(
        child: Padding(
          padding: EdgeInsets.symmetric(horizontal: 8),
          child: FormBuilder(
              key: formKey,
              child: Column(children: [
                Showcase(
                    key: sourceID,
                    description:
                        "Pools to take funds from. Uncheck any pool you do not want to use",
                    child: InputDecorator(
                        decoration: InputDecoration(labelText: "Source Pools"),
                        child: Align(
                            alignment: Alignment.centerRight,
                            child: FormBuilderField<int>(
                              name: "source pools",
                              builder: (field) => PoolSelect(
                                  onChanged: (v) => field.didChange(v)),
                            )))),
                Showcase(
                    key: feeSourceID,
                    description:
                        "Who pays the fees. Usually, the sender pays the transaction fees. Check if you want the receipient instead",
                    child: FormBuilderSwitch(
                      name: "recipientPaysFee",
                      title: Text("Recipient Pays Fee"),
                      initialValue: false,
                      onChanged: (v) => setState(() => recipientPaysFee = v!),
                    )),
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

    final srcPools = form.fields['source pools']?.value ?? 7;

    try {
      final pczt = await prepare(
          srcPools: srcPools,
          recipients: widget.recipients,
          recipientPaysFee: recipientPaysFee);

      GoRouter.of(navigatorKey.currentContext!).go("/tx", extra: pczt);
    } on AnyhowException catch (e) {
      if (mounted) await showException(context, e.message);
    }
  }
}
