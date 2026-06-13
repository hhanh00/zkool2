import 'dart:async';

import 'package:awesome_dialog/awesome_dialog.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_form_builder/flutter_form_builder.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated.dart';
import 'package:gap/gap.dart';
import 'package:go_router/go_router.dart';

import 'package:zkool/main.dart';
import 'package:zkool/pages/sweep.dart';
import 'package:zkool/src/rust/api/account.dart';
import 'package:zkool/src/rust/api/key.dart';
import 'package:zkool/src/rust/api/sync.dart';
import 'package:zkool/store.dart';
import 'package:zkool/utils.dart';
import 'package:zkool/widgets/error_display.dart';
import 'package:zkool/validators.dart';
import 'package:zkool/widgets/pool_select.dart';

class NewAccountPage extends ConsumerStatefulWidget {
  const NewAccountPage({super.key});

  @override
  ConsumerState<NewAccountPage> createState() => NewAccountPageState();
}

class NewAccountPageState extends ConsumerState<NewAccountPage> {
  late var c = coinContext.coin;
  var name = "";
  var restore = false;
  var key = "";
  var isSeed = false;
  var ledger = false;
  var isFvk = false;
  var _showAdvanced = false;
  Uint8List? iconBytes;
  final formKey = GlobalKey<FormBuilderState>();

