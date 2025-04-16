import 'package:flutter/material.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge.dart';
import 'package:zkool/src/rust/api/pay.dart';
import 'package:zkool/src/rust/pay.dart';
import 'package:zkool/utils.dart';

class TxPage extends StatefulWidget {
  final TxPlan txPlan;
  const TxPage(this.txPlan, {super.key});

  @override
  State<TxPage> createState() => TxPageState();
}

class TxPageState extends State<TxPage> {
  String? txId;

  @override
  Widget build(BuildContext context) {
    final t = Theme.of(context).textTheme;

    return Scaffold(
      appBar: AppBar(
        title: Text("Transaction"),
        actions: [
          IconButton(onPressed: onSend, icon: Icon(Icons.send)),
        ],
      ),
      body: CustomScrollView(slivers: [
        SliverToBoxAdapter(
          child: Column(children: [
            Text("Tx Plan", style: t.titleSmall),
            Text("Fee: ${zatToString(widget.txPlan.fee)}"),
            Text("Change: ${zatToString(widget.txPlan.change)}"),
            Text("Change Pool: ${widget.txPlan.changePool}"),
            if (txId != null) SelectableText("Transaction ID: ${txId!}"),
          ]),
        ),
        showTxPlan(context, widget.txPlan),
      ]),
    );
  }

  void onSend() async {
    try {
      final txId2 = await send(
        height: widget.txPlan.height,
        data: widget.txPlan.data,
      );
      setState(() => txId = txId2);
    } on AnyhowException catch (e) {
      if (mounted) await showException(context, e.message);
    }
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
            subtitle: Text("Pool: ${input.pool}"),
          );
        } else {
          final index2 = index - txPlan.inputs.length;
          final output = txPlan.outputs[index2];
          return ListTile(
            leading: Text("Output ${index2 + 1}"),
            title: Text("Address: ${output.address}"),
            trailing: Text("Value: ${zatToString(output.amount)}"),
            subtitle: Text("Pool: ${output.pool}"),
          );
        }
      });
}
