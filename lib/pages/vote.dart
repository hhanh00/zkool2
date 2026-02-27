import 'dart:typed_data';

import 'package:convert/convert.dart';
import 'package:flutter/material.dart';
import 'package:flutter_form_builder/flutter_form_builder.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:gap/gap.dart';
import 'package:go_router/go_router.dart';
import 'package:zkool/main.dart';
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
      body: (vc == null) ? showLoading("Vote Context") : SingleChildScrollView(
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
                    Gap(16),
                    if (scanning) Text("Please wait while we compute your voting power"),
                  ],
                ),
              ))),
    );
  }

  onNext() async {
    final vc = this.vc!;
    if (vc.election != null)
      await GoRouter.of(context).push(
        "/vote/page2",
        extra: vc,
      );
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

      await GoRouter.of(context).push(
        "/vote/page2",
        extra: vc.copyWith(
          id: eid,
          election: election,
        ),
      );
    }
  }
}

class VotePage2 extends ConsumerStatefulWidget {
  final VoteContext voteContext;
  const VotePage2(this.voteContext, {super.key});

  @override
  ConsumerState<VotePage2> createState() => VotePage2State();
}

class VotePage2State extends ConsumerState<VotePage2> {
  @override
  void initState() {
    super.initState();
    Future(() async {
      await scanVotes(hash: hex.encode(vc.id.hash), idAccount: vc.account, c: vc.context);
    });
  }

  @override
  Widget build(BuildContext context) {
    final t = Theme.of(context).textTheme;
    final question = vc.election!.questions[0];

    return Scaffold(
      appBar: AppBar(title: Text("Vote")),
      body: SingleChildScrollView(
          child: Padding(
              padding: EdgeInsetsGeometry.symmetric(horizontal: 8),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(question.title, style: t.headlineSmall),
                  Text(question.subtitle),
                ],
              ))),
    );
  }

  VoteContext get vc => widget.voteContext;
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
