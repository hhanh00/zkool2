import 'dart:async';
import 'dart:math';
import 'package:collection/collection.dart';
import 'package:convert/convert.dart';
import 'package:easy_debounce/easy_debounce.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:toastification/toastification.dart';
import 'package:flutter/material.dart';
import 'package:zkool/main.dart';
import 'package:zkool/router.dart';
import 'package:zkool/services/block_height_service.dart';
import 'package:zkool/src/rust/api/account.dart';
import 'package:zkool/src/rust/api/coin.dart';
import 'package:zkool/src/rust/api/contacts.dart';
import 'package:zkool/src/rust/api/db.dart';
import 'package:zkool/src/rust/api/init.dart';
import 'package:zkool/src/rust/api/mempool.dart';
import 'package:zkool/src/rust/api/network.dart';
import 'package:zkool/src/rust/api/plugin.dart' as plugin_api;
import 'package:zkool/src/rust/api/sweep.dart';
import 'package:zkool/src/rust/api/sync.dart';
import 'package:zkool/src/rust/api/voting_drive.dart';
import 'package:zkool/src/rust/api/zsa.dart';
import 'package:zkool/utils.dart';
import 'package:zkool/widgets/error_display.dart';
import 'package:zkool/vault.dart';
import 'package:zkool/widgets/theme.dart';

part 'store.g.dart';
part 'store.freezed.dart';

@riverpod
class HasDb extends _$HasDb {
  @override
  bool build() => false;

  void setHasDb() {
    state = true;
  }
}

@Riverpod(keepAlive: true)
class SelectedAccountId extends _$SelectedAccountId {
  @override
  int build() => 0;

  Future<void> set(int account) async {
    state = account;
    await putProp(key: "selected_account", value: account.toString(), c: coinContext.coin);
  }
}

// Singleton coin context - not a provider, just a data container for Rust
class CoinContext {
  Coin _coin = Coin();

  Coin get coin => _coin;

  Future<void> setAccount({required int account}) async {
    _coin = await _coin.setAccount(account: account);
  }

  void set({required Coin coin}) {
    _coin = coin;
  }
}

final coinContext = CoinContext();
final blockHeightService = BlockHeightService(
  fetchHeight: () => getCurrentHeight(c: coinContext.coin),
);

@freezed
sealed class SyncState with _$SyncState {
  factory SyncState({
    required int start,
    required int end,
    required int height,
    required int time,
    required List<Account> accounts,
  }) = _SyncState;
}

@riverpod
class SyncStateAccount extends _$SyncStateAccount {
  @override
  Future<SyncProgressAccount> build(int accountId) async {
    final accounts = await ref.watch(getAccountsProvider.future);
    final account = accounts.firstWhere((a) => a.id == accountId);
    final ss = ref.watch(synchronizerProvider);
    if (ss.accounts.any((a) => a.id == account.id)) {
      return SyncProgressAccount(
        account: account,
        start: max(ss.start, account.height),
        end: ss.end,
        height: max(ss.height, account.height),
        time: max(ss.time, account.time),
      );
    } else {
      return SyncProgressAccount(
        account: account,
        start: 0,
        end: 0,
        height: account.height,
        time: account.time,
      );
    }
  }

  void updateHeight(int height, int time) {
    state = state.whenData((s) => s.copyWith(height: height, time: time));
  }
}

@freezed
sealed class SyncProgressAccount with _$SyncProgressAccount {
  const SyncProgressAccount._();

  factory SyncProgressAccount({
    required Account account,
    required int start,
    required int end,
    required int height,
    required int time,
  }) = _SyncProgressAccount;

  double progress() => (height - start) / (end - start);
}

class ProgressWidget extends ConsumerWidget {
  final Account account;
  final double? width;
  final TextStyle? style;
  final Widget Function(BuildContext context, SyncProgressAccount status, TextStyle? style) builder;
  const ProgressWidget(
    this.account, {
    super.key,
    this.width,
    this.style,
    required this.builder,
  });

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final ssAV = ref.watch(syncStateAccountProvider(account.id));
    switch (ssAV) {
      case AsyncLoading():
        return showLoading("Sync State");
      case AsyncError(:final error):
        return showError(error);
      default:
    }
    final ss = ssAV.requireValue;
    final t = Theme.of(context);
    final timestamp = DateTime.fromMillisecondsSinceEpoch(ss.time * 1000);
    final syncAge = DateTime.now().difference(timestamp);
    final old = syncAge > Duration(minutes: 30);
    final s = style ?? TextStyle();
    final s2 = old ? s.copyWith(color: Colors.red) : s;

    return IntrinsicHeight(
        child: SizedBox(
      child: Stack(
        children: [
          if (ss.start != ss.end)
            Positioned.fill(
              child: LinearProgressIndicator(
                color: t.colorScheme.primary.withAlpha(128),
                value: ss.progress(),
              ),
            ),
          builder(context, ss, s2),
        ],
      ),
    ));
  }
}

class SmallProgressWidget extends StatelessWidget {
  final Account account;
  final TextStyle? style;
  const SmallProgressWidget(this.account, {this.style, super.key});
  @override
  Widget build(BuildContext context) => ProgressWidget(account, style: style, builder: (context, status, style) => Text("${status.height}", style: style));
}

class HeroProgressWidget extends StatelessWidget {
  final Account account;
  const HeroProgressWidget(this.account, {super.key});

  @override
  Widget build(BuildContext context) {
    final t = Theme.of(context).textTheme;
    Widget child = ProgressWidget(account, builder: (context, status, style) {
      return Center(
          child: Text.rich(
        TextSpan(
          children: [
            TextSpan(text: "${status.height}", style: t.bodyLarge!.merge(style)),
            if (status.end - status.height > 0)
              TextSpan(
                text: " tip-${status.end - status.height}",
                style: t.labelSmall,
              ),
          ],
        ),
      ));
    });

    return DisplayPanel(
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            mainAxisAlignment: MainAxisAlignment.spaceBetween,
            children: [
              Text(
                "Height",
                style: t.bodyLarge,
              ),
            ],
          ),
          child,
        ],
      ),
    );
  }
}

