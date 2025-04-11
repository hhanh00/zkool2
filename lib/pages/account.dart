import 'dart:async';

import 'package:convert/convert.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_form_builder/flutter_form_builder.dart';
import 'package:flutter_mobx/flutter_mobx.dart';
import 'package:gap/gap.dart';
import 'package:go_router/go_router.dart';
import 'package:image_picker/image_picker.dart';
import 'package:zkool/src/rust/api/account.dart';
import 'package:zkool/src/rust/api/network.dart';
import 'package:zkool/src/rust/api/sync.dart';
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
  late int height = widget.account.height;
  PoolBalance? poolBalance;

  @override
  void initState() {
    super.initState();
    setAccount(id: widget.account.id);
    Future(() async {
      await getTxDetails();

      final b = await balance();
      await AppStoreBase.loadTxHistory();
      await AppStoreBase.loadMemos();
      setState(() {
        poolBalance = b;
      });
    });
  }

  @override
  Widget build(BuildContext context) {
    final t = Theme.of(context).textTheme;
    final h = height;
    final b = poolBalance;

    return Scaffold(
        appBar: AppBar(
          title: Text(widget.account.name),
          actions: [
            IconButton(onPressed: onSync, icon: Icon(Icons.sync)),
            IconButton(onPressed: onRewind, icon: Icon(Icons.fast_rewind)),
            IconButton(onPressed: onSend, icon: Icon(Icons.send)),
          ],
        ),
        body: Padding(
          padding: const EdgeInsets.symmetric(horizontal: 16),
          child: Observer(builder: (context) {
            // make sure there is a dependency on transactions
            appStore.transactions.length;

            return Column(
              children: [
                Text(h.toString(), style: t.bodyLarge),
                if (b != null)
                  Row(children: [
                    Text("T: ${zatToString(b.field0[0])}"),
                    const Gap(8),
                    Text("S: ${zatToString(b.field0[1])}"),
                    const Gap(8),
                    Text("O: ${zatToString(b.field0[2])}"),
                  ]),
                ...showTxHistory(appStore.transactions),
                ...showMemos(appStore.memos),
              ],
            );
          }),
        ));
  }

  void onSync() async {
    await progressSubscription?.cancel();
    final currentHeight = await getCurrentHeight();
    final progress = synchronize(
        accounts: [widget.account.id], currentHeight: currentHeight);
    progressSubscription = progress.listen(
      (event) async {
        setState(() {
          height = event.height;
        });
      },
      onDone: () async {
        final b = await balance();
        await AppStoreBase.loadAccounts();
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

  void onSend() async {
    await GoRouter.of(context).push("/send");
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
              FormBuilderSwitch(
                name: "enabled",
                title: Text("Enabled"),
                initialValue: account.enabled,
                onChanged: onEditEnabled,
              ),
              FormBuilderSwitch(
                name: "hidden",
                title: Text("Hidden"),
                initialValue: account.hidden,
                onChanged: onEditHidden,
              )
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

  void onEditEnabled(v) async {
    account = account.copyWith(enabled: v);
    await updateAccount(
        update: AccountUpdate(coin: account.coin, id: account.id, enabled: v));
    await AppStoreBase.loadAccounts();
    setState(() {});
  }

  void onEditHidden(v) async {
    account = account.copyWith(hidden: v);
    await updateAccount(
        update: AccountUpdate(coin: account.coin, id: account.id, hidden: v));
    await AppStoreBase.loadAccounts();
    setState(() {});
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

List<Widget> showTxHistory(List<Tx> transactions) {
  return [
    const Text("Transaction History"),
    const Gap(8),
    ListView.builder(
      shrinkWrap: true,
      itemCount: transactions.length,
      itemBuilder: (context, index) {
        final tx = transactions[index];
        // encode tx.txid to hex string
        String txId = txIdToString(tx.txid);

        return ListTile(
          leading: Text("${tx.height}"),
          title: SelectableText(txId),
          subtitle: Text(timeToString(tx.time)),
          trailing: Text(zatToString(BigInt.from(tx.value))),
        );
      },
    ),
  ];
}

List<Widget> showMemos(List<Memo> memos) {
  return [
    const Text("Memos"),
    const Gap(8),
    ListView.builder(
      shrinkWrap: true,
      itemCount: memos.length,
      itemBuilder: (context, index) {
        final memo = memos[index];

        return ListTile(
          leading: Text("${memo.height}"),
          title: SelectableText(memo.memo ?? hex.encode(memo.memoBytes)),
          subtitle: Text(timeToString(memo.time)),
          trailing: Text(memo.idNote != null ? "In" : "Out"),
        );
      },
    ),
  ];
}
