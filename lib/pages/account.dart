import 'dart:math';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_form_builder/flutter_form_builder.dart';
import 'package:image_picker/image_picker.dart';
import 'package:zkool/pages/accounts.dart';
import 'package:zkool/src/rust/api/account.dart';

class AccountEditPage extends StatefulWidget {
  final Account account;
  const AccountEditPage(this.account, {super.key});

  @override
  State<AccountEditPage> createState() => AccountEditPageState();
}

class AccountEditPageState extends State<AccountEditPage> {
  late Account account = widget.account;

  @override
  void didUpdateWidget(covariant AccountEditPage oldWidget) {
    account = widget.account;
    super.didUpdateWidget(oldWidget);
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: Text('Account Edit'),
      ),
      body: Padding(
        padding: const EdgeInsets.symmetric(horizontal: 16),
        child: FormBuilder(
          child: Column(
        children: [
          Row(
            children: [
              Expanded(
                  child: FormBuilderTextField(
                name: 'name',
                decoration: InputDecoration(labelText: 'Name'),
                initialValue: account.name,
                onChanged: onEditName,
              )),
              GestureDetector(
                  onTap: onEditIcon,
                  child: account.avatar)
            ],
          ),
          FormBuilderTextField(
            name: 'birth',
            decoration: InputDecoration(labelText: 'Birth Height'),
            initialValue: account.birth.toString(),
            keyboardType: TextInputType.number,
            inputFormatters: [FilteringTextInputFormatter.digitsOnly],
            onChanged: onEditBirth,
          ),
        ],
      )),
    ));
  }

  void onEditName(String? name) {
    if (name != null) {
      setState(() {
        print("Edit name: $name");
        account = account.copyWith(name: name);
        updateAccount(coin: account.coin, id: account.id, name: name);
        updateList();
      });
    }
  }

  void onEditIcon() async {
    final picker = ImagePicker();
    final icon = await picker.pickImage(source: ImageSource.gallery);
    if (icon != null) {
      final bytes = await icon.readAsBytes();
      setState(() {
        account = account.copyWith(icon: bytes);
        updateAccount(coin: account.coin, id: account.id, icon: bytes);
        updateList();
      });
    }
  }

  void onEditBirth(String? birth) {
    if (birth != null) {
      setState(() {
        account = account.copyWith(birth: int.parse(birth));
        updateAccount(coin: account.coin, id: account.id, birth: int.parse(birth));
        updateList();
      });
    }
  }
}

extension AccountExtension on Account {
  CircleAvatar get avatar {
    final initials = name.substring(0, min(2, name.length));
    return CircleAvatar(
      child: icon != null ? Image.memory(icon!) : Text(initials),
    );
  }
}