// AppStore get appStore => AppStoreBase.instance;

@riverpod
Future<Account?> selectedAccount(Ref ref) async {
  final accountId = ref.watch(selectedAccountIdProvider);
  if (accountId == 0) return null;
  final accounts = await ref.watch(getAccountsProvider.future);
  final acc = accounts.firstWhereOrNull((a) => a.id == accountId);
  return acc;
}

@riverpod
class SelectedFolder extends _$SelectedFolder {
  @override
  Folder? build() {
    return null;
  }

  void selectFolder(Folder folder) {
    state = folder;
  }

  void unselect() {
    state = null;
  }
}

@Riverpod(keepAlive: true)
Future<List<Account>> getAccounts(Ref ref) async {
  final c = coinContext.coin;
  final as = await listAccounts(c: c);
  return as;
}

@riverpod
Future<List<Folder>> getFolders(Ref ref) async {
  final c = coinContext.coin;
  return await listFolders(c: c);
}

@riverpod
Future<List<Category>> getCategories(Ref ref) async {
  final c = coinContext.coin;
  return await listCategories(c: c);
}

@riverpod
Future<List<Contact>> getContacts(Ref ref) async {
  final c = coinContext.coin;
  return await listContacts(c: c);
}

@riverpod
Future<List<ContactMatch>> contactsForAddress(Ref ref, String address) async {
  if (address.isEmpty) return [];
  final c = coinContext.coin;
  return await findContactsForAddress(address: address, c: c);
}

@riverpod
Future<AccountData> account(Ref ref, int id) async {
  final c = coinContext.coin;
  final accounts = await ref.watch(getAccountsProvider.future);
  final account = accounts.firstWhere((a) => a.id == id);
  final poolBalance = await balance(c: c);
  final pool = await getAccountPools(account: id, c: c);
  final frostParams = await getAccountFrostParams(c: c);
  final transactions = await listTxHistory(c: c);
  final memos = await listMemos(c: c);
  final notes = await listNotes(c: c);
  final zsas = await listZsaHoldings(c: c);
  zsas.sort((a, b) {
    final nameA = a.assetName.isNotEmpty ? a.assetName : hex.encode(a.assetDescHash.sublist(0, 4));
    final nameB = b.assetName.isNotEmpty ? b.assetName : hex.encode(b.assetDescHash.sublist(0, 4));
    return nameA.toLowerCase().compareTo(nameB.toLowerCase());
  });

  return AccountData(
    account: account,
    balance: poolBalance,
    pool: pool,
    transactions: transactions,
    memos: memos,
    notes: notes,
    zsas: zsas,
    frostParams: frostParams,
  );
}

@Riverpod(keepAlive: true)
Future<AccountData?> getCurrentAccount(Ref ref) async {
  final selectedAccount = await ref.watch(selectedAccountProvider.future);
  if (selectedAccount == null) {
    return null;
  }
  return await ref.watch(accountProvider(selectedAccount.id).future);
}

@freezed
sealed class AccountData with _$AccountData {
  factory AccountData({
    required Account account,
    required int pool,
    required PoolBalance balance,
    required List<Tx> transactions,
    required List<Memo> memos,
    required List<TxNote> notes,
    required List<ZsaHolding> zsas,
    FrostParams? frostParams,
  }) = _AccountData;
}

@Riverpod(keepAlive: true)
class AppSettingsNotifier extends _$AppSettingsNotifier {
  @override
  Future<AppSettings> build() async {
    final c = coinContext.coin;
    final hasDb = ref.watch(hasDbProvider);
    final prefs = SharedPreferencesAsync();
    String dbName = await prefs.getString("database") ?? appName;
    final needPin = await prefs.getBool("pin_lock") ?? false;
    final offline = await prefs.getBool("offline") ?? false;
    final proxy = (hasDb ? await getProp(key: "proxy", c: c) : null) ?? "";
    // Transport: 0 = direct, 1 = Tor (arti), 2 = Nym mixnet, 3 = proxy.
    // Migrate from the legacy use_tor bool / proxy-implies-proxy behavior.
    int transport = await prefs.getInt("transport") ??
        ((await prefs.getBool("use_tor") ?? false)
            ? 1
            : proxy.isNotEmpty
                ? 3
                : 0);
    final getFx = await prefs.getBool("get_fx") ?? false;
    final coingecko = await prefs.getString("coingecko") ?? "";
    final recovery = await prefs.getBool("recovery") ?? false;
    final net = (hasDb ? await getNetworkName(c: c) : null) ?? "mainnet";
    final isLightNode = (hasDb ? await getProp(key: "is_light_node", c: c) : null) ?? "true";
    final lwd = (hasDb ? await getProp(key: "lwd", c: c) : null) ?? "https://zec.rocks";
    final syncInterval = (hasDb ? await getProp(key: "sync_interval", c: c) : null) ?? "30";
    final votingConfigUrl =
        (hasDb ? await getProp(key: "voting_config_url", c: c) : null) ?? "";
    final actionsPerSync = (hasDb ? await getProp(key: "actions_per_sync", c: c) : null) ?? "10000";
    final blockExplorer = (hasDb ? await getProp(key: "block_explorer", c: c) : null) ?? "https://cipherscan.app/tx/{txid}";
    final qrEnabled = (hasDb ? await getProp(key: "qr_enabled", c: c) : null) ?? "false";
    final qrSize = (hasDb ? await getProp(key: "qr_size", c: c) : null) ?? "20";
    final qrEC = (hasDb ? await getProp(key: "qr_ecLevel", c: c) : null) ?? "1";
    final qrDelay = (hasDb ? await getProp(key: "qr_delay", c: c) : null) ?? "500";
    final qrRepair = (hasDb ? await getProp(key: "qr_repair", c: c) : null) ?? "2";
    final qrSettings = QRSettings(
      enabled: qrEnabled == "true",
      size: double.parse(qrSize),
      ecLevel: int.parse(qrEC),
      delay: int.parse(qrDelay),
      repair: int.parse(qrRepair),
    );
    final vault = await prefs.getBool("vault") ?? false;
    final expertMode = await prefs.getBool("expert_mode") ?? false;
    setExpertMode(enabled: expertMode);
    final paletteName = await prefs.getString("palette_name") ?? 'blue';
    final darkMode = await prefs.getBool("dark_mode") ?? true;
    final txTableMode = await prefs.getBool("tx_table_mode") ?? false;
    final collapsePoolBalances =
        (hasDb ? await getProp(key: "collapse_pool_balances", c: c) : null) == "true";
    final currency = (hasDb ? await getProp(key: "currency", c: c) : null) ?? "usd";
    final price = ref.watch(priceProvider.notifier);
    price.setAutoFetchFx(getFx, coingecko, currency);

    return AppSettings(
      dbName: dbName,
      net: net,
      isLightNode: isLightNode == "true",
      lwd: lwd,
      needPin: needPin,
      pinUnlockedAt: DateTime.now(),
      offline: offline,
      transport: transport,
      proxy: proxy,
      getFx: getFx,
      coingecko: coingecko,
      recovery: recovery,
      syncInterval: syncInterval,
      actionsPerSync: actionsPerSync,
      blockExplorer: blockExplorer,
      qrSettings: qrSettings,
      vault: vault,
      expertMode: expertMode,
      paletteName: paletteName,
      darkMode: darkMode,
      votingConfigUrl: votingConfigUrl,
      transactionTableMode: txTableMode,
      collapsePoolBalances: collapsePoolBalances,
      currency: currency,
    );
  }

