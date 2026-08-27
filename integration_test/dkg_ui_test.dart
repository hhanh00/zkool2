/// Integration test: run the Flutter app as FROST DKG participant #1 against a
/// regtest chain, while the other participants run headless as `zkool_graphql`
/// instances driven by `tests/tests/test_dkg_ui.py`.
///
/// The Python side owns the chain (funding, mining) and the peer participants.
/// Both sides exchange addresses through JSON files in a rendezvous directory
/// that lives inside the macOS app sandbox container — see `tests/tests/dkg.py`
/// for the other half of the protocol.
library;


import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:zkool/main.dart';
import 'package:zkool/router.dart';
import 'package:zkool/src/rust/api/account.dart';
import 'package:zkool/store.dart';

import 'support.dart';

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  testWidgets("DKG participant #1 through the Flutter UI", (tester) async {
    final docsDir = await appDocumentsDir();
    final rendezvous = Rendezvous.fromEnvironment(docsDir);
    expect(rendezvous.dir.existsSync(), isTrue,
        reason: "rendezvous dir ${rendezvous.dir.path} missing — "
            "start this test from test_dkg_ui.py",);

    final config = rendezvous.readConfig();
    final dbPath = config["db_path"] as String;
    // The database filename must contain "regtest": that substring is what
    // selects Network::Regtest in rust/src/api/coin.rs.
    expect(dbPath, contains("regtest"));

    final prefs = SharedPreferencesAsync();
    final savedPrefs = await SavedPrefs.forceTestValues(prefs);

    try {
      await tester.runAsync(() async {
        final account = await setUpTestWallet(
          dbPath: dbPath,
          lwd: config["lwd"] as String,
          docsDir: docsDir.path,
        );
        // uaPools 6 = sapling|orchard, matching the GraphQL `ironwood` address.
        final addresses = await getAddresses(uaPools: 6, c: coinContext.coin);
        debugPrint("[dkg-ui] funding account $account address ${addresses.oaddr}");
        rendezvous.publish("funding_address", addresses.oaddr!);
      });

      await tester.pumpWidget(
        ZkoolApp(router: router(true, false, initialLocation: "/dkg1")),
      );
      await tester.pump(const Duration(seconds: 1));

      final view = tester.view;
      final logicalSize = view.physicalSize / view.devicePixelRatio;
      debugPrint(
        "[dkg-ui] window ${logicalSize.width.toStringAsFixed(0)}"
        "x${logicalSize.height.toStringAsFixed(0)} logical "
        "(dpr ${view.devicePixelRatio})",
      );

      final container =
          ProviderScope.containerOf(tester.element(find.byType(MaterialApp)));
      await primeProviders(tester, container);

      final sharedAddress = await driveDkg(
        tester,
        rendezvous,
        name: config["name"] as String,
        n: config["n"] as int,
        t: config["t"] as int,
        myId: config["my_id"] as int,
      );
      rendezvous.publish("shared_address", sharedAddress);

      // Hold the finished screen briefly so the "Finalize" step and the shared
      // address are observable (on screen, and in a screen recording) instead
      // of vanishing the instant the assertion passes.
      await pumpFor(tester, const Duration(seconds: 6));
    } finally {
      await savedPrefs.restore(prefs);
    }
  }, timeout: const Timeout(Duration(minutes: 40)),);
}
