import 'package:flutter/material.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge.dart';
import 'package:go_router/go_router.dart';
import 'package:showcaseview/showcaseview.dart';
import 'package:zkool/src/rust/api/pay.dart';
import 'package:zkool/src/rust/pay.dart';
import 'package:zkool/utils.dart';

final changeID = GlobalKey();
final changePoolID = GlobalKey();
final cancelID = GlobalKey();
final sendID4 = GlobalKey();
final txID = GlobalKey();

class TxPage extends StatefulWidget {
  final TxPlan txPlan;
  const TxPage(this.txPlan, {super.key});

  @override
  State<TxPage> createState() => TxPageState();
}

class TxPageState extends State<TxPage> {
  String? txId;

  void tutorial() async {
    tutorialHelper(context, "tutSend3", [changeID, changePoolID, cancelID, sendID4]);
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
            Text("Fee: ${zatToString(widget.txPlan.fee)}"),
            Showcase(key: changeID, description: "Amount of change returned to your wallet", child:
            Text("Change: ${zatToString(widget.txPlan.change)}")),
            Showcase(key: changePoolID, description: "Pool to which the change goes to", child:
            Text("Change Pool: ${poolToString(widget.txPlan.changePool)}")),
            if (txId != null)
            Showcase(key: txID, description: "Transaction ID", child:
              SelectableText("Transaction ID: ${txId!}")),
          ]),
        ),
        showTxPlan(context, widget.txPlan),
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
      final txId2 = await send(
        height: widget.txPlan.height,
        data: widget.txPlan.data,
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
            trailing: Text("Value: ${zatToString(input.amount)}"),
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