  void unlock() {
    state = state.whenData((s) => s.copyWith(
          pinUnlockedAt: DateTime.now(),
        ));
  }

  Future<void> setTheme(String paletteName, bool darkMode) async {
    final prefs = SharedPreferencesAsync();
    await prefs.setString("palette_name", paletteName);
    await prefs.setBool("dark_mode", darkMode);
    state = state.whenData((s) => s.copyWith(
          paletteName: paletteName,
          darkMode: darkMode,
        ));
  }

  Future<void> setTransactionViewMode(bool tableMode) async {
    final prefs = SharedPreferencesAsync();
    await prefs.setBool("tx_table_mode", tableMode);
    state = state.whenData((s) => s.copyWith(
          transactionTableMode: tableMode,
        ));
  }

  Future<void> setCollapsePoolBalances(bool collapsed) async {
    await putProp(key: "collapse_pool_balances", value: collapsed.toString(), c: coinContext.coin);
    state = state.whenData((s) => s.copyWith(
          collapsePoolBalances: collapsed,
        ));
  }

  Future<void> setVotingConfigUrl(String url) async {
    await putProp(key: "voting_config_url", value: url, c: coinContext.coin);
    state = state.whenData((s) => s.copyWith(
          votingConfigUrl: url,
        ));
  }

  /// Persist all settings to prefs/DB props and apply live coin changes
  /// (lwd/transport/proxy). Called once when leaving the settings page.
  Future<void> save(AppSettings settings) async {
    final c = coinContext.coin;
    final prefs = SharedPreferencesAsync();
    await prefs.setString("database", settings.dbName);
    await putProp(key: "is_light_node", value: settings.isLightNode.toString(), c: c);
    await putProp(key: "lwd", value: settings.lwd, c: c);
    await putProp(key: "block_explorer", value: settings.blockExplorer, c: c);
    await putProp(key: "actions_per_sync", value: settings.actionsPerSync, c: c);
    await putProp(key: "sync_interval", value: settings.syncInterval, c: c);
    await prefs.setBool("pin_lock", settings.needPin);
    await prefs.setBool("offline", settings.offline);
    await prefs.setInt("transport", settings.transport);
    await putProp(key: "proxy", value: settings.proxy, c: c);
    await prefs.setBool("get_fx", settings.getFx);
    await prefs.setString("coingecko", settings.coingecko);
    await putProp(key: "qr_enabled", value: settings.qrSettings.enabled.toString(), c: c);
    await putProp(key: "qr_size", value: settings.qrSettings.size.toString(), c: c);
    await putProp(key: "qr_ecLevel", value: settings.qrSettings.ecLevel.toString(), c: c);
    await putProp(key: "qr_delay", value: settings.qrSettings.delay.toString(), c: c);
    await putProp(key: "qr_repair", value: settings.qrSettings.repair.toString(), c: c);
    await prefs.setBool("vault", settings.vault);
    await prefs.setBool("expert_mode", settings.expertMode);
    await prefs.setString("palette_name", settings.paletteName);
    await prefs.setBool("dark_mode", settings.darkMode);
    await putProp(key: "currency", value: settings.currency, c: c);
    await putProp(key: "collapse_pool_balances", value: settings.collapsePoolBalances.toString(), c: c);
    await putProp(key: "voting_config_url", value: settings.votingConfigUrl, c: c);
    coinContext.set(
      coin: c
          .setLwd(url: settings.lwd, serverType: settings.isLightNode ? 0 : 1)
          .setTransport(transport: settings.transport)
          .setProxy(proxy: settings.proxy),
    );
    ref.read(priceProvider.notifier).setAutoFetchFx(
      settings.getFx,
      settings.coingecko,
      settings.currency,
    );
    state = AsyncValue.data(settings);
  }
}

@Riverpod(keepAlive: true)
class PriceNotifier extends _$PriceNotifier {
  @override
  double? build() => null;

  void setPrice(double price) {
    state = price;
  }

  Timer? fetchFxTimer;
  String _lastApi = "";
  String _lastCurrency = "usd";

  void setAutoFetchFx(bool autoGetFx, String api, String currency) async {
    _lastApi = api;
    _lastCurrency = currency;
    if (autoGetFx) {
      await fetch(api, currency);
      fetchFxTimer = Timer.periodic(Duration(minutes: 1), (_) async {
        await fetch(_lastApi, _lastCurrency);
      });
    } else {
      fetchFxTimer?.cancel();
      fetchFxTimer = null;
    }
  }

