import 'dart:async';

import 'package:awesome_dialog/awesome_dialog.dart';
import 'package:convert/convert.dart';
import 'package:flutter/material.dart';
import 'package:flutter_form_builder/flutter_form_builder.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge.dart';
import 'package:gap/gap.dart';
import 'package:go_router/go_router.dart';
import 'package:zkool/main.dart';
import 'package:zkool/src/rust/api/vote.dart';
import 'package:zkool/store.dart';
import 'package:zkool/utils.dart';
import 'package:async/async.dart';
import 'package:zkool/widgets/input_amount.dart';

final contextMemoizer = AsyncMemoizer<Context>();

class VotePage1 extends ConsumerStatefulWidget {
  const VotePage1({super.key});

  @override
  ConsumerState<VotePage1> createState() => VotePage1State();
}

class VotePage1State extends ConsumerState<VotePage1> {
  final formKey = GlobalKey<FormBuilderState>();
  bool scanning = false;
  double? progress;
  ElectionPropsPub? election;
  String? url;
  int? account;

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: Text("Vote")),
      body: SingleChildScrollView(
          child: Padding(
              padding: EdgeInsetsGeometry.symmetric(horizontal: 8),
              child: FormBuilder(
                key: formKey,
                child: Column(children: [
                  FormBuilderTextField(
                    key: ValueKey(url),
                    name: "url",
                    decoration: InputDecoration(label: Text("Election URL")),
                  ),
                  Gap(16),
                  ElevatedButton(
                    onPressed: onNext,
                    child: Text("Next"),
                  ),
                ]),
              ))),
    );
  }

  void onNext() async {
    try {
      final form = formKey.currentState!;
      if (!form.validate()) return;
      final fields = form.fields;
      final url = fields["url"]!.value as String;

      final c = ref.read(coinContextProvider);
      final e = ref.read(electionProvider.notifier);
      AwesomeDialog? dialog;
      try {
        dialog = await showMessage(context, "Please wait while we mint voting tokens from your orchard funds", dismissable: false);
        await e.fetch(c.account, url);
      } finally {
        dialog?.dismiss();
      }
      await GoRouter.of(context).pushReplacement('/vote/page2');
    } on AnyhowException catch (e) {
      await showException(context, e.message);
    }
  }
}

class VotePage2 extends ConsumerStatefulWidget {
  const VotePage2({super.key});

  @override
  ConsumerState<VotePage2> createState() => VotePage2State();
}

class VotePage2State extends ConsumerState<VotePage2> {
  final formKey = GlobalKey<FormBuilderState>();
  String amount = "";
  String? balance;
  late ElectionPropsPub election;
  late List<int> answers;

  @override
  void initState() {
    super.initState();
    final e = ref.read(electionProvider)!;
    election = e.election!;
    answers = [for (var _ in election.questions) 1];
    Future(() async {
      balance = zatToString(await refresh(context, ref));
      amount = balance!;
      setState(() {});
    });
  }

  @override
  Widget build(BuildContext context) {
    final t = Theme.of(context).textTheme;

    final max = balance?.let((b) => stringToZat(b));
    return Scaffold(
      appBar: AppBar(title: Text("Vote"), actions: [
        IconButton(
          onPressed: onQuit,
          icon: Icon(Icons.cancel),
          tooltip: "Quit",
        ),
        IconButton(
          onPressed: onDelegate,
          icon: Icon(Icons.forward),
          tooltip: "Delegate",
        ),
        IconButton(
          onPressed: onVote,
          icon: Icon(Icons.how_to_vote),
          tooltip: "Vote",
        )
      ]),
      body: SingleChildScrollView(
          child: Padding(
              padding: EdgeInsetsGeometry.symmetric(horizontal: 8),
              child: FormBuilder(
                  key: formKey,
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(election.name, style: t.headlineSmall),
                      Text(election.caption),
                      Gap(16),
                      InputAmount(
                        name: "amount",
                        showFx: false,
                        initialValue: balance,
                        onChanged: (v) => setState(() => amount = v!),
                        max: max,
                      ),
                      Gap(8),
                      if (balance != null) Center(child: Text("Max available: $balance", style: t.bodySmall)),
                      Gap(16),
                      ...[
                        for (var (i, c) in election.questions.indexed)
                          QuestionWidget(
                            c,
                            answers[i],
                            onChanged: (v) => answers[i] = v!,
                          )
                      ]
                    ],
                  )))),
    );
  }

  void onQuit() async {
    final confirmed = await confirmDialog(context, title: "Close Election?", message: "Do you want to quit this election?");
    if (confirmed) {
      final c = ref.read(coinContextProvider);
      await deleteElection(c: c);
      GoRouter.of(context).pop();
    }
  }

  void onDelegate() async {
    await GoRouter.of(context).push("/vote/delegate", extra: amount);
  }

  void onVote() async {
    if (!formKey.currentState!.validate()) return;
    final confirmed = await confirmDialog(context, title: "Vote", message: "Do you want to submit this vote of $amount?");
    if (!confirmed) return;
    final c = ref.read(coinContextProvider);
    final voteContent = hex.encode(answers);
    AwesomeDialog? dialog;
    try {
      dialog = await showMessage(context, "Please wait while we compute the ballot", dismissable: false);

      logger.i("amount $amount");
      final id = await vote(
        idAccount: c.account,
        vote: voteContent,
        amount: stringToZat(amount),
        c: c,
      );
      showSnackbar("Vote $id submitted");
    } finally {
      dialog?.dismiss();
    }
    final newBalance = await refresh(context, ref);
    amount = zatToString(newBalance);
    balance = amount;
    setState(() {});
  }
}

