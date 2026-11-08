/// Integration test: every key type that is not a seed, through the real app.
///
/// Each key is imported on the new-account page and synced, and its addresses,
/// pool balances and incoming-only flag are checked. All the accounts then sync
/// together through `syncIfNeeded`. What follows depends on the key: a spending
/// key broadcasts a payment, a full viewing key plans one the app can only save
/// unsigned, and an incoming key shows its banner and no send button. The
/// Sapling secret key pays one block behind the tip, so taking the Sync Warning's
/// "Sync Now" runs `syncIfNeeded` as a manual sync too.
/// `tests/tests/test_key_types_ui.py` owns the chain, the funding and the keys.
library;

import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:go_router/go_router.dart';
import 'package:integration_test/integration_test.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:zkool/main.dart';
import 'package:zkool/pages/tx.dart';
import 'package:zkool/router.dart';
import 'package:zkool/src/rust/api/account.dart';
import 'package:zkool/src/rust/api/network.dart';
import 'package:zkool/src/rust/api/sync.dart';
import 'package:zkool/store.dart';
import 'package:zkool/utils.dart';

import 'support.dart';

const _sendTooltip = "Send funds to one or many addresses";
const _broadcastTooltip = "Confirm, broadcast transaction";
const _saveUnsignedTooltip = "Save the unsigned transaction";
const _incomingBanner = "This account was created from an incoming viewing key. It shows the payments received; "
    "spends are not visible to it, so the shielded figures only grow.";

/// One key to import, as the orchestrator describes it.
class _KeyCase {
  final String name;
  final String key;
  final String mode; // spend, watch or incoming
  final String? taddr;
  final String? saddr;
  final bool orchard;
  final List<BigInt> minimum; // by pool: transparent, Sapling, Orchard, Ironwood
  final bool syncWarning;

