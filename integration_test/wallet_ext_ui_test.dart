/// Integration test: the pages the everyday wallet test leaves out, one app
/// and no peers.
///
/// Opens every page in scope that `wallet_ui_test.dart` never reaches and taps
/// through its actions, checking each one's effect: settings and its
/// sub-pages, the account view tabs, contacts, the address list, the error
/// display and the database manager. The flows the base test already covers
/// (creating accounts, receiving, paying) are not repeated: the accounts are
/// created through the bridge and `tests/tests/test_wallet_ext_ui.py` funds
/// one of them directly, with notes in every pool, a dust note and a memo.
///
/// Every setting the test changes is changed back through the same page, and
/// the stored value is checked both times, so the chain, the server and the
/// developer's own preferences are as before when the run ends.
///
/// Two accounts: A (funded, the pages are opened on it) and B (a contact).
library;

import 'dart:async';
import 'dart:io';

import 'package:file_picker/file_picker.dart';
import 'package:flutter/material.dart';
import 'package:flutter_form_builder/flutter_form_builder.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:zkool/main.dart';
import 'package:zkool/pages/account.dart';
import 'package:zkool/router.dart';
import 'package:zkool/src/rust/api/account.dart';
import 'package:zkool/src/rust/api/coin.dart' hide initDatadir;
import 'package:zkool/src/rust/api/contacts.dart';
import 'package:zkool/src/rust/api/db.dart';
import 'package:zkool/store.dart';
import 'package:zkool/utils.dart';

import 'support.dart';

const _accountA = "Alice";
const _accountB = "Bob";

// Values no setting has by default, so a saved change is told from the original.
const _proxy = "socks5h://127.0.0.1:19050";
const _votingUrl = "https://example.invalid/voting-config.json";
const _actionsPerSync = "123";
const _coingeckoKey = "wallet-ext-key";
const _blockExplorer = "https://example.invalid/tx/";
const _qrDelay = "250";
const _qrRepair = "7";

const _transportLabels = {0: "Direct", 1: "Tor", 2: "Nym", 3: "Proxy"};

