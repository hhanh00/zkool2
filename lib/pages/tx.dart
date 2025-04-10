import 'package:flutter/material.dart';
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
    return Scaffold(
      appBar: AppBar(
        title: Text("Transaction"),
        actions: [
          IconButton(onPressed: onSend, icon: Icon(Icons.send)),
        ],
      ),
      body: SingleChildScrollView(
        child: Padding(
          padding: EdgeInsets.symmetric(horizontal: 8),
          child: Column(children: [
            if (txId != null) SelectableText("Transaction ID: ${txId!}"),
            ...showTxPlan(context, widget.txPlan),
          ]),
        ),
      ),
    );
  }

  void onSend() async {
    final _txId = await send(
      height: widget.txPlan.height,
      data: widget.txPlan.data,
    );
    print("Transaction sent: $_txId");
    setState(() => txId = _txId);
  }
}

List<Widget> showTxPlan(BuildContext context, TxPlan txPlan) {
  final t = Theme.of(context).textTheme;

  final inouts = ListView.builder(
      shrinkWrap: true,
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

  return [
    Text("Tx Plan", style: t.titleSmall),
    Text("Fee: ${zatToString(txPlan.fee)}"),
    Text("Change: ${zatToString(txPlan.change)}"),
    Text("Change Pool: ${txPlan.changePool}"),
    inouts
  ];
}