  Future<double?> fetch(String api, String currency) async {
    try {
      final p = await getCoingeckoPrice(api: api, currency: currency, c: coinContext.coin);
      setPrice(p);
      return p;
    } catch (_) {
      return null;
    }
  }
}

@Riverpod(keepAlive: true)
class SupportedCurrenciesNotifier extends _$SupportedCurrenciesNotifier {
  @override
  Future<List<String>> build() async {
    final settings = await ref.watch(appSettingsProvider.future);
    return await getSupportedVsCurrencies(api: settings.coingecko, c: coinContext.coin);
  }
}

@freezed
sealed class AppSettings with _$AppSettings {
  factory AppSettings({
    required String dbName,
    required String net,
    required bool isLightNode,
    required String lwd,
    required String blockExplorer,
    required String syncInterval, // in blocks
    required String actionsPerSync,
    required int transport,
    required String proxy,
    required String coingecko,
    required bool recovery,
    required bool needPin,
    required DateTime pinUnlockedAt,
    required bool offline,
    required bool getFx,
    required QRSettings qrSettings,
    required bool vault,
    required bool expertMode,
    required String paletteName,
    required bool darkMode,
    required bool transactionTableMode,
    required bool collapsePoolBalances,
    required String currency,
    required String votingConfigUrl,
  }) = _AppSettings;
}

@Riverpod(keepAlive: true)
class LogNotifier extends _$LogNotifier {
  @override
  List<String> build() {
    return [];
  }

  void append(String logLine) {
    state.add(logLine);
  }
}

@Riverpod(keepAlive: true)
class CurrentHeight extends _$CurrentHeight {
  int? _cachedHeight;
  DateTime? _lastFetch;
  static const _ttl = Duration(seconds: 15);

  @override
  Future<int?> build() async {
    _cachedHeight = blockHeightService.lastHeight;
    return _cachedHeight;
  }

  /// Get current height, cached up to 15s. Respects offline mode.
  /// Set [force] to true to bypass the TTL cache.
  Future<int?> fetch({bool force = false}) async {
    final settings = await ref.read(appSettingsProvider.future);
    if (settings.offline) {
      return _cachedHeight;
    }
    if (!force) {
      final now = DateTime.now();
      if (_cachedHeight != null && _lastFetch != null && now.difference(_lastFetch!) < _ttl) {
        return _cachedHeight;
      }
    }
    return await _doFetch();
  }

  Future<int?> _doFetch() async {
    _cachedHeight = await blockHeightService.fetchCurrent();
    _lastFetch = DateTime.now();
    state = AsyncData(_cachedHeight);
    return _cachedHeight;
  }

  void updateFromService(int height) {
    _cachedHeight = height;
    _lastFetch = DateTime.now();
    state = AsyncData(height);
  }
}

Mempool mempool = Mempool();

@Freezed(makeCollectionsUnmodifiable: false)
sealed class MempoolState with _$MempoolState {
  factory MempoolState({
    required bool running,
    required Map<int, int> unconfirmedFunds,
    required List<(String, String, int)> unconfirmedTx,
  }) = _MempoolState;
}

@Riverpod(keepAlive: true)
class MempoolNotifier extends _$MempoolNotifier {
  @override
  MempoolState build() {
    return MempoolState(running: false, unconfirmedFunds: {}, unconfirmedTx: []);
  }

  void runMempoolListener() async {
    final c = coinContext.coin;
    final settings = await ref.read(appSettingsProvider.future);
    if (settings.offline) return;

    while (true) {
      try {
        if (settings.offline) return;
        state = MempoolState(running: true, unconfirmedFunds: {}, unconfirmedTx: []);

        final comp = Completer();
        mempool.run(c: c).listen(
              (msg) {
                if (msg is MempoolMsg_TxId) {
                  final mempoolTx = msg.field0; // txid hash
                  final amounts = mempoolTx.amounts; // list of (account id, name, value unconfirmed)
                  final size = mempoolTx.size; // size in bytes of the tx
                  addTx(mempoolTx.txid, amounts, size);
                }
                if (msg is MempoolMsg_BlockHeight) {
                  clear();
                }
              },
              onDone: comp.complete,
              onError: (e) {
                comp.complete();
              },
            );
        await comp.future; // wait for the stream to complete
        await Future.delayed(Duration(seconds: 5));
      } catch (_) {}
    }
  }

  void addTx(String txId, List<MempoolAmount> unconfirmedValues, int size) {
    final unconfirmed = unconfirmedValues.map((a) => "${a.name} ${zatToString(BigInt.from(a.value))}").join(", ");
    final unconfirmedTx = state.unconfirmedTx;
    unconfirmedTx.add((txId, unconfirmed, size));

    final unconfirmedFunds = state.unconfirmedFunds;
    for (var a in unconfirmedValues) {
      final account = a.account;
      final amount = a.value;
      unconfirmedFunds.update(
        account,
        (value) => value + amount,
        ifAbsent: () => amount,
      );
    }
    state = state.copyWith(unconfirmedTx: unconfirmedTx, unconfirmedFunds: unconfirmedFunds);
  }

  void clear() {
    state = state.copyWith(unconfirmedFunds: {}, unconfirmedTx: []);
  }
}

void runLogListener() async {
  final stream = setLogStream();
  final scope = ProviderScope.containerOf(appKey.currentContext!);
  final log = scope.read(logProvider.notifier);
  stream.listen((m) {
    log.append(m.message);
    if (m.span == "transaction") {
      toastification.show(
        description: Text(m.message),
        margin: EdgeInsets.all(8),
        borderRadius: BorderRadius.circular(8),
        animationDuration: Durations.long1,
        autoCloseDuration: Duration(seconds: 3),
      );
    }
  });
}

// Need a mempool provider to inform accounts
// that their balance may have changed due to
// new txs in the mempool

//   // Only settings from SharedPreferences
//   // This is called before getting the database

//   Future<void> loadSettings() async {
//     net = await getNetworkName();
//     lwd = await getProp(key: "lwd") ?? lwd;
//     syncInterval = await getProp(key: "sync_interval") ?? syncInterval;
//     actionsPerSync = await getProp(key: "actions_per_sync") ?? actionsPerSync;
//     blockExplorer = await getProp(key: "block_explorer") ?? blockExplorer;
//   }

