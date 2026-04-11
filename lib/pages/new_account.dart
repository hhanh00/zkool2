import 'package:awesome_dialog/awesome_dialog.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_form_builder/flutter_form_builder.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated.dart';
import 'package:gap/gap.dart';
import 'package:go_router/go_router.dart';
import 'package:showcaseview/showcaseview.dart';
import 'package:zkool/main.dart';
import 'package:zkool/pages/sweep.dart';
import 'package:zkool/src/rust/api/account.dart';
import 'package:zkool/src/rust/api/key.dart';
import 'package:zkool/src/rust/api/sync.dart';
import 'package:zkool/store.dart';
import 'package:zkool/utils.dart';
import 'package:zkool/validators.dart';
import 'package:zkool/widgets/pool_select.dart';

final dkgID = GlobalKey();
final importID = GlobalKey();
final saveID = GlobalKey();
final iconID = GlobalKey();
final nameID = GlobalKey();
final internalID = GlobalKey();
final restoreID = GlobalKey();

final keyID = GlobalKey();
final generateID = GlobalKey();
final passphraseID = GlobalKey();
final accountIndexID = GlobalKey();
final birthID = GlobalKey();
final accountPoolsID = GlobalKey();

class NewAccountPage extends ConsumerStatefulWidget {
  const NewAccountPage({super.key});

  @override
  ConsumerState<NewAccountPage> createState() => NewAccountPageState();
}

class NewAccountPageState extends ConsumerState<NewAccountPage> {
  late var c = ref.read(coinContextProvider);
  var name = "";
  var restore = false;
  var key = "";
  var isSeed = false;
  var ledger = false;
  var isFvk = false;
  var multifactorEnabled = false;
  Uint8List? iconBytes;
  final formKey = GlobalKey<FormBuilderState>();

  void tutorial() async {
    tutorialHelper(
      context,
      "tutNew0",
      [nameID, iconID, internalID, restoreID, dkgID, importID, saveID],
    );
    if (restore) tutorialHelper(context, "tutNew1", [keyID, generateID, birthID, accountPoolsID]);
    if (restore && isSeed) tutorialHelper(context, "tutNew2", [passphraseID, accountIndexID]);
  }