const _userMemo = "edited by the wallet-ext test";
const _exportPassword = "wallet-ext";
const _contactNotes = "wallet-ext contact";
const _contactImported = "Carol";
const _contactImportedCopy = "Carol-copy";
const _contactListImported = "Dave";
const _scratchDb = "wallet-ext-scratch";

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();
  // debugPrint throttles and drops queued lines when the test dies; the log
  // must show the last steps before a failure.
  // ignore: avoid_print
  debugPrint = (String? message, {int? wrapWidth}) => print(message ?? "");

  testWidgets("Remaining wallet pages through the Flutter UI", (tester) async {
    final docsDir = await appDocumentsDir();
    final rendezvous = Rendezvous.fromEnvironment(docsDir);
    expect(rendezvous.dir.existsSync(), isTrue,
        reason: "rendezvous dir ${rendezvous.dir.path} missing — "
            "start this test from test_wallet_ext_ui.py",);

    final config = rendezvous.readConfig();
    final dbPath = config["db_path"] as String;
    final coingeckoUrl = config["coingecko_url"] as String;
    final fundTaddr = config["fund_taddr"] as String;
    final fundSaddr = config["fund_saddr"] as String;
    final fundOaddr = config["fund_oaddr"] as String;
    final fundDust = config["fund_dust"] as String;
    final memo = config["memo"] as String;
    // The seed phrase is behind authenticate(): open where the device has no
    // biometrics (Linux, CI), Touch ID on macOS, where the step is skipped.
    final biometricGate = config["biometric_gate"] as bool? ?? false;
    expect(dbPath, contains("regtest"));

    final prefs = SharedPreferencesAsync();
    final savedPrefs = await SavedPrefs.forceTestValues(prefs);
    // The settings page shows and edits the key; start from a known value.
    final savedApiKey = await prefs.getString("coingecko");
    await prefs.setString("coingecko", "");
    // The settings phase puts these back through the UI; a run that dies
    // halfway must not leave the developer's app with the test values.
    final savedTransport = await prefs.getInt("transport");
    final savedDarkMode = await prefs.getBool("dark_mode");
    final savedPalette = await prefs.getString("palette_name");
    // The database manager points the app at the test database.
    final savedDatabase = await prefs.getString("database");
    final savedRecovery = await prefs.getBool("recovery");

    void phase(String name) {
      debugPrint("[wallet-ext] phase: $name");
      rendezvous.publish("phase", name);
    }

    // The runner drops log lines printed just before a failure; a step written
    // to the rendezvous file survives, and pytest prints it.
    void step(String what) {
      debugPrint("[wallet-ext] step: $what");
      rendezvous.publish("step", what);
    }

    // Set once the next process is to put the database preferences back.
    var restarting = false;
    try {
      // ── Set up ───────────────────────────────────────────────────────────
      // Accounts come from the bridge: the new-account page is the base
      // test's job, and the budget here goes to the pages under test.
      final (a, b) = (await tester.runAsync(() async {
        await openTestWallet(
          dbPath: dbPath,
          lwd: config["lwd"] as String,
          docsDir: docsDir.path,
          props: {"coingecko_url": coingeckoUrl},
        );
        return (await _newAccount(_accountA), await _newAccount(_accountB));
      }))!;
      debugPrint("[wallet-ext] accounts A=$a B=$b; documents directory ${docsDir.path}");

      // Exports and imports go through the system file dialogs, which the
      // test cannot drive: a fake saves into the rendezvous directory.
      final picker = FakeFilePicker(Directory("${rendezvous.dir.path}/files")..createSync());
      FilePicker.platform = picker;

      final appRouter = router(true, false, initialLocation: "/accounts");
      await tester.pumpWidget(ZkoolApp(router: appRouter));
      await tester.pump(const Duration(seconds: 1));
      final container = ProviderScope.containerOf(tester.element(find.byType(MaterialApp)));
      await primeProviders(tester, container);
      // The splash starts the height stream at launch; without it the current
      // height stays unknown and the notes tab's lock-recent does nothing.
      unawaited(container.read(synchronizerProvider.notifier).autoSync());
      await waitForAccountList(tester);

      // ── Fund ─────────────────────────────────────────────────────────────
      // pytest pays A at every pool in one transaction, plus a dust note.
      phase("fund");
      final aAddresses = (await tester.runAsync(
        () => getAddresses(uaPools: 7, account: a, c: coinContext.coin),
      ))!;
      rendezvous.publish("a_taddr", aAddresses.taddr!);
      rendezvous.publish("a_saddr", aAddresses.saddr!);
      rendezvous.publish("a_oaddr", aAddresses.oaddr!);
      appRouter.go(accountRoute("/", a));
      await waitForAccountView(tester, _accountA);
      await waitForBalance(tester, a,
          atLeast: zecToZat(fundTaddr) + zecToZat(fundSaddr) + zecToZat(fundOaddr) + zecToZat(fundDust),
          what: "A to receive the funding",);

      // ── Settings ─────────────────────────────────────────────────────────
      // The page saves when it is left, so the fields are changed together,
      // the page left, the stored values checked, and the same again to put
      // the originals back. The light-node switch and the server are only
      // toggled on the page: saved, they would point the sync elsewhere.
      phase("settings");
      final original = container.read(appSettingsProvider).value!;
      final palette = original.paletteName == "red" ? "blue" : "red";
      final changed = original.copyWith(
        transport: 3,
        proxy: _proxy,
        votingConfigUrl: _votingUrl,
        actionsPerSync: _actionsPerSync,
        syncInterval: "0",
        offline: !original.offline,
        getFx: !original.getFx,
        coingecko: _coingeckoKey,
        blockExplorer: _blockExplorer,
        qrSettings: original.qrSettings.copyWith(
          enabled: !original.qrSettings.enabled,
          delay: int.parse(_qrDelay),
          repair: int.parse(_qrRepair),
        ),
        darkMode: !original.darkMode,
        paletteName: palette,
      );

      step("open settings from the account menu");
      await openMenuItem(tester, "Settings");
      await _waitForSettings(tester);

      step("light node switch relabels the server field");
      expect(find.text("Light Node Server"), findsOneWidget);
      await _tapSwitch(tester, switchNamed("light"));
      expect(find.text("Full Node Server"), findsOneWidget);
      await _tapSwitch(tester, switchNamed("light"));
      expect(find.text("Light Node Server"), findsOneWidget);

      step("server picker: regtest has no public list");
      await tapTooltip(tester, "Select from server list");
      await pumpUntil(tester, () => find.text("Select Lightwalletd Server").evaluate().isNotEmpty,
          timeout: shortTimeout, what: "the server picker",);
      await pumpUntil(tester, () => find.text("No servers available").evaluate().isNotEmpty,
          timeout: shortTimeout, what: "the empty server list",);
      await _back(tester);
      await _waitForSettings(tester);
      expect(fieldValue(tester, "lwd"), original.lwd, reason: "leaving the picker keeps the server");

      step("proxy transport enables the proxy field");
      expect(_proxyEnabled(tester), original.transport == 3);
      await _selectTransport(tester, "Proxy", proxyEnabled: true);
      await _enterField(tester, "proxy", _proxy);
      await _enterField(tester, "voting_config_url", _votingUrl);
      await _enterField(tester, "actions_per_sync", _actionsPerSync);

      step("cancel sync zeroes the autosync interval");
      await tapTooltip(tester, "This will cancel the current sync and disable AutoSync");
      await answerDialog(tester, "Do you want to cancel the current sync? AutoSync will be disabled too", "Ok");
      expect(fieldValue(tester, "autosync"), "0");

      await _tapSwitch(tester, switchNamed("offline"));
      await _tapSwitch(tester, switchNamed("fx"));
      await _enterField(tester, "coingecko", _coingeckoKey);
      await _enterField(tester, "block_explorer", _blockExplorer);

      step("QR settings page");
      await _editQrSettings(tester, enabled: changed.qrSettings.enabled, delay: _qrDelay, repair: _qrRepair);

      step("theme page saves at once");
      await _setTheme(tester, container, darkMode: changed.darkMode, palette: palette);

      step("log page");
      await tapTooltip(tester, "View Log");
      await pumpUntil(tester, () => find.text("Log").evaluate().isNotEmpty,
          timeout: shortTimeout, what: "the log page",);
      await _back(tester);
      await _waitForSettings(tester);

      step("expert mode: long-press the version");
      expect(find.text("expert"), findsNothing);
      await _longPressVersion(tester);
      expect(find.text("expert"), findsOneWidget);
      expect(find.text("Cloud Vault"), findsOneWidget, reason: "expert mode shows the vault switch");
      await _longPressVersion(tester);
      expect(find.text("expert"), findsNothing);

      step("leave settings: saves the changes");
      await _back(tester);
      await waitForAccountView(tester, _accountA);
      await _expectSettingsSaved(tester, container, prefs, changed, what: "the changed settings");

      // The Lock button only locks with Pin Lock on, which stays off (#1361).

      step("put the settings back");
      await openMenuItem(tester, "Settings");
      await _waitForSettings(tester);
      await _selectTransport(tester, _transportLabels[original.transport]!, proxyEnabled: original.transport == 3);
      await _enterField(tester, "proxy", original.proxy);
      await _enterField(tester, "voting_config_url", original.votingConfigUrl);
      await _enterField(tester, "actions_per_sync", original.actionsPerSync);
      await _enterField(tester, "autosync", original.syncInterval);
      await _tapSwitch(tester, switchNamed("offline"));
      await _tapSwitch(tester, switchNamed("fx"));
      await _enterField(tester, "coingecko", original.coingecko);
      await _enterField(tester, "block_explorer", original.blockExplorer);
      await _editQrSettings(tester,
          enabled: original.qrSettings.enabled,
          delay: original.qrSettings.delay.toString(),
          repair: original.qrSettings.repair.toString(),);
      await _setTheme(tester, container, darkMode: original.darkMode, palette: original.paletteName);
      await _back(tester);
      await waitForAccountView(tester, _accountA);
      await _expectSettingsSaved(tester, container, prefs, original, what: "the original settings");

      // ── Account view ─────────────────────────────────────────────────────
      // A's four notes (transparent, Sapling, Ironwood with the memo, and
      // Ironwood dust) feed the notes and memos tabs; the exports and the
      // encrypted account file land in the fake picker's directory.
      phase("account");
      await waitForSync(tester, a);

      step("table view of the transactions");
      await openMenuItem(tester, "View: Table");
      await pumpUntil(tester, () => find.text("TxID").evaluate().isNotEmpty,
          timeout: shortTimeout, what: "the transaction table",);
      expect(container.read(appSettingsProvider).value!.transactionTableMode, isTrue);
      await openMenuItem(tester, "View: List");
      await pumpUntil(tester, () => find.text("TxID").evaluate().isEmpty,
          timeout: shortTimeout, what: "the transaction list",);
      expect(container.read(appSettingsProvider).value!.transactionTableMode, isFalse);
      await _export(tester, picker, "Export Transactions", "transactions");

      step("memos tab: search, edit");
      await tapTab(tester, "Memos");
      await pumpUntil(tester, () => find.text(memo).evaluate().isNotEmpty,
          timeout: shortTimeout, what: "the memo in the memos tab",);
      final search = find.ancestor(of: find.text("Search Memos"), matching: find.byType(TextField)).first;
      await tester.enterText(search, "no such memo");
      await pumpFor(tester, const Duration(seconds: 1));
      expect(find.text(memo), findsNothing, reason: "the search filters the memos");
      await tester.enterText(search, memo.substring(0, 6));
      await pumpFor(tester, const Duration(seconds: 1));
      expect(find.text(memo), findsOneWidget);
      // A long press on the bubble turns it into a field, and losing focus
      // saves it. The memo text itself is a CopyableText with its own long
      // press, so press the time label next to it.
      await tester.longPress(find.descendant(of: find.byType(MemoWidget), matching: find.byType(Text)).first);
      await pumpFor(tester, const Duration(seconds: 1));
      final memoField = find.descendant(of: find.byType(MemoWidget), matching: find.byType(TextField));
      await tester.enterText(memoField, _userMemo);
      await tester.pump();
      await tapTab(tester, "Memos");
      await pumpUntil(tester, () => find.text(_userMemo).evaluate().isNotEmpty,
          timeout: shortTimeout, what: "the edited memo",);
      await _export(tester, picker, "Export Memos", "memos");

      step("notes tab: lock, dust, group by pool");
      await tapTab(tester, "Notes");
      var notes = (await tester.runAsync(() => listNotes(account: a, c: coinContext.coin)))!;
      expect(notes.length, 4, reason: "transparent, Sapling, Ironwood and Ironwood dust");
      expect(notes.where((n) => n.locked), isEmpty);
      final dust = notes.firstWhere((n) => n.value == zecToZat(fundDust));
      final transparent = notes.firstWhere((n) => n.pool == 0);
      final sapling = notes.firstWhere((n) => n.pool == 1);
      await pumpUntil(tester, () => find.byKey(ValueKey(dust.id)).evaluate().isNotEmpty,
          timeout: shortTimeout, what: "the notes list",);
      await tester.tap(find.byKey(ValueKey(transparent.id)));
      await pumpFor(tester, const Duration(seconds: 1));
      expect(await _lockedNotes(tester, a), {transparent.id}, reason: "a tap locks the note");
      await tapTooltip(tester, "Hide dust notes");
      expect(find.byKey(ValueKey(dust.id)), findsNothing);
      await tapTooltip(tester, "Show dust notes");
      expect(find.byKey(ValueKey(dust.id)), findsOneWidget);
      await tapTooltip(tester, "Group by pool");
      // Three pools; the transparent one is already locked.
      expect(find.byTooltip("Lock all in pool"), findsNWidgets(2));
      expect(find.byTooltip("Unlock all in pool"), findsOneWidget);
      await tester.tap(find.byTooltip("Lock all in pool").first);
      await pumpFor(tester, const Duration(seconds: 1));
      expect(await _lockedNotes(tester, a), {transparent.id, sapling.id}, reason: "the Sapling pool is locked");
      await tester.tap(find.byTooltip("Unlock all in pool").first);
      await pumpFor(tester, const Duration(seconds: 1));
      expect(await _lockedNotes(tester, a), {sapling.id}, reason: "the transparent pool is unlocked");
      await tapTooltip(tester, "Ungroup notes");
      await tapTooltip(tester, "Toggle all notes");
      await answerDialog(tester, "Do you want to toggle the lock state of every note?", "Ok");
      expect(await _lockedNotes(tester, a), {transparent.id, dust.id, notes.firstWhere((n) => n.value == zecToZat(fundOaddr)).id},
          reason: "every lock state flipped",);
      await tapTooltip(tester, "Unlock all notes");
      await answerDialog(tester, "Do you want to unlock every note?", "Ok");
      expect(await _lockedNotes(tester, a), isEmpty);
      // The lock-recent button does nothing without a current height.
      await pumpUntil(tester, () => container.read(currentHeightProvider).value != null,
          timeout: shortTimeout, what: "the chain height to be known",);
      await tapTooltip(tester, "Lock recently mined notes");
      await pumpUntil(tester, () => find.text("Enter confirmation threshold").evaluate().isNotEmpty && find.byType(TextField).evaluate().isNotEmpty,
          timeout: shortTimeout, what: "the confirmation threshold dialog",);
      await tester.enterText(find.byType(TextField).last, "1000");
      await tester.pump();
      await tester.tap(find.text("Ok").last);
      await pumpFor(tester, const Duration(seconds: 1));
      expect(await _lockedNotes(tester, a), notes.map((n) => n.id).toSet(), reason: "every note is within 1000 blocks of the tip");
      await tapTooltip(tester, "Unlock all notes");
      await answerDialog(tester, "Do you want to unlock every note?", "Ok");
      expect(await _lockedNotes(tester, a), isEmpty);
      await _export(tester, picker, "Export Notes", "notes");

      step("viewing keys");
      await openMenuItem(tester, "Backup");
      await pumpUntil(tester, () => find.text("Viewing Keys").evaluate().isNotEmpty && find.byType(CopyableText).evaluate().isNotEmpty,
          timeout: shortTimeout, what: "the viewing keys page",);
      final pools = (await tester.runAsync(() => getAccountPools(account: a, c: coinContext.coin)))!;
      final ufvk = (await tester.runAsync(() => getAccountUfvk(account: a, pools: pools, c: coinContext.coin)))!;
      expect(copyableWith(ufvk), findsOneWidget, reason: "the full viewing key for every pool");
      await tester.tap(find.text("Incoming"));
      await pumpFor(tester, const Duration(seconds: 1));
      final uivk = (await tester.runAsync(() => getAccountUivk(account: a, pools: pools, c: coinContext.coin)))!;
      expect(copyableWith(uivk), findsOneWidget, reason: "the incoming viewing key");
      // Dropping the transparent pool changes the key.
      await tester.tap(find.text("Trp"));
      await pumpFor(tester, const Duration(seconds: 1));
      final shielded = (await tester.runAsync(() => getAccountUivk(account: a, pools: pools & ~1, c: coinContext.coin)))!;
      expect(copyableWith(shielded), findsOneWidget, reason: "the incoming key without the transparent pool");
      if (!biometricGate) {
        await tapTooltip(tester, "Show Seed Phrase");
        await pumpUntil(tester, () => find.text("Mnemonic").evaluate().isNotEmpty,
            timeout: shortTimeout, what: "the seed phrase",);
      }
      await _back(tester);
      await waitForAccountView(tester, _accountA);

      step("edit account: export, rewind, reset");
      await openMenuItem(tester, "Edit Account");
      await pumpUntil(tester, () => find.text("Account Edit").evaluate().isNotEmpty,
          timeout: shortTimeout, what: "the account edit page",);
      await tapTooltip(tester, "Export an encrypted file of this account");
      await pumpUntil(tester, () => fieldNamed("repeated_password").evaluate().isNotEmpty,
          timeout: shortTimeout, what: "the export password dialog",);
      await tester.enterText(fieldNamed("password"), _exportPassword);
      await tester.enterText(fieldNamed("repeated_password"), _exportPassword);
      await tester.pump();
      await tester.tap(find.text("Ok").last);
      await _savedFile(tester, picker, "$_accountA.bin");
      final heightBefore = await _accountHeight(tester, a);
      await tapTooltip(tester, "Rewind back a few blocks");
      await answerDialog(tester, "Are you sure you want to rewind this account? This will rollback the account to a previous height. You will not lose any funds, but you will need to resync the account", "Ok");
      await pumpUntil(tester, () => find.text("Account Edit").evaluate().isNotEmpty,
          timeout: shortTimeout, what: "the account edit page after the rewind",);
      final heightRewound = await _accountHeight(tester, a);
      expect(heightRewound, lessThan(heightBefore), reason: "rewound");
      await tapTooltip(tester, "Clear and reset account to birth height");
      await answerDialog(tester, "Are you sure you want to reset this account? This will clear all sync data and reset the account to the birth height. You will not lose any funds, but you will need to resync the account", "Ok");
      await pumpFor(tester, const Duration(seconds: 1));
      // The rewind may already land on the birth checkpoint; the reset also
      // drops everything scanned since.
      expect(await _accountHeight(tester, a), lessThanOrEqualTo(heightRewound), reason: "reset to the birth height");
      expect(await tester.runAsync(() => totalBalance(a)), BigInt.zero, reason: "the reset clears the sync data");
      await _back(tester);
      await waitForAccountView(tester, _accountA);
      await waitForSync(tester, a);
      expect(await tester.runAsync(() => totalBalance(a)), zecToZat(fundTaddr) + zecToZat(fundSaddr) + zecToZat(fundOaddr) + zecToZat(fundDust),
          reason: "the balance is back after the resync",);

      // ── Contacts ─────────────────────────────────────────────────────────
      // B is the contact: created and edited on the editor page, exported
      // and imported as vCards through the fake picker, picked on A's send
      // page by name, and deleted.
      phase("contacts");
      final bAddresses = (await tester.runAsync(
        () => getAddresses(uaPools: 7, account: b, c: coinContext.coin),
      ))!;
      step("new contact");
      await openMenuItem(tester, "Account Manager");
      await waitForAccountList(tester);
      await openMenuItem(tester, "Contacts");
      await _waitForContacts(tester);
      await tapTooltip(tester, "New contact");
      await pumpUntil(tester, () => find.text("New Contact").evaluate().isNotEmpty && fieldNamed("address_0").evaluate().isNotEmpty,
          timeout: shortTimeout, what: "the new contact page",);
      await _enterField(tester, "name", _accountB);
      await _enterField(tester, "address_0", bAddresses.ua!);
      await _enterField(tester, "notes", _contactNotes);
      await _back(tester); // saves on the way out
      await _waitForContacts(tester);
      var contacts = await _contacts(tester);
      expect(contacts.single.name, _accountB);
      expect(contacts.single.addresses, [bAddresses.ua!]);
      expect(contacts.single.notes, _contactNotes);
      expect(find.widgetWithText(ListTile, _accountB), findsOneWidget, reason: "the contact is listed");

      step("edit contact: add and remove an address");
      await tester.tap(find.widgetWithText(ListTile, _accountB));
      await pumpUntil(tester, () => find.text("Edit Contact").evaluate().isNotEmpty,
          timeout: shortTimeout, what: "the contact editor",);
      await tester.tap(find.text("Add Address"));
      await pumpFor(tester, const Duration(seconds: 1));
      await _enterField(tester, "address_1", bAddresses.taddr!);
      await _back(tester);
      await _waitForContacts(tester);
      contacts = await _contacts(tester);
      expect(contacts.single.addresses, [bAddresses.ua!, bAddresses.taddr!], reason: "the second address is saved");
      await tester.tap(find.widgetWithText(ListTile, _accountB));
      await pumpUntil(tester, () => fieldNamed("address_1").evaluate().isNotEmpty,
          timeout: shortTimeout, what: "the contact editor with two addresses",);
      await tapTooltip(tester, "Remove address");
      expect(fieldNamed("address_1"), findsNothing);
      // The removed address is still saved and exported (#1362): the checks
      // below expect both addresses until that is fixed.
      await _enterField(tester, "notes", "$_contactNotes edited");
      step("export the contact as a vCard");
      await tapTooltip(tester, "Export as vCard");
      await _savedFile(tester, picker, "$_accountB.vcf");
      final vcard = File(picker.saved.last).readAsStringSync();
      expect(vcard, contains("FN:$_accountB"));
      expect(vcard, contains("zcash:${bAddresses.ua!}"));
      await _back(tester);
      await _waitForContacts(tester);
      contacts = await _contacts(tester);
      expect(contacts.single.addresses, [bAddresses.ua!, bAddresses.taddr!], reason: "both addresses kept (#1362)");
      expect(contacts.single.notes, "$_contactNotes edited");

      step("import a vCard into a new contact, and into the list");
      // The editor's export is not importable (#1363); the list's export
      // is, and it has only B so far.
      await tapTooltip(tester, "Export as vCard");
      await _savedFile(tester, picker, "contacts.vcf");
      final exported = File(picker.saved.last).readAsStringSync();
      expect(exported, contains("FN:$_accountB"));
      // vCard folds long lines at 75 characters.
      expect(exported.replaceAll("\r\n ", ""), contains("zcash:${bAddresses.ua!}"));
      final carolVcard = File("${picker.dir.path}/$_contactImported.vcf")
        ..writeAsStringSync(exported.replaceAll("FN:$_accountB", "FN:$_contactImported"));
      picker.picks.add(carolVcard.path);
      await tapTooltip(tester, "New contact");
      await pumpUntil(tester, () => find.text("New Contact").evaluate().isNotEmpty,
          timeout: shortTimeout, what: "the new contact page for the import",);
      await tapTooltip(tester, "Import from vCard");
      await pumpUntil(tester, () => fieldValue(tester, "name") == _contactImported,
          timeout: shortTimeout, what: "the imported name in the editor",);
      // The export's "Zcash addresses:" label comes back as a note line (#1365).
      expect(fieldValue(tester, "notes"), startsWith("$_contactNotes edited"), reason: "the imported notes");
      // The import has already stored the contact (#1366) and left the first
      // address field empty (#1364): fill it and save under another name.
      await _enterField(tester, "address_0", bAddresses.ua!);
      await _enterField(tester, "name", _contactImportedCopy);
      await _back(tester);
      await _waitForContacts(tester);
      await pumpUntil(tester, () => find.widgetWithText(ListTile, _contactImportedCopy).evaluate().isNotEmpty,
          timeout: shortTimeout, what: "the saved copy in the list",);
      await pumpUntil(tester, () => find.widgetWithText(ListTile, _contactImported).evaluate().isNotEmpty,
          timeout: shortTimeout, what: "the imported contact in the list",);
      final daveVcard = File("${picker.dir.path}/$_contactListImported.vcf")
        ..writeAsStringSync(exported.replaceAll("FN:$_accountB", "FN:$_contactListImported"));
      picker.picks.add(daveVcard.path);
      await tapTooltip(tester, "Import from vCard");
      await pumpUntil(tester, () => find.widgetWithText(ListTile, _contactListImported).evaluate().isNotEmpty,
          timeout: shortTimeout, what: "the contact imported into the list",);
      contacts = await _contacts(tester);
      expect(contacts.map((x) => x.name).toSet(), {_accountB, _contactImported, _contactImportedCopy, _contactListImported});

      step("export every contact as a vCard");
      await tapTooltip(tester, "Export as vCard");
      await _savedFile(tester, picker, "contacts.vcf");
      final all = File(picker.saved.last).readAsStringSync();
      for (final name in [_accountB, _contactImported, _contactImportedCopy, _contactListImported]) {
        expect(all, contains("FN:$name"));
      }

      step("delete the imported contacts");
      for (final name in [_contactImported, _contactImportedCopy, _contactListImported]) {
        await tester.tap(find.widgetWithText(ListTile, name));
        await pumpUntil(tester, () => find.text("Edit Contact").evaluate().isNotEmpty,
            timeout: shortTimeout, what: "the editor of $name",);
        await tapTooltip(tester, "Delete contact");
        await answerDialog(tester, "'$name' will be permanently deleted.", "Ok");
        await _waitForContacts(tester);
        await pumpUntil(tester, () => find.widgetWithText(ListTile, name).evaluate().isEmpty,
            timeout: shortTimeout, what: "$name to leave the list",);
      }
      contacts = await _contacts(tester);
      expect(contacts.map((x) => x.name), [_accountB]);

      step("pick the contact as a send recipient");
      await _back(tester);
      await waitForAccountList(tester);
      appRouter.go(accountRoute("/", a));
      await waitForAccountView(tester, _accountA);
      await pumpFor(tester, const Duration(seconds: 1)); // the route transition
      await tapTooltip(tester, "Send funds to one or many addresses");
      await pumpUntil(tester, () => fieldNamed("address").evaluate().isNotEmpty,
          timeout: shortTimeout, what: "the send page",);
      await tester.enterText(fieldNamed("address"), "@${_accountB.substring(0, 2)}");
      await pumpUntil(tester, () => find.widgetWithText(ListTile, _accountB).evaluate().isNotEmpty,
          timeout: shortTimeout, what: "the contact suggestion",);
      await tester.tap(find.widgetWithText(ListTile, _accountB));
      await pumpUntil(tester, () => fieldValue(tester, "address") == bAddresses.ua!,
          timeout: shortTimeout, what: "the contact's address in the recipient field",);
      await _back(tester);
      await waitForAccountView(tester, _accountA);

      // ── Receive ──────────────────────────────────────────────────────────
      // The receive page's pool picker, QR page and new-address generation,
      // the address list with its filters, and the transparent address scan.
      phase("receive");
      step("receive page: pools, QR, new addresses");
      await tapTooltip(tester, "Show the account receiving addresses");
      await pumpUntil(tester, () => find.text("Receive Funds").evaluate().isNotEmpty && find.byType(CopyableText).evaluate().isNotEmpty,
          timeout: shortTimeout, what: "the receive page",);
      final aPools = (await tester.runAsync(() => getAccountPools(account: a, c: coinContext.coin)))!;
      // The unified address starts with the shielded pools only.
      final shieldedUa = (await tester.runAsync(() => getAddresses(uaPools: aPools & 6, account: a, c: coinContext.coin)))!.ua!;
      expect(copyableWith(shieldedUa), findsOneWidget, reason: "the shielded-only unified address");
      await tester.tap(find.text("Trp"));
      await pumpFor(tester, const Duration(seconds: 1));
      final fullUa = (await tester.runAsync(() => getAddresses(uaPools: (aPools & 6) | 1, account: a, c: coinContext.coin)))!.ua!;
      expect(copyableWith(fullUa), findsOneWidget, reason: "the unified address with the transparent receiver");
      await tapTooltip(tester, "Show address as a QR Code");
      await pumpUntil(tester, () => find.text("Unified Address").evaluate().isNotEmpty && find.text("Receive Funds").evaluate().isEmpty,
          timeout: shortTimeout, what: "the QR page",);
      await _back(tester);
      await pumpUntil(tester, () => find.text("Receive Funds").evaluate().isNotEmpty,
          timeout: shortTimeout, what: "the receive page after the QR page",);
      final oldTaddr = aAddresses.taddr!;
      expect(copyableWith(oldTaddr), findsOneWidget);
      await tapTooltip(tester, "Generate a new set of addresses (transparent/sapling and orchard). Previous addresses can still receive funds");
      await answerDialog(tester, "Do you want to generate a new set of addresses? Previous addresses can still receive funds", "Ok");
      await pumpUntil(tester, () => copyableWith(oldTaddr).evaluate().isEmpty && find.text("Receive Funds").evaluate().isNotEmpty,
          timeout: shortTimeout, what: "the new addresses",);
      final newTaddr = (await tester.runAsync(() => getAddresses(uaPools: 7, account: a, c: coinContext.coin)))!.taddr!;
      expect(newTaddr, isNot(oldTaddr));
      expect(copyableWith(newTaddr), findsOneWidget, reason: "the next transparent address");

      step("address list and its filters");
      await tapTooltip(tester, "View All Addresses");
      await pumpUntil(tester, () => find.text("Addresses").evaluate().isNotEmpty && find.text("Scope").evaluate().isNotEmpty,
          timeout: shortTimeout, what: "the address list",);
      // One row per address and index. The funded index and the one just
      // generated depend on the seed (indexes without a Sapling address are
      // skipped), so they come from the data. The rows are built as they
      // scroll into view.
      final counts = (await tester.runAsync(() => fetchAddressTxCount(aggregate: false, poolFilter: aPools, account: a, c: coinContext.coin)))!;
      debugPrint("[wallet-ext] address rows: ${counts.map((x) => "p${x.pool} s${x.scope} i${x.dindex} ${x.txCount}tx").join(", ")}");
      final funded = counts.firstWhere((x) => x.pool == 0 && x.scope == 0 && x.txCount > 0);
      final newest = counts.where((x) => x.scope == 0).fold(0, (m, x) => x.dindex > m ? x.dindex : m);
      expect(newest, greaterThan(funded.dindex), reason: "a newer index after the generation");
      await _scrollTo(tester, find.textContaining("Idx ${funded.dindex} · 1 txs").first);
      await _scrollTo(tester, find.textContaining("Idx $newest · 0 txs").first);
      await _scrollTo(tester, find.text(_trimmed(oldTaddr)));
      await _pickFilter(tester, "Show", "Used");
      expect(find.textContaining("· 0 txs"), findsNothing, reason: "only used addresses");
      await _scrollTo(tester, find.text(_trimmed(oldTaddr)));
      await _pickFilter(tester, "Show", "Unused");
      expect(find.text(_trimmed(oldTaddr)), findsNothing, reason: "only unused addresses");
      expect(find.textContaining("· 0 txs"), findsWidgets);
      await _pickFilter(tester, "Show", "All");
      // The change addresses are derived up front, one per pool.
      await _pickFilter(tester, "Scope", "Change");
      expect(find.byType(ListTile), findsWidgets);
      expect(find.text(_trimmed(oldTaddr)), findsNothing, reason: "the funded address is external");
      await _pickFilter(tester, "Scope", "External");
      await _scrollTo(tester, find.text(_trimmed(oldTaddr)));
      await _pickFilter(tester, "Scope", "All");
      await _pickFilter(tester, "UA", "On");
      await pumpUntil(tester, () => find.textContaining("Unified · ").evaluate().isNotEmpty,
          timeout: shortTimeout, what: "the unified rows",);
      expect(find.text(_trimmed(oldTaddr)), findsNothing, reason: "the pools are aggregated into unified addresses");
      await _pickFilter(tester, "UA", "Off");
      await _scrollTo(tester, find.text(_trimmed(oldTaddr)));
      // Dropping the transparent pool removes its rows.
      await tester.tap(find.text("Trp"));
      await pumpUntil(tester, () => find.text(_trimmed(oldTaddr)).evaluate().isEmpty,
          timeout: shortTimeout, what: "the transparent rows to go",);
      await tester.tap(find.text("Trp"));
      await pumpFor(tester, const Duration(seconds: 1));
      await _scrollTo(tester, find.text(_trimmed(oldTaddr)));
      await tapTooltip(tester, "Hide filters");
      expect(find.text("Scope"), findsNothing);
      await tapTooltip(tester, "Show filters");
      expect(find.text("Scope"), findsOneWidget);
      await _back(tester);
      await pumpUntil(tester, () => find.text("Receive Funds").evaluate().isNotEmpty,
          timeout: shortTimeout, what: "the receive page after the address list",);

      step("scan for transparent addresses");
      await tapTooltip(tester, "Find other transparent addresses. If you restored from a wallet that has address rotation (such as Ledger, Exodus, etc), Tap, then Reset and Sync");
      await pumpUntil(tester, () => find.text("Scan for alternative transparent addresses?").evaluate().isNotEmpty && fieldNamed("gap").evaluate().isNotEmpty,
          timeout: shortTimeout, what: "the transparent scan dialog",);
      await _enterField(tester, "gap", "3");
      await tester.tap(find.text("Run"));
      await pumpUntil(tester, () => find.text("Scan Completed").evaluate().isNotEmpty,
          timeout: chainTimeout, what: "the scan to complete",);
      await tester.tap(find.text("Close"));
      await pumpUntil(tester, () => find.text("Scan for alternative transparent addresses?").evaluate().isEmpty,
          timeout: shortTimeout, what: "the scan dialog to close",);
      await _back(tester);
      await waitForAccountView(tester, _accountA);

      // ── Errors ───────────────────────────────────────────────────────────
      // Two failing calls on the send page: an unknown contact name, which
      // the page rejects, and a payment above the balance, which the summary
      // page fails to build. Both end in the error dialog.
      phase("errors");
      step("unknown contact");
      await tapTooltip(tester, "Send funds to one or many addresses");
      await pumpUntil(tester, () => fieldNamed("address").evaluate().isNotEmpty,
          timeout: shortTimeout, what: "the send page for the errors",);
      await tester.enterText(fieldNamed("address"), "@nobody");
      await tester.pump();
      await enterAmount(tester, "0.01");
      await tapTooltip(tester, "Send transaction (including queued recipients)");
      await _expectErrorDialog(tester, "Unknown contact: nobody");
      step("payment above the balance");
      await tapTooltip(tester, "Clear Form Inputs");
      await enterRecipient(tester, bAddresses.ua!, "100");
      await tapTooltip(tester, "Send transaction (including queued recipients)");
      await pumpUntil(tester, () => find.text("Extra Options").evaluate().isNotEmpty,
          timeout: shortTimeout, what: "the send options page",);
      await tapTooltip(tester, "Send (Summary and Confirmation)");
      await _expectErrorDialog(tester, null);
      appRouter.go(accountRoute("/", a));
      await waitForAccountView(tester, _accountA);
      await pumpFor(tester, const Duration(seconds: 1));

      // ── Database manager ─────────────────────────────────────────────────
      // Last, since it changes databases. The manager is scheduled from the
      // settings page and opens when the app starts again, and a start is a
      // new process (the Rust side initialises once): this process ends here
      // and the orchestrator runs wallet_ext_db_ui_test.dart, which works the
      // manager on a scratch database created now.
      phase("database");
      final testDbName = dbPath.split("/").last.replaceFirst(RegExp(r"\.db$"), "");
      step("create a scratch database");
      await tester.runAsync(() => Coin().openDatabase(dbFilepath: "${docsDir.path}/$_scratchDb.db", password: null));
      expect(File("${docsDir.path}/$_scratchDb.db").existsSync(), isTrue);
      step("schedule the database manager");
      await openMenuItem(tester, "Settings");
      await _waitForSettings(tester);
      await tapTooltip(tester, "Database Manager");
      await answerDialog(tester, "The Database Manager will open when you restart the app. Do you want to schedule it now?", "Ok");
      await answerDialog(tester, "Restart the app to enter the database manager", "Ok");
      expect(await prefs.getBool("recovery"), isTrue, reason: "the manager is scheduled for the next start");
      // Opened the usual way, this database would be the one the settings
      // name; the next process reopens it by that name and puts the
      // developer's values back.
      await prefs.setString("database", testDbName);
      rendezvous.publish("saved_database", savedDatabase ?? "");
      rendezvous.publish("saved_recovery", savedRecovery?.toString() ?? "");
      debugPrint("[wallet-ext] database preference was '$savedDatabase', recovery $savedRecovery");
      restarting = true;
      phase("restart");
      await pumpFor(tester, const Duration(seconds: 2));
    } finally {
      await savedPrefs.restore(prefs);
      await _restorePref(prefs, "coingecko", savedApiKey);
      await _restorePref(prefs, "transport", savedTransport);
      await _restorePref(prefs, "dark_mode", savedDarkMode);
      await _restorePref(prefs, "palette_name", savedPalette);
      if (!restarting) {
        await _restorePref(prefs, "database", savedDatabase);
        await _restorePref(prefs, "recovery", savedRecovery);
      }
    }
  }, timeout: const Timeout(Duration(minutes: 12)));
}