@Riverpod(keepAlive: true)
class SynchronizerNotifier extends _$SynchronizerNotifier {
  bool syncInProgress = false;
  StreamSubscription<int>? _autoSyncSubscription;
  bool _handlingAutoSyncHeight = false;
  bool _forceNextAutoSync = false;
  int? _pendingAutoSyncHeight;
  StreamSubscription<SyncProgress>? syncProgressSubscription;
  int retryCount = 0;

  @override
  SyncState build() {
    ref.onDispose(() {
      unawaited(_autoSyncSubscription?.cancel());
    });
    return SyncState(
      start: 0,
      end: 0,
      height: 0,
      time: 0,
      accounts: [],
    );
  }

  void begin(List<Account> accounts, int endHeight) {
    final minAccount = accounts.fold((0, 0), (a, b) {
      if (b.height < a.$1) return (b.height, b.time);
      return a;
    });
    state = SyncState(
      start: minAccount.$1,
      end: endHeight,
      height: minAccount.$1,
      accounts: accounts,
      time: minAccount.$2,
    );
  }

  void update(int height, int time) {
    state = state.copyWith(height: height, time: time);
  }

  void end() {
    state = SyncState(
      start: 0,
      end: 0,
      height: 0,
      time: 0,
      accounts: [],
    );
  }

  Future<void> startSynchronize(
    List<Account> accounts, {
    int? currentHeight,
  }) async {
    if (syncInProgress) return;

    final c = coinContext.coin;
    final settings = ref.read(appSettingsProvider).requireValue;
    if (settings.offline) return;

    syncInProgress = true;
    retryCount = 0;
    final completer = Completer<void>();
    var requestedHeight = currentHeight;

    while (true) {
      try {
        logger.i("Starting Synchronization");
        if (WidgetsBinding.instance.lifecycleState == AppLifecycleState.resumed) {
          showSnackbar("Starting Synchronization");
        }
        final syncHeight = requestedHeight ?? await blockHeightService.fetchCurrent();
        requestedHeight = null;

        begin(accounts, syncHeight);

        final progress = synchronize(
          accounts: accounts.map((a) => a.id).toList(),
          currentHeight: syncHeight,
          actionsPerSync: int.parse(settings.actionsPerSync),
          transparentLimit: 100,
          checkpointAge: 500_000,
          fast: true,
          c: c,
        );
        await syncProgressSubscription?.cancel();

        final done = Completer<void>();
        syncProgressSubscription = progress.listen(
          (p) {
            retryCount = 0;
            update(p.height, p.time);
          },
          onError: (e) {
            done.completeError(e is AnyhowException ? e : AnyhowException(e.toString()));
          },
          onDone: () {
            done.complete();
          },
          cancelOnError: true,
        );

        await done.future;

        // Sync completed successfully
        end();
        syncInProgress = false;
        await syncProgressSubscription?.cancel();
        syncProgressSubscription = null;
        ref.invalidate(getAccountsProvider);
        ref.invalidate(accountProvider);
        if (WidgetsBinding.instance.lifecycleState == AppLifecycleState.resumed) {
          showSnackbar("Synchronization Completed");
        }
        logger.i("Synchronization Completed");
        // Fetch tx details in the background for all accounts
        unawaited(Future(() async {
          try {
            for (final account in accounts) {
              await fetchTxDetails(account: account.id, c: c);
            }
            ref.invalidate(accountProvider);
          } on AnyhowException catch (e) {
            logger.e("Error fetching tx details: $e");
          }
        }));

        completer.complete();
        return await completer.future;
      } on AnyhowException catch (e) {
        retryCount++;
        final maxDelay = pow(2, min(retryCount, 10)).toInt();
        final delay = 30 + Random().nextInt(maxDelay);
        logger.e("Sync error: $e\n\nRetrying in $delay seconds (attempt $retryCount)");

        final context = navigatorKey.currentContext;
        if (context != null && context.mounted) {
          await ErrorDialog.show(
            context,
            error: e,
            customMessage: "Sync error (attempt $retryCount of ~10). Retrying in $delay seconds...",
          );
        }

        await Future.delayed(Duration(seconds: delay));
      } finally {
        syncInProgress = false;
      }
    }
  }

  Future<void> autoSync({bool now = false, int? interval}) async {
    final settings = await ref.read(appSettingsProvider.future);
    final effectiveInterval = interval ?? (int.tryParse(settings.syncInterval) ?? 0);

    if (settings.offline || effectiveInterval <= 0) {
      await _autoSyncSubscription?.cancel();
      _autoSyncSubscription = null;
      return;
    }

    if (_autoSyncSubscription == null) {
      var forceFirstHeight = now;
      _autoSyncSubscription = blockHeightService.heights.listen(
        (height) {
          ref.read(currentHeightProvider.notifier).updateFromService(height);
          _queueAutoSync(height, force: forceFirstHeight);
          forceFirstHeight = false;
        },
        onError: (Object error, StackTrace stackTrace) {
          logger.e("Block height polling failed", error: error, stackTrace: stackTrace);
        },
      );
    } else if (now) {
      final height = blockHeightService.lastHeight ?? await blockHeightService.fetchCurrent();
      _queueAutoSync(height, force: true);
    }
  }

  void _queueAutoSync(int height, {required bool force}) {
    _pendingAutoSyncHeight = max(_pendingAutoSyncHeight ?? height, height);
    _forceNextAutoSync |= force;
    if (_handlingAutoSyncHeight) return;
    unawaited(_drainAutoSyncQueue());
  }

  Future<void> _drainAutoSyncQueue() async {
    _handlingAutoSyncHeight = true;
    try {
      while (_pendingAutoSyncHeight != null) {
        final height = _pendingAutoSyncHeight!;
        final force = _forceNextAutoSync;
        _pendingAutoSyncHeight = null;
        _forceNextAutoSync = false;
        try {
          await syncIfNeeded(height, now: force);
        } on AnyhowException catch (error, stackTrace) {
          logger.e("AutoSync failed", error: error, stackTrace: stackTrace);
        }
      }
    } finally {
      _handlingAutoSyncHeight = false;
    }
  }

