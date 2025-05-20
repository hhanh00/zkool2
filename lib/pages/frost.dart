import 'dart:async';

import 'package:easy_stepper/easy_stepper.dart';
import 'package:flutter/material.dart';
import 'package:flutter_form_builder/flutter_form_builder.dart';
import 'package:form_builder_validators/form_builder_validators.dart';
import 'package:gap/gap.dart';
import 'package:go_router/go_router.dart';
import 'package:zkool/src/rust/api/frost.dart';
import 'package:zkool/src/rust/api/network.dart';
import 'package:zkool/src/rust/api/pay.dart';
import 'package:zkool/store.dart';

class FrostPage1 extends StatefulWidget {
  final PcztPackage pczt;
  const FrostPage1(this.pczt, {super.key});

  @override
  State<FrostPage1> createState() => FrostPage1State();
}

Widget buildFrostPage(BuildContext context,
    {required int index, required Widget child}) {
  return Scaffold(
      appBar: AppBar(title: const Text("Frost Multi Party Signature")),
      body: CustomScrollView(slivers: [
        PinnedHeaderSliver(child: FrostSteps(currentIndex: index)),
        SliverPadding(
            padding: EdgeInsets.symmetric(horizontal: 8),
            sliver: SliverToBoxAdapter(child: child)),
      ]));
}

class FrostPage1State extends State<FrostPage1> {
  final formKey = GlobalKey<FormBuilderState>();
  final frostParams = AppStoreBase.instance.frostParams!;
  late final accounts = AppStoreBase.instance.accounts.where((e) => !e.hidden);

  @override
  void initState() {
    super.initState();
    Future(() async {
      final signing = await isSigningInProgress();
      if (signing) {
        if (context.mounted) {
          WidgetsBinding.instance.addPostFrameCallback((_) {
            GoRouter.of(context).pushReplacement("/frost2");
          });
        }
      }
    });
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text("Frost Multi Party Signature")),
      body:
      FormBuilder(
        key: formKey,
        child: Column(children: [
          ListTile(
            title: Text("Your Participant ID"),
            subtitle: Text(frostParams.id.toString()),
          ),
          Padding(
              padding: EdgeInsets.symmetric(horizontal: 16),
              child: FormBuilderDropdown(
                  name: "coordinator",
                  decoration: const InputDecoration(
                    labelText: "ID of the coordinator",
                  ),
                  initialValue: 1,
                  items: List.generate(
                    5,
                    (i) => DropdownMenuItem(
                      value: i + 1,
                      child: Text("${i + 1}"),
                    ),
                  ))),
          Padding(
              padding: EdgeInsets.symmetric(horizontal: 16),
              child: FormBuilderDropdown(
                name: "account",
                decoration: const InputDecoration(labelText: "Funding Account"),
                items: accounts
                    .map((a) => DropdownMenuItem(
                          value: a.id,
                          child: Text(a.name),
                        ))
                    .toList(),
                validator: FormBuilderValidators.required(),
              )),
          Gap(16),
          ElevatedButton.icon(
              onPressed: onNext,
              label: Text("Next"),
              icon: Icon(Icons.arrow_forward))
        ])));
  }

  void onNext() async {
    final form = formKey.currentState!;
    if (form.saveAndValidate()) {
      final coordinator = form.fields["coordinator"]!.value as int;
      final fundingAccount = form.fields["account"]!.value as int;
      await initSign(
          pczt: widget.pczt,
          coordinator: coordinator,
          fundingAccount: fundingAccount);
      if (!mounted) return;
      await GoRouter.of(context).pushReplacement("/frost2");
    }
  }
}

class FrostPage2 extends StatefulWidget {
  const FrostPage2({super.key});

  @override
  State<FrostPage2> createState() => FrostPage2State();
}

class FrostPage2State extends State<FrostPage2> {
  String message = "";
  Timer? timer;
  int currentIndex = 0;

  @override
  void initState() {
    super.initState();
    runFrost();
    timer = Timer.periodic(Duration(seconds: 30), (_) async {
      runFrost();
    });
  }

  @override
  void dispose() {
    timer?.cancel();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final t = Theme.of(context).textTheme;
    return buildFrostPage(context,
        index: currentIndex,
        child: Column(children: [
          Text(message, style: t.bodyLarge)
        ]));
  }

  int? currentHeight;

  void runFrost() async {
    final h = await getCurrentHeight();
    if (currentHeight != null && currentHeight == h) return;
    currentHeight = h;
    final accounts = AppStoreBase.instance.accounts
        .where((e) => e.enabled)
        .map((e) => e.id)
        .toList();
    await AppStoreBase.instance.startSynchronize(accounts,
        int.parse(AppStoreBase.instance.actionsPerSync));

    final status = doSign();
    status.listen((s) {
      if (s is SigningStatus_WaitingForCommitments) {
        setState(() {
          message = "Waiting for other participants to send their commitments";
          currentIndex = 1; // coordinator
        });
      } else if (s is SigningStatus_SendingCommitment) {
        setState(() {
          message = "Sending our commitments to the coordinator";
          currentIndex = 1; // other
        });
      } else if (s is SigningStatus_SendingSigningPackage) {
        setState(() {
          message = "Broadcasting the signing package to all participants";
          currentIndex = 2; // coordinator
        });
      } else if (s is SigningStatus_WaitingForSigningPackage) {
        setState(() {
          message = "Waiting for the signing package from the coordinator";
          currentIndex = 2; // other
        });
      } else if (s is SigningStatus_SendingSignatureShare) {
        setState(() {
          message = "Sending our signature share to the coordinator";
          currentIndex = 3; // other
        });
      } else if (s is SigningStatus_WaitingForSignatureShares) {
        setState(() {
          message = "Waiting for the signature share from the other participants";
          currentIndex = 2; // coordinator
        });
      } else if (s is SigningStatus_PreparingTransaction) {
        setState(() {
          message = "Assembling the transaction";
          currentIndex = 3; // coordinator
        });
      } else if (s is SigningStatus_SendingTransaction) {
        setState(() {
          message = "Sending the transaction to the network";
          currentIndex = 3; // coordinator
        });
      } else if (s is SigningStatus_TransactionSent) {
        setState(() {
          message = "TX ID: ${s.field0}";
          currentIndex = 3; // coordinator
        });
      }
    });
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
          title: "Commitments",
          icon: Icon(Icons.receipt_long),
        ),
        EasyStep(
          title: "Signatures",
          icon: Icon(Icons.draw),
        ),
        EasyStep(
          title: "Finalize",
          icon: Icon(Icons.flag),
        ),
      ],
    );
  }
}