  _KeyCase(Map<String, dynamic> json)
      : name = json["name"] as String,
        key = json["key"] as String,
        mode = json["mode"] as String,
        taddr = json["taddr"] as String?,
        saddr = json["saddr"] as String?,
        orchard = json["orchard"] as bool,
        minimum = (json["minimum"] as List).map((v) => zecToZat(v as String)).toList(),
        syncWarning = json["sync_warning"] as bool;
}

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();
  // ignore: avoid_print
  debugPrint = (String? message, {int? wrapWidth}) => print(message ?? "");

  testWidgets("Every other key type imports and does what its key allows", (tester) async {
    final docsDir = await appDocumentsDir();
    final rendezvous = Rendezvous.fromEnvironment(docsDir);
    expect(rendezvous.dir.existsSync(), isTrue,
        reason: "rendezvous dir ${rendezvous.dir.path} missing — "
            "start this test from test_key_types_ui.py",);

    final config = rendezvous.readConfig();
    final dbPath = config["db_path"] as String;
    final recipient = config["recipient"] as String;
    final spend = config["spend"] as String;
    final cases = (config["cases"] as List).map((c) => _KeyCase(c as Map<String, dynamic>)).toList();
    final biometricGate = config["biometric_gate"] as bool? ?? false;
    expect(dbPath, contains("regtest"));

    final prefs = SharedPreferencesAsync();
    final savedPrefs = await SavedPrefs.forceTestValues(prefs);

    void phase(String name) {
      debugPrint("[key-types-ui] phase: $name");
      rendezvous.publish("phase", name);
    }

    // A failure's last log lines can be lost; the rendezvous file survives.
    void step(String what) {
      debugPrint("[key-types-ui] step: $what");
      rendezvous.publish("step", what);
    }

    try {
      await tester.runAsync(() async {
        await openTestWallet(dbPath: dbPath, lwd: config["lwd"] as String, docsDir: docsDir.path);
      });

      final appRouter = router(true, false, initialLocation: "/accounts");
      await tester.pumpWidget(ZkoolApp(router: appRouter));
      await tester.pump(const Duration(seconds: 1));
      final container = ProviderScope.containerOf(tester.element(find.byType(MaterialApp)));
      await primeProviders(tester, container);

      // ── Import, sync from the dialog, check what each key sees ───────────
      phase("import");
      final ids = <String, int>{};
      for (final keyCase in cases) {
        step("import ${keyCase.name}");
        await importKey(tester, appRouter, name: keyCase.name, key: keyCase.key, birth: "1");
        final id = (await tester.runAsync(() => accountIds()))![keyCase.name]!;
        ids[keyCase.name] = id;
        await _checkAccount(tester, keyCase, id);
        step("balance of ${keyCase.name}");
        await _waitForPools(tester, keyCase, id);
      }

      // ── Each key shows itself again on the backup block and the key tree ─
      phase("keys");
      await _checkKeyTree(tester, appRouter, {for (final c in cases) c.name: c}, ids, biometricGate: biometricGate);

      // ── syncIfNeeded, across every key type at once ──────────────────────
      phase("auto-sync");
      final tip = await _mineBlock(tester, rendezvous, "auto-sync");
      final notifier = container.read(synchronizerProvider.notifier);
      await tester.runAsync(() => notifier
          .syncIfNeeded(tip, now: true, kind: SyncKind.auto)
          .timeout(const Duration(minutes: 3)),);
      final synced = (await tester.runAsync(() => listAccounts(c: coinContext.coin)))!;
      for (final account in synced) {
        expect(account.height, greaterThanOrEqualTo(tip), reason: "${account.name} reached the tip");
      }

      // ── Full viewing keys plan a payment but cannot sign it ──────────────
      phase("watch");
      for (final keyCase in cases.where((c) => c.mode == "watch")) {
        step("plan from ${keyCase.name}");
        await _planPayment(tester, appRouter, keyCase, ids[keyCase.name]!, recipient, spend);
        expect(find.byTooltip(_saveUnsignedTooltip), findsOneWidget, reason: "${keyCase.name} can save the plan");
        expect(find.byTooltip(_broadcastTooltip), findsNothing, reason: "${keyCase.name} has no key to sign with");
      }

      // ── Spending keys pay ────────────────────────────────────────────────
      phase("spend");
      for (final keyCase in cases.where((c) => c.mode == "spend")) {
        final id = ids[keyCase.name]!;
        step("pay from ${keyCase.name}");
        // An earlier payment may have spent a coin this key shares.
        appRouter.go(accountRoute("/", id));
        await waitForAccountView(tester, keyCase.name);
        await waitForSync(tester, id);
        final before = (await tester.runAsync(() => totalBalance(id)))!;
        var height = (await tester.runAsync(() => getCurrentHeight(c: coinContext.coin)))!;
        if (keyCase.syncWarning) {
          height = await _mineBlock(tester, rendezvous, "sync-warning-${keyCase.name}");
          container.read(currentHeightProvider.notifier).updateFromService(height);
        }
        await _planPayment(tester, appRouter, keyCase, id, recipient, spend);
        if (keyCase.syncWarning) {
          final account = (await tester.runAsync(() => listAccounts(c: coinContext.coin)))!.firstWhere((a) => a.id == id);
          expect(account.height, greaterThanOrEqualTo(height), reason: "the manual sync reached the tip");
        }
        await tapTooltip(tester, _broadcastTooltip);
        await answerDialog(tester, "Are you sure you want to send this transaction?", "Ok");
        final txLine = await awaitStatus(
          tester,
          rendezvous,
          readStatus: () {
            final found = find.byWidgetPredicate((w) => w is CopyableText && w.text.startsWith("Transaction ID: ")).evaluate();
            return found.isEmpty ? "" : (found.first.widget as CopyableText).text;
          },
          isDone: (s) => s.startsWith("Transaction ID: "),
          timeout: const Duration(minutes: 3),
          what: "the transaction id of ${keyCase.name}",
        );
        rendezvous.publish("txid_${keyCase.name}", txLine.substring("Transaction ID: ".length).trim());

        step("settle ${keyCase.name}");
        await _waitForTip(tester, above: height);
        appRouter.go(accountRoute("/", id));
        await waitForAccountView(tester, keyCase.name);
        await waitForSync(tester, id);
        final after = (await tester.runAsync(() => totalBalance(id)))!;
        debugPrint("[key-types-ui] ${keyCase.name}: $before -> $after");
        expect(after, lessThanOrEqualTo(before - zecToZat(spend)), reason: "${keyCase.name} paid");
      }

      phase("done");
      rendezvous.publish("done", "true");
      await pumpFor(tester, const Duration(seconds: 3));
    } finally {
      await savedPrefs.restore(prefs);
    }
  }, timeout: const Timeout(Duration(minutes: 25)));
}

