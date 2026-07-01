import 'dart:async';

import 'package:convert/convert.dart';
import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge.dart';
import 'package:gap/gap.dart';
import 'package:go_router/go_router.dart';

import 'package:zkool/main.dart';
import 'package:zkool/pages/raptor.dart';
import 'package:zkool/src/rust/api/account.dart';
import 'package:zkool/src/rust/api/mempool.dart';
import 'package:zkool/src/rust/api/pay.dart';
import 'package:zkool/src/rust/pay.dart';
import 'package:zkool/store.dart';
import 'package:zkool/utils.dart';
import 'package:zkool/widgets/error_display.dart';

class TxPage extends ConsumerStatefulWidget {
  final PcztPackage pczt;
  const TxPage(this.pczt, {super.key});

  @override
  ConsumerState<TxPage> createState() => TxPageState();
}

class TxPageState extends ConsumerState<TxPage> {
  late final c = coinContext.coin;
  String? txId;
  late final TxPlan txPlan = toPlan(package: widget.pczt, c: c);
  bool canBroadcast = false;
  Account? account;
  AccountData? details;
  bool _sending = false;
  String _sendStep = "";

  @override
  void initState() {
    super.initState();
    Future(() async {
      final settings = ref.read(appSettingsProvider).requireValue;
      final canBroadcast = !settings.offline;
      final account = await ref.read(selectedAccountProvider.future);
      final details = await ref.read(accountProvider(account!.id).future);
      setState(() {
        this.account = account;
        this.details = details;
        this.canBroadcast = canBroadcast;
      });
    });
  }

  @override
  Widget build(BuildContext context) {
    final pinlock = ref.watch(lifecycleProvider);
    if (pinlock.value ?? false) return PinLock();

    if (account == null) return blank(context);
    final t = Theme.of(context).textTheme;

    final canSend = (txPlan.canSign || account!.hw != 0) && canBroadcast;
    final hasFrost = details!.frostParams != null;

    final success = txId != null;

    return Scaffold(
      appBar: AppBar(
        title: Text(success ? "Transaction Sent" : "Transaction"),
        actions: [
          if (!success) ...[
            if (hasFrost) IconButton(onPressed: onFrost, icon: Icon(Icons.group)),
            IconButton(onPressed: onCancel, tooltip: "Cancel, do NOT send", icon: Icon(Icons.cancel)),
            IconButton(
              onPressed: _sending ? null : (canSend ? onSend : onSave),
              tooltip: "Confirm, broadcast transaction",
              icon: Icon(
                canSend
                    ? Icons.send
                    : txPlan.canSign
                        ? Icons.draw
                        : Icons.save,
              ),
            ),
          ],
        ],
      ),
      body: Column(
        children: [
          if (_sending) const LinearProgressIndicator(),
          Expanded(
            child: CustomScrollView(
              slivers: [
                SliverToBoxAdapter(
                  child: Padding(
                    padding: EdgeInsets.symmetric(horizontal: 16),
                    child: Column(
                      children: [
                        if (_sending) ...[
                          SizedBox(height: 16),
                          Row(
                            children: [
                              SizedBox(
                                width: 20,
                                height: 20,
                                child: CircularProgressIndicator(strokeWidth: 2),
                              ),
                              Gap(12),
                              Expanded(
                                child: Column(
                                  crossAxisAlignment: CrossAxisAlignment.start,
                                  children: [
                                    Text(_sendStep,
                                        style: t.bodyMedium
                                            ?.copyWith(fontWeight: FontWeight.w600,),),
                                    Text(
                                      "Tx will be sent in the background if this page is closed",
                                      style: t.bodySmall,
                                    ),
                                  ],
                                ),
                              ),
                            ],
                          ),
                          Divider(height: 24),
                        ],
                        if (success)
                          Text("Tx Sent Successfully", style: t.titleSmall),
                        Text("Fee: ${zatToString(txPlan.fee)}"),
                        Gap(8),
                        if (txId != null)
                          Tooltip(
                            message: "Transaction ID",
                            child: CopyableText("Transaction ID: ${txId!}"),
                          ),
                      ],
                    ),
                  ),
                ),
                if (!success)
                  showTxPlan(context, txPlan),
              ],
            ),
          ),
        ],
      ),
      bottomNavigationBar: success
          ? SafeArea(
              child: Padding(
                padding: const EdgeInsets.all(16),
                child: SizedBox(
                  width: double.infinity,
                  child: ElevatedButton(
                    onPressed: onCancel,
                    child: const Text("Dismiss"),
                  ),
                ),
              ),
            )
          : null,
    );
  }

  void onFrost() async {
    await GoRouter.of(context).push("/frost1", extra: widget.pczt);
  }