/// Picks "Export …" from the account view's menu and checks the file the
/// fake picker saved; [what] names it in the log.
Future<void> _export(WidgetTester tester, FakeFilePicker picker, String item, String what) async {
  final before = picker.saved.length;
  await openMenuItem(tester, item);
  await pumpUntil(tester, () => picker.saved.length > before,
      timeout: shortTimeout, what: "the $what export to be saved",);
  final path = picker.saved.last;
  await answerDialog(tester, "$path Saved", "Ok");
  expect(File(path).lengthSync(), greaterThan(0), reason: "the $what export is not empty");
}

/// Waits for the fake picker to save [fileName] with some content.
Future<void> _savedFile(WidgetTester tester, FakeFilePicker picker, String fileName) async {
  await pumpUntil(tester, () => picker.saved.any((p) => p.endsWith("/$fileName")),
      timeout: shortTimeout, what: "$fileName to be saved",);
  expect(File(picker.saved.lastWhere((p) => p.endsWith("/$fileName"))).lengthSync(), greaterThan(0));
}

/// Waits for the error dialog, logs its message, checks it holds [contains]
/// when given, and closes it.
Future<void> _expectErrorDialog(WidgetTester tester, String? contains) async {
  // The dialog's title is not a plain Text widget; its message is selectable.
  await pumpUntil(tester, () => find.byType(SelectableText).evaluate().isNotEmpty && find.text("Ok").evaluate().isNotEmpty,
      timeout: const Duration(minutes: 2), what: "the error dialog",);
  // The page underneath may show selectable text too (a payment URI); the
  // dialog's is the topmost.
  final message = (find.byType(SelectableText).evaluate().last.widget as SelectableText).data ?? "";
  debugPrint("[wallet-ext] error dialog: $message");
  if (contains != null) expect(message, stringContainsInOrder([contains]));
  // The dialog slides in; a tap before it settles misses the button.
  await pumpFor(tester, const Duration(seconds: 1));
  final ok = find.text("Ok").last;
  final okRect = tester.getRect(ok);
  final window = tester.view.physicalSize / tester.view.devicePixelRatio;
  debugPrint("[wallet-ext] error dialog Ok at $okRect, message at ${tester.getRect(find.byType(SelectableText).last)}, window $window");
  if (okRect.bottom <= window.height) {
    await tester.tap(ok);
  } else {
    // The dialog grows with the backtrace in the message and its button
    // leaves the window (#1368); a tap outside the dialog dismisses it too.
    debugPrint("[wallet-ext] the Ok button is off screen; dismissing the dialog by its barrier");
    await tester.tapAt(const Offset(4, 4));
  }
  await pumpFor(tester, const Duration(seconds: 1));
  expect(find.text("Ok"), findsNothing, reason: "the error dialog closed");
  await pumpFor(tester, const Duration(milliseconds: 500));
}

