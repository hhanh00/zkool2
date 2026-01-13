import 'dart:async';

import 'package:easy_stepper/easy_stepper.dart';
import 'package:flutter/material.dart';
import 'package:flutter_form_builder/flutter_form_builder.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge.dart';
import 'package:form_builder_validators/form_builder_validators.dart';
import 'package:gap/gap.dart';
import 'package:go_router/go_router.dart';
import 'package:showcaseview/showcaseview.dart';
import 'package:zkool/main.dart';
import 'package:zkool/src/rust/api/account.dart';
import 'package:zkool/src/rust/api/frost.dart';
import 'package:zkool/src/rust/api/network.dart';
import 'package:zkool/store.dart';
import 'package:zkool/utils.dart';
import 'package:zkool/validators.dart';

final nameID3 = GlobalKey();
final participantID = GlobalKey();
final pID = GlobalKey();
final thresholdID = GlobalKey();
final fundingID = GlobalKey();

Widget buildDKGPage(
  BuildContext context,
  WidgetRef ref, {
  required int index,
  required bool finished,
  required Widget child,
}) {
  return Scaffold(
    appBar: AppBar(
      title: const Text("Distributed Key Generation"),
      actions: [
        finished
            ? IconButton(
                onPressed: () => onClose(context),
                icon: const Icon(Icons.close),
              )
            : IconButton(
                onPressed: () => onCancel(context, ref),
                icon: const Icon(Icons.cancel),
              ),
      ],
    ),
    body: CustomScrollView(
      slivers: [
        PinnedHeaderSliver(child: DKGSteps(currentIndex: index)),
        SliverPadding(
          padding: EdgeInsets.symmetric(horizontal: 8),
          sliver: SliverToBoxAdapter(child: child),
        ),
      ],
    ),
  );
}

class DKGPage1 extends ConsumerStatefulWidget {
  const DKGPage1({super.key});

  @override
  ConsumerState<DKGPage1> createState() => DKGPage1State();
}

class DKGPage1State extends ConsumerState<DKGPage1> {
  late final c = ref.read(coinContextProvider);
  final formKey = GlobalKey<FormBuilderState>();
  List<Account> accounts = [];

  @override
  void initState() {
    super.initState();
    Future(() async {
      final accounts = (await ref.read(getAccountsProvider.future)).where((e) => !e.hidden).toList();
      final dkgInProgress = await hasDkgParams(c: c);
      if (dkgInProgress && mounted) {
        WidgetsBinding.instance.addPostFrameCallback((_) {
          GoRouter.of(context).pushReplacement("/dkg2");
        });
      }
      setState(() => this.accounts = accounts);
    });
  }

  void tutorial() async {
    tutorialHelper(
      context,
      "dkg",
      [nameID3, participantID, pID, thresholdID, fundingID],
    );
  }

