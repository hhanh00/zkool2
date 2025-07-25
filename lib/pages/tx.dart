import 'package:connectivity_plus/connectivity_plus.dart';
import 'package:convert/convert.dart';
import 'package:file_picker/file_picker.dart';
import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter_mobx/flutter_mobx.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge.dart';
import 'package:go_router/go_router.dart';
import 'package:qr_flutter/qr_flutter.dart';
import 'package:showcaseview/showcaseview.dart';
import 'package:zkool/src/rust/api/mempool.dart';
import 'package:zkool/src/rust/api/pay.dart';
import 'package:zkool/src/rust/pay.dart';
import 'package:zkool/store.dart';
import 'package:zkool/utils.dart';

final cancelID = GlobalKey();
final sendID4 = GlobalKey();
final txID = GlobalKey();

class TxPage extends StatefulWidget {
  final PcztPackage pczt;
  const TxPage(this.pczt, {super.key});

  @override
  State<TxPage> createState() => TxPageState();
}

class TxPageState extends State<TxPage> {
  String? txId;
  late final TxPlan txPlan = toPlan(package: widget.pczt);
  bool canBroadcast = false;

  void tutorial() async {
    tutorialHelper(context, "tutSend3", [cancelID, sendID4]);
    if (txId != null) {
      tutorialHelper(context, "tutSend4", [txID]);
    }
  }

  @override
  void initState() {
    super.initState();
    Future(() async {
      final state = await (Connectivity().checkConnectivity());
      if (!state.contains(ConnectivityResult.none)) {
        canBroadcast = true;
      }
      setState(() {});
    });
  }

  @override
  Widget build(BuildContext context) {
    final t = Theme.of(context).textTheme;

    Future(tutorial);

    final canSend = txPlan.canSign && canBroadcast;
    final hasFrost = appStore.frostParams != null;

    return Scaffold(
      appBar: AppBar(
        title: Text("Transaction"),
        actions: [
          if (hasFrost) IconButton(onPressed: onFrost, icon: Icon(Icons.group)),
          Showcase(
              key: cancelID,
              description: "Cancel, do NOT send",
              child: IconButton(onPressed: onCancel, icon: Icon(Icons.cancel))),
          Showcase(
              key: sendID4,
              description: "Confirm, broadcast transaction",
              child: IconButton(
                  onPressed: canSend ? onSend : onSave,
                  icon: Icon(canSend
                      ? Icons.send
                      : txPlan.canSign
                          ? Icons.draw
                          : Icons.save))),
        ],
      ),
      body: CustomScrollView(slivers: [
        SliverToBoxAdapter(
            child: Padding(
          padding: EdgeInsets.symmetric(horizontal: 16),
          child: Column(children: [
            Text("Tx Plan", style: t.titleSmall),
            Text("Fee: ${zatToString(txPlan.fee)}"),
            SelectableText.rich(
              TextSpan(
                children: [
                  TextSpan(
                      text: "Payment URI: ${widget.pczt.puri} ",
                      style: t.titleSmall),
                  WidgetSpan(
                      child: IconButton(
                    tooltip: "Show Payment URI",
                    icon: Icon(Icons.qr_code),
                    onPressed: onUriQr,
                  )),
                ],
              ),
            ),
            if (txId != null)
              Showcase(
                  key: txID,
                  description: "Transaction ID",
                  child: SelectableText("Transaction ID: ${txId!}")),
          ]),
        )),
        showTxPlan(context, txPlan),
      ]),
    );
  }

  void onFrost() async {
    await GoRouter.of(context).push("/frost1", extra: widget.pczt);
  }

  void onSend() async {
    try {
      final confirmed = await confirmDialog(
        context,
        title: "Confirm Transaction",
        message: "Are you sure you want to send this transaction?",
      );
      if (!confirmed) return;
      var pczt = widget.pczt;
      if (!txPlan.canBroadcast)
        pczt = await signTransaction(
          pczt: widget.pczt,
        );

      final txBytes = await extractTransaction(package: pczt);
      final txId2 = await broadcastTransaction(
        height: txPlan.height,
        txBytes: txBytes,
      );
      showSnackbar("Transaction broadcasted successfully");
      setState(() => txId = txId2);
    } on AnyhowException catch (e) {
      if (mounted) await showException(context, e.message);
    }
  }

  void onSave() async {
    try {
      var pczt = widget.pczt;
      if (txPlan.canSign) {
        pczt = await signTransaction(
          pczt: widget.pczt,
        );
      }
      final pcztData = await packTransaction(pczt: pczt);
      final prefix = txPlan.canSign ? "signed" : "unsigned";
      await FilePicker.platform.saveFile(
        dialogTitle:
            'Please select an output file for the unsigned transaction',
        fileName: '$prefix-tx.bin',
        bytes: pcztData,
      );
    } on AnyhowException catch (e) {
      if (!mounted) return;
      await showException(context, e.message);
    }
  }

  void onCancel() {
    GoRouter.of(context).go("/account");
  }

  void onUriQr() async {
    await showDialog(
        context: context,
        barrierDismissible: true,
        builder: (context) {
          return AlertDialog(
            title: Text("Payment URI"),
            content: SizedBox(
                width: 250,
                height: 250,
                child: QrImageView(
                  data: widget.pczt.puri,
                  version: QrVersions.auto,
                  backgroundColor: Colors.white,
                  size: 200.0,
                )),
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

String poolToString(int pool) {
  switch (pool) {
    case 0:
      return "Transparent";
    case 1:
      return "Sapling";
    case 2:
      return "Orchard";
    default:
      return "Unknown";
  }
}

SliverList showTxPlan(BuildContext context, TxPlan txPlan) {
  return SliverList.builder(
      itemCount: txPlan.inputs.length + txPlan.outputs.length,
      itemBuilder: (context, index) {
        if (index < txPlan.inputs.length) {
          final input = txPlan.inputs[index];
          return ListTile(
            leading: Text("Input ${index + 1}"),
            trailing: input.amount != null ? zatToText(input.amount!) : null,
            subtitle: Text("Pool: ${poolToString(input.pool)}"),
          );
        } else {
          final index2 = index - txPlan.inputs.length;
          final output = txPlan.outputs[index2];
          return ListTile(
            leading: Text("Output ${index2 + 1}"),
            title: Text("Address: ${output.address}"),
            trailing: zatToText(output.amount),
            subtitle: Text("Pool: ${poolToString(output.pool)}"),
          );
        }
      });
}

class MempoolPage extends StatelessWidget {
  const MempoolPage({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: Text("Mempool")),
      body: Observer(builder: (context) {
        final mempool = appStore.mempoolTxIds;
        return ListView.builder(
            itemBuilder: (context, index) {
              final tx = mempool[index];
              return ListTile(
                onTap: () => onMempoolTx(context, tx.$1),
                title: SelectableText(tx.$1),
                subtitle: Text(tx.$2),
                trailing: Text(tx.$3.toString()),
              );
            },
            itemCount: mempool.length);
      }),
    );
  }

  onMempoolTx(BuildContext context, String txId) async {
    final mempoolTx = await getMempoolTx(txId: txId);
    if (!context.mounted) return;
    await GoRouter.of(context).push("/mempool_view", extra: mempoolTx);
  }
}

class MempoolTxViewPage extends StatelessWidget {
  final Uint8List rawTx;
  const MempoolTxViewPage(this.rawTx, {super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: Text("Mempool Transaction")),
      body: Padding(
          padding: EdgeInsets.all(16),
          child: SingleChildScrollView(child: SelectableText(
            hex.encode(rawTx),
          ))),
    );
  }
}
