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
  void initState() {
    super.initState();
    Future(() async {
      try {
        final c = ref.read(coinContextProvider);
        final (url, account) = await getElectionUrl(c: c);
        setState(() {
          this.url = url ?? "";
          this.account = account;
        });
        if (url != null && account == c.account) await loadElection(url);
      } on AnyhowException catch (e) {
        if (!mounted) return;
        await showException(context, e.message);
      }
    });
  }

  @override
  Widget build(BuildContext context) {
    final t = Theme.of(context);
    final c = ref.watch(coinContextProvider);
    return Scaffold(
      appBar: AppBar(title: Text("Vote"), actions: [IconButton(onPressed: onQuit, icon: Icon(Icons.delete))]),
      body: SingleChildScrollView(
          child: Padding(
              padding: EdgeInsetsGeometry.symmetric(horizontal: 8),
              child: FormBuilder(
                key: formKey,
                child: Column(
                  children: [
                    FormBuilderTextField(
                      key: ValueKey(url),
                      name: "url",
                      decoration: InputDecoration(label: Text("Election URL")),
                      initialValue: url,
                      readOnly: url?.isNotEmpty == true,
                      onSubmitted: loadElection,
                    ),
                    Gap(16),
                    if (account != null && account != c.account) ...[
                      Text("Election was opened in another account",
                          style: t.textTheme.bodyLarge!.copyWith(
                            color: t.colorScheme.tertiary,
                          )),
                      Gap(8),
                    ],
                    ElevatedButton(onPressed: election != null ? onNext : null, child: Text("Next")),
                    Gap(16),
                  ],
                ),
              ))),
    );
  }

  void onQuit() async {
    final confirmed = await confirmDialog(context,
        title: "Leave Election", message: "Are you sure you want to leave this election? This will delete the cached election data");
    if (confirmed) {
      final c = ref.read(coinContextProvider);
      await deleteElection(c: c);
      GoRouter.of(context).pop();
    }
  }

  void onNext() async {
    if (election != null)
      await scan();
    else {
      final form = formKey.currentState!;
      if (!form.validate()) return;
      final fields = form.fields;
      final url = fields["url"]!.value as String;
      logger.i("url: $url");

      final c = ref.read(coinContextProvider);
      election = await fetchElection(url: url, account: c.account, c: c);

      await scan();
    }
  }

  Future<void> loadElection(String? url) async {
    if (url == null) return;
    final c = ref.read(coinContextProvider);
    final election = await fetchElection(url: url, account: c.account, c: c);
    setState(() {
      this.election = election;
    });
  }

  Future<void> scan() async {
    try {
      final t = Theme.of(context).textTheme;
      final c = ref.read(coinContextProvider);
      final progressSub = scanVotes(idAccount: c.account, c: c);
      var progress = ValueNotifier<double>(0.0);
      progressSub.listen((p) {
        setState(() => progress.value = p.toDouble());
      }, onDone: () async {
        setState(() {
          GoRouter.of(context).pushReplacement("/vote/page2", extra: election!);
        });
      });
      await showDialog(
        context: context,
        barrierDismissible: false,
        builder: (context) => AlertDialog(
          title: Text("Please wait while we scan for your voting funds", style: t.headlineSmall),
          content:
              ValueListenableBuilder<double>(valueListenable: progress, builder: (context, progress, _) => LinearProgressIndicator(value: progress * 0.01)),
        ),
      );
    } on AnyhowException catch (e) {
      await showException(context, e.message);
    }
  }
}

class VotePage2 extends ConsumerStatefulWidget {
  final ElectionPropsPub election;
  const VotePage2(this.election, {super.key});

  @override
  ConsumerState<VotePage2> createState() => VotePage2State();
}

class VotePage2State extends ConsumerState<VotePage2> {
  final formKey = GlobalKey<FormBuilderState>();
  String amount = "";
  String? balance;
  late List<int> answers = [for (var _ in widget.election.questions) 1];

  void init() async {
    try {
      final newBalance = await refresh(context, ref);
      amount = zatToString(newBalance);
      balance = amount;
      setState(() {});
    } on AnyhowException catch (e) {
      await showException(context, e.message);
    }
  }

  @override
  Widget build(BuildContext context) {
    final t = Theme.of(context).textTheme;

    if (balance == null) {
      WidgetsBinding.instance.addPostFrameCallback((_) => init());
    }

    final max = balance?.let((b) => stringToZat(b));
    final election = widget.election;
    return Scaffold(
      appBar: AppBar(title: Text("Vote"), actions: [
        IconButton(
          onPressed: onDelegate,
          icon: Icon(Icons.forward),
        ),
        IconButton(
          onPressed: onVote,
          icon: Icon(Icons.how_to_vote),
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
                        key: ValueKey(balance),
                        name: "amount",
                        showFx: false,
                        initialValue: amount,
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

  void onDelegate() async {
    await GoRouter.of(context).push("/vote/delegate", extra: amount);
  }

  void onVote() async {
    if (!formKey.currentState!.validate()) return;
    final confirmed = await confirmDialog(context, title: "Vote", message: "Do you want to submit this vote?");
    if (!confirmed) return;
    final c = ref.read(coinContextProvider);
    final voteContent = hex.encode(answers);
    AwesomeDialog? dialog;
    try {
      dialog = await showMessage(context, "Please wait while we compute the ballot", dismissable: false);

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
    await Future.delayed(Duration(seconds: 2));
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