  Future<void> syncIfNeeded(int currentHeight, {required bool now}) async {
    final settings = ref.read(appSettingsProvider).requireValue;
    List<Account> accountsToSync = [];
    final accounts = await ref.read(getAccountsProvider.future);
    for (var account in accounts) {
      if (account.enabled) {
        final height = account.height;
        if (now || currentHeight - height >= int.parse(settings.syncInterval)) {
          logger.i("Sync needed for ${account.name}");
          accountsToSync.add(account);
        }
      }
    }
    if (accountsToSync.isNotEmpty) {
      await startSynchronize(accountsToSync, currentHeight: currentHeight);
    }
  }
}

@Riverpod(keepAlive: true)
class TransparentScan extends _$TransparentScan {
  int gapLimit = 40;
  StreamSubscription? progressSubscription;
  TransparentScanner? scanner;

  @override
  String build() {
    return "";
  }

  bool get running => state.isNotEmpty;

  Future<void> run(BuildContext context, int gapLimit, {required void Function() onComplete}) async {
    try {
      final c = coinContext.coin;
      final sc = await TransparentScanner.newInstance();
      scanner = sc;
      final endHeight = await getCurrentHeight(c: c);
      final sub = sc.run(endHeight: endHeight, gapLimit: gapLimit, c: c);
      progressSubscription = sub.listen(
        (a) {
          state = a;
        },
        onDone: () {
          state = "";
          onComplete();
        },
        onError: (e) {
          final exception = e as AnyhowException;
          if (context.mounted) showException(context, exception.message);
        },
        cancelOnError: true,
      );
    } on AnyhowException catch (e) {
      if (context.mounted) await showException(context, e.message);
    }
  }

  Future<void> cancel() async {
    final sc = scanner;
    scanner = null;
    if (sc != null) {
      await sc.cancel();
    }
    await progressSubscription?.cancel();
    progressSubscription = null;
    state = "";
  }
}

@riverpod
class GetTxDetails extends _$GetTxDetails {
  @override
  Future<TxAccount> build(int id) async {
    final c = coinContext.coin;
    return await getTxDetails(idTx: id, c: c);
  }
}

@Riverpod(keepAlive: true)
class Lifecycle extends _$Lifecycle {
  DateTime unlockTime = DateTime.fromMillisecondsSinceEpoch(0);
  bool? locked;

  @override
  Future<bool> build() async {
    if (locked == null) {
      final settings = await ref.watch(appSettingsProvider.future);
      locked = settings.needPin;
    }
    return locked!;
  }

  void unlock() {
    unlockTime = DateTime.now();
    locked = false;
    state = AsyncData(false);
  }

  Future<void> lock({bool force = true}) async {
    final settings = await ref.read(appSettingsProvider.future);
    if (!settings.needPin) return;
    if (force || DateTime.now().difference(unlockTime).inSeconds > 30) {
      unlockTime = DateTime.fromMillisecondsSinceEpoch(0);
      locked = true;
      state = AsyncData(true);
    }
  }
}

class LifecycleWatcher with WidgetsBindingObserver {
  static LifecycleWatcher instance = LifecycleWatcher();

  bool disabled = false;

  void init() {
    WidgetsBinding.instance.addObserver(this);
  }

  @override
  void didChangeAppLifecycleState(AppLifecycleState state) {
    if (state == AppLifecycleState.resumed) {
      final scope = ProviderScope.containerOf(appKey.currentContext!);
      scope.read(lifecycleProvider.notifier).lock(force: false);
    }
    if (state == AppLifecycleState.paused) {
      // Voting driver runs are wallet-scoped background work: stop them at
      // the boundary instead of letting them race the suspended process.
      // Share tracking is deliberately left alone — it is durable and
      // resumes on the next round open.
      final scope = ProviderScope.containerOf(appKey.currentContext!);
      unawaited(scope.read(votingDriveStopAllProvider.notifier).run());
    }
  }
}

@freezed
sealed class AccountsPageData with _$AccountsPageData {
  const factory AccountsPageData({
    required AppSettings settings,
    required List<Account> accounts,
    required double? price,
    required Folder? selectedFolder,
  }) = _AccountsPageData;
}

@riverpod
Future<AccountsPageData> accountsPageData(Ref ref) async {
  final settings = await ref.watch(appSettingsProvider.future);
  final accounts = await ref.watch(getAccountsProvider.future);
  final price = ref.watch(priceProvider);
  final selectedFolder = ref.watch(selectedFolderProvider);

  return AccountsPageData(
    settings: settings,
    accounts: accounts,
    price: price,
    selectedFolder: selectedFolder,
  );
}

// Base account data - accounts + currentAccount
@freezed
sealed class BasicAccountData with _$BasicAccountData {
  const factory BasicAccountData({
    required List<Account> allAccounts,
    required AccountData? currentAccount,
  }) = _BasicAccountData;
}

@riverpod
Future<BasicAccountData> basicAccountData(Ref ref) async {
  final allAccounts = await ref.watch(getAccountsProvider.future);
  final currentAccount = await ref.watch(getCurrentAccountProvider.future);

  return BasicAccountData(
    allAccounts: allAccounts,
    currentAccount: currentAccount,
  );
}

// Account page data - extends BasicAccountData with syncState
@freezed
sealed class AccountPageData with _$AccountPageData {
  const factory AccountPageData({
    required List<Account> allAccounts,
    required AccountData? currentAccount,
    required SyncProgressAccount? syncState,
  }) = _AccountPageData;
}

@riverpod
Future<AccountPageData> accountPageData(Ref ref) async {
  final basicData = await ref.watch(basicAccountDataProvider.future);
  final accountId = basicData.currentAccount?.account.id;
  final syncState = accountId != null ? await ref.watch(syncStateAccountProvider(accountId).future) : null;

  return AccountPageData(
    allAccounts: basicData.allAccounts,
    currentAccount: basicData.currentAccount,
    syncState: syncState,
  );
}

