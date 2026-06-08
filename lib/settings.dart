import 'dart:io';

import 'package:awesome_dialog/awesome_dialog.dart';
import 'package:flutter/material.dart';
import 'package:flutter_passkey_service/pigeons/messages.g.dart' show PasskeyException;
import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated.dart';
import 'package:flutter/services.dart';
import 'package:flutter_form_builder/flutter_form_builder.dart';
import 'package:flutter_svg/flutter_svg.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:form_builder_validators/form_builder_validators.dart';
import 'package:gap/gap.dart';
import 'package:go_router/go_router.dart';
import 'package:package_info_plus/package_info_plus.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:showcaseview/showcaseview.dart';
import 'package:zkool/network.dart';
import 'package:zkool/router.dart';
import 'package:zkool/src/rust/api/coin.dart';
import 'package:zkool/src/rust/api/db.dart';
import 'package:zkool/src/rust/api/sync.dart';
import 'package:zkool/src/rust/api/account.dart';
import 'package:zkool/store.dart';
import 'package:zkool/theme_mode.dart';
import 'package:zkool/utils.dart';
import 'package:zkool/widgets/error_display.dart';
import 'package:zkool/main.dart';

/// Named mainnet block-explorer templates. Each maps a display label to a URL
/// template with a {txid} placeholder (no network placeholder — these are the
/// mainnet hosts).
const Map<String, String> kBlockExplorers = {
  "zcashexplorer.app": "https://mainnet.zcashexplorer.app/transactions/{txid}",
  "zcashinfo.com": "https://zcashinfo.com/tx/{txid}",
  "cipherscan.app": "https://cipherscan.app/tx/{txid}",
};

/// Named testnet block-explorer templates (testnet hosts of the same explorers).
const Map<String, String> kTestnetBlockExplorers = {
  "testnet.zcashexplorer.app": "https://testnet.zcashexplorer.app/transactions/{txid}",
  "testnet.cipherscan.app": "https://testnet.cipherscan.app/tx/{txid}",
};

/// The explorer set to offer for [net] (testnet has its own hosts; regtest has
/// none; everything else uses the mainnet set).
Map<String, String> blockExplorersFor(ZNetwork net) {
  switch (net) {
    case ZNetwork.testnet:
      return kTestnetBlockExplorers;
    case ZNetwork.regtest:
      return const {};
    case ZNetwork.mainnet:
      return kBlockExplorers;
  }
}

final logID = GlobalKey();
final lightnodeID = GlobalKey();
final lwdID = GlobalKey();
final torID = GlobalKey();
final coingeckoID = GlobalKey();
final actionsID = GlobalKey();
final autosyncID = GlobalKey();
final cancelID = GlobalKey();
final fxID = GlobalKey();
final pinLockID = GlobalKey();
final offlineID = GlobalKey();
final useQRID = GlobalKey();
final blockExplorerID = GlobalKey();

class SettingsPage extends ConsumerStatefulWidget {
  const SettingsPage({super.key});

  @override
  ConsumerState<SettingsPage> createState() => SettingsPageState();
}

class SettingsPageState extends ConsumerState<SettingsPage> with RouteAware {
  late Coin c = coinContext.coin;
  AppSettings? settings;

  @override
  void initState() {
    super.initState();
    Future(() async {
      final settings = await ref.read(appSettingsProvider.future);
      setState(() => this.settings = settings);
    });
  }

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    final route = ModalRoute.of(context);
    if (route is PageRoute) {
      routeObserver.subscribe(this, route);
    }
  }

  @override
  Widget build(BuildContext context) {
    if (settings == null) return blank(context);
    return SettingsForm(
      settings!,
      onChanged: (settings) async {
        final prefs = SharedPreferencesAsync();
        await prefs.setString("database", settings.dbName);
        await putProp(key: "is_light_node", value: settings.isLightNode.toString(), c: c);
        await putProp(key: "lwd", value: settings.lwd, c: c);
        await putProp(key: "block_explorer", value: settings.blockExplorer, c: c);
        await putProp(key: "actions_per_sync", value: settings.actionsPerSync, c: c);
        await putProp(key: "sync_interval", value: settings.syncInterval, c: c);
        await prefs.setBool("pin_lock", settings.needPin);
        await prefs.setBool("offline", settings.offline);
        await prefs.setBool("use_tor", settings.useTor);
        await putProp(key: "proxy", value: settings.proxy, c: c);
        await prefs.setBool("get_fx", settings.getFx);
        await prefs.setString("coingecko", settings.coingecko);
        await prefs.setString("fx_currency", settings.fxCurrency);
        await putProp(key: "qr_enabled", value: settings.qrSettings.enabled.toString(), c: c);
        await putProp(key: "qr_size", value: settings.qrSettings.size.toString(), c: c);
        await putProp(key: "qr_ecLevel", value: settings.qrSettings.ecLevel.toString(), c: c);
        await putProp(key: "qr_delay", value: settings.qrSettings.delay.toString(), c: c);
        await putProp(key: "qr_repair", value: settings.qrSettings.repair.toString(), c: c);
        c = c.setLwd(url: settings.lwd, serverType: settings.isLightNode ? 0 : 1);
        c = await c.setUseTor(useTor: settings.useTor);
        c = c.setProxy(proxy: settings.proxy);
        await prefs.setBool("vault", settings.vault);
        await prefs.setBool("expert_mode", settings.expertMode);
        coinContext.set(coin: c);
        ref.read(priceProvider.notifier).setAutoFetchFx(settings.getFx, settings.coingecko, settings.fxCurrency);
        ref.invalidate(appSettingsProvider);
      },
    );
  }
}