  @override
  Widget build(BuildContext context) {
    final pinlock = ref.watch(lifecycleProvider);
    if (pinlock.value ?? false) return PinLock();

    Future(tutorial);

    final ib = iconBytes;
    isSeed = isValidPhrase(phrase: key);
    isFvk = isValidFvk(fvk: key, c: c);
    final keyPools = ledger ? 3 : getKeyPools(key: key, c: c); // 3 is T+S

    return Scaffold(
      appBar: AppBar(
        title: const Text("New Account"),
        actions: [
          Showcase(
            key: dkgID,
            description: "Start Distributed Key Generation",
            child: IconButton(onPressed: onFrost, icon: Icon(Icons.group)),
          ),
          Showcase(
            key: importID,
            description: "Import an account from file",
            child: IconButton(
              onPressed: onImport,
              icon: Icon(Icons.file_open),
            ),
          ),
          Showcase(
            key: saveID,
            description: "Save",
            child: IconButton(
              icon: const Icon(Icons.save),
              onPressed: onSave,
            ),
          ),
        ],
      ),
      body: Padding(
        padding: const EdgeInsets.symmetric(horizontal: 8),
        child: SingleChildScrollView(
          child: FormBuilder(
            key: formKey,
            child: Column(
              children: [
                Stack(
                  children: [
                    Showcase(
                      key: iconID,
                      description: "Upload a icon",
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
                Showcase(
                  key: nameID,
                  description: "Enter a name that identifies this account",
                  child: FormBuilderTextField(
                    name: "name",
                    decoration: const InputDecoration(labelText: "Account Name"),
                    initialValue: name,
                    onChanged: (v) => setState(() => name = v!),
                  ),
                ),
                Gap(16),
                if (!ledger) Showcase(
                  key: internalID,
                  description: "Check if you want this account to use an internal address for the change like Zashi (ZIP 316)",
                  child: FormBuilderSwitch(
                    name: "useInternal",
                    title: const Text("Use Internal Change"),
                  ),
                ),
                Gap(16),
                Showcase(
                  key: restoreID,
                  description: "Check if you want to restore an existing account",
                  child: FormBuilderSwitch(
                    name: "restore",
                    title: const Text("Restore Account?"),
                    initialValue: restore,
                    onChanged: (v) => setState(() => restore = v ?? false),
                  ),
                ),
                Gap(16),
                if (!restore)
                  FormBuilderSwitch(
                    name: "multifactor",
                    title: const Text("Enable Multifactor Recovery"),
                    subtitle: Text("Use passkeys and recovery code", style: TextStyle(fontSize: 12)),
                    initialValue: multifactorEnabled,
                    onChanged: (v) {
                      setState(() => multifactorEnabled = v ?? false);
                    },
                  ),
                if (!restore && multifactorEnabled) ...[
                  Gap(8),
                  _buildMultifactorInfoCard(),
                ],
                Gap(16),
                if (restore)
                  FormBuilderSwitch(
                    name: "ledger",
                    title: const Text("H/W Ledger"),
                    initialValue: ledger,
                    onChanged: (v) => setState(() => ledger = v ?? false),
                  ),
                Gap(16),
                if (restore)
                  Row(
                    children: [
                      Expanded(
                        child: Showcase(
                          key: keyID,
                          description:
                              "Seed phrase (12, 18, 21, 24 words), a Sapling secret key, a viewing key, a unified viewing key, a xpub/xprv transparent key or a BIP-38 key (starting with K or L)",
                          child: FormBuilderTextField(
                            name: "key",
                            decoration: const InputDecoration(
                              labelText: "Key (Seed Phrase, Private Key, or Viewing Key)",
                            ),
                            validator: (s) => validKey(s, restore: restore && !ledger, c: c),
                            initialValue: key,
                            onChanged: (v) => setState(() => key = v!),
                          ),
                        ),
                      ),
                      Gap(8),
                      Showcase(
                        key: generateID,
                        description: "Generate a new Seed Phrase",
                        child: IconButton.outlined(
                          onPressed: onGenerate,
                          icon: Icon(Icons.refresh),
                        ),
                      ),
                    ],
                  ),
                Gap(16),
                if (restore && isSeed && !ledger)
                  Showcase(
                    key: passphraseID,
                    description: "An optional extra word/phrase added to the seed phrase (like in Trezor)",
                    child: FormBuilderTextField(
                      name: "passphrase",
                      decoration: const InputDecoration(
                        labelText: "Extra Passphrase (optional)",
                      ),
                    ),
                  ),
                Gap(16),
                if (restore && (isSeed || ledger))
                  Showcase(
                    key: accountIndexID,
                    description: "The derivation account index. Usually 0, but could be 1, 2, etc if you have additional accounts under the same seed",
                    child: FormBuilderTextField(
                      name: "aindex",
                      decoration: const InputDecoration(
                        labelText: "Account Index",
                      ),
                      keyboardType: TextInputType.number,
                      inputFormatters: [
                        FilteringTextInputFormatter.digitsOnly,
                      ],
                    ),
                  ),
                Gap(16),
                if (restore)
                  Showcase(
                    key: birthID,
                    description: "Block height when the wallet was created. Save synchronization time by skipping blocks before the birth height",
                    child: FormBuilderTextField(
                      name: "birth",
                      decoration: const InputDecoration(
                        labelText: "Birth Height",
                      ),
                      keyboardType: TextInputType.number,
                      inputFormatters: [
                        FilteringTextInputFormatter.digitsOnly,
                      ],
                    ),
                  ),
                if (restore && keyPools != 0)
                  Showcase(
                    key: accountPoolsID,
                    description: "Pools this account can receive funds",
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
              ],
            ),
          ),
        ),
      ),
    );
  }

  void onFrost() => GoRouter.of(context).push("/dkg1");

  Widget _buildMultifactorInfoCard() {
    return const MultifactorInfoCard();
  }

  Future<void> _createMultifactorAccount({
    required String name,
    required Uint8List? icon,
    required bool useInternal,
    required int birth,
  }) async {
    try {
      // Placeholder for multifactor account creation
      // TODO: Implement full flow with:
      // - Recovery code input
      // - Passkey registration
      // - iCloud sync check
      // - Account creation with multifactor setup

      if (!mounted) return;
      await showMessage(
        context,
        'Multifactor Recovery\n\n'
        'This feature will add:\n'
        '• Face ID / Touch ID authentication\n'
        '• iCloud sync for passkeys\n'
        '• Base32-encoded recovery string\n'
        '• Recovery code (e.g., "john-wallet")\n\n'
        'Coming soon!',
      );
    } catch (e) {
      if (mounted) await showException(context, e.toString());
    }
  }

  void onGenerate() async {
    final seed = generateSeed();
    formKey.currentState!.fields["key"]!.didChange(seed);
  }

  void onSave() async {
    if (formKey.currentState?.saveAndValidate() ?? false) {
      final currentHeight = ref.read(currentHeightProvider);

      // Handle the save logic here
      final formData = formKey.currentState?.value;
      final String? name = formData?["name"];
      final bool? restore = formData?["restore"];
      final bool multifactor = formData?["multifactor"] as bool? ?? false;
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

      // Handle multifactor account creation
      if (!r && multifactor) {
        await _createMultifactorAccount(
          name: name ?? "",
          icon: icon,
          useInternal: useInternal ?? false,
          birth: birth != null ? int.parse(birth) : currentHeight ?? 1,
        );
        return;
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
          c: c
        );
        dialog.dismiss();
        dialog = null;
        try {
          // ignore errors since it's just caching
          final settings = ref.read(appSettingsProvider).requireValue;
          if (!settings.offline) await cacheBlockTime(height: bh, c: c);
        } on AnyhowException catch (_) {}

        await ref.read(coinContextProvider.notifier).setAccount(account: account);
        c = ref.read(coinContextProvider);

        if ((key.isNotEmpty && await hasTransparentPubKey(c: c)) || ledger) {
          await showTransparentScan(ref, context);
        }

        final seed = await getAccountSeed(account: account, c: c);
        if (mounted && key.isEmpty && seed != null) {
          await showSeed(context, seed.mnemonic);
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

class MultifactorInfoCard extends StatelessWidget {
  const MultifactorInfoCard({super.key});

  @override
  Widget build(BuildContext context) {
    final t = Theme.of(context);
    final tt = t.textTheme;
    final cs = t.colorScheme;
    final bodySmall = tt.bodySmall;

    return Container(
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        color: cs.primaryContainer.withValues(alpha: 0.3),
        borderRadius: BorderRadius.circular(8),
        border: Border.all(color: cs.primary.withValues(alpha: 0.3)),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              Icon(Icons.info_outline, color: cs.primary, size: 20),
              const SizedBox(width: 8),
              Text(
                'Multifactor Recovery Setup',
                style: tt.titleSmall?.copyWith(
                  fontWeight: FontWeight.bold,
                ),
              ),
            ],
          ),
          const Gap(16),
          Text(
            'How it works:',
            style: tt.bodyMedium?.copyWith(
              fontWeight: FontWeight.bold,
              color: t.colorScheme.primary,
            ),
          ),
          const Gap(8),
          Text(
            'Your secret keys are derived from either:',
            style: bodySmall,
          ),
          Text(
            '• a device secret (cloud synced passkey)',
            style: bodySmall,
          ),
          Text(
            '• or a Recovery code for backup',
            style: bodySmall,
          ),
          const Gap(8),
          Text(
            'Restore via: iCloud / Google Password Manager OR recovery code',
            style: bodySmall?.copyWith(
              fontStyle: FontStyle.italic,
            ),
          ),
          const Gap(12),
          InkWell(
            onTap: () async {
              // TODO: Replace with actual documentation URL
              // await launchUrl(Uri.parse('https://your-docs-url.com/multifactor'));
              // For now, show a placeholder message
              if (context.mounted) {
                ScaffoldMessenger.of(context).showSnackBar(
                  const SnackBar(
                    content: Text('Documentation link coming soon!'),
                    duration: Duration(seconds: 2),
                  ),
                );
              }
            },
            child: Row(
              mainAxisSize: MainAxisSize.min,
              children: [
                Icon(Icons.description, size: 16, color: t.colorScheme.primary),
                const SizedBox(width: 4),
                Text(
                  'Learn more in documentation',
                  style: tt.bodySmall?.copyWith(
                    color: t.colorScheme.primary,
                    decoration: TextDecoration.underline,
                  ),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }
}
