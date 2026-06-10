import 'dart:typed_data';

import 'package:awesome_dialog/awesome_dialog.dart';
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
import 'package:zkool/src/rust/api/zsa.dart';
import 'package:zkool/store.dart';
import 'package:zkool/utils.dart';
import 'package:zkool/widgets/error_display.dart';

class IssuanceArgs {
  final String? assetName;
  final bool isReissuance;
  final Uint8List? assetDescHash;
  const IssuanceArgs({this.assetName, this.isReissuance = false, this.assetDescHash});
}

class ZsaHoldingsPage extends ConsumerStatefulWidget {
  const ZsaHoldingsPage({super.key});

  @override
  ConsumerState<ZsaHoldingsPage> createState() => _ZsaHoldingsPageState();
}

class _ZsaHoldingsPageState extends ConsumerState<ZsaHoldingsPage> {
  int? _editingIndex;
  late final TextEditingController _nameController;
  late final FocusNode _focusNode;

  @override
  void initState() {
    super.initState();
    _nameController = TextEditingController();
    _focusNode = FocusNode();
    _focusNode.addListener(_onFocusChange);
  }

  @override
  void dispose() {
    _focusNode.removeListener(_onFocusChange);
    _nameController.dispose();
    _focusNode.dispose();
    super.dispose();
  }

  void _onFocusChange() {
    if (!_focusNode.hasFocus && _editingIndex != null) {
      _commitEditing();
    }
  }

  void _startEditing(int index, ZsaHolding holding) {
    if (_editingIndex != null && _editingIndex != index) {
      _commitEditing(); // save then switch
    }
    _editingIndex = index;
    if (holding.assetName.isNotEmpty) {
      _nameController.text = holding.assetName;
    } else {
      _nameController.clear();
    }
    _nameController.selection = TextSelection.fromPosition(
      TextPosition(offset: _nameController.text.length),
    );
    _focusNode.requestFocus();
    setState(() {});
  }

  Future<void> _commitEditing() async {
    final index = _editingIndex;
    if (index == null) return;

    final accountData = ref.read(getCurrentAccountProvider).value;
    if (accountData == null) return;

    final h = accountData.zsas[index];
    final newName = _nameController.text;

    // Close editor immediately (optimistic)
    _editingIndex = null;
    if (mounted) setState(() {});

    // Noop if unchanged
    if (newName == h.assetName) return;

    try {
      await setAssetName(
        idAsset: h.idAsset,
        name: newName,
        c: coinContext.coin,
      );
      ref.invalidate(accountProvider);
    } on AnyhowException catch (e) {
      if (mounted) {
        await showException(context, e.message);
      }
    }
  }

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

                    final displayName = h.assetName.isNotEmpty ? h.assetName : hex.encode(h.assetDescHash.sublist(0, 4));

                    final isEditing = _editingIndex == index;

                    return Column(
                      children: [
                        Expanded(
                          child: ListTile(
                            onTap: () => GoRouter.of(context).push(
                              "/zsa/issue",
                              extra: IssuanceArgs(
                                assetName: displayName,
                                isReissuance: true,
                                assetDescHash: h.assetDescHash,
                              ),
                            ),
                            leading: CircleAvatar(
                              backgroundColor: Colors.blue,
                              child: Text(
                                initials(displayName),
                                style: tt.titleMedium?.copyWith(color: Colors.white),
                              ),
                            ),
                            title: isEditing
                                ? TextField(
                                    controller: _nameController,
                                    focusNode: _focusNode,
                                    textInputAction: TextInputAction.done,
                                    onEditingComplete: _commitEditing,
                                    decoration: InputDecoration(
                                      isDense: true,
                                      contentPadding: const EdgeInsets.symmetric(vertical: 4),
                                      border: const OutlineInputBorder(),
                                      hintText: h.assetName.isEmpty ? hex.encode(h.assetDescHash.sublist(0, 4)) : null,
                                    ),
                                    style: tt.titleMedium,
                                  )
                                : GestureDetector(
                                    onLongPress: () => _startEditing(index, h),
                                    child: Text(displayName),
                                  ),
                            subtitle: Text(hex.encode(h.assetDescHash.sublist(0, 4))),
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
  final IssuanceArgs? args;
  const IssueAssetPage({super.key, this.args});

  @override
  ConsumerState<IssueAssetPage> createState() => _IssueAssetPageState();
}

class _IssueAssetPageState extends ConsumerState<IssueAssetPage> {
  static final _maxSupply = BigInt.from(21000000) * BigInt.from(100000000);
  final _formKey = GlobalKey<FormBuilderState>();

  bool get _isReissuance => widget.args?.isReissuance == true;
  String? get _prefilledName => widget.args?.assetName;
  Uint8List? get _descHash => widget.args?.assetDescHash;

  @override
  Widget build(BuildContext context) {
    final reissuance = _isReissuance;
    final name = _prefilledName ?? "";

    return Scaffold(
      appBar: AppBar(
        title: Text(reissuance ? "Issue More $name" : "Issue New Token"),
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
          initialValue: {
            "asset_name": name,
            "first_issuance": reissuance ? false : false,
            "finalize": false,
          },
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              FormBuilderTextField(
                name: "asset_name",
                decoration: const InputDecoration(labelText: "Asset Name"),
                enabled: !reissuance,
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
                enabled: !reissuance,
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
    final firstIssuance = _isReissuance ? false : form.value["first_issuance"] as bool;
    final finalize = form.value["finalize"] as bool;

    final label = _isReissuance ? "more " : "";
    final confirmed = await confirmDialog(
      context,
      title: "Issue $assetName",
      message: "Issue $amount ${label}units of $assetName?${finalize ? ' This will finalize the asset.' : ''}",
    );
    if (!confirmed) return;

    AwesomeDialog? dialog;
    try {
      dialog = await showMessage(
        context,
        "Building and broadcasting issuance transaction...",
        dismissable: false,
      );

      final txBytes = await issueAsset(
        assetName: assetName,
        amount: BigInt.parse(amount),
        firstIssuance: firstIssuance,
        finalize: finalize,
        descHash: _descHash,
        idAccount: coinContext.coin.account,
        c: coinContext.coin,
      );
      final height = ref.read(currentHeightProvider) ?? 1;
      final txid = await broadcastTransaction(
        height: height,
        txBytes: txBytes,
        c: coinContext.coin,
      );

      dialog.dismiss();
      dialog = null;

      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text("Transaction broadcast: $txid")),
      );
      GoRouter.of(context).pop();
    } on AnyhowException catch (e) {
      dialog?.dismiss();
      if (mounted) await showException(context, e.message);
    }
  }
}