/// Picks [value] on the address list's filter row labelled [label].
Future<void> _pickFilter(WidgetTester tester, String label, String value) async {
  final row = find.ancestor(of: find.text(label), matching: find.byType(Row)).first;
  await tester.tap(find.descendant(of: row, matching: find.text(value)));
  await pumpFor(tester, const Duration(seconds: 1));
}

/// Scrolls the address list until [finder] is on screen: the rows are
/// built lazily, so an off-screen row is not in the tree.
Future<void> _scrollTo(WidgetTester tester, Finder finder) async {
  if (finder.evaluate().isEmpty) {
    debugPrint("[wallet-ext] scrolling for $finder; scrollables: ${find.bySubtype<ScrollView>().evaluate().map((e) => e.widget.runtimeType).join(", ")}");
    await tester.dragUntilVisible(finder, find.byType(ListView).last, const Offset(0, -100));
    await tester.pump();
  }
  expect(finder, findsOneWidget);
}

/// An address as the address list shows it.
String _trimmed(String address) =>
    address.length > 20 ? '${address.substring(0, 10)}...${address.substring(address.length - 8)}' : address;

Future<void> _waitForContacts(WidgetTester tester) => pumpUntil(
      tester,
      () => find.text("Contacts").evaluate().isNotEmpty && find.byTooltip("New contact").evaluate().isNotEmpty,
      timeout: shortTimeout,
      what: "the contacts page",
    );

