import 'package:collection/collection.dart';
import 'package:flutter/material.dart';
import 'package:flutter_form_builder/flutter_form_builder.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge.dart';
import 'package:form_builder_validators/form_builder_validators.dart';
import 'package:gap/gap.dart';
import 'package:go_router/go_router.dart';
import 'package:qr_flutter/qr_flutter.dart';
import 'package:showcaseview/showcaseview.dart';
import 'package:zkool/main.dart';
import 'package:zkool/pages/account.dart';
import 'package:zkool/router.dart';
import 'package:zkool/src/rust/api/account.dart';
import 'package:zkool/src/rust/api/key.dart';
import 'package:zkool/src/rust/api/pay.dart';
import 'package:zkool/src/rust/api/sync.dart';
import 'package:zkool/src/rust/pay.dart';
import 'package:zkool/store.dart';
import 'package:zkool/utils.dart';
import 'package:zkool/validators.dart';
import 'package:zkool/widgets/input_amount.dart';
import 'package:zkool/widgets/pool_select.dart';
import 'package:zkool/widgets/scanner.dart';

final addressID = GlobalKey();
final scanID = GlobalKey();
final amountID = GlobalKey();
final openTxID = GlobalKey();
final addTxID = GlobalKey();
final clearID = GlobalKey();
final sendID2 = GlobalKey();
final dustChangePolicyID = GlobalKey();

class SendPage extends StatefulWidget {
  const SendPage({super.key});

  @override
  State<SendPage> createState() => SendPageState();
}

class SendPageState extends State<SendPage> {
  final formKey = GlobalKey<FormBuilderState>();
  List<Recipient> recipients = [];
  bool supportsMemo = false;
  PoolBalance? pbalance;
  Addresses? addresses;
  int? editingIndex;

  void tutorial() async {
    tutorialHelper(context, "tutSend0", [addressID, scanID, amountID, openTxID, addTxID, sendID2]);
  }

  @override
  void initState() {
    super.initState();
    Future(() async {
      final b = await balance();
      final a = await getAddresses(uaPools: appStore.pools);

      setState(() {
        pbalance = b;
        addresses = a;
      });
    });
  }

