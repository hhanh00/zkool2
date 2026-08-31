import 'dart:async';

import 'package:easy_stepper/easy_stepper.dart';
import 'package:flutter/material.dart';
import 'package:flutter_form_builder/flutter_form_builder.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge.dart';
import 'package:form_builder_validators/form_builder_validators.dart';
import 'package:gap/gap.dart';
import 'package:go_router/go_router.dart';

import 'package:zkool/main.dart';
import 'package:zkool/src/rust/api/account.dart';
import 'package:zkool/src/rust/api/frost.dart';
import 'package:zkool/store.dart';
import 'package:zkool/utils.dart';
import 'package:zkool/validators.dart';

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
  late final c = coinContext.coin;
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

  @override
  Widget build(BuildContext context) {
    final pinlock = ref.watch(lifecycleProvider);
    if (pinlock.value ?? false) return PinLock();

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
                Tooltip(
                  message: "The name of the multisig account, once created. It can be changed later in the Edit Page",
                  child: FormBuilderTextField(
                    name: "name",
                    decoration: const InputDecoration(labelText: "Name"),
                    validator: FormBuilderValidators.required(),
                  ),
                ),
                Tooltip(
                  message: "Number of signers",
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
                Tooltip(
                  message: "Every participant should choose a different slot ID",
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
                Tooltip(
                  message: "Minimum number of signers",
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
                Tooltip(
                  message: "DKG uses messages in memos. The process needs a ~0.0001 ZEC to pay for the fees. This account is used to pay for them.",
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
  late final c = coinContext.coin;
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
  late final c = coinContext.coin;
  // Held in a field because `ref` is unsafe to use once the widget is disposed.
  late final SynchronizerNotifier _synchronizer =
      ref.read(synchronizerProvider.notifier);
  String message = "";
  int index = 0;
  StreamSubscription<int>? _heightSub;
  int? _lastStepHeight;
  // Guards against overlapping passes: a slow sync+step must finish before the
  // next block starts another, or two `doDkg` runs would each publish the same
  // round and double-spend the funding note.
  bool _stepping = false;
  bool finished = false;

  @override
  void initState() {
    super.initState();
    // `doDkg` no longer syncs itself; it steps against what is already synced.
    // Drive the wallet's synchronizer once per block and then step, rather than
    // on a fixed timer: the block-height stream delivers the current tip on
    // subscribe (kicking off round 0) and then only on changes. A round waiting
    // for its own change to confirm defers to the next block, so
    // `_lastStepHeight` guards against stepping the same height twice — keeping
    // the "waiting for funds" warning to at most once per block.
    _heightSub = blockHeightService.heights.listen((height) async {
      if (_stepping || _lastStepHeight == height) return;
      _stepping = true;
      _lastStepHeight = height;
      try {
        // Force a sync of the funding and internal frost accounts (which stores
        // the incoming package memos), then step against the fresh state.
        await _synchronizer.syncIfNeeded(height, now: true);
        if (!mounted) return;
        await runDkg();
      } finally {
        _stepping = false;
      }
    });
  }

  @override
  void dispose() {
    unawaited(_heightSub?.cancel());
    super.dispose();
  }

  Future<void> runDkg() async {
    try {
      await ref.read(currentHeightProvider.notifier).fetch();

      // No startSynchronize here: the block-height handler syncs first, and
      // doDkg steps against what is already synced. Await the stream to
      // completion so the handler serializes passes — overlapping doDkg runs
      // would each publish the same round and double-spend the funding note.
      await for (final s in doDkg(c: c)) {
        if (!mounted) return;
        if (s is DKGStatus_PublishRound0Pkg) {
          setState(() {
            message = "Broadcasting participant keys";
            index = 0;
          });
        } else if (s is DKGStatus_WaitRound0Pkg) {
          setState(() {
            message = "Waiting for other participants to send their keys";
            index = 0;
          });
        } else if (s is DKGStatus_PublishRound1Pkg) {
          setState(() {
            message = "Broadcasting round 1 packages";
            index = 1;
          });
        } else if (s is DKGStatus_WaitRound1Pkg) {
          setState(() {
            message = "Waiting for other participants to send their round 1 packages";
            index = 1;
          });
        } else if (s is DKGStatus_PublishRound2Pkg) {
          setState(() {
            message = "Broadcasting round 2 packages";
            index = 2;
          });
        } else if (s is DKGStatus_WaitRound2Pkg) {
          setState(() {
            message = "Waiting for other participants to send their round 2 packages";
            index = 2;
          });
        } else if (s is DKGStatus_WaitingForFunds) {
          // The previous round's change is not mined yet, so this round's
          // publish has nothing to spend. The next block retries; surface it
          // as a warning rather than failing so the user knows why it paused.
          showWarningSnackbar(
            "Waiting for the previous round's change to be confirmed…",
          );
        } else if (s is DKGStatus_Finalize) {
          setState(() {
            message = "Deriving the shared key";
            index = 3;
          });
        } else if (s is DKGStatus_SharedAddress) {
          final sharedUA = s.field0;
          ref.invalidate(getAccountsProvider);
          setState(() {
            message = "The shared address is: $sharedUA";
            index = 3;
            finished = true;
          });
        }
      }
    } on AnyhowException catch (e) {
      if (!context.mounted) return;
      // Transient: the next block retries, so warn instead of a modal error.
      showWarningSnackbar(e.message);
    }
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
    final c = coinContext.coin;
    await cancelDkg(c: c);
    if (!context.mounted) return;
    GoRouter.of(context).pop();
  }
}
