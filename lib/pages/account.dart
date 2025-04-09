import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_form_builder/flutter_form_builder.dart';
import 'package:image_picker/image_picker.dart';
import 'package:zkool/src/rust/api/account.dart';
import 'package:zkool/src/rust/api/network.dart';
import 'package:zkool/src/rust/api/pay.dart';
import 'package:zkool/src/rust/api/sync.dart';
import 'package:zkool/src/rust/pay.dart';
import 'package:zkool/store.dart';
import 'package:zkool/utils.dart';

class AccountViewPage extends StatefulWidget {
  final Account account;
  const AccountViewPage(this.account, {super.key});

  @override
  State<AccountViewPage> createState() => AccountViewPageState();
}

class AccountViewPageState extends State<AccountViewPage> {
  StreamSubscription<SyncProgress>? progressSubscription;
  int? height;
  PoolBalance? poolBalance;

  @override
  void initState() {
    super.initState();
    setAccount(id: widget.account.id);
    Future(() async {
      final b = await balance();
      final recipients = [
        Recipient(
            address:
                "zs1n55f4yctfdjflu75vx4vys3xgs6qzxd26qmhmvwxj9jdwxg8sswznpvu7elkccmddfdn5hnfseq",
            amount: BigInt.from(1480000)),
        Recipient(
            address:
                "u1ydx7cvpul4v8z29q4vuqczalmsztn5dlxrmujvavzsxyxjk3evpuerqhgwnhemdw9t3q6mpk3klk8ss7803lsv400zax2wrw8cacmzaz",
            amount: BigInt.from(280000)),
      ];
      await wipPlan(
          account: widget.account.id, srcPools: 7, recipients: recipients,
          recipientPaysFee: false);
      setState(() {
        poolBalance = b;
      });
    });
  }

  @override
  Widget build(BuildContext context) {
    final h = height;
    final b = poolBalance;

    return Scaffold(
        appBar: AppBar(
          title: Text(widget.account.name),
        ),
        body: Padding(
          padding: const EdgeInsets.symmetric(horizontal: 16),
          child: Column(
            children: [
              if (h != null) Text(h.toString()),
              if (b != null)
                Text("${b.field0[0]} ${b.field0[1]} ${b.field0[2]}"),
              IconButton.filled(onPressed: onSync, icon: Icon(Icons.sync)),
              IconButton.filled(
                  onPressed: onRewind, icon: Icon(Icons.fast_rewind)),
              IconButton.filled(
                  onPressed: onPrepare, icon: Icon(Icons.play_arrow)),
            ],
          ),
        ));
  }

  void onSync() async {
    final ids = appStore.accounts.map((a) => a.id).toList();
    await progressSubscription?.cancel();
    final currentHeight = await getCurrentHeight();
    final progress = synchronize(accounts: ids, currentHeight: currentHeight);
    progressSubscription = progress.listen(
      (event) async {
        setState(() {
          height = event.height;
        });
      },
      onDone: () async {
        final b = await balance();
        setState(() {
          poolBalance = b;
        });
      },
    );
  }

  void onRewind() async {
    final dbHeight = await getDbHeight(account: widget.account.id);
    await rewindSync(height: dbHeight - 60);
  }

  void onPrepare() async {
    await prepare(account: widget.account.id, senderPayFees: true, srcPools: 7);
  }
}

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
                  GestureDetector(onTap: onEditIcon, child: account.avatar)
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

  void onEditName(String? name) async {
    if (name != null) {
      account = account.copyWith(name: name);
      await updateAccount(
          update:
              AccountUpdate(coin: account.coin, id: account.id, name: name));
      await AppStoreBase.loadAccounts();
      setState(() {});
    }
  }

  void onEditIcon() async {
    final picker = ImagePicker();
    final icon = await picker.pickImage(source: ImageSource.gallery);
    if (icon != null) {
      final bytes = await icon.readAsBytes();
      account = account.copyWith(icon: bytes);
      await updateAccount(
          update:
              AccountUpdate(coin: account.coin, id: account.id, icon: bytes));
      await AppStoreBase.loadAccounts();
      setState(() {});
    }
  }

  void onEditBirth(String? birth) async {
    if (birth != null) {
      account = account.copyWith(birth: int.parse(birth));
      await updateAccount(
          update: AccountUpdate(
              coin: account.coin, id: account.id, birth: int.parse(birth)));
      await AppStoreBase.loadAccounts();
      setState(() {});
    }
  }
}

extension AccountExtension on Account {
  CircleAvatar get avatar {
    final i = initials(name);
    return CircleAvatar(
      child: icon != null ? Image.memory(icon!) : Text(i),
    );
  }
}