// Full account page data - extends AccountPageData with price + mempool
@freezed
sealed class FullAccountPageData with _$FullAccountPageData {
  const factory FullAccountPageData({
    required List<Account> allAccounts,
    required AccountData? currentAccount,
    required SyncProgressAccount? syncState,
    required double? price,
    required MempoolState mempool,
  }) = _FullAccountPageData;
}

@riverpod
Future<FullAccountPageData> fullAccountPageData(Ref ref) async {
  final accountData = await ref.watch(accountPageDataProvider.future);
  final price = ref.watch(priceProvider);
  final mempool = ref.watch(mempoolProvider);

  return FullAccountPageData(
    allAccounts: accountData.allAccounts,
    currentAccount: accountData.currentAccount,
    syncState: accountData.syncState,
    price: price,
    mempool: mempool,
  );
}

@freezed
sealed class QRSettings with _$QRSettings {
  factory QRSettings({
    required bool enabled,
    required double size,
    required int ecLevel,
    required int delay,
    required int repair,
  }) = _QRSettings;
}

@Riverpod(keepAlive: true)
class VaultNotifier extends _$VaultNotifier {
  @override
  Future<Vault> build() async {
    return Vault.create();
  }

  Future<void> test() async {
    final vault = await future;
    await vault.rustVault.test();
  }

  Future<bool> hasVault() async {
    logger.i("VaultNotifier.hasVault");
    final vault = await future;
    return vault.hasVault();
  }

  Future<Uint8List?> get masterPk async {
    final vault = await future;
    return vault.masterPk;
  }

  Future<void> initialize(String password) async {
    final vault = await future;
    await vault.initialize(password);
  }

  Future<void> deleteLocalVault() async {
    final vault = await future;
    await vault.deleteLocalVault();
  }

  Future<void> resetDevicePart() async {
    final vault = await future;
    await vault.resetDevicePart();
  }

  Future<void> registerDevice({required String password, required Uint8List prf}) async {
    logger.i("VaultNotifier.registerDevice");
    final vault = await future;
    await vault.registerDevice(password: password, prf: prf);
  }

  Future<Uint8List> downloadVaultBytes() async {
    logger.i("VaultNotifier.downloadVaultBytes");
    final vault = await future;
    return vault.downloadVaultBytes();
  }

  Future<List<RestoredAccount>> recoverWithPrf({required Uint8List vaultBytes, required Uint8List prf}) async {
    logger.i("VaultNotifier.recoverWithPrf");
    final vault = await future;
    return vault.recoverWithPrf(vaultBytes: vaultBytes, prf: prf);
  }

  Future<List<RestoredAccount>> recoverVault({required Uint8List vaultBytes, required String masterPassword}) async {
    logger.i("VaultNotifier.recoverVault");
    final vault = await future;
    return vault.recoverVault(vaultBytes: vaultBytes, masterPassword: masterPassword);
  }

  Future<void> storeAccount({required String name, required String seed, required int aindex, required bool useInternal, required int birthHeight}) async {
    EasyDebounce.debounce('vault-store', Duration(milliseconds: 5000), () async {
      logger.i("Storing account into vault: name=$name, aindex=$aindex, useInternal=$useInternal, birthHeight=$birthHeight");
      final vault = await future;
      final pk = (await vault.masterPk)!;
      await vault.storeAccount(name: name, seed: seed, aindex: aindex, useInternal: useInternal, birthHeight: birthHeight, pk: pk);
    });
  }

  Future<void> signOut() async {
    logger.i("VaultNotifier.signOut");
    if (ref.read(votingSubmissionGuardProvider)) {
      throw Exception(
        "A voting submission is in progress. Wait for it to finish before signing out.",
      );
    }
    // Wallet switch: stop this wallet's driver runs (none live past the
    // guard above; this also clears any finished registry entries' wakeups).
    unawaited(ref.read(votingDriveStopAllProvider.notifier).run());
    final vault = await future;
    await vault.signOut();
  }

  Future<void> signIn() async {
    logger.i("VaultNotifier.signIn");
    final vault = await future;
    await vault.signIn(silent: false);
  }
}

// ── Plugin providers ────────────────────────────────────────────────────

@riverpod
Future<List<plugin_api.PluginInfo>> pluginList(Ref ref) async {
  final c = coinContext.coin;
  return await plugin_api.listPlugins(c: c);
}

@riverpod
Future<List<plugin_api.MemoSection>> pluginMemoSections(
  Ref ref,
  List<int> memoBytes,
  Coin c,
) async {
  return await plugin_api.parseMemoWithPlugins(memoBytes: memoBytes, c: c);
}

// ── Migration providers ────────────────────────────────────────────────────

final ironwoodActiveProvider = FutureProvider<bool>((ref) async {
  return await isIronwoodActive(c: coinContext.coin);
});

// ── Voting round drive providers ────────────────────────────────────────────
//
// Live counterpart of the round driver described in
// docs/voting-submission-orchestration.md: the UI saves only `Decision`
// values; prepare, start, and status polling go through the Rust host
// adapter, and the durable truth lives in the voting sidecar.

/// Blocks destructive wallet actions (account deletion, vault sign-out,
/// wallet removal) while a voting submission run is in flight.
@Riverpod(keepAlive: true)
class VotingSubmissionGuard extends _$VotingSubmissionGuard {
  @override
  bool build() => false;

  void setActive(bool active) {
    state = active;
  }
}

/// Rounds whose driver run is live in this process; keeps the submission
/// guard truthful across concurrent per-round jobs.
final Set<String> _votingDriveRunningRounds = {};

void _syncVotingSubmissionGuard(Ref ref) {
  ref
      .read(votingSubmissionGuardProvider.notifier)
      .setActive(_votingDriveRunningRounds.isNotEmpty);
}

/// State of the submission job for one round: an explicit prepare step
/// (eligibility preview), then a driven run observed by polling.
@freezed
sealed class VotingDriveJobState with _$VotingDriveJobState {
  const factory VotingDriveJobState({
    required String stage, // idle|preparing|ready|driving|error
    VotingEligibilityPreview? eligibility,
    /// Last polled driver status; null before the first poll.
    VotingDriveStatus? status,
    String? error,
  }) = _VotingDriveJobState;
}