  Future<void> _confirmAndSend() async {
    try {
      final confirmed = await confirmDialog(
        context,
        title: "Confirm Transaction",
        message: "Are you sure you want to send this transaction?",
      );
      if (!confirmed) return;

      setState(() => _sendStep = "Signing transaction...");
      var pczt = widget.pczt;
      if (!txPlan.canBroadcast) {
        if (account!.hw != 0) {
          final comp = Completer();
          signLedgerTransaction(package: widget.pczt, c: c).listen(
            (e) {
              switch (e) {
                case SigningEvent_Progress p:
                  showSnackbar(p.field0);
                case SigningEvent_Result r:
                  pczt = r.field0;
                  comp.complete();
              }
            },
            onError: (e) {
              final exc = e as AnyhowException;
              comp.completeError(exc);
            },
          );
          await comp.future;
        } else {
          pczt = await signTransaction(
            pczt: widget.pczt,
            c: c,
          );
        }
      }

      setState(() => _sendStep = "Extracting transaction...");
      final txBytes = await extractTransaction(package: pczt);

      setState(() => _sendStep = "Broadcasting...");
      final result = await broadcastTransaction(
        height: txPlan.height,
        txBytes: txBytes,
        c: c,
      );
      logger.i("tx result $result");

      Uint8List? txidHex;
      try {
        txidHex = Uint8List.fromList(hex.decode(result));
      } on FormatException {
        // result is not a valid hex txid — broadcast returned an error message
      }

      if (txidHex != null) {
        await storePendingTx(
          height: txPlan.height,
          txid: txidHex,
          price: pczt.price,
          category: pczt.category,
          c: c,
        );
        if (mounted) {
          setState(() => txId = result);
          showSnackbar("Transaction broadcasted successfully");
        } else {
          showSnackbar("Transaction sent successfully (background)");
        }
      } else {
        if (mounted) {
          setState(() => _sendStep = "");
          await showException(context, result);
        }
      }
      if (mounted) setState(() => _sendStep = "");
    } on AnyhowException catch (e) {
      if (mounted) {
        setState(() => _sendStep = "");
        await showException(context, e.message);
      } else {
        showSnackbar("Send failed: ${e.message}");
      }
    }
  }

  void onSend() async {
    setState(() {
      _sending = true;
      _sendStep = "Preparing...";
    });
    try {
      await _confirmAndSend();
    } on AnyhowException catch (e) {
      if (mounted) {
        await showException(context, e.message);
      } else {
        showSnackbar("Send failed: ${e.message}");
      }
    } finally {
      if (mounted) {
        setState(() {
          _sending = false;
          _sendStep = "";
        });
      }
    }
  }

  void onSave() async {
    try {
      var pczt = widget.pczt;
      if (txPlan.canSign) {
        pczt = await signTransaction(
          pczt: widget.pczt,
          c: c,
        );
      }
      final pcztData = await packTransaction(pczt: pczt);
      final prefix = txPlan.canSign ? "signed" : "unsigned";
      final path = await saveFile(
        title: "Please select an output file for the unsigned transaction",
        fileName: "$prefix-tx.bin",
        data: pcztData,
      );
      final appSettings = await ref.read(appSettingsProvider.future);
      if (path != null && appSettings.qrSettings.enabled) await showAnimatedQR(context, ref, path);
    } on AnyhowException catch (e) {
      if (!mounted) return;
      await showException(context, e.message);
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
        final isZsa = input.assetName != "ZEC";
        return ListTile(
          leading: Text("Input ${index + 1}"),
          trailing: input.amount != null
              ? (isZsa
                  ? Text(input.amount.toString(), style: TextStyle(color: Colors.purple, fontWeight: FontWeight.bold))
                  : zatToText(input.amount!, selectable: true))
              : null,
          subtitle: Text([
            "Pool: ${poolToString(input.pool)}",
            if (isZsa) input.assetName,
          ].join(" · ")),
        );
      } else {
        final index2 = index - txPlan.inputs.length;
        final output = txPlan.outputs[index2];
        final isZsa = output.assetName != "ZEC";
        return ListTile(
          leading: Text("Output ${index2 + 1}"),
          title: Text("Address: ${output.address}"),
          trailing: isZsa
              ? Text(output.amount.toString(), style: TextStyle(color: Colors.purple, fontWeight: FontWeight.bold))
              : zatToText(output.amount, selectable: true),
          subtitle: Text([
            "Pool: ${poolToString(output.pool)}",
            if (isZsa) output.assetName,
          ].join(" · ")),
        );
      }
    },
  );
}

class MempoolPage extends ConsumerWidget {
  const MempoolPage({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final pinlock = ref.watch(lifecycleProvider);
    if (pinlock.value ?? false) return PinLock();

    return Scaffold(
      appBar: AppBar(title: Text("Mempool")),
      body: Builder(
        builder: (context) {
          final mempool = ref.watch(mempoolProvider);
          return ListView.builder(
            itemBuilder: (context, index) {
              final tx = mempool.unconfirmedTx[index];
              return ListTile(
                onTap: () => onMempoolTx(context, ref, tx.$1),
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

  onMempoolTx(BuildContext context, WidgetRef ref, String txId) async {
    final c = coinContext.coin;
    final mempoolTx = await getMempoolTx(txId: txId, c: c);
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
