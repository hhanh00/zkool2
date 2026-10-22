import 'dart:async';

import 'package:convert/convert.dart';
import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge.dart';
import 'package:gap/gap.dart';
import 'package:go_router/go_router.dart';

import 'package:zkool/main.dart';
import 'package:zkool/services/continuation_context.dart';
import 'package:zkool/pages/swaps.dart';
import 'package:zkool/pages/raptor.dart';
import 'package:zkool/src/rust/api/account.dart';
import 'package:zkool/src/rust/api/frost.dart';
import 'package:zkool/src/rust/api/mempool.dart';
import 'package:zkool/src/rust/api/pay.dart';
import 'package:zkool/src/rust/pay.dart';
import 'package:zkool/store.dart';
import 'package:zkool/utils.dart';
import 'package:zkool/widgets/error_display.dart';
import 'package:zkool/router.dart';

class TxPage extends ConsumerStatefulWidget {
  final PcztPackage pczt;
  final ContinuationContext? continuation;
  final int account;
  const TxPage(this.pczt, {this.continuation, required this.account, super.key});

  @override
  ConsumerState<TxPage> createState() => TxPageState();
}

class TxPageState extends ConsumerState<TxPage> {
  late final c = coinContext.coin;
  String? txId;
  late final TxPlan txPlan = toPlan(package: widget.pczt, c: c);
  bool canBroadcast = false;
  Account? account;
  bool _sending = false;
  String _sendStep = "";

  @override
  void initState() {
    super.initState();
    Future(() async {
      final settings = ref.read(appSettingsProvider).requireValue;
      final canBroadcast = !settings.offline;
      final accounts = await ref.read(getAccountsProvider.future);
      final account = accounts.firstWhere((a) => a.id == widget.account);
      setState(() {
        this.account = account;
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
    final hasFrost = account!.frost;

    final success = txId != null;

    return Scaffold(
      appBar: AppBar(
        title: Text(success ? "Transaction Sent" : "Transaction"),
        actions: [
          if (!success) ...[
            IconButton(onPressed: onCancel, tooltip: "Cancel, do NOT send", icon: Icon(Icons.cancel)),
            // A group account has no spending key, so the planner never marks
            // its transaction signable; the loader always does. A loaded one
            // is the coordinator's, to sign with the group. A planned one is
            // saved and handed to the other signers first, as before.
            if (hasFrost && txPlan.canSign)
              IconButton(
                onPressed: onFrost,
                tooltip: "Sign with the group and send",
                icon: Icon(Icons.send),
              )
            else
              IconButton(
                onPressed: _sending ? null : (canSend ? onSend : onSave),
                // One button, three jobs: name the one it has now.
                tooltip: canSend
                    ? "Confirm, broadcast transaction"
                    : txPlan.canSign
                        ? "Sign and save the transaction"
                        : hasFrost
                            ? "Save the transaction for the group signers"
                            : "Save the unsigned transaction",
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
                                    Text(
                                      _sendStep,
                                      style: t.bodyMedium?.copyWith(
                                        fontWeight: FontWeight.w600,
                                      ),
                                    ),
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
                        if (success) Text("Tx Sent Successfully", style: t.titleSmall),
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
                if (!success) showTxPlan(context, txPlan),
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

  Future<void> onFrost() async {
    // A file made for another account would start a session that can never
    // sign: refuse it here, where the message makes sense.
    try {
      await checkGroupPczt(account: widget.account, pczt: widget.pczt, c: c);
    } on AnyhowException catch (e) {
      if (mounted) await showException(context, e.message);
      return;
    }
    if (!mounted) return;
    await GoRouter.of(context).push(accountRoute("/frost1", widget.account), extra: widget.pczt);
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
        // The signer (wallet keys or the account's Ledger app) is chosen on
        // the Rust side; a Ledger app reports its prompts as progress.
        final comp = Completer<PcztPackage>();
        signTransactionWithProgress(package: widget.pczt, account: account!.id, c: c).listen(
          (e) {
            switch (e) {
              case SigningEvent_Progress p:
                showSnackbar(p.field0);
              case SigningEvent_Result r:
                comp.complete(r.field0);
            }
          },
          onError: (e) {
            final exc = e as AnyhowException;
            comp.completeError(exc);
          },
        );
        pczt = await comp.future;
      }

      setState(() => _sendStep = "Broadcasting...");
      // Extracts, posts, locks the spent notes and records the price and
      // category; a rejection by the server throws.
      final result = await broadcast(package: pczt, account: account!.id, c: c);
      logger.i("tx result $result");

      {
        if (mounted) setState(() => txId = result);
        switch (widget.continuation) {
          case SwapContext continuation:
            final summary = await continuation.complete(result, account!.id, c);
            if (mounted) {
              ref.invalidate(savedSwapsProvider);
              unawaited(GoRouter.of(context).pushReplacement<void>(accountRoute('/swap/summary', widget.account), extra: summary));
            }
          case null:
            break;
        }
        if (mounted) {
          showSnackbar("Transaction broadcasted successfully");
        } else {
          showSnackbar("Transaction sent successfully (background)");
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
    if (_sending || txId != null) return;
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
          account: account!.id,
          c: c,
        );
      }
      final pcztData = await packTransaction(pczt: pczt);
      final prefix = txPlan.canSign ? "signed" : "unsigned";
      final path = await saveFile(
        title: "Please select an output file for the $prefix transaction",
        fileName: "$prefix-tx.bin",
        data: pcztData,
      );
      final appSettings = await ref.read(appSettingsProvider.future);
      if (mounted && path != null && appSettings.qrSettings.enabled) await showAnimatedQR(context, ref, path);
      // A group's planned transaction is saved for the other signers; the
      // one who planned it is often the coordinator, and loading the file
      // back just to start is a detour.
      if (mounted && path != null && account!.frost && !txPlan.canSign) {
        final start = await confirmDialog(
          context,
          title: "Coordinate the signing?",
          message: "The transaction is saved. Share the file with the other signers. "
              "Start the signing session now, as the coordinator?",
          btnOkText: "Start",
          btnCancelText: "Not now",
        );
        if (start && mounted) await onFrost();
      }
    } on AnyhowException catch (e) {
      if (!mounted) return;
      await showException(context, e.message);
    }
  }

  void onCancel() {
    GoRouter.of(context).go(accountRoute("/", widget.account));
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
    case 3:
      return "Ironwood";
    default:
      return "Unknown";
  }
}

SliverList showTxPlan(BuildContext context, TxPlan txPlan) {
  final hasInputs = txPlan.inputs.isNotEmpty;
  final hasOutputs = txPlan.outputs.isNotEmpty;
  final separatorCount = (hasInputs && hasOutputs) ? 1 : 0;
  return SliverList.builder(
    itemCount: txPlan.inputs.length + txPlan.outputs.length + separatorCount,
    itemBuilder: (context, index) {
      if (hasInputs && index < txPlan.inputs.length) {
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
      } else if (separatorCount == 1 && index == txPlan.inputs.length) {
        return const Divider(height: 24, thickness: 1, indent: 16, endIndent: 16);
      } else {
        final outputIndex = index - txPlan.inputs.length - separatorCount;
        final output = txPlan.outputs[outputIndex];
        final isZsa = output.assetName != "ZEC";
        return ListTile(
          leading: Text("Output ${outputIndex + 1}"),
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