  @override
  Widget build(BuildContext context) {
    final pinlock = ref.watch(lifecycleProvider);
    if (pinlock.value ?? false) return PinLock();

    final ib = iconBytes;
    isSeed = isValidPhrase(phrase: key);
    isFvk = isValidFvk(fvk: key, c: c);
    final keyPools = ledger ? 3 : getKeyPools(key: key, c: c); // 3 is T+S

    return Scaffold(
      appBar: AppBar(
        title: const Text("New Account"),
        actions: [
          IconButton(onPressed: onFrost, tooltip: "Start Distributed Key Generation", icon: Icon(Icons.group)),
          IconButton(
            onPressed: onImport,
            tooltip: "Import an account from file",
            icon: Icon(Icons.file_open),
          ),
          IconButton(
            icon: const Icon(Icons.save),
            tooltip: "Save",
            onPressed: onSave,
          ),
        ],
      ),
      body: Padding(
        padding: const EdgeInsets.symmetric(horizontal: 12),
        child: SingleChildScrollView(
          child: FormBuilder(
            key: formKey,
            child: Column(
              children: [
                Gap(12),
                Text(
                  "Create a new wallet or restore an existing one",
                  textAlign: TextAlign.center,
                  style: Theme.of(context).textTheme.bodyMedium?.copyWith(
                        color: Theme.of(context).colorScheme.onSurface.withAlpha(150),
                      ),
                ),
                Gap(20),
                Card(
                  child: Padding(
                    padding: const EdgeInsets.all(20),
                    child: Column(
                      children: [
                        Stack(
                          children: [
                            Tooltip(
                              message: "Upload a icon",
                              child: CircleAvatar(
                                radius: 60,
                                backgroundImage: ib != null ? Image.memory(ib).image : null,
                                child: ib == null ? Text(initials(name)) : null,
                              ),
                            ),
                            Positioned(
                              right: 0,
                              bottom: 0,
                              child: IconButton.filled(
                                onPressed: onEdit,
                                icon: Icon(Icons.edit),
                              ),
                            ),
                          ],
                        ),
                        Gap(16),
                        Tooltip(
                          message: "Enter a name that identifies this account",
                          child: FormBuilderTextField(
                            name: "name",
                            decoration: const InputDecoration(
                              labelText: "Account Name",
                              prefixIcon: Icon(Icons.person_outline),
                            ),
                            initialValue: name,
                            onChanged: (v) => setState(() => name = v!),
                          ),
                        ),
                      ],
                    ),
                  ),
                ),
                Gap(12),
                Card(
                  child: Padding(
                    padding: const EdgeInsets.all(20),
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Tooltip(
                          message: "Check if you want to restore an existing account",
                          child: FormBuilderSwitch(
                            name: "restore",
                            title: Row(
                              children: [
                                SizedBox(width: 38, child: Icon(Icons.restore, size: 20)),
                                Gap(8),
                                const Text("Restore Account?"),
                              ],
                            ),
                            initialValue: restore,
                            onChanged: (v) => setState(() => restore = v ?? false),
                          ),
                        ),
                        if (restore) ...[
                          Gap(8),
                          Text(
                            "Enter your seed phrase, private key, or viewing key to restore",
                            style: Theme.of(context).textTheme.bodySmall?.copyWith(
                                  color: Theme.of(context).colorScheme.onSurface.withAlpha(120),
                                ),
                          ),
                          Gap(12),
                          Row(
                            children: [
                              Expanded(
                                child: Tooltip(
                                  message:
                                      "Seed phrase (12, 18, 21, 24 words), a Sapling secret key, a viewing key, a unified viewing key, a xpub/xprv transparent key or a BIP-38 key (starting with K or L)",
                                  child: FormBuilderTextField(
                                    name: "key",
                                    decoration: const InputDecoration(
                                      labelText: "Key (Seed Phrase, Private Key, or Viewing Key)",
                                      prefixIcon: Icon(Icons.key),
                                    ),
                                    validator: (s) => validKey(s, restore: restore && !ledger, c: c),
                                    initialValue: key,
                                    onChanged: (v) => setState(() => key = v!),
                                  ),
                                ),
                              ),
                              Gap(8),
                              Tooltip(
                                message: "Generate a new Seed Phrase",
                                child: IconButton.outlined(
                                  onPressed: onGenerate,
                                  icon: Icon(Icons.refresh),
                                ),
                              ),
                            ],
                          ),
                          Gap(12),
                          Tooltip(
                            message: "Block height when the wallet was created. Save synchronization time by skipping blocks before the birth height",
                            child: FormBuilderTextField(
                              name: "birth",
                              decoration: const InputDecoration(
                                labelText: "Birth Height",
                                prefixIcon: Icon(Icons.height),
                              ),
                              keyboardType: TextInputType.number,
                              inputFormatters: [
                                FilteringTextInputFormatter.digitsOnly,
                              ],
                            ),
                          ),
                        ],
                      ],
                    ),
                  ),
                ),
                Gap(12),
                Card(
                  child: Padding(
                    padding: const EdgeInsets.all(20),
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        FormBuilderSwitch(
                          name: "advanced",
                          title: Row(
                            children: [
                              SizedBox(width: 38, child: Icon(Icons.tune, size: 20)),
                              Gap(8),
                              const Text("Advanced Options"),
                            ],
                          ),
                          initialValue: _showAdvanced,
                          onChanged: (v) => setState(() => _showAdvanced = v ?? false),
                        ),
                        if (_showAdvanced) ...[
                          Gap(8),
                          Text(
                            "Extra derivation and pool settings",
                            style: Theme.of(context).textTheme.bodySmall?.copyWith(
                                  color: Theme.of(context).colorScheme.onSurface.withAlpha(120),
                                ),
                          ),
                          Gap(12),
                          if (!ledger && (isSeed || key.isEmpty))
                            Tooltip(
                              message: "Check if you want this account to use an internal address for the change like Zashi (ZIP 316)",
                              child: FormBuilderSwitch(
                                name: "useInternal",
                                title: const Text("Use Internal Change"),
                              ),
                            ),
                          if (restore) ...[
                            Gap(12),
                            if (isSeed && !ledger)
                              Tooltip(
                                message: "An optional extra word/phrase added to the seed phrase (like in Trezor)",
                                child: FormBuilderTextField(
                                  name: "passphrase",
                                  decoration: const InputDecoration(
                                    labelText: "Extra Passphrase (optional)",
                                    prefixIcon: Icon(Icons.lock_outline),
                                  ),
                                ),
                              ),
                            Gap(12),
                            if (isSeed || ledger)
                              Tooltip(
                                message: "The derivation account index. Usually 0, but could be 1, 2, etc if you have additional accounts under the same seed",
                                child: FormBuilderTextField(
                                  name: "aindex",
                                  decoration: const InputDecoration(
                                    labelText: "Account Index",
                                    prefixIcon: Icon(Icons.tag),
                                  ),
                                  keyboardType: TextInputType.number,
                                  inputFormatters: [
                                    FilteringTextInputFormatter.digitsOnly,
                                  ],
                                ),
                              ),
                            Gap(12),
                            if (keyPools != 0)
                              Tooltip(
                                message: "Pools this account can receive funds",
                                child: InputDecorator(
                                  decoration: InputDecoration(labelText: "Pools"),
                                  child: Align(
                                    alignment: Alignment.centerRight,
                                    child: FormBuilderField<int>(
                                      name: "pools",
                                      initialValue: keyPools,
                                      builder: (field) => PoolSelect(
                                        enabled: keyPools,
                                        initialValue: field.value!,
                                        onChanged: (v) => field.didChange(v),
                                      ),
                                    ),
                                  ),
                                ),
                              ),
                            Gap(12),
                            FormBuilderSwitch(
                              name: "ledger",
                              title: Row(
                                children: [
                                  SizedBox(width: 38, child: Icon(Icons.usb, size: 20)),
                                  Gap(8),
                                  const Text("H/W Ledger"),
                                ],
                              ),
                              initialValue: ledger,
                              onChanged: (v) => setState(() => ledger = v ?? false),
                            ),
                          ],
                        ],
                      ],
                    ),
                  ),
                ),
                Gap(16),
              ],
            ),
          ),
        ),
      ),
    );
  }

  void onFrost() => GoRouter.of(context).push("/dkg1");

  void onGenerate() async {
    final seed = generateSeed();
    formKey.currentState!.fields["key"]!.didChange(seed);
  }

  void onSave() async {
    if (formKey.currentState?.saveAndValidate() ?? false) {
      final currentHeight = ref.read(currentHeightProvider).value;

      // Handle the save logic here
      final formData = formKey.currentState?.value;
      final String? name = formData?["name"];
      final bool? restore = formData?["restore"];
      final bool ledger = formData?["ledger"] as bool? ?? false;
      final String? passphrase = formData?["passphrase"];
      final String? aindex = formData?["aindex"];
      final String? birth = formData?["birth"];
      final bool? useInternal = formData?["useInternal"];
      final int? pools = formData!["pools"];

      final icon = iconBytes;

      final r = restore ?? false;
      if (r && birth == null) {
        final confirmed = await confirmDialog(
          context,
          title: "No Birth Height",
          message: "Are you sure you don't want to enter the birth height?",
        );
        if (!confirmed) return;
      }

      final bh = birth != null ? int.parse(birth) : currentHeight ?? 1;
      AwesomeDialog? dialog;
      try {
        String message = "Please wait while we create the account";
        if (ledger) message += "\nConfirm on your Ledger device";
        dialog = await showMessage(context, message, dismissable: false);
        final account = await newAccount(
            na: NewAccount(
              icon: icon,
              name: name ?? "",
              restore: r,
              key: key,
              passphrase: passphrase,
              aindex: int.parse(aindex ?? "0"),
              birth: bh,
              folder: "",
              pools: pools,
              useInternal: useInternal ?? false,
              internal: false,
              ledger: ledger,
            ),
            c: c);
        dialog.dismiss();
        dialog = null;
        final settings = ref.read(appSettingsProvider).requireValue;
        try {
          // ignore errors since it's just caching
          if (!settings.offline) await cacheBlockTime(height: bh, c: c);
        } on AnyhowException catch (_) {}

        await coinContext.setAccount(account: account);
        c = coinContext.coin;

        if ((key.isNotEmpty && await hasTransparentPubKey(c: c)) || ledger) {
          await showTransparentScan(ref, context);
        }

        final seed = await getAccountSeed(account: account, c: c);
        if (seed != null && settings.vault) {
          await ref.read(vaultProvider.notifier).storeAccount(
                name: name ?? "",
                seed: seed.mnemonic,
                aindex: int.parse(aindex ?? "0"),
                useInternal: useInternal ?? false,
                birthHeight: bh,
              );
        }
        if (mounted && key.isEmpty && seed != null) {
          await showSeed(context, seed.mnemonic);
        }
        if (mounted && r && currentHeight != null) {
          final shouldSync = await confirmDialog(
            context,
            title: "Account Imported",
            message: "Account imported successfully. Would you like to synchronize it now?",
          );
          if (shouldSync && mounted) {
            unawaited(ref.read(synchronizerProvider.notifier).syncIfNeeded(currentHeight, now: true));
          }
        }
        ref.invalidate(getAccountsProvider);
        if (mounted) GoRouter.of(context).pop();
      } on AnyhowException catch (e) {
        await showException(context, e.message);
        dialog?.dismiss();
      }
    }
  }

  void onEdit() async {
    final icon = await pickImage();
    if (icon != null) {
      final bytes = await icon.readAsBytes();
      setState(() => iconBytes = bytes);
    }
  }

  onImport() async {
    try {
      final data = await openFile(title: "Please select an encrypted account file for import");
      if (data == null) return;
      if (!mounted) return;
      final password = await inputPassword(
        context,
        title: "Import File",
        message: "File Password",
      );
      if (password != null) {
        await importAccount(passphrase: password, data: data, c: c);
        if (mounted) await showMessage(context, "Account imported successfully");
        ref.invalidate(getAccountsProvider);
      }
    } on AnyhowException catch (e) {
      logger.e(e);
      if (mounted) await showException(context, e.message);
    }
  }
}
