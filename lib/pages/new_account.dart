import 'dart:io';

import 'package:convert/convert.dart';
import 'package:file_picker/file_picker.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_form_builder/flutter_form_builder.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated.dart';
import 'package:gap/gap.dart';
import 'package:go_router/go_router.dart';
import 'package:image_picker/image_picker.dart';
import 'package:showcaseview/showcaseview.dart';
import 'package:zkool/main.dart';
import 'package:zkool/src/rust/api/account.dart';
import 'package:zkool/src/rust/api/key.dart';
import 'package:zkool/store.dart';
import 'package:zkool/utils.dart';
import 'package:zkool/validators.dart';

final importID = GlobalKey();
final saveID = GlobalKey();
final iconID = GlobalKey();
final nameID = GlobalKey();
final internalID = GlobalKey();
final restoreID = GlobalKey();

final keyID = GlobalKey();
final passphraseID = GlobalKey();
final accountIndexID = GlobalKey();
final fingerprintID = GlobalKey();
final birthID = GlobalKey();

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
  var isFvk = false;
  Uint8List? iconBytes;
  final formKey = GlobalKey<FormBuilderState>();

  void tutorial() async {
    tutorialHelper(context, "tutNew0", [
      nameID, iconID, internalID, restoreID, importID, saveID
    ]);
    if (restore) tutorialHelper(context, "tutNew1", [
      keyID, birthID
    ]);
    if (restore && isSeed) tutorialHelper(context, "tutNew2", [
      passphraseID, accountIndexID
    ]);
    if (restore && isFvk) tutorialHelper(context, "tutNew3", [
      fingerprintID
    ]);
  }

  @override
  Widget build(BuildContext context) {
    Future(tutorial);

    final ib = iconBytes;
    isSeed = isValidPhrase(phrase: key);
    isFvk = isValidFvk(fvk: key);

    return Scaffold(
        appBar: AppBar(
          title: const Text("New Account"),

          actions: [
            IconButton(
              onPressed: onFrost, icon: Icon(Icons.ac_unit)),
            Showcase(
                key: importID,
                description: "Import an account from file",
                child: IconButton(
                    onPressed: onImport, icon: Icon(Icons.file_download))),
            Showcase(
                key: saveID,
                description: "Save",
                child: IconButton(
                  icon: const Icon(Icons.save),
                  onPressed: onSave,
                )),
          ],
        ),
        body: Padding(
            padding: const EdgeInsets.symmetric(horizontal: 8),
            child: SingleChildScrollView(
              child: FormBuilder(
                key: formKey,
                child: Column(
                  children: [
                    Stack(children: [
                      Showcase(
                          key: iconID,
                          description: "Upload a icon",
                          child: CircleAvatar(
                            radius: 60,
                            backgroundImage:
                                ib != null ? Image.memory(ib).image : null,
                            child: ib == null ? Text(initials(name)) : null,
                          )),
                      Positioned(
                          right: 0,
                          bottom: 0,
                          child: IconButton.filled(
                            onPressed: onEdit,
                            icon: Icon(Icons.edit),
                          ))
                    ]),
                    Gap(16),
                    Showcase(
                        key: nameID,
                        description:
                            "Enter a name that identifies this account",
                        child: FormBuilderTextField(
                          name: "name",
                          decoration:
                              const InputDecoration(labelText: "Account Name"),
                          initialValue: name,
                          onChanged: (v) => setState(() => name = v!),
                        )),
                    Gap(16),
                    Showcase(
                        key: internalID,
                        description:
                            "Check if you want this account to use an internal address for the change like Zashi (ZIP 315)",
                        child: FormBuilderSwitch(
                            name: "useInternal",
                            title: const Text("Use Internal Change"))),
                    Gap(16),
                    Showcase(
                        key: restoreID,
                        description:
                            "Check if you want to restore an existing account",
                        child: FormBuilderSwitch(
                            name: "restore",
                            title: const Text("Restore Account?"),
                            initialValue: restore,
                            onChanged: (v) =>
                                setState(() => restore = v ?? false))),
                    Gap(16),
                    if (restore)
                      Showcase(
                          key: keyID,
                          description:
                              "Seed phrase (12, 18, 21, 24 words), a Sapling secret key, a viewing key, a unified viewing key, a xpub/xprv transparent key or a BIP-38 key (starting with K or L)",
                          child: FormBuilderTextField(
                            name: "key",
                            decoration: const InputDecoration(
                                labelText:
                                    "Key (Seed Phrase, Private Key, or Viewing Key)"),
                            validator: (s) => validKey(s, restore: restore),
                            initialValue: key,
                            onChanged: (v) => setState(() => key = v!),
                          )),
                    Gap(16),
                    if (restore && isSeed)
                    Showcase(key: passphraseID, description: "An optional extra word/phrase added to the seed phrase (like in Trezor)", child:
                      FormBuilderTextField(
                        name: "passphrase",
                        decoration: const InputDecoration(
                            labelText: "Extra Passphrase (optional)"),
                      )),
                    Gap(16),
                    if (restore && isSeed)
                    Showcase(key: accountIndexID, description: "The derivation account index. Usually 0, but could be 1, 2, etc if you have additional accounts under the same seed", child:
                      FormBuilderTextField(
                        name: "aindex",
                        decoration:
                            const InputDecoration(labelText: "Account Index"),
                        keyboardType: TextInputType.number,
                        inputFormatters: [
                          FilteringTextInputFormatter.digitsOnly
                        ],
                      )),
                    Gap(16),
                    if (restore && isFvk)
                    Showcase(key: fingerprintID, description: "The seed fingerprint is needed for cold wallet transactions. It is the 4-byte number displayed under the UFVK", child:
                      FormBuilderTextField(
                        name: "fingerprint",
                        decoration:
                            const InputDecoration(labelText: "Seed Fingerprint"),
                        validator: (s) => (s == null || s.isEmpty) ? null : validHexString(s, 4),
                      )),
                    if (restore)
                      Showcase(key: birthID, description: "Block height when the wallet was created. Save synchronization time by skipping blocks before the birth height", child:
                      FormBuilderTextField(
                        name: "birth",
                        decoration:
                            const InputDecoration(labelText: "Birth Height"),
                        keyboardType: TextInputType.number,
                        inputFormatters: [
                          FilteringTextInputFormatter.digitsOnly
                        ],
                      )),
                  ],
                ),
              ),
            )));
  }

  void onFrost() => GoRouter.of(context).push("/dkg1");

  void onSave() async {
    if (formKey.currentState?.saveAndValidate() ?? false) {
      // Handle the save logic here
      final formData = formKey.currentState?.value;
      final String? name = formData?["name"];
      final bool? restore = formData?["restore"];
      final String? passphrase = formData?["passphrase"];
      final String? aindex = formData?["aindex"];
      final String? birth = formData?["birth"];
      final String? fingerprint = formData?["fingerprint"];
      final bool? useInternal = formData?["useInternal"];

      final icon = iconBytes;

      final account = await newAccount(
          na: NewAccount(
        icon: icon,
        name: name ?? "",
        restore: restore ?? false,
        key: key,
        passphrase: passphrase,
        aindex: int.parse(aindex ?? "0"),
        birth: birth != null
            ? int.parse(birth)
            : AppStoreBase.instance.currentHeight,
        fingerprint: fingerprint != null ? Uint8List.fromList(hex.decode(fingerprint)) : null,
        useInternal: useInternal ?? false,
        internal: false,
      ));
      final seed = await getAccountSeed(account: account);
      if (mounted && key.isEmpty) {
        await showSeed(context, seed!.mnemonic);
      }
      await AppStoreBase.instance.loadAccounts();
      if (mounted) GoRouter.of(context).pop();
    }
  }

  void onEdit() async {
    final picker = ImagePicker();
    final icon = await picker.pickImage(source: ImageSource.gallery);
    if (icon != null) {
      final bytes = await icon.readAsBytes();
      setState(() => iconBytes = bytes);
    }
  }

  onImport() async {
    try {
      final files = await FilePicker.platform.pickFiles(
        dialogTitle: 'Please select an encrypted account file for import',
      );
      if (files == null) return;
      if (!mounted) return;
      final file = files.files.first;
      final password = await inputPassword(context,
          title: "Import File", message: "File Password");
      if (password != null) {
        final encryptedFile = File(file.path!);
        final encrypted = encryptedFile.readAsBytesSync();
        await importAccount(passphrase: password, data: encrypted);
        if (mounted)
          await showMessage(context, "Account imported successfully");
        await AppStoreBase.instance.loadAccounts();
      }
    } on AnyhowException catch (e) {
      logger.e(e);
      if (mounted) await showException(context, e.message);
    }
  }
}