Future<void> _checkAccount(WidgetTester tester, _KeyCase keyCase, int id) async {
  final addresses = (await tester.runAsync(
    () => getAddresses(uaPools: 7, account: id, c: coinContext.coin),
  ))!;
  final account = (await tester.runAsync(() => listAccounts(c: coinContext.coin)))!
      .firstWhere((a) => a.id == id);
  debugPrint("[key-types-ui] ${keyCase.name}: account $id taddr=${addresses.taddr} "
      "saddr=${addresses.saddr} oaddr=${addresses.oaddr} incoming=${account.incomingOnly}");
  expect(addresses.taddr, keyCase.taddr, reason: "${keyCase.name} transparent address");
  expect(addresses.saddr, keyCase.saddr, reason: "${keyCase.name} Sapling address");
  expect(addresses.oaddr != null, keyCase.orchard, reason: "${keyCase.name} Orchard pool");

  final incoming = keyCase.mode == "incoming";
  expect(account.incomingOnly, incoming, reason: "${keyCase.name} incoming-only flag");
  expect(find.text(_incomingBanner), incoming ? findsOneWidget : findsNothing, reason: "${keyCase.name} banner");
  expect(find.byTooltip(_sendTooltip), incoming ? findsNothing : findsOneWidget, reason: "${keyCase.name} send button");
}

Finder _copyable(String text) => find.byWidgetPredicate((w) => w is CopyableText && w.text == text);

/// The xprv account shows its xprv on the backup block; its key tree has the
/// WIF of the address the WIF case was made from, and behind the switch the
/// xpub and public key the watch cases were made from. The Sapling spending
/// key shows itself and, behind the switch, the viewing key the S-vk case
/// imported. Every string is compared with the key the orchestrator holds,
/// so the tree is checked against an independent derivation. Secret keys sit
/// behind `authenticate()`, Touch ID on macOS, so those checks run only where
/// the gate opens by itself; a viewing-only account has no gate.
Future<void> _checkKeyTree(
  WidgetTester tester,
  GoRouter appRouter,
  Map<String, _KeyCase> byName,
  Map<String, int> ids, {
  required bool biometricGate,
}) async {
  Future<void> open(_KeyCase keyCase) async {
    unawaited(appRouter.push("/viewing_keys", extra: ids[keyCase.name]!));
    await pumpUntil(tester, () => find.text("Viewing Keys").evaluate().isNotEmpty,
        timeout: shortTimeout, what: "the viewing keys page of ${keyCase.name}",);
    await tapTooltip(tester, "Show Backup Keys");
    await pumpUntil(tester, () => _copyable(keyCase.key).evaluate().isNotEmpty,
        timeout: shortTimeout, what: "the key of ${keyCase.name} on the backup block",);
    await tester.ensureVisible(find.text("Show Key Tree"));
    await tester.tap(find.text("Show Key Tree"));
    await pumpUntil(tester, () => find.text("Key Tree").evaluate().isNotEmpty,
        timeout: shortTimeout, what: "the key tree of ${keyCase.name}",);
    await pumpUntil(tester, () => _copyable(keyCase.key).evaluate().isNotEmpty,
        timeout: shortTimeout, what: "the root of the key tree of ${keyCase.name}",);
  }

  Future<void> flip() async {
    await tester.tap(find.byType(Switch));
    await pumpFor(tester, const Duration(seconds: 1));
  }

  Future<void> close() async {
    appRouter.pop();
    await pumpFor(tester, const Duration(milliseconds: 500));
    appRouter.pop();
    await pumpFor(tester, const Duration(milliseconds: 500));
  }

  if (!biometricGate) {
    final xprv = byName["T-xprv"]!;
    await open(xprv);
    expect(find.text("External #0"), findsOneWidget, reason: "the first external address hangs off the xprv");
    expect(_copyable(byName["T-wif"]!.key), findsOneWidget, reason: "External #0 is the WIF the T-wif case imported");
    await flip();
    expect(_copyable(byName["T-xpub"]!.key), findsOneWidget, reason: "the xpub the T-xpub case imported");
    expect(_copyable(byName["T-zpk"]!.key), findsOneWidget, reason: "the public key the T-zpk case imported");
    expect(_copyable(xprv.key), findsNothing, reason: "no secret in viewing mode");
    await close();

    final ssk = byName["S-sk"]!;
    await open(ssk);
    await flip();
    expect(_copyable(byName["S-vk"]!.key), findsOneWidget, reason: "the viewing key the S-vk case imported");
    await close();
  }

  // A viewing-only account opens on its viewing face and has no switch.
  final svk = byName["S-vk"]!;
  await open(svk);
  expect(find.byType(Switch), findsNothing, reason: "nothing secret to switch to");
  await close();
}