Future<List<Contact>> _contacts(WidgetTester tester) async =>
    (await tester.runAsync(() => listContacts(c: coinContext.coin)))!;

Future<Set<int>> _lockedNotes(WidgetTester tester, int account) async {
  final notes = (await tester.runAsync(() => listNotes(account: account, c: coinContext.coin)))!;
  return notes.where((n) => n.locked).map((n) => n.id).toSet();
}

Future<int> _accountHeight(WidgetTester tester, int account) async {
  final accounts = (await tester.runAsync(() => listAccounts(c: coinContext.coin)))!;
  return accounts.firstWhere((x) => x.id == account).height;
}

Future<void> _restorePref(SharedPreferencesAsync prefs, String key, Object? value) => switch (value) {
      null => prefs.remove(key),
      final bool v => prefs.setBool(key, v),
      final int v => prefs.setInt(key, v),
      final String v => prefs.setString(key, v),
      _ => throw ArgumentError("unsupported preference type for $key"),
    };

Future<int> _newAccount(String name) => newAccount(
      na: NewAccount(
        name: name,
        restore: false,
        key: "",
        aindex: 0,
        birth: 1,
        folder: "",
        useInternal: false,
        internal: false,
        hw: 0,
      ),
      c: coinContext.coin,
    );

Future<void> _waitForSettings(WidgetTester tester) => pumpUntil(
      tester,
      () => find.text("Settings").evaluate().isNotEmpty && fieldNamed("lwd").evaluate().isNotEmpty,
      timeout: shortTimeout,
      what: "the settings page",
    );

