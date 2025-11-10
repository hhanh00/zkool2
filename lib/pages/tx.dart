import 'dart:async';
import 'dart:convert';

import 'package:convert/convert.dart';
import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge.dart';
import 'package:gap/gap.dart';
import 'package:go_router/go_router.dart';
import 'package:showcaseview/showcaseview.dart';
import 'package:zkool/main.dart';
import 'package:zkool/src/rust/api/account.dart';
import 'package:zkool/src/rust/api/mempool.dart';
import 'package:zkool/src/rust/api/pay.dart';
import 'package:zkool/src/rust/pay.dart';
import 'package:zkool/store.dart';
import 'package:zkool/utils.dart';

final cancelID = GlobalKey();
final sendID4 = GlobalKey();
final txID = GlobalKey();

class TxPage extends ConsumerStatefulWidget {
  final PcztPackage pczt;
  const TxPage(this.pczt, {super.key});

  @override
  ConsumerState<TxPage> createState() => TxPageState();
}

class TxPageState extends ConsumerState<TxPage> {
  String? txId;
  late final TxPlan txPlan = toPlan(package: widget.pczt);
  late bool canBroadcast;
  late Account account;
  late AccountData details;

  @override
  void initState() {
    super.initState();
    final settings = ref.read(appSettingsProvider);
    canBroadcast = !settings.offline;
    account = ref.read(selectedAccountProvider)!;
    details = ref.read(accountProvider(account.id)).requireValue;
  }

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


    final canSend = (txPlan.canSign || account.hw != 0) && canBroadcast;
    final hasFrost = details.frostParams != null;

    return Scaffold(
      appBar: AppBar(
        title: Text("Transaction"),
        actions: [
          if (hasFrost) IconButton(onPressed: onFrost, icon: Icon(Icons.group)),
          Showcase(
            key: cancelID,
            description: "Cancel, do NOT send",
            child: IconButton(onPressed: onCancel, icon: Icon(Icons.cancel)),
          ),
          Showcase(
            key: sendID4,
            description: "Confirm, broadcast transaction",
            child: IconButton(
              onPressed: canSend ? onSend : onSave,
              icon: Icon(
                canSend
                    ? Icons.send
                    : txPlan.canSign
                        ? Icons.draw
                        : Icons.save,
              ),
            ),
          ),
        ],
      ),
      body: CustomScrollView(
        slivers: [
          SliverToBoxAdapter(
            child: Padding(
              padding: EdgeInsets.symmetric(horizontal: 16),
              child: Column(
                children: [
                  Text("Tx Plan", style: t.titleSmall),
                  Text("Fee: ${zatToString(txPlan.fee)}"),
                  Gap(8),
                  if (txId != null)
                    Showcase(
                      key: txID,
                      description: "Transaction ID",
                      child: CopyableText("Transaction ID: ${txId!}"),
                    ),
                ],
              ),
            ),
          ),
          showTxPlan(context, txPlan),
        ],
      ),
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
      if (!txPlan.canBroadcast) {
        if (account.hw != 0) {
          final c = Completer();
          signLedgerTransaction(pczt: widget.pczt).listen((e) {
            switch (e) {
              case SigningEvent_Progress p:
                showSnackbar(p.field0);
              case SigningEvent_Result r:
                pczt = r.field0;
                c.complete();
            }
          });
          await c.future;
        } else {
          pczt = await signTransaction(
            pczt: widget.pczt,
          );
        }
      }

      final txBytes = await extractTransaction(package: pczt);
      final result = await broadcastTransaction(
        height: txPlan.height,
        txBytes: txBytes,
      );
      try {
        final txid = jsonDecode(result) as String;
        final txidHex = hex.decode(txid);
        await storePendingTx(
          height: txPlan.height,
          txid: txidHex,
          price: pczt.price,
          category: pczt.category,
        );
        await showMessage(context, txid);
        showSnackbar("Transaction broadcasted successfully");
      } catch (_) {
        if (mounted) await showException(context, result);
      }
      if (mounted) setState(() => txId = result);
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
      await appWatcher.saveFile(
        title: "Please select an output file for the unsigned transaction",
        fileName: "$prefix-tx.bin",
        data: pcztData,
      );
    } on AnyhowException catch (e) {
      if (!mounted) return;
      await showException(context, e.message);
    }
  }

  void onCancel() {
    GoRouter.of(context).go("/account");
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
          trailing: input.amount != null ? zatToText(input.amount!, selectable: true) : null,
          subtitle: Text("Pool: ${poolToString(input.pool)}"),
        );
      } else {
        final index2 = index - txPlan.inputs.length;
        final output = txPlan.outputs[index2];
        return ListTile(
          leading: Text("Output ${index2 + 1}"),
          title: Text("Address: ${output.address}"),
          trailing: zatToText(output.amount, selectable: true),
          subtitle: Text("Pool: ${poolToString(output.pool)}"),
        );
      }
    },
  );
}

class MempoolPage extends ConsumerWidget {
  const MempoolPage({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return Scaffold(
      appBar: AppBar(title: Text("Mempool")),
      body: Builder(
        builder: (context) {
          final mempool = ref.watch(mempoolProvider);
          return ListView.builder(
            itemBuilder: (context, index) {
              final tx = mempool.unconfirmedTx[index];
              return ListTile(
                onTap: () => onMempoolTx(context, tx.$1),
                title: CopyableText(tx.$1),
                subtitle: Text(tx.$2),
                trailing: Text(tx.$3.toString()),
              );
            },
            itemCount: mempool.unconfirmedTx.length,
          );
        },
      ),
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
        child: SingleChildScrollView(
          child: CopyableText(
            hex.encode(rawTx),
          ),
        ),
      ),
    );
  }
}
