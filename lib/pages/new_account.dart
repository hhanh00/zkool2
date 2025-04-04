import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_form_builder/flutter_form_builder.dart';
import 'package:gap/gap.dart';
import 'package:go_router/go_router.dart';
import 'package:image_picker/image_picker.dart';
import 'package:zkool/src/rust/api/account.dart';
import 'package:zkool/src/rust/api/key.dart';
import 'package:zkool/store.dart';
import 'package:zkool/utils.dart';

class NewAccountPage extends StatefulWidget {
  const NewAccountPage({super.key});

  @override
  State<NewAccountPage> createState() => NewAccountPageState();
}

class NewAccountPageState extends State<NewAccountPage> {
  var name = "";
  var restore = false;
  Uint8List? iconBytes;
  final formKey = GlobalKey<FormBuilderState>();

  @override
  Widget build(BuildContext context) {
    final ib = iconBytes;

    return Scaffold(
        appBar: AppBar(
          title: const Text("New Account"),
          actions: [
            IconButton(
              icon: const Icon(Icons.save),
              onPressed: onSave,
            ),
          ],
        ),
        body: Padding(
          padding: const EdgeInsets.symmetric(horizontal: 8),
          child: FormBuilder(
            key: formKey,
            child: Column(
              children: [
                Stack(children: [
                  CircleAvatar(
                    radius: 60,
                    backgroundImage: ib != null ? Image.memory(ib).image : null,
                    child: ib == null ? Text(initials(name)) : null,
                  ),
                  Positioned(
                      right: 0,
                      bottom: 0,
                      child: IconButton.filled(
                        onPressed: onEdit,
                        icon: Icon(Icons.edit),
                      ))
                ]),
                Gap(16),
                FormBuilderTextField(
                  name: "name",
                  decoration: const InputDecoration(labelText: "Account Name"),
                  initialValue: name,
                  onChanged: (v) => setState(() => name = v!),
                ),
                Gap(16),
                FormBuilderSwitch(
                    name: "restore",
                    title: const Text("Restore Account?"),
                    initialValue: restore,
                    onChanged: (v) => setState(() => restore = v ?? false)),
                Gap(16),
                if (restore)
                  FormBuilderTextField(
                    name: "key",
                    decoration: const InputDecoration(
                        labelText:
                            "Key (Seed Phrase, Private Key, or Viewing Key)"),
                    validator: validKey,
                  ),
                Gap(16),
                if (restore)
                  FormBuilderTextField(
                    name: "aindex",
                    decoration: const InputDecoration(
                        labelText:
                            "Account Index (when using a seed phrase)"),
                    keyboardType: TextInputType.number,
                    inputFormatters: [FilteringTextInputFormatter.digitsOnly],
                  ),
                Gap(16),
                if (restore)
                  FormBuilderTextField(
                    name: "birth",
                    decoration:
                        const InputDecoration(labelText: "Birth Height"),
                    keyboardType: TextInputType.number,
                    inputFormatters: [FilteringTextInputFormatter.digitsOnly],
                  ),
              ],
            ),
          ),
        ));
  }

  void onSave() async {
    if (formKey.currentState?.saveAndValidate() ?? false) {
      // Handle the save logic here
      final formData = formKey.currentState?.value;
      final String? name = formData?["name"];
      final bool? restore = formData?["restore"];
      final String? key = formData?["key"];
      final String? aindex = formData?["aindex"];
      final String? birth = formData?["birth"];
      final icon = iconBytes;

      await newAccount(
          na: NewAccount(
            icon: icon,
            name: name ?? "",
            restore: restore ?? false,
            key: key ?? "",
            aindex: int.parse(aindex ?? "0"),
            birth: birth != null ? int.parse(birth) : null,
          ));
      await AppStoreBase.loadAccounts();
      GoRouter.of(context).pop();
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

  String? validKey(String? key) {
    if ((key == null || key.isEmpty)) {
      return restore ? "Key is required" : null;
    }
    if (!isValidKey(key: key)) {
      return "Invalid Key";
    }
    return null;
  }
}
