import 'package:convert/convert.dart';
import 'package:flutter/material.dart';
import 'package:flutter_form_builder/flutter_form_builder.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge.dart';
import 'package:form_builder_validators/form_builder_validators.dart';
import 'package:gap/gap.dart';
import 'package:go_router/go_router.dart';
import 'package:zkool/src/rust/api/issuance.dart';
import 'package:zkool/src/rust/api/pay.dart';
import 'package:zkool/store.dart';
import 'package:zkool/utils.dart';
import 'package:zkool/widgets/error_display.dart';

class ZsaHoldingsPage extends ConsumerStatefulWidget {
  const ZsaHoldingsPage({super.key});

  @override
  ConsumerState<ZsaHoldingsPage> createState() => _ZsaHoldingsPageState();
}

class _ZsaHoldingsPageState extends ConsumerState<ZsaHoldingsPage> {
  @override
  Widget build(BuildContext context) {
    final tt = Theme.of(context).textTheme;

    final accountData = ref.watch(getCurrentAccountProvider);

    return accountData.when(
      loading: () => blank(context),
      error: (error, stack) => showError(error),
      data: (data) {
        final zsas = data?.zsas ?? [];

        return Scaffold(
          appBar: AppBar(
            title: const Text("ZSA Holdings"),
            leading: IconButton(
              icon: const Icon(Icons.arrow_back),
              onPressed: () => GoRouter.of(context).pop(),
            ),
            actions: [
              IconButton(
                tooltip: "Issue new token",
                icon: const Icon(Icons.add),
                onPressed: () => GoRouter.of(context).push("/zsa/issue"),
              ),
            ],
          ),
          body: CustomScrollView(
            slivers: [
              if (zsas.isEmpty)
                SliverFillRemaining(
                  child: Center(
                    child: Text("Any ZSA tokens you receive will appear here.", style: tt.bodyMedium),
                  ),
                )
              else
                SliverFixedExtentList.builder(
                  itemCount: zsas.length,
                  itemExtent: 64,
                  itemBuilder: (context, index) {
                    final h = zsas[index];

                    final displayName = h.assetName.isNotEmpty
                        ? h.assetName
                        : hex.encode(h.assetDescHash.sublist(0, 8));

                    return Column(
                      children: [
                        Expanded(
                          child: ListTile(
                            leading: CircleAvatar(
                              backgroundColor: Colors.blue,
                              child: Text(
                                initials(displayName),
                                style: tt.titleMedium?.copyWith(color: Colors.white),
                              ),
                            ),
                            title: Text(displayName),
                            subtitle: Text(hex.encode(h.assetDescHash.sublist(0, 8))),
                            trailing: Text(h.balance.toString(), style: tt.titleMedium),
                          ),
                        ),
                        const Divider(height: 1, thickness: 1, indent: 16, endIndent: 16),
                      ],
                    );
                  },
                ),
            ],
          ),
        );
      },
    );
  }
}

class IssueAssetPage extends ConsumerStatefulWidget {
  const IssueAssetPage({super.key});

  @override
  ConsumerState<IssueAssetPage> createState() => _IssueAssetPageState();
}

class _IssueAssetPageState extends ConsumerState<IssueAssetPage> {
  static final _maxSupply = BigInt.from(21000000) * BigInt.from(100000000);
  final _formKey = GlobalKey<FormBuilderState>();

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text("Issue New Token"),
        actions: [
          IconButton(
            tooltip: "Issue",
            icon: const Icon(Icons.check),
            onPressed: _issue,
          ),
        ],
      ),
      body: Padding(
        padding: const EdgeInsets.all(16),
        child: FormBuilder(
          key: _formKey,
          initialValue: const {
            "first_issuance": false,
            "finalize": false,
          },
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              // TODO: to support adding supply to an existing asset, we need
              // an optional asset_desc_hash field. The asset name alone doesn't
              // identify the asset — the desc_hash is the canonical identifier.
              FormBuilderTextField(
                name: "asset_name",
                decoration: const InputDecoration(labelText: "Asset Name"),
                validator: FormBuilderValidators.required(),
              ),
              const Gap(16),
              FormBuilderTextField(
                name: "amount",
                decoration: const InputDecoration(labelText: "Amount"),
                keyboardType: TextInputType.number,
                validator: FormBuilderValidators.compose([
                  FormBuilderValidators.required(),
                  FormBuilderValidators.integer(),
                  (v) {
                    if (v == null) return null;
                    final n = BigInt.tryParse(v);
                    if (n == null) return "Invalid number";
                    if (n <= BigInt.zero) return "Must be greater than 0";
                    if (n > _maxSupply) return "Exceeds max supply of 21 million";
                    return null;
                  },
                ]),
              ),
              const Gap(16),
              FormBuilderSwitch(
                name: "first_issuance",
                title: const Text("First Issuance"),
                subtitle: const Text("Include a zero-value reference note (ZIP-227)"),
              ),
              FormBuilderSwitch(
                name: "finalize",
                title: const Text("Finalize"),
                subtitle: const Text("Prevent any future issuance of this asset"),
              ),
            ],
          ),
        ),
      ),
    );
  }

  Future<void> _issue() async {
    final form = _formKey.currentState!;
    if (!form.saveAndValidate()) return;

    final assetName = form.value["asset_name"] as String;
    final amount = form.value["amount"] as String;
    final firstIssuance = form.value["first_issuance"] as bool;
    final finalize = form.value["finalize"] as bool;

    final confirmed = await confirmDialog(
      context,
      title: "Issue $assetName",
      message: "Issue $amount units of $assetName?${finalize ? ' This will finalize the asset.' : ''}",
    );
    if (!confirmed) return;

    try {
      final txBytes = await issueAsset(
        assetName: assetName,
        amount: BigInt.parse(amount),
        firstIssuance: firstIssuance,
        finalize: finalize,
        idAccount: coinContext.coin.account,
        c: coinContext.coin,
      );
      final height = ref.read(currentHeightProvider) ?? 1;
      final txid = await broadcastTransaction(
        height: height,
        txBytes: txBytes,
        c: coinContext.coin,
      );
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text("Transaction broadcast: $txid")),
      );
      GoRouter.of(context).pop();
    } on AnyhowException catch (e) {
      if (mounted) await showException(context, e.message);
    }
  }
}