  @override
  Widget build(BuildContext context) {
    final pinlock = ref.watch(lifecycleProvider);
    if (pinlock.value ?? false) return PinLock();

    Future(tutorial);

    return Scaffold(
      appBar: AppBar(
        title: const Text("Distributed Key Generation"),
        actions: [
          IconButton(
            onPressed: () => onCancel(context, ref),
            icon: const Icon(Icons.cancel),
          ),
        ],
      ),
      body: SingleChildScrollView(
        child: Padding(
          padding: EdgeInsets.symmetric(horizontal: 16),
          child: FormBuilder(
            key: formKey,
            child: Column(
              children: [
                Showcase(
                  key: nameID3,
                  description: "The name of the multisig account, once created. It can be changed later in the Edit Page",
                  child: FormBuilderTextField(
                    name: "name",
                    decoration: const InputDecoration(labelText: "Name"),
                    validator: FormBuilderValidators.required(),
                  ),
                ),
                Showcase(
                  key: participantID,
                  description: "Number of signers",
                  child: FormBuilderDropdown(
                    name: "participants",
                    decoration: const InputDecoration(
                      labelText: "Number of Participants",
                    ),
                    initialValue: 2,
                    items: List.generate(
                      4,
                      (i) => DropdownMenuItem(
                        value: i + 2,
                        child: Text("${i + 2}"),
                      ),
                    ),
                  ),
                ),
                Showcase(
                  key: pID,
                  description: "Every participant should choose a different slot ID",
                  child: FormBuilderDropdown(
                    name: "id",
                    decoration: const InputDecoration(
                      labelText: "Your Participant ID",
                    ),
                    initialValue: 1,
                    items: List.generate(
                      5,
                      (i) => DropdownMenuItem(
                        value: i + 1,
                        child: Text("${i + 1}"),
                      ),
                    ),
                  ),
                ),
                Showcase(
                  key: thresholdID,
                  description: "Minimum number of signers",
                  child: FormBuilderDropdown(
                    name: "threshold",
                    decoration: const InputDecoration(
                      labelText: "Number of Signers Required (Threshold)",
                    ),
                    initialValue: 2,
                    items: List.generate(
                      4,
                      (i) => DropdownMenuItem(
                        value: i + 2,
                        child: Text("${i + 2}"),
                      ),
                    ),
                    validator: (v) {
                      final n = formKey.currentState?.fields["participants"]!.value as int;
                      if (v! > n) return "Threshold must be less than participants";
                      return null;
                    },
                  ),
                ),
                Showcase(
                  key: fundingID,
                  description: "DKG uses messages in memos. The process needs a ~0.0001 ZEC to pay for the fees. This account is used to pay for them.",
                  child: FormBuilderDropdown(
                    name: "account",
                    decoration: const InputDecoration(
                      labelText: "Funding Account for the DKG messages",
                    ),
                    items: accounts
                        .map(
                          (a) => DropdownMenuItem(
                            value: a.id,
                            child: Text(a.name),
                          ),
                        )
                        .toList(),
                    validator: FormBuilderValidators.required(),
                  ),
                ),
                Gap(16),
                ElevatedButton.icon(
                  onPressed: () => onNext(context),
                  label: Text("Next"),
                  icon: Icon(Icons.arrow_forward),
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }

  onNext(BuildContext context) async {
    final form = formKey.currentState!;
    if (form.saveAndValidate()) {
      final name = form.fields["name"]!.value as String;
      final participants = form.fields["participants"]!.value as int;
      final id = form.fields["id"]!.value as int;
      final threshold = form.fields["threshold"]!.value as int;
      final account = form.fields["account"]!.value as int;
      await setDkgParams(
        name: name,
        id: id,
        n: participants,
        t: threshold,
        fundingAccount: account,
        c: c,
      );
      if (!context.mounted) return;
      await GoRouter.of(context).push("/dkg2");
    }
  }
}

class DKGPage2 extends ConsumerStatefulWidget {
  const DKGPage2({super.key});

  @override
  ConsumerState<DKGPage2> createState() => DKGPage2State();
}

class DKGPage2State extends ConsumerState<DKGPage2> {
  late final c = ref.read(coinContextProvider);
  final formKey = GlobalKey<FormBuilderState>();
  List<String> addresses = [];

  @override
  void initState() {
    super.initState();
    Future(() async {
      await initDkg(c: c);
      final addresses = await getDkgAddresses(c: c);
      setState(() => this.addresses = addresses);
      if (await hasDkgAddresses(c: c)) {
        WidgetsBinding.instance.addPostFrameCallback((_) {
          GoRouter.of(context).pushReplacement("/dkg3");
        });
      }
    });
  }

  @override
  Widget build(BuildContext context) {
    final pinlock = ref.watch(lifecycleProvider);
    if (pinlock.value ?? false) return PinLock();

    return Scaffold(
      appBar: AppBar(
        title: const Text("DKG Addresses"),
        actions: [
          IconButton(
            onPressed: () => onCancel(context, ref),
            icon: const Icon(Icons.cancel),
          ),
        ],
      ),
      body: SingleChildScrollView(
        child: Padding(
          padding: EdgeInsets.symmetric(horizontal: 16),
          child: FormBuilder(
            key: formKey,
            child: Column(
              children: [
                ...addresses.asMap().entries.map((kv) {
                  final i = kv.key;
                  final address = kv.value;

                  return FormBuilderTextField(
                    name: "$i",
                    decoration: InputDecoration(
                      labelText: "Address for Participant #${i + 1}",
                    ),
                    initialValue: address,
                    validator: FormBuilderValidators.compose([
                      FormBuilderValidators.required(),
                      validAddress,
                    ]),
                  );
                }),
                Gap(16),
                ElevatedButton.icon(
                  onPressed: () => onNext(context),
                  label: Text("Next"),
                  icon: Icon(Icons.arrow_forward),
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }

  onNext(BuildContext context) async {
    final form = formKey.currentState!;
    if (form.saveAndValidate()) {
      for (var i = 0; i < addresses.length; i++) {
        final address = form.fields["$i"]!.value as String;
        await setDkgAddress(id: i + 1, address: address, c: c);
      }
      if (!context.mounted) return;
      await GoRouter.of(context).pushReplacement("/dkg3");
    }
  }
}

class DKGPage3 extends ConsumerStatefulWidget {
  const DKGPage3({super.key});

  @override
  ConsumerState<DKGPage3> createState() => DKGPage3State();
}

class DKGPage3State extends ConsumerState<DKGPage3> {
  late final c = ref.read(coinContextProvider);
  String message = "";
  int index = 0;
  Timer? runTimer;
  int? currentHeight;
  bool finished = false;

  @override
  void initState() {
    super.initState();
    runTimer = Timer.periodic(const Duration(seconds: 30), (_) async {
      await runDkg();
    });
    unawaited(runDkg());
  }

  @override
  void dispose() {
    runTimer?.cancel();
    super.dispose();
  }

  Future<void> runDkg() async {
    final h = await getCurrentHeight(c: c);
    if (currentHeight != null && currentHeight == h) return;
    currentHeight = h;
    final as = await ref.read(getAccountsProvider.future);
    final accounts = as.where((e) => e.enabled).toList();
    final synchronizer = ref.read(synchronizerProvider.notifier);
    await synchronizer.startSynchronize(
      accounts,
    );

    final status = doDkg(c: c);
    status.listen(
      (s) {
        if (s is DKGStatus_PublishRound1Pkg) {
          setState(() {
            message = "Broadcasting round 1 packages";
            index = 1;
          });
        }
        if (s is DKGStatus_WaitRound1Pkg) {
          setState(() {
            message = "Waiting for other participants to send their round 1 packages";
            index = 1;
          });
        }
        if (s is DKGStatus_PublishRound2Pkg) {
          setState(() {
            message = "Broadcasting round 2 packages";
            index = 2;
          });
        }
        if (s is DKGStatus_WaitRound2Pkg) {
          setState(() {
            message = "Waiting for other participants to send their round 2 packages";
            index = 2;
          });
        }
        if (s is DKGStatus_SharedAddress) {
          final sharedUA = s.field0;
          ref.invalidate(getAccountsProvider);
          setState(() {
            message = "The shared address is: $sharedUA";
            index = 3;
            finished = true;
          });
        }
      },
      onError: (Object e) async {
        final exc = e as AnyhowException;
        if (!context.mounted) return;
        await showException(context, exc.message);
      },
    );
  }

  @override
  Widget build(BuildContext context) {
    final t = Theme.of(context).textTheme;
    return buildDKGPage(
      context,
      ref,
      index: index,
      finished: finished,
      child: CopyableText(message, style: t.bodyLarge),
    );
  }
}

class DKGSteps extends StatelessWidget {
  final int currentIndex;

  const DKGSteps({super.key, required this.currentIndex});

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
          title: "Round 1",
          icon: Icon(Icons.mail),
        ),
        EasyStep(
          title: "Round 2",
          icon: Icon(Icons.mail),
        ),
        EasyStep(
          title: "Finalize",
          icon: Icon(Icons.flag),
        ),
      ],
    );
  }
}

void onClose(BuildContext context) => GoRouter.of(context).go("/");

void onCancel(BuildContext context, WidgetRef ref) async {
  final confirmed = await confirmDialog(
    context,
    title: "Cancel DKG",
    message: "Are you sure you want to cancel the DKG process?",
  );
  if (confirmed) {
    final c = ref.read(coinContextProvider);
    await cancelDkg(c: c);
    if (!context.mounted) return;
    GoRouter.of(context).pop();
  }
}