  @override
  Widget build(BuildContext context) {
    Future(tutorial);
    final t = Theme.of(context).textTheme;
    final cs = Theme.of(context).colorScheme;

    final balance = pbalance;
    final address = formKey.currentState?.fields['address']?.value as String? ?? "";
    final recipientTiles = recipients
        .mapIndexed((i, r) => ListTile(
            title: Text(r.address),
            subtitle: zatToText(r.amount, selectable: false),
            trailing: IconButton(
                icon: Icon(Icons.delete),
                onPressed: () {
                  setState(() {
                    editingIndex = null;
                    recipients.remove(r);
                  });
                }),
            onTap: () => onEdit(i),
            selectedTileColor: cs.inversePrimary,
            selected: i == editingIndex))
        .toList();

    supportsMemo = address.isNotEmpty && validAddress(address) == null && !isValidTransparentAddress(address: address);
    return Scaffold(
        appBar: AppBar(
          title: Text("Recipient"),
          actions: [
            Showcase(
                key: openTxID,
                description: "Load an unsigned transaction",
                child: IconButton(tooltip: "Load Tx", onPressed: onLoad, icon: Icon(Icons.file_open))),
            Showcase(key: clearID, description: "Clear Form Inputs", child: IconButton(tooltip: "Clear", onPressed: reset, icon: Icon(Icons.clear))),
            Showcase(
                key: addTxID,
                description: "Queue this recipient to create a multi send",
                child: IconButton(tooltip: "Add to Multi Tx", onPressed: onAdd, icon: Icon(Icons.add))),
            Showcase(
                key: sendID2,
                description: "Send transaction (including queued recipients)",
                child: IconButton(tooltip: "Send (Next Step)", onPressed: onSend, icon: Icon(Icons.send))),
          ],
        ),
        body: SingleChildScrollView(
            child: Padding(
                padding: EdgeInsets.symmetric(horizontal: 8),
                child: FormBuilder(
                    key: formKey,
                    child: Column(children: [
                      ...recipientTiles,
                      if (balance != null) BalanceWidget(balance, onPoolSelected: onPoolSelected),
                      Gap(16),
                      OverflowBar(spacing: 16, children: [
                        if (addresses?.taddr != null) IconButton(onPressed: onUnshield, tooltip: "Unshield All", icon: Icon(Icons.lock_open)),
                        if (addresses?.saddr != null || addresses?.oaddr != null) ...[
                          IconButton(onPressed: () => onShield(true), tooltip: "Shield One", icon: Icon(Icons.shield_outlined)),
                          IconButton(onPressed: () => onShield(false), tooltip: "Shield All", icon: Icon(Icons.shield)),
                        ]
                      ]),
                      if (appStore.notes.any((n) => n.locked))
                        Container(
                          color: cs.secondaryContainer,
                          child: Text("Some notes are disabled", style: t.bodyLarge!.copyWith(color: cs.onSecondaryContainer))),
                      Row(children: [
                        Expanded(
                            child: Showcase(
                                key: addressID,
                                description: "Receiver Address (Transparent, Sapling or UA)",
                                child: FormBuilderTextField(
                                  name: "address",
                                  decoration: const InputDecoration(labelText: "Address"),
                                  validator: FormBuilderValidators.compose([FormBuilderValidators.required(), validAddressOrPaymentURI]),
                                  onChanged: onAddressChanged,
                                  onEditingComplete: onAddressEditComplete,
                                ))),
                        Showcase(
                            key: scanID,
                            description: "Open the QR Scanner",
                            child: IconButton(tooltip: "Scan", onPressed: onScan, icon: Icon(Icons.qr_code_scanner))),
                      ]),
                      InputAmount(name: "amount", onMax: onMax),
                      // Row(children: [
                      //   Expanded(
                      //       child: Showcase(
                      //           key: amountID,
                      //           description: "Amount to send",
                      //           child: FormBuilderTextField(
                      //             name: "amount",
                      //             decoration: const InputDecoration(labelText: "Amount"),
                      //             validator: FormBuilderValidators.compose([FormBuilderValidators.required(), validAmount]),
                      //             keyboardType: TextInputType.numberWithOptions(decimal: true),
                      //           ))),
                      //   IconButton(tooltip: "Set amount to entire balance", onPressed: onMax, icon: Icon(Icons.vertical_align_top)),
                      // ]),
                      Visibility(
                          visible: supportsMemo,
                          maintainState: true,
                          child: FormBuilderTextField(
                            name: "memo",
                            decoration: const InputDecoration(labelText: "Memo"),
                            maxLines: 8,
                          )),
                    ])))));
  }

  void onLoad() async {
    final data = await appWatcher.openFile(title: "Please select a transaction to sign");
    if (data == null) return;
    final pczt = await unpackTransaction(bytes: data);
    if (!mounted) return;
    GoRouter.of(context).go("/tx", extra: pczt.copyWith(canSign: true));
  }

  void onMax() async {
    final form = formKey.currentState!;
    final b = await balance();
    final total = b.field0[0] + b.field0[1] + b.field0[2];
    form.fields['amount']?.didChange(zatToString(total));
  }

  void onAdd() async {
    final recipient = await validateAndGetRecipient();
    if (recipient != null) {
      setState(() {
        recipients.add(recipient);
      });
      reset();
    }
  }

  void onShield(bool smartTransparent) async {
    if (!smartTransparent) {
      final confirmed = await confirmDialog(context,
          title: 'Shield All Privacy Warning',
          message: 'Shielding all your transparent funds may result in a transaction that links multiple t-addresses.\nPrefer using "Shield One".');
      if (!confirmed) return;
    }
    try {
      final options = PaymentOptions(
          srcPools: 1, // Only the transparent pool (mask)
          recipientPaysFee: true,
          smartTransparent: smartTransparent,
          dustChangePolicy: DustChangePolicy.sendToRecipient);
      final pczt = await prepare(
        recipients: [
          Recipient(
              address: addresses?.oaddr ?? addresses?.saddr ?? "", // Shield to Orchard or Sapling address
              amount: pbalance?.field0[0] ?? BigInt.zero)
        ],
        options: options,
      );

      GoRouter.of(navigatorKey.currentContext!).go("/tx", extra: pczt);
    } on AnyhowException catch (e) {
      if (mounted) await showException(context, e.message);
    }
  }