/// Taps the back arrow and lets the route transition finish: a tap during
/// the transition lands on the page that is leaving.
Future<void> _back(WidgetTester tester) async {
  await tester.pageBack();
  await pumpFor(tester, const Duration(seconds: 1));
}

bool _proxyEnabled(WidgetTester tester) => tester.widget<FormBuilderTextField>(fieldNamed("proxy")).enabled;

/// Picks [label] on the transport selector and waits for the proxy field
/// to follow, retrying the tap: an overlay (a tooltip, a transition) can
/// take the first one. The texts on screen are logged when it does.
Future<void> _selectTransport(WidgetTester tester, String label, {required bool proxyEnabled}) async {
  for (var attempt = 1; attempt <= 5; attempt++) {
    final segment = find.text(label);
    await tester.ensureVisible(segment);
    await tester.pump();
    await tester.tap(segment, warnIfMissed: false);
    await pumpFor(tester, const Duration(seconds: 1));
    if (_proxyEnabled(tester) == proxyEnabled) return;
    debugPrint("[wallet-ext] transport $label not taken, attempt $attempt; visible: ${visibleTexts().join(" | ")}");
  }
  fail("the transport $label was not selected");
}

Future<void> _tapSwitch(WidgetTester tester, Finder finder) async {
  await tester.ensureVisible(finder);
  await tester.pump();
  await tester.tap(finder);
  await pumpFor(tester, const Duration(seconds: 1));
}

