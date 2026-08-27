/// Integration test: run the Flutter app as a FROST *signing* participant.
///
/// Signing needs a shared key, so the test runs the DKG wizard first (the app
/// as participant #1, two headless `zkool_graphql` peers), then drives the
/// signing pages: /frost1 to pick the coordinator and the funding account, and
/// /frost2 for the signing rounds. `tests/tests/test_frost_ui.py` owns the
/// chain, funds the shared address, prepares the PCZT on the coordinator and
/// verifies that the receiver is actually paid.
library;

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:zkool/main.dart';
import 'package:zkool/router.dart';
import 'package:zkool/src/rust/api/account.dart';
import 'package:zkool/src/rust/api/pay.dart';
import 'package:zkool/store.dart';

import 'support.dart';

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  testWidgets("FROST signing participant through the Flutter UI", (tester) async {
    final docsDir = await appDocumentsDir();
    final rendezvous = Rendezvous.fromEnvironment(docsDir);
    expect(rendezvous.dir.existsSync(), isTrue,
        reason: "rendezvous dir ${rendezvous.dir.path} missing — "
            "start this test from test_frost_ui.py",);

    final config = rendezvous.readConfig();
    final dbPath = config["db_path"] as String;
    final myId = config["my_id"] as int;
    final coordinator = config["coordinator"] as int;
    expect(dbPath, contains("regtest"));
    expect(coordinator, isNot(myId),
        reason: "this test drives a non-coordinator participant",);

    final prefs = SharedPreferencesAsync();
    final savedPrefs = await SavedPrefs.forceTestValues(prefs);

    try {
      // ── Phase 1: DKG, to get a shared key to sign with ────────────────────
      await tester.runAsync(() async {
        final account = await setUpTestWallet(
          dbPath: dbPath,
          lwd: config["lwd"] as String,
          docsDir: docsDir.path,
        );
        final addresses = await getAddresses(uaPools: 6, c: coinContext.coin);
        debugPrint("[dkg-ui] funding account $account address ${addresses.oaddr}");
        rendezvous.publish("funding_address", addresses.oaddr!);
      });

      final appRouter = router(true, false, initialLocation: "/dkg1");
      await tester.pumpWidget(ZkoolApp(router: appRouter));
      await tester.pump(const Duration(seconds: 1));

      final container =
          ProviderScope.containerOf(tester.element(find.byType(MaterialApp)));
      await primeProviders(tester, container);

      final sharedAddress = await driveDkg(
        tester,
        rendezvous,
        name: config["name"] as String,
        n: config["n"] as int,
        t: config["t"] as int,
        myId: myId,
      );
      rendezvous.publish("shared_address", sharedAddress);

      // Leave the DKG page so it is disposed. Its 30s timer keeps calling
      // doDkg, which starts throwing `get_funding_account: no rows` once the
      // DKG is finished and the selected account moves to the FROST one.
      appRouter.go("/accounts");
      await pumpFor(tester, const Duration(seconds: 2));

      // ── Phase 2: wait for the orchestrator to fund the shared address and
      // hand us the PCZT the coordinator prepared ───────────────────────────
      Map<String, dynamic>? fromOrchestrator;
      await pumpUntil(
        tester,
        () {
          fromOrchestrator = rendezvous.readOrchestrator();
          return fromOrchestrator?["pczt"] != null;
        },
        timeout: const Duration(minutes: 15),
        what: "the PCZT from the coordinator",
      );
      final pcztHex = fromOrchestrator!["pczt"] as String;
      debugPrint("[dkg-ui] received PCZT (${pcztHex.length ~/ 2} bytes)");

      // The FROST account has to be the selected one: FrostPage1 reads
      // `frostParams` off the current account, and the Rust side keys the
      // signing state off the coin's account.
      // DKGPage3 invalidates the account list when it finishes, but make sure
      // we read it after the FROST account was created.
      container.invalidate(getAccountsProvider);
      late final PcztPackage pczt;
      await tester.runAsync(() async {
        final accounts = await container.read(getAccountsProvider.future);
        final frostAccount = accounts.firstWhere(
          (a) => a.name == config["name"] as String,
          orElse: () => throw StateError(
            "no FROST account named ${config["name"]} after the DKG",
          ),
        );
        debugPrint("[dkg-ui] FROST account ${frostAccount.id}");
        await coinContext.setAccount(account: frostAccount.id);
        await container.read(selectedAccountIdProvider.notifier).set(frostAccount.id);
        // Interchangeable with zkool_graphql now that both use bincode
        // standard() — see rust/src/api/pay.rs.
        pczt = await unpackTransaction(bytes: _hexToBytes(pcztHex));
      });
      container.invalidate(selectedAccountProvider);
      await pumpFor(tester, const Duration(seconds: 2));

      // ── Phase 3: the signing pages ────────────────────────────────────────
      // /frost1 takes the PCZT as a route `extra`, so navigate there on the
      // router we built rather than starting the app on that route.
      appRouter.go("/frost1", extra: pczt);
      await pumpUntil(
        tester,
        () => find.text("ID of the coordinator").evaluate().isNotEmpty,
        timeout: const Duration(minutes: 2),
        what: "FROST page 1",
      );

      await selectDropdown(tester, "ID of the coordinator", "$coordinator");
      await selectDropdown(
        tester,
        "Funding Account for the FROST messages",
        "DKG-Fund",
      );
      await tapNext(tester);

      await pumpUntil(
        tester,
        () => find.text("Frost Multi Party Signature").evaluate().isNotEmpty,
        timeout: const Duration(minutes: 2),
        what: "FROST page 2",
      );

      // FrostPage2 renders its status in a plain Text, not a CopyableText.
      const completed = "Signing completed";
      await awaitStatus(
        tester,
        rendezvous,
        readStatus: () {
          final found = find
              .byWidgetPredicate(
                (w) => w is Text && (w.data ?? "").isNotEmpty && _isSigningStatus(w.data!),
              )
              .evaluate();
          if (found.isEmpty) return "";
          return (found.first.widget as Text).data!;
        },
        isDone: (s) => s == completed,
        timeout: const Duration(minutes: 20),
        what: "signing to complete",
      );
      rendezvous.publish("signing_status", completed);

      await pumpFor(tester, const Duration(seconds: 6));
    } finally {
      await savedPrefs.restore(prefs);
    }
  }, timeout: const Timeout(Duration(minutes: 60)));
}

/// The messages FrostPage2 can show; used to pick the status Text out of the
/// page without matching the AppBar title or the stepper labels.
const _signingStatuses = [
  "Waiting for other participants to send their commitments",
  "Sending our commitments to the coordinator",
  "Broadcasting the signing package to all participants",
  "Waiting for the signing package from the coordinator",
  "Sending our signature share to the coordinator",
  "Signing completed",
  "Waiting for the signature share from the other participants",
  "Assembling the transaction",
  "Sending the transaction to the network",
];

bool _isSigningStatus(String s) =>
    _signingStatuses.contains(s) || s.startsWith("TX ID: ");

List<int> _hexToBytes(String hex) => [
      for (var i = 0; i < hex.length; i += 2)
        int.parse(hex.substring(i, i + 2), radix: 16),
    ];
