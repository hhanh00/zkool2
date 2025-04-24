import 'package:flutter/material.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge.dart';
import 'package:go_router/go_router.dart';
import 'package:showcaseview/showcaseview.dart';
import 'package:zkool/src/rust/api/pay.dart';
import 'package:zkool/src/rust/pay.dart';
import 'package:zkool/src/rust/pay/plan.dart';
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

  void tutorial() async {
    tutorialHelper(context, "tutSend3", [cancelID, sendID4]);
    if (txId != null) {
      tutorialHelper(context, "tutSend4", [txID]);
    }
  }

  @override
  Widget build(BuildContext context) {
    final t = Theme.of(context).textTheme;

    Future(tutorial);

    return Scaffold(
      appBar: AppBar(
        title: Text("Transaction"),
        actions: [
          Showcase(key: cancelID, description: "Cancel, do NOT send", child:
          IconButton(onPressed: onCancel, icon: Icon(Icons.cancel))),
          Showcase(key: sendID4, description: "Confirm, broadcast transaction", child:
          IconButton(onPressed: onSend, icon: Icon(Icons.send))),
        ],
      ),
      body: CustomScrollView(slivers: [
        SliverToBoxAdapter(
          child: Column(children: [
            Text("Tx Plan", style: t.titleSmall),
            Text("Fee: ${zatToString(txPlan.fee)}"),
            if (txId != null)
            Showcase(key: txID, description: "Transaction ID", child:
              SelectableText("Transaction ID: ${txId!}")),
          ]),
        ),
        showTxPlan(context, txPlan),
      ]),
    );
  }

  void onSend() async {
    try {
      final confirmed = await confirmDialog(
        context,
        title: "Confirm Transaction",
        message: "Are you sure you want to send this transaction?",
      );
      if (!confirmed) return;
      final txBytes = await signTransaction(
        pczt: widget.pczt,
      );
      final txId2 = await broadcastTransaction(
        height: txPlan.height,
        txBytes: txBytes,
      );
      setState(() => txId = txId2);
    } on AnyhowException catch (e) {
      if (mounted) await showException(context, e.message);
    }
  }

  void onCancel() {
    GoRouter.of(context).go("/");
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
            trailing: input.amount != null ? Text("Value: ${zatToString(input.amount!)}") : null,
            subtitle: Text("Pool: ${poolToString(input.pool)}"),
          );
        } else {
          final index2 = index - txPlan.inputs.length;
          final output = txPlan.outputs[index2];
          return ListTile(
            leading: Text("Output ${index2 + 1}"),
            title: Text("Address: ${output.address}"),
            trailing: Text("Value: ${zatToString(output.amount)}"),
            subtitle: Text("Pool: ${poolToString(output.pool)}"),
          );
        }
      });
}