/// Waits until every pool holds what was paid to the key.
Future<void> _waitForPools(WidgetTester tester, _KeyCase keyCase, int id) async {
  final deadline = DateTime.now().add(chainTimeout);
  var polls = 0;
  List<BigInt> pools = [];
  while (DateTime.now().isBefore(deadline)) {
    final PoolBalance b = (await tester.runAsync(() => balance(account: id, c: coinContext.coin)))!;
    pools = b.field0.toList();
    if ([for (var i = 0; i < keyCase.minimum.length; i++) pools[i] >= keyCase.minimum[i]].every((ok) => ok)) {
      debugPrint("[key-types-ui] ${keyCase.name}: pools $pools");
      return;
    }
    if (polls++ % 6 == 5) await tapTooltip(tester, "Sync this account");
    await pumpFor(tester, const Duration(seconds: 5));
  }
  fail("${keyCase.name} holds $pools, wanted at least ${keyCase.minimum}");
}

/// Has the orchestrator mine a block, and returns the new tip.
Future<int> _mineBlock(WidgetTester tester, Rendezvous rendezvous, String request) async {
  final before = (await tester.runAsync(() => getCurrentHeight(c: coinContext.coin)))!;
  rendezvous.publish("mine", request);
  return _waitForTip(tester, above: before);
}

Future<int> _waitForTip(WidgetTester tester, {required int above}) async {
  final deadline = DateTime.now().add(chainTimeout);
  while (DateTime.now().isBefore(deadline)) {
    final tip = (await tester.runAsync(() => getCurrentHeight(c: coinContext.coin)))!;
    if (tip > above) return tip;
    await pumpFor(tester, const Duration(seconds: 1));
  }
  fail("the chain did not pass height $above");
}

/// Takes the send page as far as the transaction page.
Future<void> _planPayment(
  WidgetTester tester,
  GoRouter appRouter,
  _KeyCase keyCase,
  int id,
  String recipient,
  String amount,
) async {
  // Through the account view: the previous account's send page is still in the
  // route stack when leaving a transaction page, and would match the finder.
  appRouter.go(accountRoute("/", id));
  await waitForAccountView(tester, keyCase.name);
  appRouter.go(accountRoute("/send", id));
  await pumpUntil(tester, () => fieldNamed("address").evaluate().isNotEmpty,
      timeout: shortTimeout, what: "the send page of ${keyCase.name}",);
  await enterRecipient(tester, recipient, amount);
  await tapTooltip(tester, "Send transaction (including queued recipients)");
  await pumpUntil(tester, () => find.text("Extra Options").evaluate().isNotEmpty,
      timeout: shortTimeout, what: "the send options of ${keyCase.name}",);
  await tapTooltip(tester, "Send (Summary and Confirmation)");
  // The height the page holds can be ahead of the account: sync if it asks.
  // A case that expects the warning must see it.
  var warned = false;
  final deadline = DateTime.now().add(const Duration(minutes: 3));
  while (DateTime.now().isBefore(deadline)) {
    if (find.byType(TxPage).evaluate().isNotEmpty) {
      expect(warned || !keyCase.syncWarning, isTrue, reason: "${keyCase.name} saw the Sync Warning");
      return;
    }
    if (find.text("Sync Now").evaluate().isNotEmpty) {
      warned = true;
      await tester.tap(find.text("Sync Now").last);
    }
    await pumpFor(tester, const Duration(seconds: 1));
  }
  fail("no transaction page for ${keyCase.name}: ${visibleTexts().join(" | ")}");
}
