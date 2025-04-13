import 'dart:async';

import 'package:convert/convert.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_form_builder/flutter_form_builder.dart';
import 'package:flutter_mobx/flutter_mobx.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated.dart';
import 'package:gap/gap.dart';
import 'package:go_router/go_router.dart';
import 'package:image_picker/image_picker.dart';
import 'package:zkool/main.dart';
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
      final b = await balance();
      await AppStoreBase.instance.loadTxHistory();
      await AppStoreBase.instance.loadMemos();
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

    logger.i("Memos #${AppStoreBase.instance.memos.length}");

    return DefaultTabController(
        length: 2,
        child: Scaffold(
            appBar: AppBar(
              title: Text(widget.account.name),
              actions: [
                IconButton(
                    tooltip: "Open Log",
                    onPressed: () => onOpenLog(context),
                    icon: Icon(Icons.description)),
                IconButton(
                    tooltip: "Sync this account",
                    onPressed: onSync,
                    icon: Icon(Icons.sync)),
                IconButton(
                    tooltip: "Rewind to previous checkpoint",
                    onPressed: onRewind,
                    icon: Icon(Icons.fast_rewind)),
                IconButton(
                    tooltip: "Receive Funds",
                    onPressed: onReceive,
                    icon: Icon(Icons.download)),
                IconButton(
                    tooltip: "Send Funds",
                    onPressed: onSend,
                    icon: Icon(Icons.send)),
              ],
              bottom: TabBar(
                tabs: [
                  Tab(text: 'Transactions'),
                  Tab(text: 'Memos'),
                ],
              ),
            ),
            body: Padding(
              padding: const EdgeInsets.symmetric(horizontal: 16),
              child: Observer(builder: (context) {
                // make sure there is a dependency on transactions
                AppStoreBase.instance.transactions.length;

                return TabBarView(children: [
                  SingleChildScrollView(
                      child: Column(children: [
                    Text("Height"),
                    Gap(8),
                    Text(h.toString(), style: t.bodyLarge),
                    Gap(16),
                    Text("Balance"),
                    Gap(8),
                    if (b != null)
                      Row(children: [
                        Text("T: ${zatToString(b.field0[0])}"),
                        const Gap(8),
                        Text("S: ${zatToString(b.field0[1])}"),
                        const Gap(8),
                        Text("O: ${zatToString(b.field0[2])}"),
                      ]),
                    Gap(8),
                    if (b != null)
                      Text(
                          "\u2211: ${zatToString(b.field0[0] + b.field0[1] + b.field0[2])}"),
                    Gap(16),
                    ...showTxHistory(AppStoreBase.instance.transactions),
                  ])),
                  SingleChildScrollView(
                      child: Column(
                    children: [
                      ...showMemos(AppStoreBase.instance.memos),
                    ],
                  ))
                ]);
              }),
            )));
  }

  void onSync() async {
    try {
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
          final h = await getDbHeight();
          await AppStoreBase.instance.loadAccounts();
          setState(() {
            poolBalance = b;
            height = h;
          });
        },
      );
    } on AnyhowException catch (e) {
      if (mounted)
        await showException(context, e.message);
    }
  }

  void onRewind() async {
    final dbHeight = await getDbHeight();
    await rewindSync(height: dbHeight - 60);
    final h = await getDbHeight();
    setState(() => height = h);
  }

  void onReceive() async {
    await GoRouter.of(context).push("/receive");
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
      await AppStoreBase.instance.loadAccounts();
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
      await AppStoreBase.instance.loadAccounts();
      setState(() {});
    }
  }

  void onEditBirth(String? birth) async {
    if (birth != null && birth.isNotEmpty) {
      account = account.copyWith(birth: int.parse(birth));
      await updateAccount(
          update: AccountUpdate(
              coin: account.coin, id: account.id, birth: int.parse(birth)));
      await AppStoreBase.instance.loadAccounts();
      setState(() {});
    }
  }

  void onEditEnabled(v) async {
    account = account.copyWith(enabled: v);
    await updateAccount(
        update: AccountUpdate(coin: account.coin, id: account.id, enabled: v));
    await AppStoreBase.instance.loadAccounts();
    setState(() {});
  }

  void onEditHidden(v) async {
    account = account.copyWith(hidden: v);
    await updateAccount(
        update: AccountUpdate(coin: account.coin, id: account.id, hidden: v));
    await AppStoreBase.instance.loadAccounts();
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
      physics: const NeverScrollableScrollPhysics(),
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
    ListView.builder(
      shrinkWrap: true,
      physics: const NeverScrollableScrollPhysics(),
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

void onOpenLog(BuildContext context) async {
  await GoRouter.of(context).push("/log");
}
