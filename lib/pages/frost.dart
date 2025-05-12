import 'package:easy_stepper/easy_stepper.dart';
import 'package:flutter/material.dart';
import 'package:flutter_form_builder/flutter_form_builder.dart';
import 'package:form_builder_validators/form_builder_validators.dart';
import 'package:gap/gap.dart';
import 'package:go_router/go_router.dart';
import 'package:zkool/main.dart';
import 'package:zkool/src/rust/api/frost.dart';
import 'package:zkool/store.dart';
import 'package:zkool/validators.dart';

Widget buildDKGPage(BuildContext context,
    {required int index, required Widget child}) {
  return Scaffold(
      appBar: AppBar(
        title: const Text("Distributed Key Generation"),
      ),
      body: CustomScrollView(slivers: [
        PinnedHeaderSliver(child: FrostSteps(currentIndex: index)),
        SliverPadding(
            padding: EdgeInsets.symmetric(horizontal: 8),
            sliver: SliverToBoxAdapter(child: child)),
      ]));
}

class DKGPage1 extends StatefulWidget {
  const DKGPage1({super.key});

  @override
  State<StatefulWidget> createState() => DKGPage1State();
}

class DKGPage1State extends State<DKGPage1> {
  final formKey = GlobalKey<FormBuilderState>();
  late final accounts = AppStoreBase.instance.accounts.where((e) => !e.hidden);

  @override
  void initState() {
    super.initState();
    Future(() async {
      logger.i("DKGPage1 initState");
      final package = await loadFrost();
      if (package != null) {
        logger.i("package: $package");
        final userInputCompleted = await package.userInputCompleted();
        WidgetsBinding.instance.addPostFrameCallback((_) async {
          logger.i("context mounted: ${context.mounted}");
          if (!context.mounted) return;
          GoRouter.of(context).pop();
          if (!userInputCompleted) {
            await GoRouter.of(context).push("/dkg/step2", extra: package);
          } else {
            await GoRouter.of(context).push("/dkg/step3", extra: package);
          }
        });
      }
    });
  }

  @override
  Widget build(BuildContext context) {
    return buildDKGPage(context, index: 0, child: FormBuilder(
        key: formKey,
        child: Column(
          children: [
            FormBuilderTextField(
              name: "name",
              decoration: const InputDecoration(labelText: "Name"),
              validator: FormBuilderValidators.required(),
            ),
            FormBuilderDropdown(
              name: "participants",
              decoration:
                  const InputDecoration(labelText: "Number of Participants"),
              initialValue: 2,
              items: List.generate(
                4,
                (i) => DropdownMenuItem(
                  value: i + 2,
                  child: Text("${i + 2}"),
                ),
              ),
            ),
            FormBuilderDropdown(
              name: "id",
              decoration:
                  const InputDecoration(labelText: "Your Participant ID"),
              initialValue: 1,
              items: List.generate(
                5,
                (i) => DropdownMenuItem(
                  value: i + 1,
                  child: Text("${i + 1}"),
                ),
              ),
            ),
            FormBuilderDropdown(
                name: "threshold",
                decoration: const InputDecoration(
                    labelText: "Number of Signers Required (Threshold)"),
                initialValue: 2,
                items: List.generate(
                  4,
                  (i) => DropdownMenuItem(
                    value: i + 2,
                    child: Text("${i + 2}"),
                  ),
                ),
                validator: (v) {
                  final n = formKey.currentState?.fields["participants"]!.value
                      as int;
                  if (v! > n) return "Threshold must be less than participants";
                  return null;
                }),
            FormBuilderDropdown(
              name: "account",
              decoration: const InputDecoration(labelText: "Funding Account"),
              items: accounts
                  .map((a) => DropdownMenuItem(
                        value: a.id,
                        child: Text(a.name),
                      ))
                  .toList(),
              validator: FormBuilderValidators.required(),
            ),
            Gap(16),
            ElevatedButton.icon(
                onPressed: () => onNext(context),
                label: Text("Next"),
                icon: Icon(Icons.arrow_forward))
          ],
        )));
  }

