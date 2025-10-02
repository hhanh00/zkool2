import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_form_builder/flutter_form_builder.dart';
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

class NewAccountPage extends StatefulWidget {
  const NewAccountPage({super.key});

  @override
  State<NewAccountPage> createState() => NewAccountPageState();
}

class NewAccountPageState extends State<NewAccountPage> {
  var name = "";
  var restore = false;
  var key = "";
  var isSeed = false;
  var ledger = false;
  var isFvk = false;
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
    Future(tutorial);

    final ib = iconBytes;
    isSeed = isValidPhrase(phrase: key);
    isFvk = isValidFvk(fvk: key);
    final keyPools = getKeyPools(key: key);

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
                Showcase(
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
                if (restore)
                  FormBuilderSwitch(
                    name: "ledger",
                    title: const Text("H/W Ledger"),
                    initialValue: ledger,
                    onChanged: (v) => setState(() => ledger = v ?? false),
                  ),
                Gap(16),
                if (restore && !ledger)
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
                            validator: (s) => validKey(s, restore: restore),
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
                if (restore && isSeed)
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

  void onGenerate() async {
    final seed = generateSeed();
    formKey.currentState!.fields["key"]!.didChange(seed);
  }

  void onSave() async {
    if (formKey.currentState?.saveAndValidate() ?? false) {
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

      final bh = birth != null ? int.parse(birth) : appStore.currentHeight;
      try {
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
        );
        try {
          // ignore errors since it's just caching
          if (!appStore.offline) await cacheBlockTime(height: bh);
        } on AnyhowException catch (_) {}

        await setAccount(account: account);

        if (key.isNotEmpty && await hasTransparentPubKey()) {
          await showTransparentScan(context);
        }

        final seed = await getAccountSeed(account: account);
        if (mounted && key.isEmpty && seed != null) {
          await showSeed(context, seed.mnemonic);
        }
        await appStore.loadAccounts();
        if (mounted) GoRouter.of(context).pop();
      } on AnyhowException catch (e) {
        await showException(context, e.message);
      }
    }
  }

  void onEdit() async {
    final icon = await appWatcher.pickImage();
    if (icon != null) {
      final bytes = await icon.readAsBytes();
      setState(() => iconBytes = bytes);
    }
  }

  onImport() async {
    try {
      final data = await appWatcher.openFile(title: "Please select an encrypted account file for import");
      if (data == null) return;
      if (!mounted) return;
      final password = await inputPassword(
        context,
        title: "Import File",
        message: "File Password",
      );
      if (password != null) {
        await importAccount(passphrase: password, data: data);
        if (mounted) await showMessage(context, "Account imported successfully");
        await appStore.loadAccounts();
      }
    } on AnyhowException catch (e) {
      logger.e(e);
      if (mounted) await showException(context, e.message);
    }
  }
}