/// Submission job for one round. `prepare` persists the bundle layout and
/// returns the eligibility preview the UI shows before the voter commits;
/// `start` hands the round to the Rust driver, which runs to quiescence on
/// its own task. Status polling is owned here, so it survives page
/// navigation; a run already live in Rust is adopted on first read.
@Riverpod(keepAlive: true)
class VotingDriveJob extends _$VotingDriveJob {
  String _roundId = "";
  Timer? _poll;

  @override
  VotingDriveJobState build(String roundId) {
    _roundId = roundId;
    ref.onDispose(_stopPolling);
    // Runs are process-local in Rust; a page reopened mid-run adopts the
    // live registry entry instead of showing a stale idle ballot.
    unawaited(_adoptLiveRun());
    return const VotingDriveJobState(stage: "idle");
  }

  /// Persists the round's bundle layout and records the eligibility preview.
  /// Returns false on failure; the reason is in `state.error`.
  Future<bool> prepare({required String lightwalletdUrl, required String roundName}) async {
    if (state.stage == "preparing" || state.stage == "starting" || state.stage == "driving") {
      return false;
    }
    state = state.copyWith(stage: "preparing", error: null, eligibility: null);
    try {
      final eligibility = await votingPrepareRound(
        roundId: _roundId,
        lightwalletdUrl: lightwalletdUrl,
        roundName: roundName,
        c: coinContext.coin,
      );
      state = state.copyWith(stage: "ready", eligibility: eligibility);
      return true;
    } on Exception catch (e) {
      state = state.copyWith(stage: "error", error: e.toString());
      return false;
    }
  }

  /// Starts the driver run after the ballot was confirmed. A run already
  /// live for the round is adopted rather than duplicated.
  Future<void> start({required String lightwalletdUrl, required String roundName}) async {
    if (state.stage == "starting" || state.stage == "driving") return;
    // The start call re-gathers every round input over the network before
    // returning; reflect it immediately so the UI is not stuck on the
    // ballot behind a closed dialog.
    state = state.copyWith(stage: "starting", error: null);
    try {
      await votingDriveStart(
        roundId: _roundId,
        lightwalletdUrl: lightwalletdUrl,
        roundName: roundName,
        c: coinContext.coin,
      );
    } on Exception catch (e) {
      state = state.copyWith(stage: "error", error: e.toString());
      return;
    }
    state = state.copyWith(stage: "driving", error: null);
    _votingDriveRunningRounds.add(_roundId);
    _syncVotingSubmissionGuard(ref);
    _startPolling();
  }

  /// Cooperatively cancels the run; durable effects stay and a later start
  /// re-plans from the sidecar. Polling continues until Rust reports the run
  /// stopped.
  Future<void> cancel() async {
    try {
      await votingDriveCancel(roundId: _roundId, c: coinContext.coin);
    } on Exception catch (e) {
      state = state.copyWith(error: e.toString());
    }
  }

  Future<void> _adoptLiveRun() async {
    try {
      final status = await votingDriveStatus(roundId: _roundId, c: coinContext.coin);
      if (status.running && state.stage != "driving") {
        state = state.copyWith(stage: "driving", status: status);
        _votingDriveRunningRounds.add(_roundId);
        _syncVotingSubmissionGuard(ref);
        _startPolling();
      }
    } on Exception {
      // No registry entry (or wallet not open yet); the idle ballot is right.
    }
  }

  void _startPolling() {
    _stopPolling();
    unawaited(_pollOnce());
    _poll = Timer.periodic(const Duration(seconds: 2), (_) => unawaited(_pollOnce()));
  }

  void _stopPolling() {
    _poll?.cancel();
    _poll = null;
  }

  Future<void> _pollOnce() async {
    try {
      final status = await votingDriveStatus(roundId: _roundId, c: coinContext.coin);
      state = state.copyWith(status: status);
      if (!status.running) {
        _stopPolling();
        _votingDriveRunningRounds.remove(_roundId);
        _syncVotingSubmissionGuard(ref);
      }
    } on Exception {
      // Transient status errors keep the poll loop alive; the run outlives
      // them and the next tick retries.
    }
  }
}

/// Friendly headline for a driver quiescence label, as formatted by
/// `quiescence_label` in `rust/src/voting/drive.rs`.
String votingQuiescenceLabel(String? quiescence) {
  final q = quiescence;
  if (q == null) return "Submitting votes";
  if (q == "done") return "Votes submitted";
  if (q.startsWith("background_shares")) {
    return "Votes confirmed — delivering helper shares";
  }
  if (q == "cancelled") return "Cancelled — resume to continue";
  if (q.startsWith("needs_ballot")) return "Ballot incomplete";
  if (q.startsWith("needs_delegation_signatures")) {
    return "Delegation signatures needed";
  }
  if (q == "needs_bundle_setup") return "Bundle setup needed — prepare again";
  if (q == "chain_terminal") return "Chain rejected the submission";
  if (q == "chain_recovery_stalled") {
    return "Submission recovery stalled — try again later";
  }
  if (q == "failures") return "Submission failed";
  if (q.startsWith("pass_budget_exhausted")) return "Paused — resume to continue";
  return q.replaceAll('_', ' ');
}

/// Re-runnable trigger to stop every live driver run for the current wallet.
/// Backgrounding and wallet switches invoke it so no run outlives its wallet;
/// durable effects stay and a later start re-plans from the sidecar.
@Riverpod(keepAlive: true)
class VotingDriveStopAll extends _$VotingDriveStopAll {
  @override
  void build() {}

  Future<void> run() async {
    try {
      final cancelled = await votingDriveCancelAll(c: coinContext.coin);
      if (cancelled > BigInt.zero) {
        logger.i("[Voting] stopped $cancelled driver run(s) for this wallet");
      }
    } on Exception catch (e) {
      // Best-effort: backgrounding and sign-out never surface errors.
      logger.i("[Voting] drive stop-all skipped: $e");
    }
  }
}