Future<void> _enterField(WidgetTester tester, String name, String value) async {
  final field = fieldNamed(name);
  await tester.ensureVisible(field);
  await tester.pump();
  if (value.isEmpty) {
    // enterText("") leaves the field as it was on the desktop binding; the
    // form field follows its controller, so clear that instead.
    tester.widget<TextField>(find.descendant(of: field, matching: find.byType(TextField))).controller!.clear();
  } else {
    await tester.enterText(field, value);
  }
  await tester.pump();
  expect(fieldValue(tester, name), value, reason: "field $name reads back");
}

/// Opens the QR settings sub-page, sets its fields and leaves; the page hands
/// its values to the settings page when it pops.
Future<void> _editQrSettings(WidgetTester tester, {required bool enabled, required String delay, required String repair}) async {
  final row = find.ancestor(of: find.text("File Transmission via QR Codes"), matching: find.byType(Row)).first;
  await tester.ensureVisible(row);
  await tester.pump();
  await tester.tap(find.descendant(of: row, matching: find.byIcon(Icons.chevron_right)));
  await pumpUntil(tester, () => find.text("QR Codes").evaluate().isNotEmpty && switchNamed("enabled").evaluate().isNotEmpty,
      timeout: shortTimeout, what: "the QR settings page",);
  final state = (switchNamed("enabled").evaluate().first as StatefulElement).state as FormBuilderFieldState;
  if ((state.value as bool) != enabled) await _tapSwitch(tester, switchNamed("enabled"));
  await _enterField(tester, "delay", delay);
  await _enterField(tester, "repair", repair);
  await _back(tester);
  await _waitForSettings(tester);
}