  void onUnshield() async {
    try {
      final options = PaymentOptions(
          srcPools: 6, // Only the sapling and orchard pool (mask)
          recipientPaysFee: true,
          dustChangePolicy: DustChangePolicy.sendToRecipient,
          smartTransparent: false);
      final pczt = await prepare(
        recipients: [Recipient(address: addresses?.taddr ?? "", amount: (pbalance?.field0[1] ?? BigInt.zero) + (pbalance?.field0[2] ?? BigInt.zero))],
        options: options,
      );

      GoRouter.of(navigatorKey.currentContext!).go("/tx", extra: pczt);
    } on AnyhowException catch (e) {
      if (mounted) await showException(context, e.message);
    }
  }

  void onEdit(int index) async {
    final currentEditingIndex = editingIndex;
    await finishEditing();
    if (currentEditingIndex != index) {
      editingIndex = index;
      final fields = formKey.currentState!.fields;
      final recipient = recipients[index];
      fields["address"]!.didChange(recipient.address);
      fields["amount"]!.didChange(zatToString(recipient.amount));
      fields["memo"]!.didChange(recipient.userMemo);
    }
    setState(() {});
  }

  Future<void> finishEditing() async {
    if (editingIndex != null) {
      final recipient = await validateAndGetRecipient();
      if (recipient != null) recipients[editingIndex!] = recipient;
      editingIndex = null;
      reset();
    }
  }

  void onSend() async {
    await finishEditing();

    logger.i(formKey.currentState!.isDirty);
    if (formKey.currentState!.isDirty) {
      final recipient = await validateAndGetRecipient();
      if (recipient != null) {
        recipients.add(recipient);
      } else
        return;
    }

    if (!mounted) return;
    reset();
    if (recipients.isNotEmpty) await GoRouter.of(context).push("/send2", extra: recipients);
  }

  void onScan() async {
    final address2 = await showScanner(context, validator: validAddressOrPaymentURI);
    if (address2 != null) {
      formKey.currentState!.fields["address"]!.didChange(address2);
    }
  }

  void onAddressChanged(String? v) {
    if (v == null || v.isEmpty) return;
    final recipients2 = parsePaymentUri(uri: v);
    if (recipients2 != null) {
      if (recipients2.length == 1) {
        final fields = formKey.currentState!.fields;
        final recipient = recipients2.first;
        WidgetsBinding.instance.addPostFrameCallback((_) {
          fields["address"]!.didChange(recipient.address);
          if (recipient.amount > BigInt.zero)
            fields["amount"]!.didChange(zatToString(recipient.amount));
          fields["memo"]!.didChange(recipient.userMemo);
          setState(() {});
        });
      } else {
        setState(() => recipients = recipients2);
        WidgetsBinding.instance.addPostFrameCallback((_) {
          if (formKey.currentState!.isDirty) reset();
        });
      }
    }
  }

  void onAddressEditComplete() {
    setState(() {});
  }

  Future<Recipient?> validateAndGetRecipient() async {
    final form = formKey.currentState!;
    if (form.saveAndValidate()) {
      final address = form.fields['address']?.value as String;
      final amount = form.fields['amount']?.value as String;
      final memo = form.fields['memo']?.value as String?;
      logger.i("Send $amount to $address");

      final recipient = Recipient(address: address, amount: stringToZat(amount), userMemo: memo);
      return recipient;
    }
    return null;
  }

  void onPoolSelected(int pool) {
    final a = addresses;
    if (a == null) return;
    final addressField = formKey.currentState!.fields["address"]!;
    switch (pool) {
      case 0:
        addressField.didChange(a.taddr ?? "");
      case 1:
        addressField.didChange(a.saddr ?? "");
      case 2:
        addressField.didChange(a.oaddr ?? "");
      default:
        logger.w("Unknown pool selected: $pool");
    }
    setState(() {});
  }