class SettingsForm extends ConsumerStatefulWidget {
  final void Function(AppSettings) onChanged;
  final AppSettings settings; // original settings
  const SettingsForm(this.settings, {super.key, required this.onChanged});
  @override
  ConsumerState<SettingsForm> createState() => SettingsFormState();
}

class SettingsFormState extends ConsumerState<SettingsForm> {
  final formKey = GlobalKey<FormBuilderState>();
  late AppSettings settings = widget.settings; // updated settings

  String dbFullPath = "";
  String versionString = "";
  List<String> topServers = [];
  bool forceCustomServer = false;
  static const String customServer = "__custom__";
  bool forceCustomExplorer = false;
  static const String customExplorer = "__custom__";

  @override
  void initState() {
    super.initState();
    Future(() async {
      dbFullPath = await getFullDatabasePath(settings.dbName);
      // Server list is network-specific: mainnet uses the bundled top servers;
      // testnet/regtest offer their known defaults.
      final net = networkForName(settings.net);
      topServers = net == ZNetwork.mainnet ? await loadTopServers() : networkInfo(net).servers;
      final packageInfo = await PackageInfo.fromPlatform();
      final version = packageInfo.version;
      final buildNumber = packageInfo.buildNumber;
      versionString = "$version+$buildNumber";
      setState(() {});
    });
  }

  // Whether to show the free-form LWD URL field: when the user explicitly chose
  // "Custom…", or the current server isn't one of the bundled top servers.
  bool get showCustomServerField => forceCustomServer || !topServers.contains(settings.lwd);

  // The explorers offered for the active network.
  Map<String, String> get explorers => blockExplorersFor(networkForName(settings.net));

  // The label of the currently-selected named explorer, or null if the stored
  // template isn't one of the bundled explorers (i.e. a custom URL).
  String? get currentExplorerLabel {
    for (final e in explorers.entries) {
      if (e.value == settings.blockExplorer) return e.key;
    }
    return null;
  }

  // Show the free-form explorer URL field when "Custom Explorer" was chosen, or
  // the stored template isn't one of the bundled explorers.
  bool get showCustomExplorerField => forceCustomExplorer || currentExplorerLabel == null;

  void tutorial() async {
    tutorialHelper(context, "tutSettings0",
        [logID, lightnodeID, lwdID, torID, coingeckoID, actionsID, autosyncID, cancelID, pinLockID, offlineID, fxID, useQRID, blockExplorerID]);
  }

