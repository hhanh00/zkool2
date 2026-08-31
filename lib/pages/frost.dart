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
import 'package:zkool/src/rust/api/pay.dart';
import 'package:zkool/store.dart';
import 'package:zkool/utils.dart';

class FrostPage1 extends ConsumerStatefulWidget {
  final PcztPackage pczt;
  const FrostPage1(this.pczt, {super.key});

  @override
  ConsumerState<FrostPage1> createState() => FrostPage1State();
}

Widget buildFrostPage(
  BuildContext context,
  WidgetRef ref, {
  required int index,
  required bool finished,
  required Widget child,
}) {
  return Scaffold(
    appBar: AppBar(
      title: const Text("Frost Multi Party Signature"),
      actions: [
        (finished)
            ? IconButton(
                onPressed: () {
                  GoRouter.of(context).go("/");
                },
                icon: Icon(Icons.close),
              )
            : IconButton(
                onPressed: () => onCancel(context, ref),
                icon: const Icon(Icons.cancel),
              ),
      ],
    ),
    body: CustomScrollView(
      slivers: [
        PinnedHeaderSliver(child: FrostSteps(currentIndex: index)),
        SliverPadding(
          padding: EdgeInsets.symmetric(horizontal: 8),
          sliver: SliverToBoxAdapter(child: child),
        ),
      ],
    ),
  );
}

class FrostPage1State extends ConsumerState<FrostPage1> {
  late final c = coinContext.coin;
  final formKey = GlobalKey<FormBuilderState>();
  List<Account> accounts = [];
  AccountData? account;
  FrostParams? frostParams;

  @override
  void initState() {
    super.initState();
  }

  @override
  Widget build(BuildContext context) {
    final pinlock = ref.watch(lifecycleProvider);
    if (pinlock.value ?? false) return PinLock();

    final accountDataAV = ref.watch(basicAccountDataProvider);

    return accountDataAV.when(
      loading: () => blank(context),
      error: (error, stack) => showError(error),
      data: (accountData) {
        accounts = accountData.allAccounts.where((e) => !e.hidden).toList();
        account = accountData.currentAccount;

        if (frostParams == null) {
          frostParams = account!.frostParams;
          Future(() async {
            final signing = await isSigningInProgress(c: c);
            if (signing) {
              if (context.mounted) {
                WidgetsBinding.instance.addPostFrameCallback((_) {
                  GoRouter.of(context).pushReplacement("/frost2");
                });
              }
            }
            setState(() {});
          });
        }

        return Scaffold(
          appBar: AppBar(title: const Text("Frost Multi Party Signature")),
          body: FormBuilder(
            key: formKey,
            child: Column(
              children: [
                ListTile(
                  title: Text("Your Participant ID"),
                  subtitle: Text(frostParams!.id.toString()),
                ),
                Padding(
                  padding: EdgeInsets.symmetric(horizontal: 16),
                  child: Tooltip(
                    message: "Participant ID who is coordinating the multisignature",
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
                      ),
                    ),
                  ),
                ),
                Padding(
                  padding: EdgeInsets.symmetric(horizontal: 16),
                  child: Tooltip(
                    message: "Multisig uses messages in memos. The process needs a ~0.0001 ZEC to pay for the fees. This account is used to pay for them.",
                    child: FormBuilderDropdown(
                      name: "account",
                      decoration: const InputDecoration(
                        labelText: "Funding Account for the FROST messages",
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
                ),
                Gap(16),
                ElevatedButton.icon(
                  onPressed: onNext,
                  label: Text("Next"),
                  icon: Icon(Icons.arrow_forward),
                ),
              ],
            ),
          ),
        );
      },
    );
  }

  void onNext() async {
    final form = formKey.currentState!;
    if (form.saveAndValidate()) {
      final coordinator = form.fields["coordinator"]!.value as int;
      final fundingAccount = form.fields["account"]!.value as int;
      await initSign(pczt: widget.pczt, coordinator: coordinator, fundingAccount: fundingAccount, c: c);
      if (!mounted) return;
      await GoRouter.of(context).pushReplacement("/frost2");
    }
  }
}

class FrostPage2 extends ConsumerStatefulWidget {
  const FrostPage2({super.key});

  @override
  ConsumerState<FrostPage2> createState() => FrostPage2State();
}

class FrostPage2State extends ConsumerState<FrostPage2> {
  late final c = coinContext.coin;
  // Held in a field because `ref` is unsafe to use once the widget is disposed.
  late final SynchronizerNotifier _synchronizer =
      ref.read(synchronizerProvider.notifier);
  String message = "";
  StreamSubscription<int>? _heightSub;
  int? _lastStepHeight;
  // Guards against overlapping passes: a slow sync+step must finish before the
  // next block starts another, or two `doSign` runs would each spend the same
  // funding note and double-spend.
  bool _stepping = false;
  int currentIndex = 0;
  bool finished = false;

  @override
  void initState() {
    super.initState();
    // `doSign` no longer syncs itself; it steps against what is already synced.
    // Drive the wallet's synchronizer once per block and then step: the
    // block-height stream delivers the current tip on subscribe (kicking off
    // the first round) and then only on changes. A message waiting for its
    // change to confirm defers to the next block, so `_lastStepHeight` guards
    // against stepping the same height twice.
    _heightSub = blockHeightService.heights.listen((height) async {
      if (_stepping || _lastStepHeight == height) return;
      _stepping = true;
      _lastStepHeight = height;
      try {
        // Force a sync of the funding and internal frost accounts (which stores
        // the incoming message memos), then step against the fresh state.
        await _synchronizer.syncIfNeeded(height, now: true);
        if (!mounted) return;
        await runFrost();
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

  @override
  Widget build(BuildContext context) {
    final t = Theme.of(context).textTheme;
    return buildFrostPage(
      context,
      ref,
      index: currentIndex,
      finished: finished,
      child: Column(children: [Text(message, style: t.bodyLarge)]),
    );
  }

  Future<void> runFrost() async {
    try {
      await ref.read(currentHeightProvider.notifier).fetch();

      // No startSynchronize here: the block-height handler syncs first, and
      // doSign steps against what is already synced. Await the stream to
      // completion so the handler serializes passes — overlapping doSign runs
      // would each spend the same funding note and double-spend.
      await for (final s in doSign(c: c)) {
        if (!mounted) return;
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
        } else if (s is SigningStatus_SigningCompleted) {
          setState(() {
            message = "Signing completed";
            currentIndex = 3; // other
            finished = true;
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
            finished = true;
          });
        } else if (s is SigningStatus_WaitingForFunds) {
          // The previous message's change is not mined yet, so this one has
          // nothing to spend. The next block retries; surface it as a warning
          // rather than failing.
          showWarningSnackbar(
            "Waiting for the previous message's change to be confirmed…",
          );
        }
      }
    } on AnyhowException catch (e) {
      if (!context.mounted) return;
      // Transient: the next block retries, so warn instead of a modal error.
      showWarningSnackbar(e.message);
    }
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

void onCancel(BuildContext context, WidgetRef ref) async {
  final c = coinContext.coin;
  final confirmed = await confirmDialog(
    context,
    title: "Cancel Multi Signature",
    message: "Are you sure you want to cancel the multi signature process?",
  );
  if (!confirmed) return;
  await resetSign(c: c);
  if (!context.mounted) return;
  GoRouter.of(context).pop();
}