/// Opens the theme sub-page and picks [darkMode] and [palette]; unlike the
/// rest of the settings, the theme is stored as soon as it changes.
Future<void> _setTheme(WidgetTester tester, ProviderContainer container, {required bool darkMode, required String palette}) async {
  await tapTooltip(tester, "Theme");
  await pumpUntil(tester, () => find.text("Theme").evaluate().isNotEmpty && find.text("Dark Mode").evaluate().isNotEmpty,
      timeout: shortTimeout, what: "the theme page",);
  if (container.read(appSettingsProvider).value!.darkMode != darkMode) {
    await tester.tap(find.text("Dark Mode"));
    await pumpFor(tester, const Duration(seconds: 1));
  }
  final button = find.widgetWithText(ElevatedButton, palette);
  await tester.ensureVisible(button);
  await tester.pump();
  await tester.tap(button);
  await pumpUntil(tester, () {
    final s = container.read(appSettingsProvider).value;
    return s != null && s.darkMode == darkMode && s.paletteName == palette;
  }, timeout: shortTimeout, what: "the theme $palette (dark: $darkMode) to be stored",);
  await _back(tester);
  await _waitForSettings(tester);
}

/// Long-presses the version line (`version+build`) at the bottom of the
/// settings page; the page fills it in after an async lookup.
Future<void> _longPressVersion(WidgetTester tester) async {
  final version = find.byWidgetPredicate((w) => w is Text && w.data != null && RegExp(r"^\d+(\.\d+)*\+").hasMatch(w.data!));
  await pumpUntil(tester, () => version.evaluate().isNotEmpty,
      timeout: shortTimeout, what: "the version line on the settings page",);
  await tester.ensureVisible(version);
  await tester.pump();
  await tester.longPress(version);
  await pumpFor(tester, const Duration(seconds: 1));
}

/// Checks the settings the page writes when it is left: the provider, the
/// wallet props and the preferences.
Future<void> _expectSettingsSaved(
  WidgetTester tester,
  ProviderContainer container,
  SharedPreferencesAsync prefs,
  AppSettings expected, {
  required String what,
}) async {
  await pumpUntil(tester, () => container.read(appSettingsProvider).value?.proxy == expected.proxy,
      timeout: shortTimeout, what: "$what to be saved",);
  final s = container.read(appSettingsProvider).value!;
  expect(s.transport, expected.transport, reason: "transport");
  expect(s.proxy, expected.proxy, reason: "proxy");
  expect(s.votingConfigUrl, expected.votingConfigUrl, reason: "voting config url");
  expect(s.actionsPerSync, expected.actionsPerSync, reason: "actions per sync");
  expect(s.syncInterval, expected.syncInterval, reason: "autosync interval");
  expect(s.offline, expected.offline, reason: "offline");
  expect(s.getFx, expected.getFx, reason: "auto fetch price");
  expect(s.coingecko, expected.coingecko, reason: "coingecko key");
  expect(s.blockExplorer, expected.blockExplorer, reason: "block explorer");
  expect(s.qrSettings.enabled, expected.qrSettings.enabled, reason: "qr enabled");
  expect(s.qrSettings.delay, expected.qrSettings.delay, reason: "qr delay");
  expect(s.qrSettings.repair, expected.qrSettings.repair, reason: "qr repair");
  expect(s.darkMode, expected.darkMode, reason: "dark mode");
  expect(s.paletteName, expected.paletteName, reason: "palette");
  expect(s.lwd, expected.lwd, reason: "server unchanged");
  expect(s.isLightNode, expected.isLightNode, reason: "light node unchanged");
  final stored = (await tester.runAsync(() async => (
        proxy: await getProp(key: "proxy", c: coinContext.coin),
        syncInterval: await getProp(key: "sync_interval", c: coinContext.coin),
        qrDelay: await getProp(key: "qr_delay", c: coinContext.coin),
        offline: await prefs.getBool("offline"),
        transport: await prefs.getInt("transport"),
        coingecko: await prefs.getString("coingecko"),
      )))!;
  expect(stored.proxy, expected.proxy, reason: "proxy prop");
  expect(stored.syncInterval, expected.syncInterval, reason: "sync_interval prop");
  expect(stored.qrDelay, expected.qrSettings.delay.toString(), reason: "qr_delay prop");
  expect(stored.offline, expected.offline, reason: "offline pref");
  expect(stored.transport, expected.transport, reason: "transport pref");
  expect(stored.coingecko, expected.coingecko, reason: "coingecko pref");
}