  onNext(BuildContext context) async {
    final form = formKey.currentState!;
    if (form.saveAndValidate()) {
      final name = form.fields["name"]!.value as String;
      final participants = form.fields["participants"]!.value as int;
      final id = form.fields["id"]!.value as int;
      final threshold = form.fields["threshold"]!.value as int;
      final account = form.fields["account"]!.value as int;
      final frost = await newFrost(
        name: name,
        id: id,
        n: participants,
        t: threshold,
        fundingAccount: account,
      );
      if (!context.mounted) return;
      await GoRouter.of(context).push("/dkg/step2", extra: frost);
    }
  }
}

class DKGPage2 extends StatefulWidget {
  final FrostPackage package;
  const DKGPage2(this.package, {super.key});

  @override
  State<StatefulWidget> createState() => DKGPage2State();
}

class DKGPage2State extends State<DKGPage2> {
  late FrostPackage package = widget.package;
  final formKey = GlobalKey<FormBuilderState>();

  @override
  Widget build(BuildContext context) {
    logger.i("DKGPage2: $package");

    return buildDKGPage(context, index: 1, child: FormBuilder(
      key: formKey,
      child: Column(
      children: [...package.addresses.asMap().entries.map((kv) {
        final i = kv.key;
        final address = kv.value;

        return FormBuilderTextField(name: "$i",
            decoration: InputDecoration(labelText: "Address for Participant #${i + 1}"),
            initialValue: address,
            readOnly: i == package.id - 1,
            validator: FormBuilderValidators.compose([
              FormBuilderValidators.required(),
              validAddress,
            ]));
      }),
      Gap(16),
      ElevatedButton.icon(
          onPressed: () => onNext(context),
          label: Text("Next"),
          icon: Icon(Icons.arrow_forward))
      ]
    )));
  }

  onNext(BuildContext context) async {
    final form = formKey.currentState!;
    if (form.saveAndValidate()) {
      final addresses = List.generate(package.n, (i) {
        final address = form.fields["$i"]!.value as String;
        return address;
      });
      package = package.copyWith(addresses: addresses);
      await submitDkg(package: package);
      if (!context.mounted) return;
      GoRouter.of(context).go("/");
    }
  }
}

class DKGPage3 extends StatefulWidget {
  const DKGPage3({super.key});

  @override
  State<StatefulWidget> createState() => DKGPage3State();
}

class DKGPage3State extends State<DKGPage3> {
  FrostPackage? package;

  @override
  void initState() {
    super.initState();
    Future(() async {
      final package = (await loadFrost())!;
      final state = await package.toState();
      if (state == null) {
        logger.e("DKG state is incomplete");
        return;
      }
      await state.run();
      setState(() => this.package = package);
    });
  }

  @override
  Widget build(BuildContext context) {
    return buildDKGPage(context, index: 2, child: SizedBox.shrink());
  }
}

class FrostSteps extends StatelessWidget {
  final int currentIndex;

  const FrostSteps({super.key, required this.currentIndex});

  @override
  Widget build(BuildContext context) {
    return EasyStepper(
      activeStep: currentIndex,
      showLoadingAnimation: false,
      stepRadius: 20,
      fitWidth: false,
      steps: [
        EasyStep(
          title: "Participants",
          icon: Icon(Icons.people),
        ),
        EasyStep(
          title: "Mailbox",
          icon: Icon(Icons.mail),
        ),
        EasyStep(
          title: "Mailbox",
          icon: Icon(Icons.mail),
        ),
        EasyStep(
          title: "Distribute",
          lineText: "Share 1/2",
          icon: Icon(Icons.podcasts),
        ),
        EasyStep(
          title: "Distribute",
          lineText: "Share 2/2",
          icon: Icon(Icons.podcasts),
        ),
        EasyStep(
          title: "Finalize",
          icon: Icon(Icons.flag),
        ),
      ],
    );
  }
}