  void reset() {
    formKey.currentState!.reset();
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
  late final hasTex = widget.recipients.any((r) => isTexAddress(address: r.address));
  var recipientPaysFee = false;
  var discardDustChange = true;
  var puri = "";
  final formKey = GlobalKey<FormBuilderState>();

  void tutorial() async {
    tutorialHelper(context, "tutSend2", [sourceID, feeSourceID, sendID3]);
  }

  @override
  void initState() {
    super.initState();
    Future(() async {
      final uri = await buildPuri(recipients: widget.recipients);
      setState(() => puri = uri);
    });
  }

  @override
  Widget build(BuildContext context) {
    Future(tutorial);
    logger.i("hasTex: $hasTex, recipients: ${widget.recipients.length}");

    return Scaffold(
      appBar: AppBar(
        title: Text("Extra Options"),
        actions: [
          Showcase(
              key: sendID3,
              description: "Send (Summary and Confirmation)",
              child: IconButton(tooltip: "Send (Compute Tx)", onPressed: onSend, icon: Icon(Icons.send))),
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
                    description: "Pools to take funds from. Uncheck any pool you do not want to use",
                    child: InputDecorator(
                        decoration: InputDecoration(labelText: "Source Pools"),
                        child: Align(
                            alignment: Alignment.centerRight,
                            child: FormBuilderField<int>(
                              name: "source pools",
                              initialValue: hasTex ? 1 : appStore.pools,
                              builder: (field) =>
                                  PoolSelect(enabled: appStore.pools, initialValue: field.value!, onChanged: hasTex ? null : (v) => field.didChange(v)),
                            )))),
                Showcase(
                    key: feeSourceID,
                    description: "Who pays the fees. Usually, the sender pays the transaction fees. Check if you want the recipient instead",
                    child: FormBuilderSwitch(
                      name: "recipientPaysFee",
                      title: Text("Recipient Pays Fee"),
                      initialValue: false,
                      onChanged: (v) => setState(() => recipientPaysFee = v!),
                    )),
                Showcase(
                    key: dustChangePolicyID,
                    description: "If the change amount is below the dust limit, it can be sent to the recipient or discarded.",
                    child: FormBuilderSwitch(
                      name: "dustChangePolicy",
                      title: Text("Discard Dust Change"),
                      initialValue: true,
                      onChanged: (v) => setState(() => discardDustChange = v!),
                    )),
                Gap(16),
                Divider(),
                Gap(8),
                InputDecorator(
                  decoration: InputDecoration(
                      label: Text("Payment URI"),
                      suffixIcon: IconButton(
                        tooltip: "Show Payment URI",
                        icon: Icon(Icons.qr_code),
                        onPressed: onUriQr,
                      )),
                  child: CopyableText(puri),
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

    final srcPools = form.fields['source pools']?.value ?? 7;

    try {
      final options = PaymentOptions(
          srcPools: srcPools,
          recipientPaysFee: recipientPaysFee,
          dustChangePolicy: discardDustChange ? DustChangePolicy.discard : DustChangePolicy.sendToRecipient,
          smartTransparent: false);
      final pczt = await prepare(
        recipients: widget.recipients,
        options: options,
      );

      await GoRouter.of(navigatorKey.currentContext!).push("/tx", extra: pczt);
    } on AnyhowException catch (e) {
      if (mounted) await showException(context, e.message);
    }
  }

  void onUriQr() async {
    await showDialog(
        context: context,
        barrierDismissible: true,
        builder: (context) {
          return AlertDialog(
            title: Text("Payment URI"),
            content: GestureDetector(
                onTap: () => copyToClipboard(puri),
                child: SizedBox(
                    width: 250,
                    height: 250,
                    child: QrImageView(
                      data: puri,
                      version: QrVersions.auto,
                      backgroundColor: Colors.white,
                      size: 200.0,
                    ))),
            actions: [
              TextButton(
                onPressed: () => Navigator.of(context).pop(),
                child: Text("Close"),
              ),
            ],
          );
        });
  }
}
