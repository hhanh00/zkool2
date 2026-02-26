import 'dart:typed_data';

import 'package:convert/convert.dart';
import 'package:flutter/material.dart';
import 'package:flutter_form_builder/flutter_form_builder.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:gap/gap.dart';
import 'package:zkool/main.dart';
import 'package:zkool/src/rust/api/vote.dart';
import 'package:zkool/store.dart';
import 'package:zkool/utils.dart';

class VotePage1 extends ConsumerStatefulWidget {
  @override
  ConsumerState<VotePage1> createState() => VotePage1State();
}

class VotePage1State extends ConsumerState<VotePage1> {
  final formKey = GlobalKey<FormBuilderState>();

  @override
  Widget build(BuildContext context) {
    ElectionId? eid;
    ElectionPropsPub? election;
    try {
      final electionIdAV = ref.watch(electionIdProvider);
      ensureAV(context, electionIdAV);
      final id = electionIdAV.requireValue;
      if (id.hash.isNotEmpty) {
        eid = id;
        final e = ref.watch(electionProvider);
        ensureAV(context, e);
        election = e.requireValue;
      }
    } on Widget catch (w) {
      return w;
    }

    final h = eid?.let((h) => hex.encode(h.hash));
    final url = eid?.url;
    return Scaffold(
      appBar: AppBar(
        title: Text("Vote"),
      ),
      body: SingleChildScrollView(
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
                      readOnly: h != null,
                    ),
                    if (h == null) ...[
                      Gap(16),
                      ElevatedButton(onPressed: onSelect, child: Text("Select Election"))
                    ],
                  ],
                ),
              ))),
    );
  }

  void onSelect() async {
    final form = formKey.currentState!;
    if (!form.validate()) return;
    final fields = form.fields;
    final url = fields["url"]!.value as String;
    final hashStr = fields["hash"]!.value as String;
    logger.i("$url $hashStr");
    final hash = hex.decode(hashStr); // TODO: Validate

    final c = ref.read(coinContextProvider);
    final e = await fetchElection(url: url, hash: hash, c: c);
    ref.invalidate(electionIdProvider);
    await getElection(c: c);
    logger.i("Election: ${e.start} ${e.end}");
  }
}