  @override
  Widget build(BuildContext context) {
    final t = Theme.of(context).textTheme;
    Future(tutorial);

    return Scaffold(
      appBar: AppBar(
        title: Text("Settings"),
        actions: [
          Showcase(
            key: logID,
            description: "Open the App Log",
            child: IconButton(tooltip: "View Log", onPressed: () => onOpenLog(context), icon: Icon(Icons.description)),
          ),
          IconButton(tooltip: "Lock", onPressed: () => lockApp(ref), icon: Icon(Icons.lock)),
          IconButton(tooltip: "Show Tutorials again", onPressed: () => resetTutorial(context), icon: Icon(Icons.school)),
          IconButton(tooltip: "Database Manager", onPressed: onDatabaseManager, icon: Icon(Icons.folder)),
        ],
      ),
      body: SingleChildScrollView(
        child: FormBuilder(
          key: formKey,
          child: Padding(
            padding: const EdgeInsets.fromLTRB(8, 0, 8, 8),
            child: Column(
              children: [
                Consumer(
                  builder: (context, ref, _) {
                    final appTheme = ref.watch(themeModeProvider);
                    return FormBuilderDropdown<AppTheme>(
                      name: "theme",
                      decoration: const InputDecoration(labelText: "Theme"),
                      initialValue: appTheme,
                      items: const [
                        DropdownMenuItem(value: AppTheme.zkool, child: Text("Zkool")),
                        DropdownMenuItem(value: AppTheme.dark, child: Text("Dark")),
                        DropdownMenuItem(value: AppTheme.light, child: Text("Light")),
                        DropdownMenuItem(value: AppTheme.system, child: Text("System")),
                      ],
                      onChanged: (value) {
                        if (value != null) ref.read(themeModeProvider.notifier).set(value);
                      },
                    );
                  },
                ),
                Gap(8),
                Showcase(
                  key: lightnodeID,
                  description: "Whether the server is a light node or not",
                  child: FormBuilderSwitch(
                    name: "light",
                    title: Text("Light Node"),
                    initialValue: settings.isLightNode,
                    onChanged: onChangedIsLightNode,
                  ),
                ),
                if (settings.isLightNode && topServers.isNotEmpty)
                  Builder(builder: (context) {
                    final isMobile = MediaQuery.of(context).size.width < 600;
                    final dropdown = FormBuilderDropdown<String>(
                      name: "server",
                      isDense: true,
                      isExpanded: true,
                      decoration: InputDecoration(
                        labelText: isMobile ? "Light Node Server" : null,
                        contentPadding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
                        border: const OutlineInputBorder(),
                      ),
                      initialValue: topServers.contains(settings.lwd) ? settings.lwd : customServer,
                      items: [
                        ...topServers.map((s) => DropdownMenuItem(
                              value: s,
                              child: Text(s, overflow: TextOverflow.ellipsis, maxLines: 1),
                            )),
                        const DropdownMenuItem(value: customServer, child: Text("Custom…")),
                      ],
                      onChanged: onChangedServer,
                    );
                    if (isMobile) return dropdown;
                    return Row(
                      children: [
                        const Expanded(child: Text("Light Node Server")),
                        SizedBox(
                          width: 324,
                          child: dropdown,
                        ),
                      ],
                    );
                  }),
                if (!settings.isLightNode || showCustomServerField)
                  Showcase(
                    key: lwdID,
                    description: "Node server to connect to",
                    child: FormBuilderTextField(
                      name: "lwd",
                      decoration: InputDecoration(labelText: "${settings.isLightNode ? 'Light' : 'Full'} Node Server"),
                      initialValue: settings.lwd,
                      onChanged: onChangedLWD,
                    ),
                  ),
                Showcase(
                  key: torID,
                  description: "Route every server connection through a proxy. "
                      "Supports socks5://, socks5h://, http:// and https://. "
                      "Tap the Tor button to use a local Tor SOCKS5 proxy. Leave empty for a direct connection. Needs App Restart",
                  child: Row(
                    crossAxisAlignment: CrossAxisAlignment.center,
                    children: [
                      Expanded(
                        child: FormBuilderTextField(
                          name: "proxy",
                          decoration: InputDecoration(
                            labelText: "HTTP / SOCKS5 Proxy",
                            hintText: torProxyUrl,
                          ),
                          initialValue: settings.proxy,
                          onChanged: onChangedProxy,
                        ),
                      ),
                      const Gap(8),
                      IconButton.outlined(
                        tooltip: "Use Tor (prefill $torProxyUrl)",
                        onPressed: onEnableTor,
                        icon: SvgPicture.asset(
                          "assets/tor.svg",
                          width: 22,
                          height: 22,
                          colorFilter: ColorFilter.mode(
                            Theme.of(context).colorScheme.primary,
                            BlendMode.srcIn,
                          ),
                        ),
                      ),
                    ],
                  ),
                ),
                Showcase(
                  key: actionsID,
                  description: "Number actions per synchronization chunk",
                  child: FormBuilderTextField(
                    name: "actions_per_sync",
                    decoration: const InputDecoration(labelText: "Actions per Sync"),
                    initialValue: settings.actionsPerSync,
                    onChanged: onChangedActionsPerSync,
                    validator: FormBuilderValidators.integer(),
                    keyboardType: TextInputType.number,
                    inputFormatters: [FilteringTextInputFormatter.digitsOnly],
                  ),
                ),
                Gap(16),
                Row(
                  children: [
                    Expanded(
                      child: Showcase(
                        key: autosyncID,
                        description: "AutoSync interval in blocks. Accounts that are behind by more than this value will start synchronization",
                        child: FormBuilderTextField(
                          name: "autosync",
                          decoration: const InputDecoration(labelText: "AutoSync Interval"),
                          initialValue: settings.syncInterval,
                          onChanged: onChangedSyncInterval,
                          validator: FormBuilderValidators.integer(),
                          keyboardType: TextInputType.number,
                          inputFormatters: [FilteringTextInputFormatter.digitsOnly],
                        ),
                      ),
                    ),
                    Showcase(
                      key: cancelID,
                      description: "This will cancel the current sync and disable AutoSync",
                      child: IconButton(tooltip: "Cancel Sync", onPressed: onCancelSync, icon: Icon(Icons.cancel)),
                    ),
                  ],
                ),
                Gap(8),
                Showcase(
                  key: pinLockID,
                  description: "Ask for device pin when app opens",
                  child: FormBuilderSwitch(name: "pin_lock", title: Text("Pin Lock"), initialValue: settings.needPin, onChanged: onPinLockChanged),
                ),
                Gap(8),
                Showcase(
                  key: offlineID,
                  description: "Toggle offline mode",
                  child: FormBuilderSwitch(name: "offline", title: Text("Offline"), initialValue: settings.offline, onChanged: onOfflineChanged),
                ),
                Gap(8),
                Showcase(
                  key: fxID,
                  description: "Toggle auto update of market price",
                  child: FormBuilderSwitch(name: "fx", title: Text("Auto Fetch Market Price"), initialValue: settings.getFx, onChanged: onGetFxChanged),
                ),
                Gap(8),
                Row(
                  children: [
                    const Expanded(child: Text("Market Price Currency")),
                    SizedBox(
                      width: 120,
                      child: FormBuilderDropdown<String>(
                        name: "fx_currency",
                        isDense: true,
                        decoration: const InputDecoration(
                          contentPadding: EdgeInsets.symmetric(horizontal: 8, vertical: 4),
                          border: OutlineInputBorder(),
                        ),
                        initialValue: fxCurrencies.contains(settings.fxCurrency) ? settings.fxCurrency : "usd",
                        items: fxCurrencies
                            .map((c) => DropdownMenuItem(value: c, child: Text(c.toUpperCase())))
                            .toList(),
                        onChanged: onChangedFxCurrency,
                      ),
                    ),
                  ],
                ),
                Gap(8),
                Showcase(
                  key: coingeckoID,
                  description: "CoinGecko API Key. Register for an account on their website",
                  child: FormBuilderTextField(
                    name: "coingecko",
                    decoration: InputDecoration(
                      label: Text("CoinGecko API Key"),
                    ),
                    initialValue: settings.coingecko,
                    onChanged: onChangedCoingecko,
                  ),
                ),
                Gap(8),
                Showcase(
                  key: blockExplorerID,
                  description: "Block Explorer used to open transactions",
                  child: Row(
                    children: [
                      const Expanded(child: Text("Block Explorer")),
                      SizedBox(
                        width: 200,
                        child: FormBuilderDropdown<String>(
                          name: "explorer",
                          isDense: true,
                          isExpanded: true,
                          decoration: const InputDecoration(
                            contentPadding: EdgeInsets.symmetric(horizontal: 8, vertical: 4),
                            border: OutlineInputBorder(),
                          ),
                          initialValue: currentExplorerLabel ?? customExplorer,
                          items: [
                            ...explorers.keys.map((label) => DropdownMenuItem(
                                  value: label,
                                  child: Text(label, overflow: TextOverflow.ellipsis, maxLines: 1),
                                )),
                            const DropdownMenuItem(value: customExplorer, child: Text("Custom Explorer")),
                          ],
                          onChanged: onChangedExplorer,
                        ),
                      ),
                    ],
                  ),
                ),
                if (showCustomExplorerField)
                  FormBuilderTextField(
                    name: "block_explorer",
                    decoration: const InputDecoration(
                      labelText: "Custom Explorer URL",
                      hintText: "https://host/tx/{txid}",
                    ),
                    initialValue: settings.blockExplorer,
                    onChanged: onChangedBlockExplorer,
                  ),
                Gap(8),
                Showcase(
                  key: useQRID,
                  description: "Use QR Codes for file transmission between devices",
                  child: Row(children: [
                    Expanded(child: Text("File Transmission via QR Codes")),
                    SizedBox(width: 40, child: IconButton(onPressed: onQR, icon: Icon(Icons.chevron_right)))
                  ]),
                ),
                Gap(8),
                if (settings.expertMode)
                  FormBuilderSwitch(
                    name: "vault",
                    title: Text("Cloud Vault"),
                    initialValue: settings.vault,
                    onChanged: onChangedVault,
                  ),
                if (settings.expertMode && settings.vault)
                  Row(children: [
                    Expanded(child: Text("Recover Accounts from Vault")),
                    SizedBox(width: 40, child: IconButton(onPressed: onVaultRecover, icon: Icon(Icons.chevron_right),),),
                  ]),
                Gap(16),
                CopyableText(dbFullPath, style: t.bodySmall),
                Gap(8),
                GestureDetector(
                  onLongPress: () async {
                    final prefs = SharedPreferencesAsync();
                    final newExpertMode = !settings.expertMode;
                    await prefs.setBool("expert_mode", newExpertMode);
                    setState(() {
                      settings = settings.copyWith(expertMode: newExpertMode);
                      widget.onChanged(settings);
                    });
                  },
                  child: Row(
                    mainAxisAlignment: MainAxisAlignment.center,
                    children: [
                      Text(versionString),
                      if (settings.expertMode) ...[
                        Gap(8),
                        Text("expert", style: t.bodySmall?.copyWith(color: Colors.grey)),
                      ],
                    ],
                  ),
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }

  void onOpenLog(BuildContext context) async {
    await GoRouter.of(context).push("/log");
  }

  void onCancelSync() async {
    final confirmed = await confirmDialog(context, title: "Cancel Sync", message: "Do you want to cancel the current sync? AutoSync will be disabled too");
    if (!confirmed) return;
    formKey.currentState!.fields["autosync"]!.didChange("0");
    await cancelSync();
  }

  void onChangedDatabaseName(String? value) async {
    if (value == null) return;
    setState(() {
      settings = settings.copyWith(dbName: value);
      widget.onChanged(settings);
    });
  }

  void onChangedLWD(String? value) async {
    if (value == null) return;
    setState(() {
      settings = settings.copyWith(lwd: value);
      widget.onChanged(settings);
    });
  }

  void onChangedServer(String? value) async {
    if (value == null) return;
    if (value == customServer) {
      // Reveal the free-form URL field; keep the current lwd so it stays
      // editable.
      setState(() => forceCustomServer = true);
      return;
    }
    // A bundled server was selected: use it as the active LWD URL.
    setState(() {
      forceCustomServer = false;
      settings = settings.copyWith(lwd: value);
      widget.onChanged(settings);
    });
  }

  void onChangedCoingecko(String? value) async {
    if (value == null) return;
    setState(() {
      settings = settings.copyWith(coingecko: value);
      widget.onChanged(settings);
    });
  }

  void onChangedBlockExplorer(String? value) async {
    if (value == null) return;
    setState(() {
      settings = settings.copyWith(blockExplorer: value);
      widget.onChanged(settings);
    });
  }

  void onChangedExplorer(String? value) async {
    if (value == null) return;
    if (value == customExplorer) {
      // Reveal the free-form URL field; keep the current template editable.
      setState(() => forceCustomExplorer = true);
      return;
    }
    // A bundled explorer was selected: store its URL template.
    final template = explorers[value];
    if (template == null) return;
    setState(() {
      forceCustomExplorer = false;
      settings = settings.copyWith(blockExplorer: template);
      widget.onChanged(settings);
    });
  }

  onChangedIsLightNode(bool? value) async {
    if (value == null) return;
    setState(() {
      settings = settings.copyWith(isLightNode: value);
      widget.onChanged(settings);
    });
  }

  void onChangedProxy(String? value) async {
    if (value == null) return;
    setState(() {
      settings = settings.copyWith(proxy: value);
      widget.onChanged(settings);
    });
  }

  // On Windows the common Tor setup is the Tor Browser bundle, whose SOCKS
  // proxy listens on 9150. Elsewhere the standalone tor daemon defaults to 9050.
  //
  // We use the socks5h:// scheme (not socks5://) so DNS resolution happens at
  // the proxy. This is required for .onion server addresses to resolve and also
  // prevents DNS leaks when routing over Tor.
  static String get torProxyUrl =>
      Platform.isWindows ? "socks5h://127.0.0.1:9150" : "socks5h://127.0.0.1:9050";

  void onEnableTor() async {
    final url = torProxyUrl;
    formKey.currentState?.fields["proxy"]?.didChange(url);
    setState(() {
      settings = settings.copyWith(proxy: url);
      widget.onChanged(settings);
    });
  }

  onQR() async {
    await GoRouter.of(context).push("/settings/qr", extra: (QRSettings qrSettings) {
      setState(() {
        settings = settings.copyWith(qrSettings: qrSettings);
        widget.onChanged(settings);
      });
    });
  }

  onPinLockChanged(bool? value) async {
    if (value == null) return;
    setState(() {
      settings = settings.copyWith(needPin: value);
      widget.onChanged(settings);
    });
  }

  onOfflineChanged(bool? value) async {
    if (value == null) return;
    setState(() {
      settings = settings.copyWith(offline: value);
      widget.onChanged(settings);
    });
  }

  onGetFxChanged(bool? value) async {
    if (value == null) return;
    setState(() {
      settings = settings.copyWith(getFx: value);
      widget.onChanged(settings);
    });
  }

  void onChangedFxCurrency(String? value) async {
    if (value == null) return;
    setState(() {
      settings = settings.copyWith(fxCurrency: value);
      widget.onChanged(settings);
    });
  }

  onChangedVault(bool? value) async {
    if (value == null) return;
    if (value) {
      // Vault is activating...
      final tt = Theme.of(context).textTheme;
      // Ask for confirmation from the user
      final confirmed = await confirmDialog(context,
          title: "",
          message: "",
          body: Padding(
            padding: EdgeInsetsGeometry.all(8),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Center(child: Text("Enable Vault", style: tt.titleMedium)),
                const Gap(16),
                Divider(),
                const Gap(16),
                Text("Your vault keys will be stored securely in the Cloud:", style: tt.bodyMedium),
                const Gap(16),
                Row(
                  children: [
                    Icon(Icons.lock_outline, size: 18, color: Colors.green.shade700),
                    const Gap(8),
                    Text("End-to-end encrypted", style: tt.titleSmall?.copyWith(color: Colors.green.shade700)),
                  ],
                ),
                const Gap(4),
                Text("Your account keys are encrypted. Only you can decrypt them.", style: tt.bodySmall),
                const Gap(12),
                Row(
                  children: [
                    Icon(Icons.cloud_outlined, size: 18, color: Colors.blue.shade700),
                    const Gap(8),
                    Text("Google Drive (app data only)", style: tt.titleSmall?.copyWith(color: Colors.blue.shade700)),
                  ],
                ),
                const Gap(4),
                Text("Backups go to app-specific storage — not your entire Drive.", style: tt.bodySmall),
                if (passkeySupported) ...[
                  const Gap(12),
                  Row(
                    children: [
                      Icon(Icons.fingerprint, size: 18, color: Colors.orange.shade700),
                      const Gap(8),
                      Text("Biometric protection", style: tt.titleSmall?.copyWith(color: Colors.orange.shade700)),
                    ],
                  ),
                  const Gap(4),
                  Text("Face ID / Touch ID protects your vault on this device.", style: tt.bodySmall),
                ],
              ],
            ),
          ));

      bool success = false;
      try {
        if (!confirmed) return;

        final vault = ref.read(vaultProvider.notifier);
        // newVault is true iff there is a local master file
        final newVault = !(await vault.hasVault());
        logger.i("[Vault] enable: newVault=$newVault");

        String? password;
        if (newVault) {
          if (!mounted) return;
          // no vault at all, create a new one. Ask for the master password MP
          final p = await inputPassword(context, title: "Create Vault Password", repeated: true, required: true);
          if (p == null) return;
          password = p; // now we have MP
          try {
            logger.i("[Vault] enable: initializing new vault");
            await vault.initialize(password);
          } catch (e) {
            logger.e("[Vault] enable: initialization failed: $e");
            await vault.deleteLocalVault(); // revert the creation of the vault
            if (mounted) await ErrorDialog.show(context, error: e);
            return;
          }
        }

        if (passkeySupported) {
          logger.i("[Vault] enable: starting passkey authentication flow");
          Uint8List? prf;
          bool needsDeviceRegistration = true;

          // Always show the picker - let user select a key (including remote via QR)
          try {
            logger.i("[Vault] enable: showing passkey picker");
            prf = await authenticatePasskey();
            logger.i("[Vault] enable: got PRF from passkey, verifying against vault");

            // Verify the PRF can decrypt the vault
            if (!newVault) {
              try {
                final vaultBytes = await vault.downloadVaultBytes();
                await vault.recoverWithPrf(vaultBytes: vaultBytes, prf: prf);
                logger.i("[Vault] enable: passkey PRF verified OK, vault decrypted successfully");
                needsDeviceRegistration = false;
              } catch (e) {
                logger.e("[Vault] enable: passkey PRF cannot decrypt vault: $e");
                if (mounted) {
                  await showException(context, "This passkey cannot unlock the vault. It may not be the correct key for this vault.");
                }
                return;
              }
            }
          } on PasskeyException catch (e) {
            logger.i("[Vault] enable: passkey picker cancelled or failed (${e.errorType}): ${e.message}");
            // User cancelled or error - offer to register a new key
            if (!mounted) return;

            final registerNewKey = await confirmDialog(
              context,
              title: "Register New Passkey?",
              message: "No passkey was selected. Would you like to register a new passkey for this vault?",
            );

            if (!registerNewKey) {
              logger.i("[Vault] enable: user declined to register new key");
              return;
            }

            // Prompt for Master Password before allowing registration
            if (!mounted) return;
            final p = await inputPassword(context, title: "Master Password Required", required: true);
            if (p == null) return;
            password = p;

            // Register new passkey
            logger.i("[Vault] enable: registering new passkey");
            final registration = await registerPasskey();
            if (registration == null) {
              logger.i("[Vault] enable: passkey registration returned null (already exists?)");
              if (!newVault) {
                // Try to authenticate with existing passkey
                prf = await authenticatePasskey();
                final vaultBytes = await vault.downloadVaultBytes();
                await vault.recoverWithPrf(vaultBytes: vaultBytes, prf: prf);
                needsDeviceRegistration = false;
              }
            } else {
              prf = await authenticatePasskey();
              logger.i("[Vault] enable: new passkey registered and authenticated");
            }
          }

          if (needsDeviceRegistration) {
            if (password == null) {
              if (!mounted) return;
              final p = await inputPassword(context, title: "Vault Password", required: true);
              if (p == null) return;
              password = p;
            }
            prf ??= await authenticatePasskey();
            logger.i("[Vault] enable: registering device with PRF");
            await vault.registerDevice(password: password, prf: prf);
          }
        } else {
          logger.i("[Vault] enable: passkey not supported on this platform, skipping device registration");
        }

        if (!mounted) return;
        await showMessage(context, "Vault activated");
        success = true;
      } on AnyhowException catch (e) {
        if (mounted) await showException(context, e.message);
        return;
      } on PasskeyException catch (e) {
        logger.e("[Vault] enable: passkey error (${e.errorType}): ${e.message}");
        if (mounted) await showException(context, "Passkey error: ${e.message}");
        return;
      } finally {
        if (!success) {
          formKey.currentState?.fields['vault']?.didChange(false);
        }
      }
    }
    setState(() {
      settings = settings.copyWith(vault: value);
      widget.onChanged(settings);
    });
  }

  void onVaultRecover() async {
    // 1. Try passkey first - always show picker explicitly (local & remote)
    Uint8List? prf;
    if (passkeySupported) {
      try {
        logger.i("[Recover] step 1: showing passkey picker");
        prf = await authenticatePasskey();
        logger.i("[Recover] step 1: got PRF from passkey");
      } on PasskeyException catch (e) {
        logger.i("[Recover] step 1: passkey cancelled or failed (${e.errorType}), falling back to master password");
        // Passkey cancelled or failed - will fall back to master password below
      }
    }

    // 2. Download vault bytes once
    Uint8List vaultBytes;
    try {
      logger.i("[Recover] step 2: downloading vault bytes");
      vaultBytes = await ref.read(vaultProvider.notifier).downloadVaultBytes();
      logger.i("[Recover] step 2: downloaded ${vaultBytes.length} bytes");
    } on AnyhowException catch (e) {
      logger.e("[Recover] step 2: download failed: $e");
      if (mounted) await showException(context, e.message);
      return;
    } catch (e) {
      logger.e("[Recover] step 2: download failed: $e");
      if (mounted) await ErrorDialog.show(context, error: e);
      return;
    }

    // 3. Try PRF recovery
    List? recovered;
    if (prf != null) {
      try {
        logger.i("[Recover] step 3: trying PRF recovery");
        recovered = await ref.read(vaultProvider.notifier).recoverWithPrf(vaultBytes: vaultBytes, prf: prf);
        logger.i('[Recover] step 3: PRF recovery succeeded, ${recovered.length} accounts');
      } catch (e) {
        // PRF recovery failed, fall through to password
        logger.i('[Recover] step 3: PRF recovery failed: $e');
      }
    }

    // 4. Fall back to password recovery
    if (recovered == null) {
      logger.i("[Recover] step 4: prompting for password");
      if (!mounted) return;
      final password = await inputPassword(context, title: "Vault Password", required: true);
      if (password == null) return;
      try {
        logger.i("[Recover] step 4: recovering with password");
        recovered = await ref.read(vaultProvider.notifier).recoverVault(vaultBytes: vaultBytes, masterPassword: password);
        logger.i("[Recover] step 4: recovered ${recovered.length} accounts");
      } on AnyhowException catch (e) {
        logger.e("[Recover] step 4: password recovery failed: $e");
        if (mounted) await showException(context, e.message);
        return;
      }
    }

    if (!mounted) return;
    final existingAccounts = await ref.read(getAccountsProvider.future);
    final coin = coinContext.coin;
    final ctx = context;
    AwesomeDialog? dialog;
    try {
      dialog = await showMessage(ctx, "Importing ${recovered.length} account(s)...", dismissable: false);
      for (final ra in recovered) {
        // find existing account matching seed + aindex
        final match = existingAccounts.where((a) => a.seed == ra.seed && a.aindex == ra.aindex).firstOrNull;
        if (match != null) {
          // seed exists — only update name and birth height
          await updateAccount(
            update: AccountUpdate(
              coin: coin.coin,
              id: match.id,
              name: ra.name,
              birth: ra.birthHeight,
              folder: match.folder.id,
            ),
            c: coin,
          );
        } else {
          // create new account from seed
          await newAccount(
            na: NewAccount(
              name: ra.name,
              restore: true,
              key: ra.seed,
              aindex: ra.aindex,
              birth: ra.birthHeight,
              folder: "",
              useInternal: ra.useInternal,
              internal: false,
              ledger: false,
            ),
            c: coin,
          );
        }
      }
      ref.invalidate(getAccountsProvider);
      dialog.dismiss();
      if (mounted) await showMessage(context, "Vault recovery completed");
    } on AnyhowException catch (e) {
      dialog?.dismiss();
      if (mounted) await showException(context, e.message);
    }
  }

  onChangedActionsPerSync(String? value) async {
    if (value == null) return;
    if (int.tryParse(value) == null) {
      return;
    }
    setState(() {
      settings = settings.copyWith(actionsPerSync: value);
      widget.onChanged(settings);
    });
  }

  onChangedSyncInterval(String? value) async {
    if (value == null) return;
    if (int.tryParse(value) == null) {
      return;
    }
    setState(() {
      settings = settings.copyWith(syncInterval: value);
      widget.onChanged(settings);
    });
  }

  void onDatabaseManager() async {
    final confirmed = await confirmDialog(
      context,
      title: "Database Manager",
      message: "The Database Manager will open when you restart the app. Do you want to schedule it now?",
    );
    if (confirmed) {
      final prefs = SharedPreferencesAsync();
      await prefs.setBool("recovery", true);
      await showMessage(context, "Restart the app to enter the database manager");
    }
  }
}

typedef VoidFunction<T> = void Function(T);

class SettingsQRPage extends ConsumerStatefulWidget {
  final VoidFunction<QRSettings> onClose;
  const SettingsQRPage({required this.onClose, super.key});

  @override
  ConsumerState<SettingsQRPage> createState() => SettingsQRPageState();
}

class SettingsQRPageState extends ConsumerState<SettingsQRPage> with RouteAware {
  final formKey = GlobalKey<FormBuilderState>();
  QRSettings? settings;

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    final route = ModalRoute.of(context);
    if (route is PageRoute) {
      routeObserver.subscribe(this, route);
    }
  }

  @override
  void didPop() {
    super.didPop();
    onPop();
  }

  @override
  Widget build(BuildContext context) {
    final settingsAV = ref.watch(appSettingsProvider);
    final t = Theme.of(context).textTheme;
    return Scaffold(
      appBar: AppBar(title: Text("Settings")),
      body: settingsAV.when(
        loading: () => blank(context),
        error: (error, stack) => showError(error),
        data: (settings) {
          final qrSettings = settings.qrSettings;
          return SingleChildScrollView(
        child: Padding(
          padding: EdgeInsetsGeometry.symmetric(horizontal: 8),
          child: FormBuilder(
            key: formKey,
            child: Column(
              children: [
                Card(
                  elevation: 1,
                  margin: EdgeInsets.all(8),
                  child: Padding(
                    padding: EdgeInsetsGeometry.all(8),
                    child: Column(
                      children: [
                        Text("QR Codes", style: t.titleMedium),
                        Gap(16),
                        FormBuilderSwitch(name: "enabled", initialValue: qrSettings.enabled, title: Text("Enabled")),
                        Gap(8),
                        FormBuilderSlider(
                          name: "size",
                          decoration: InputDecoration(
                            label: Text("QR Code Size"),
                          ),
                          initialValue: qrSettings.size,
                          min: 10,
                          max: 40,
                          divisions: 30,
                        ),
                        Gap(16),
                        FormBuilderSlider(
                          name: "ecLevel",
                          decoration: InputDecoration(
                            label: Text("Error Correction Level"),
                            helper: Text(
                              "higher ECL is more robust but takes more space",
                            ),
                          ),
                          initialValue: qrSettings.ecLevel.toDouble(),
                          min: 0,
                          max: 3,
                          divisions: 3,
                        ),
                        Gap(16),
                        FormBuilderTextField(
                          name: "delay",
                          decoration: InputDecoration(
                            label: Text("Duration between QR codes (ms)"),
                          ),
                          initialValue: qrSettings.delay.toString(),
                          validator: FormBuilderValidators.integer(),
                          keyboardType: TextInputType.number,
                          inputFormatters: [
                            FilteringTextInputFormatter.digitsOnly,
                          ],
                        ),
                      ],
                    ),
                  ),
                ),
                Card(
                  elevation: 1,
                  child: Padding(
                    padding: EdgeInsetsGeometry.all(8),
                    child: Column(
                      children: [
                        Text("Fountain Codes", style: t.titleMedium),
                        Gap(8),
                        FormBuilderTextField(
                          name: "repair",
                          decoration: InputDecoration(
                            label: Text("Repair Packets"),
                          ),
                          initialValue: qrSettings.repair.toString(),
                          validator: FormBuilderValidators.integer(),
                          keyboardType: TextInputType.number,
                          inputFormatters: [
                            FilteringTextInputFormatter.digitsOnly,
                          ],
                        ),
                      ],
                    ),
                  ),
                ),
              ],
            ),
          ),
          ),
        );
        },
      ),
    );
  }

  void onPop() {
    final form = formKey.currentState!;
    if (form.validate()) {
      final fields = form.fields;
      final enabled = fields["enabled"]!.value as bool;
      final size = fields["size"]!.value as double;
      final ecLevel = fields["ecLevel"]!.value as double;
      final delay = int.parse(fields["delay"]!.value as String);
      final repair = int.parse(fields["repair"]!.value as String);
      final settings = QRSettings(
        enabled: enabled,
        size: size,
        ecLevel: ecLevel.toInt(),
        delay: delay,
        repair: repair,
      );
      widget.onClose(settings);
    }
  }
}
