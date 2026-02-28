import 'dart:typed_data';

import 'package:convert/convert.dart';
import 'package:flutter/material.dart';
import 'package:flutter_form_builder/flutter_form_builder.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:gap/gap.dart';
import 'package:go_router/go_router.dart';
import 'package:zkool/main.dart';
import 'package:zkool/pages/account.dart';
import 'package:zkool/src/rust/api/coin.dart';
import 'package:zkool/src/rust/api/vote.dart';
import 'package:zkool/store.dart';
import 'package:zkool/utils.dart';
import 'package:async/async.dart';

part 'vote.freezed.dart';

final contextMemoizer = AsyncMemoizer<Context>();

class VotePage1 extends ConsumerStatefulWidget {
  const VotePage1({super.key});

  ConsumerState<VotePage1> createState() => VotePage1State();
}

class VotePage1State extends ConsumerState<VotePage1> {
  VoteContext? vc;
  final formKey = GlobalKey<FormBuilderState>();
  bool scanning = false;
  double? progress;

  @override
  void initState() {
    super.initState();
    Future(() async {
      final c = ref.read(coinContextProvider);
      final context = await contextMemoizer.runOnce(() => getElectionContext(c: c));
      final eid = await getElectionId(c: context);
      final election = (eid.hash.isNotEmpty) ? await fetchElection(url: eid.url!, hash: eid.hash, c: context) : null;
      final voteContext = VoteContext(account: c.account, context: context, id: eid, election: election);
      setState(() => vc = voteContext);
    });
  }

  @override
  Widget build(BuildContext context) {
    final h = vc?.id.hash.let((h) => hex.encode(h));
    final url = vc?.id.url;
    return Scaffold(
      appBar: AppBar(
        title: Text("Vote"),
      ),
      body: (vc == null)
          ? showLoading("Vote Context")
          : SingleChildScrollView(
              child: Padding(
                  padding: EdgeInsetsGeometry.symmetric(horizontal: 8),
                  child: FormBuilder(
                    key: formKey,
                    child: Column(
                      children: [
                        FormBuilderTextField(
                          name: "url",
                          decoration: InputDecoration(label: Text("Election URL")),
                          initialValue: url,
                          readOnly: url != null,
                        ),
                        FormBuilderTextField(
                          name: "hash",
                          decoration: InputDecoration(label: Text("Hash")),
                          initialValue: h,
                          readOnly: h!.isNotEmpty,
                        ),
                        Gap(16),
                        ElevatedButton(onPressed: onNext, child: Text("Next")),
                      ],
                    ),
                  ))),
    );
  }

  void onNext() async {
    final vc = this.vc!;
    if (vc.election != null)
      await scan(vc);
    else {
      final form = formKey.currentState!;
      if (!form.validate()) return;
      final fields = form.fields;
      final url = fields["url"]!.value as String;
      final hashStr = fields["hash"]!.value as String;
      logger.i("$url $hashStr");
      final hash = hex.decode(hashStr); // TODO: Validate

      final election = await fetchElection(url: url, hash: hash, c: vc.context);
      final eid = await getElectionId(c: vc.context);

      try {
        setState(() => scanning = true);
        // await scanVotes(c: widget.ec);
      } finally {
        setState(() => scanning = false);
      }

      await scan(
        vc.copyWith(
          id: eid,
          election: election,
        ),
      );
    }
  }

  Future<void> scan(VoteContext vc) async {
    final t = Theme.of(context).textTheme;
    final progressSub = scanVotes(hash: hex.encode(vc.id.hash), idAccount: vc.account, c: vc.context);
    var progress = ValueNotifier<double>(0.0);
    progressSub.listen((p) {
      setState(() => progress.value = p.toDouble());
    }, onDone: () {
      setState(() {
        GoRouter.of(context).pushReplacement("/vote/page2/0", extra: vc);
      });
    });
    await showDialog(
      context: context,
      barrierDismissible: false,
      builder: (context) => AlertDialog(
        title: Text("Please wait while we scan for your voting funds", style: t.headlineSmall),
        content: ValueListenableBuilder<double>(valueListenable: progress, builder: (context, progress, _) => LinearProgressIndicator(value: progress * 0.01)),
      ),
    );
  }
}

class VotePage2 extends ConsumerStatefulWidget {
  final int idxQuestion;
  final VoteContext voteContext;
  const VotePage2(this.idxQuestion, this.voteContext, {super.key});

  @override
  ConsumerState<VotePage2> createState() => VotePage2State();
}

class VotePage2State extends ConsumerState<VotePage2> {
  String? balance;
  late final question = widget.voteContext.election!.questions[0];
  late List<int> answers = [for (var _ in question.choices) 1];

  @override
  void initState() {
    super.initState();
    Future(() async {
      final vc = widget.voteContext;
      final b = await getBalance(
        hash: hex.encode(vc.id.hash),
        idAccount: vc.account,
        idxQuestion: widget.idxQuestion,
        c: vc.context,
      );
      setState(() => balance = zatToString(b));
    });
  }

  @override
  Widget build(BuildContext context) {
    final t = Theme.of(context).textTheme;

    return Scaffold(
      appBar: AppBar(title: Text("Vote"),
      actions: [
        IconButton(onPressed: onVote, icon: Icon(Icons.how_to_vote))
      ]),
      body: SingleChildScrollView(
          child: Padding(
              padding: EdgeInsetsGeometry.symmetric(horizontal: 8),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  if (balance != null) Center(child: Text(balance!, style: t.headlineMedium)),
                  Gap(32),
                  Text(question.title, style: t.headlineSmall),
                  Text(question.subtitle),
                  Gap(16),
                  ...[for (var (i, c) in question.choices.indexed)
                    ChoiceWidget(c, answers[i], onChanged: (v) => answers[i] = v!,)]
                ],
              ))),
    );
  }

  void onVote() async {
    logger.i(answers);
  }

  VoteContext get vc => widget.voteContext;
}

class ChoiceWidget extends StatelessWidget {
  final ChoiceProp choice;
  final int answer;
  final void Function(int?) onChanged;
  const ChoiceWidget(this.choice, this.answer, {super.key, required this.onChanged});

  @override
  Widget build(BuildContext context) {
    final t = Theme.of(context).textTheme;
    return Card(
        child: Padding(
            padding: EdgeInsetsGeometry.all(8),
            child: Column(
              children: [
                if (choice.title != null) Text(choice.title!, style: t.bodyLarge),
                Gap(8),
                if (choice.subtitle != null) Align(alignment: AlignmentGeometry.bottomLeft, child: Text(choice.subtitle!)),
                FormBuilderRadioGroup<int>(
                  name: choice.title!,
                  options: [for (var (i, a) in choice.answers.indexed) FormBuilderFieldOption(value: i + 1, child: Text(a),)],
                  initialValue: answer,
                  onChanged: onChanged,
                )
              ],
            )));
  }
}

@freezed
sealed class VoteContext with _$VoteContext {
  const VoteContext._();

  factory VoteContext({
    required int account,
    required ElectionId id,
    required ElectionPropsPub? election,
    required Context context,
  }) = _VoteContext;
}