class QuestionWidget extends StatelessWidget {
  final QuestionProp question;
  final int answer;
  final void Function(int?) onChanged;
  const QuestionWidget(this.question, this.answer, {super.key, required this.onChanged});

  @override
  Widget build(BuildContext context) {
    final t = Theme.of(context).textTheme;
    return Card(
        child: Padding(
            padding: EdgeInsetsGeometry.all(8),
            child: Column(
              children: [
                Text(question.title, style: t.bodyLarge),
                Gap(8),
                Align(alignment: AlignmentGeometry.bottomLeft, child: Text(question.subtitle)),
                FormBuilderRadioGroup<int>(
                  name: question.title,
                  options: [
                    for (var (i, a) in question.answers.indexed)
                      FormBuilderFieldOption(
                        value: i + 1,
                        child: Text(a),
                      )
                  ],
                  initialValue: answer,
                  onChanged: onChanged,
                )
              ],
            )));
  }
}

class VoteDelegatePage extends ConsumerStatefulWidget {
  final String amount;
  const VoteDelegatePage(this.amount, {super.key});

  @override
  ConsumerState<ConsumerStatefulWidget> createState() => VoteDelegateState();
}

class VoteDelegateState extends ConsumerState<VoteDelegatePage> {
  final formKey = GlobalKey<FormBuilderState>();
  String? address;

  @override
  void initState() {
    super.initState();
    Future(() async {
      final c = ref.read(coinContextProvider);
      address = await getElectionAddress(idAccount: c.account, c: c);
      setState(() {});
    });
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
        appBar: AppBar(
          title: Text("Delegate"),
          actions: [IconButton(onPressed: onOK, icon: Icon(Icons.check))],
        ),
        body: SingleChildScrollView(
            child: Padding(
                padding: EdgeInsetsGeometry.symmetric(horizontal: 8),
                child: FormBuilder(
                    key: formKey,
                    child: Column(children: [
                      ListTile(
                        title: Text("Your address"),
                        subtitle: CopyableText(address ?? ""),
                      ),
                      Gap(16),
                      FormBuilderTextField(
                        name: "address",
                        decoration: InputDecoration(label: Text("Recipient")),
                      )
                    ])))));
  }

  void onOK() async {
    try {
      final c = ref.read(coinContextProvider);
      final recipient = formKey.currentState!.fields["address"]!.value as String;
      final confirmed = await confirmDialog(context, title: "Delegate", message: "Sending ${widget.amount} votes to $recipient");
      if (confirmed) {
        final a = stringToZat(widget.amount);
        AwesomeDialog? dialog;
        try {
          dialog = await showMessage(context, "Please wait while we compute the ballot", dismissable: false);
          final id = await delegate(idAccount: c.account, amount: a, address: recipient, c: c);
          showSnackbar("Delegation $id submitted");
        } finally {
          dialog?.dismiss();
        }
      }
      await refresh(context, ref);
    } on AnyhowException catch (e) {
      if (!mounted) return;
      await showException(context, e.message);
    }
    if (!mounted) return;
    GoRouter.of(context).pop();
  }
}

Future<BigInt> refresh(
  BuildContext context,
  WidgetRef ref,
) async {
  final c = ref.read(coinContextProvider);
  AwesomeDialog? dialog;
  try {
    dialog = await showMessage(
      context,
      "Synchronizing",
      dismissable: false,
    );
    await Future.delayed(Duration(seconds: 5));
    await scanBallots(
      idAccount: c.account,
      c: c,
    );
    final b = await getBalance(
      idAccount: c.account,
      c: c,
    );
    return b;
  } finally {
    dialog?.dismiss();
  }
}
