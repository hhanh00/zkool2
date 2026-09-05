import 'dart:async';
import 'dart:convert';
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
import 'package:zkool/services/votechain_backoff.dart';
import 'package:zkool/services/votechain_classify.dart';
import 'package:zkool/services/votechain_confirmation.dart';
import 'package:zkool/services/votechain_failover.dart';
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
import 'package:zkool/src/rust/api/voting.dart';
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
    final voteNodeUrl =
        (hasDb ? await getProp(key: "vote_node_url", c: c) : null) ?? "";
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
      voteNodeUrl: voteNodeUrl,
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

  Future<void> setVoteNodeUrl(String url) async {
    await putProp(key: "vote_node_url", value: url, c: coinContext.coin);
    state = state.whenData((s) => s.copyWith(
          voteNodeUrl: url,
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
    await putProp(key: "vote_node_url", value: settings.voteNodeUrl, c: c);
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
      final p = await getCoingeckoPrice(api: api, currency: currency);
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
    return await getSupportedVsCurrencies(api: settings.coingecko);
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
    required String voteNodeUrl,
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
        syncProgressSubscription?.cancel();
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
        return completer.future;
      } on AnyhowException catch (e) {
        retryCount++;
        final maxDelay = pow(2, min(retryCount, 10)).toInt();
        final delay = 30 + Random().nextInt(maxDelay);
        logger.e("Sync error: $e\n\nRetrying in $delay seconds (attempt $retryCount)");

        final context = navigatorKey.currentContext;
        if (context != null) {
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

// ── Voting providers ───────────────────────────────────────────────────────

/// Aggregated recovery-first view of one voting round: the fork-derived
/// resume plan, the full recovery snapshot, and the persisted ballot intents.
/// Every voting screen loads this before acting; nothing about a voting
/// session lives in Dart state.
@freezed
sealed class VotingSessionState with _$VotingSessionState {
  factory VotingSessionState({
    required VotingRoundPlan? plan,
    required VotingRoundRecovery? recovery,
    required List<VotingBallotIntent> intents,
  }) = _VotingSessionState;
}

/// Per-round voting session. `build()` is the recovery-first triple load;
/// `refresh()` re-runs it after an action mutates the voting DB.
@Riverpod(keepAlive: true)
class VotingSession extends _$VotingSession {
  String _roundId = "";

  @override
  Future<VotingSessionState> build(String roundId) {
    _roundId = roundId;
    return _load();
  }

  Future<VotingSessionState> _load() async {
    final c = coinContext.coin;
    // Pass the draft proposal ids so the fork's plan can see open proposals:
    // `needs_draft_setup` (the fresh-round trigger) and `all_decided` are
    // computed against them, and are vacuously false/true with an empty list.
    final draftIds = await _draftProposalIds(c);
    final plan = await votingPlan(
      roundId: _roundId,
      proposalIds: draftIds,
      c: c,
    );
    final recovery = await votingRecovery(roundId: _roundId, c: c);
    final intents = await votingBallotIntents(roundId: _roundId, c: c);
    return VotingSessionState(plan: plan, recovery: recovery, intents: intents);
  }

  Future<List<int>> _draftProposalIds(Coin c) async {
    try {
      final drafts = await votingDraftsLoad(roundId: _roundId, c: c);
      if (drafts == null || drafts.isEmpty) return const [];
      return (jsonDecode(drafts) as List<dynamic>)
          .map((d) => (d as Map<String, dynamic>)['proposal_id'] as int? ?? 0)
          .where((id) => id > 0)
          .toList();
    } on Exception {
      return const [];
    }
  }

  Future<void> refresh() async {
    state = const AsyncValue.loading();
    state = await AsyncValue.guard(_load);
  }
}

/// Rounds persisted in the voting DB for the current wallet.
@riverpod
Future<List<VotingRoundInfo>> votingRoundList(Ref ref) async {
  final c = coinContext.coin;
  return await votingRounds(c: c);
}

/// Sessions for every locally-known round, fetched in ONE Rust call that
/// holds a single pool connection (per-round loads would need one connection
/// per round and stall the pool once the page lists many rounds).
@riverpod
Future<Map<String, VotingSessionState>> votingSessionsAll(Ref ref) async {
  final rounds = await ref.watch(votingRoundListProvider.future);
  if (rounds.isEmpty) return const {};
  final c = coinContext.coin;
  final sessions = await votingSessions(
    roundIds: rounds.map((r) => r.roundId).toList(),
    c: c,
  );
  return {
    for (final s in sessions)
      s.roundId: VotingSessionState(
        plan: s.plan,
        recovery: s.recovery,
        intents: s.intents,
      ),
  };
}

/// Friendly round title from the vote chain round status, falling back to
/// the round id. The chain's `title` field is the only friendly name source
/// (the config and the local DB carry no titles).
@riverpod
Future<String> votingRoundTitle(Ref ref, String roundId, String chainUrl) async {
  final c = coinContext.coin;
  final res = await votechainRoundStatus(
    baseUrl: chainUrl,
    roundId: roundId,
    c: c,
  );
  if (res.statusCode < 200 || res.statusCode >= 300) return roundId;
  final body = jsonDecode(res.body) as Map<String, dynamic>;
  final round = body['round'] as Map<String, dynamic>? ?? {};
  final title = round['title'];
  if (title is String && title.trim().isNotEmpty) return title;
  return roundId;
}

/// Normalized chain round status ("active" / "tallying" / "closed") from the
/// vote chain round status, or null when unresolved (non-2xx or absent).
/// Mirrors vizor's `votingPollListStatus`: numeric 1/2/3 plus lenient string
/// forms. Only "tallying"/"closed" rounds have a published tally.
@riverpod
Future<String?> votingRoundStatus(Ref ref, String roundId, String chainUrl) async {
  final c = coinContext.coin;
  final res = await votechainRoundStatus(
    baseUrl: chainUrl,
    roundId: roundId,
    c: c,
  );
  if (res.statusCode < 200 || res.statusCode >= 300) return null;
  final body = jsonDecode(res.body) as Map<String, dynamic>;
  final round = body['round'] as Map<String, dynamic>? ?? {};
  final status = round['status'];
  if (status is int) {
    return switch (status) {
      2 => "tallying",
      3 => "closed",
      _ => "active",
    };
  }
  if (status is String) {
    final s = status.trim().toLowerCase();
    if (s == '2' || s.contains('tally') || s == 'pending') return "tallying";
    if (s == '3' ||
        s.contains('closed') ||
        s.contains('complete') ||
        s.contains('done') ||
        s.contains('ended') ||
        s.contains('final') ||
        s.contains('result')) {
      return "closed";
    }
    return "active";
  }
  return null;
}

/// One round proposal with its option labels, from the vote chain round
/// status — used to render human-readable ballot evidence.
class VotingProposalInfo {
  final int id;
  final String title;
  final Map<int, String> optionLabels;

  const VotingProposalInfo({
    required this.id,
    required this.title,
    required this.optionLabels,
  });
}

/// Parsed proposals (id, title, option id → label) for a round, from the
/// chain round status. Empty when the fetch fails.
@riverpod
Future<List<VotingProposalInfo>> votingRoundProposals(
  Ref ref,
  String roundId,
  String chainUrl,
) async {
  final c = coinContext.coin;
  final res = await votechainRoundStatus(
    baseUrl: chainUrl,
    roundId: roundId,
    c: c,
  );
  if (res.statusCode < 200 || res.statusCode >= 300) return const [];
  final body = jsonDecode(res.body) as Map<String, dynamic>;
  final round = body['round'] as Map<String, dynamic>? ?? {};
  final proposals = round['proposals'] as List<dynamic>? ?? [];
  final result = <VotingProposalInfo>[];
  for (final p in proposals) {
    if (p is! Map) continue;
    final pid = p['id'];
    if (pid is! int || pid < 1) continue;
    final title = (p['title'] ?? "Proposal $pid").toString();
    var options = <int, String>{};
    final opts = p['options'] as List<dynamic>? ?? [];
    for (final entry in opts.asMap().entries) {
      final o = entry.value;
      if (o is! Map) continue;
      final id = (o['index'] is int) ? o['index'] as int : entry.key;
      options[id] =
          (o['label'] ?? o['short_title'] ?? o['title'] ?? "Option").toString();
    }
    if (options.isEmpty) {
      // Vote-sdk default: Yes/No when options are missing.
      options = const {0: "Yes", 1: "No"};
    }
    result.add(VotingProposalInfo(id: pid, title: title, optionLabels: options));
  }
  return result;
}

/// Resolved and authenticated voting config for the configured source URL.
/// `build()` returns the last cached resolved config without touching the
/// network (so merely reading the provider never triggers a fetch); call
/// `resolve()` to fetch fresh, falling back to cached on failure.
@Riverpod(keepAlive: true)
class VotingConfigNotifier extends _$VotingConfigNotifier {
  @override
  Future<VotingConfig?> build() async {
    final settings = await ref.watch(appSettingsProvider.future);
    final source = settings.votingConfigUrl;
    if (source.isEmpty) return null;
    return votingConfigCached(source: source, c: coinContext.coin);
  }

  Future<VotingConfig?> _resolve(String source) async {
    final c = coinContext.coin;
    try {
      return await votingConfigResolve(source: source, c: c);
    } on AnyhowException catch (e) {
      logger.e("Voting config resolve failed for $source: ${e.message}");
      final cached = await votingConfigCached(source: source, c: c);
      if (cached != null) {
        logger.w("Serving cached voting config for $source");
        return cached;
      }
      rethrow;
    }
  }

  /// Resolve the voting config, using [source] when provided or the
  /// configured URL from app settings otherwise.
  /// Throws when resolution fails and no cached config exists.
  Future<VotingConfig?> resolve({String? source}) async {
    source ??= (await ref.read(appSettingsProvider.future)).votingConfigUrl;
    state = const AsyncValue.loading();
    try {
      final result = source.isEmpty ? null : await _resolve(source);
      state = AsyncValue.data(result);
      return result;
    } catch (e, stackTrace) {
      state = AsyncValue.error(e, stackTrace);
      rethrow;
    }
  }
}

/// Blocks destructive wallet actions (account deletion, vault sign-out,
/// wallet removal) while a voting submission job is in flight.
@Riverpod(keepAlive: true)
class VotingSubmissionGuard extends _$VotingSubmissionGuard {
  @override
  bool build() => false;

  void setActive(bool active) {
    state = active;
  }
}

/// State of the voting submission job for one round.
@freezed
sealed class VotingSubmissionJobState with _$VotingSubmissionJobState {
  factory VotingSubmissionJobState({
    required String stage, // idle|preparing|proving|submitting|confirming|voting|shares|retrying|done|error
    required double progress,
    String? error,
    /// Voting weight (zatoshi) delegated by the prepared bundle, shown in
    /// the status UI once the delegation prepare step completes.
    BigInt? eligibleWeightZatoshi,
    /// Chain evidence of what this run confirmed: the delegation tx hash, or
    /// the first confirmed vote hash when no delegation ran.
    String? txHash,
    /// Block height at which [txHash] was included; set together with it.
    int? confirmHeight,
    /// Honest "done" headline describing what THIS run actually completed.
    String? doneLabel,
    /// Current automatic retry of a transiently failed run (1-based).
    int? retryAttempt,
    /// Seconds until the next automatic retry fires.
    int? retryInSeconds,
  }) = _VotingSubmissionJobState;
}

/// Delegation execution job for one round. Runs the serialized chain:
/// prepare (or resume) → setup → build submission (progress stream) →
/// broadcast → mark submitted → poll confirmation → confirm. Vote casting
/// lands in a later phase. Restart-safe: the resume plan decides which steps
/// run (`delegate` vs `poll_delegation`), never re-broadcasting a recorded tx.
@Riverpod(keepAlive: true)
class VotingSubmissionJob extends _$VotingSubmissionJob {
  @override
  VotingSubmissionJobState build(String roundId) {
    return VotingSubmissionJobState(stage: "idle", progress: 0);
  }

  Future<void> start({
    required String chainUrl,
    required String pirServerUrl,
    VotingPirLayout? pirLayout,
    String? roundParamsJson,
    String? roundName,
    int? maxRealNotesPerBundle,
    String? lightwalletdUrl,
    String voteNodeUrl = "",
    int ceremonyStart = 0,
    int? voteEnd,
    List<String> shareServerUrls = const [],
    bool singleShare = false,
  }) async {
    if (state.stage != "idle" &&
        state.stage != "error" &&
        state.stage != "done") {
      return; // already running (or waiting to retry)
    }
    state = state.copyWith(
      stage: "running",
      progress: 0,
      error: null,
      retryAttempt: null,
      retryInSeconds: null,
    );
    ref.read(votingSubmissionGuardProvider.notifier).setActive(true);
    // Every step below is plan-driven and idempotent, so re-running the body
    // after a transient failure is always safe: recorded state is re-read and
    // the plan decides what still needs doing. Transient failures retry with
    // backoff; deterministic rejections fail fast to the error stage.
    var attempt = 0;
    while (true) {
      try {
        final chainUrls = await _effectiveChainUrls(chainUrl);
        final failover = VoteChainFailover(allServers: chainUrls);
        final delegated = await _runDelegation(
          chainUrls: chainUrls,
          failover: failover,
          pirServerUrl: pirServerUrl,
          pirLayout: pirLayout,
          roundParamsJson: roundParamsJson,
          roundName: roundName,
          maxRealNotesPerBundle: maxRealNotesPerBundle,
          lightwalletdUrl: lightwalletdUrl,
        );
        final voted = await _runVotes(
          chainUrls: chainUrls,
          failover: failover,
          voteNodeUrl: voteNodeUrl,
        );
        // The vote chain servers double as helper (share) servers; the voting
        // flow never passes shareServerUrls, so fall back to the configured
        // vote servers (mirrors vizor's context.config.voteServers).
        final shareUrls = await _effectiveShareServerUrls(shareServerUrls);
        // The voting pages never pass ceremonyStart/voteEnd either — resolve
        // them from the chain round status so the share plan can schedule.
        var effectiveCeremony = ceremonyStart;
        var effectiveVoteEnd = voteEnd;
        if (effectiveCeremony == 0 || effectiveVoteEnd == null) {
          final timing =
              await _roundShareTiming(chainUrls: chainUrls, failover: failover);
          if (timing != null) {
            effectiveCeremony = timing.ceremonyStart;
            effectiveVoteEnd = timing.voteEnd;
          }
        }
        final shared = await _submitShares(
          failover: failover,
          ceremonyStart: effectiveCeremony,
          voteEnd: effectiveVoteEnd,
          shareServerUrls: shareUrls,
          singleShare: singleShare,
        );
        final String doneLabel;
        if (delegated) {
          doneLabel = "Delegation confirmed";
        } else if (voted) {
          doneLabel = "Votes submitted";
        } else if (shared) {
          doneLabel = "Shares submitted";
        } else {
          doneLabel = await _remainingLabel();
        }
        state = state.copyWith(stage: "done", progress: 1, doneLabel: doneLabel);
        ref.read(votingSubmissionGuardProvider.notifier).setActive(false);
        // The round is now recorded locally and its plan advanced; refresh the
        // voting page so the tile leaves "Join" and shows the real status
        // without a manual refresh.
        ref.invalidate(votingRoundListProvider);
        ref.invalidate(votingSessionProvider(roundId));
        return;
      } on TransientVoteChainException catch (e) {
        attempt++;
        if (attempt > voteChainMaxAutoRetries) {
          state = state.copyWith(
            stage: "error",
            error: e.toString(),
            retryAttempt: null,
            retryInSeconds: null,
          );
          ref.read(votingSubmissionGuardProvider.notifier).setActive(false);
          return;
        }
        final delay = voteChainRetryDelay(attempt);
        state = state.copyWith(
          stage: "retrying",
          retryAttempt: attempt,
          retryInSeconds: delay.inSeconds,
        );
        await Future<void>.delayed(delay);
      } on Exception catch (e) {
        state = state.copyWith(stage: "error", error: e.toString());
        ref.read(votingSubmissionGuardProvider.notifier).setActive(false);
        return;
      }
    }
  }

  void reset() {
    state = VotingSubmissionJobState(stage: "idle", progress: 0);
  }

  /// The vote chain front the caller chose first, followed by the remaining
  /// configured vote servers — the failover candidate list for chain calls.
  Future<List<String>> _effectiveChainUrls(String chainUrl) async {
    final urls = <String>[];
    if (chainUrl.isNotEmpty) urls.add(chainUrl);
    try {
      final config = await ref.read(votingConfigProvider.future);
      for (final s in config?.voteServers ?? const <VotingServiceEndpoint>[]) {
        if (!urls.contains(s.url)) urls.add(s.url);
      }
    } on Exception {
      // Config unavailable: failover degrades to the single passed URL.
    }
    return urls;
  }

  /// Runs the delegation steps for this round; returns true when a delegation
  /// was confirmed in this run (fresh broadcast or recorded-hash poll).
  Future<bool> _runDelegation({
    required List<String> chainUrls,
    required VoteChainFailover failover,
    required String pirServerUrl,
    VotingPirLayout? pirLayout,
    String? roundParamsJson,
    String? roundName,
    int? maxRealNotesPerBundle,
    String? lightwalletdUrl,
  }) async {
    final c = coinContext.coin;
    final session = await ref.read(votingSessionProvider(roundId).future);
    final plan = session.plan;
    final steps = plan?.nextSteps ?? const <VotingNextStep>[];
    final delegateStep = steps.where((s) => s.kind == "delegate").firstOrNull;
    final pollStep =
        steps.where((s) => s.kind == "poll_delegation").firstOrNull;
    // A fresh round has no plan steps at all; the fork's plan flags it with
    // `needs_draft_setup`. Treat that as delegation work for the first bundle
    // that still needs it (mirrors vizor's roundPlanNeedsDraftSetup trigger).
    var freshDelegate = false;
    int bundleIndex;
    if (delegateStep != null || pollStep != null) {
      bundleIndex = (delegateStep ?? pollStep!).bundleIndex;
    } else {
      final p = plan;
      if (p == null || !p.needsDraftSetup) {
        return false; // no delegation work for this round
      }
      final pending = p.delegationStatuses
          .where((s) => s.phase == "prepared" || s.phase == "committed")
          .firstOrNull;
      if (p.delegationStatuses.isNotEmpty && pending == null) {
        return false; // all bundles confirmed; _runVotes handles casting
      }
      bundleIndex = pending?.bundleIndex ?? 0;
      freshDelegate = true;
    }

    String? txHash;
    if (delegateStep != null || freshDelegate) {
      state = state.copyWith(stage: "preparing");
      // Recovery with a persisted wire: a previous run already proved and
      // built the submission. Re-proving would waste a ZK proof and trip the
      // fork's round-phase bookkeeping when votes already exist for the
      // round — reuse the persisted wire for a byte-identical resubmission
      // instead (the server dedups it or the tree reconciliation records the
      // confirmation).
      var wireJson = await delegationWireJson(
        roundId: roundId,
        bundleIndex: bundleIndex,
        c: c,
      );
      if (wireJson == null || wireJson.isEmpty) {
        // The delegation keys embed an app-owned voting hotkey; auto-create
        // one when missing and the round isn't already hotkey-bound (mirrors
        // vizor's _ensureHotkey). A bound round without the stored hotkey
        // keeps failing with the load error instead of silently generating a
        // mismatched key.
        if (!(plan?.hotkeyBound ?? false)) {
          await _ensureVotingHotkey();
        }
        // The voting pages never pass a lightwalletd URL — use the app's
        // configured one for the fresh prepare (the fork needs it to fetch
        // the snapshot anchor tree state).
        final lwdUrl = (lightwalletdUrl == null || lightwalletdUrl!.isEmpty)
            ? await _appLwdUrl()
            : lightwalletdUrl!;
        final prepared = roundParamsJson != null && roundName != null
            ? await delegationPrepare(
                roundParamsJson: roundParamsJson,
                roundName: roundName,
                sessionJson: null,
                bundleIndex: bundleIndex,
                maxRealNotesPerBundle: maxRealNotesPerBundle,
                lightwalletdUrl: lwdUrl,
                c: c,
              )
            : await delegationPrepareResume(
                roundId: roundId,
                bundleIndex: bundleIndex,
                maxRealNotesPerBundle: maxRealNotesPerBundle,
                lightwalletdUrl: lightwalletdUrl,
                c: c,
              );
        state = state.copyWith(
          eligibleWeightZatoshi: prepared.eligibleWeightZatoshi,
        );

        state = state.copyWith(stage: "proving");
        final pir = await _resolvePirConfig(
          pirServerUrl: pirServerUrl,
          pirLayout: pirLayout,
        );
        await _buildDelegation(
          roundId: roundId,
          bundleIndex: bundleIndex,
          pirLayout: pir.$2,
          pirServerUrl: pir.$1,
        );

        // The FRB boundary drops the build result when a StreamSink is
        // present, so the wire body comes from the prop persisted by the
        // build.
        wireJson = await delegationWireJson(
          roundId: roundId,
          bundleIndex: bundleIndex,
          c: c,
        );
        if (wireJson == null || wireJson.isEmpty) {
          throw AnyhowException(
            "No wire JSON produced for round $roundId bundle $bundleIndex",
          );
        }
      }

      if (wireJson == null || wireJson.isEmpty) {
        throw AnyhowException(
          "No wire JSON produced for round $roundId bundle $bundleIndex",
        );
      }
      state = state.copyWith(stage: "submitting");
      final res = await failover.run(
        baseUrls: chainUrls,
        call: (u) => votechainSubmitDelegation(
          baseUrl: u,
          submissionJson: wireJson!,
          c: c,
        ),
      );
      if (res.statusCode == 422) {
        final rejection = parseVoteChainRejection(res.body);
        if (classifyVoteChainRejection(res.body) ==
            ChainRejectionKind.duplicateNullifier) {
          throw AnyhowException(
            "The vote chain says this delegation's nullifier is already "
            "spent: the delegation is recorded on-chain, but this wallet "
            "has no record of submitting it. If the wallet was restored or "
            "its voting data cleared after the original submission, it no "
            "longer holds the voting hotkey the on-chain delegation is "
            "bound to, and it cannot sign votes for this round. "
            "${rejection.log}",
          );
        }
        throw AnyhowException(
          "Delegation rejected by the vote chain: ${res.body}",
        );
      }
      final submittedHash = txHashFromVoteChainBody(res.body) ?? "";
      final rejection = parseVoteChainRejection(res.body);
      if (res.statusCode < 200 || res.statusCode >= 300) {
        // A 502 "broadcast outcome unknown" body carries the deterministic
        // tx hash: the tx may have landed — record it and let confirmation
        // polling learn the truth. Any other non-2xx without a hash fails.
        if (submittedHash.isEmpty) {
          throw AnyhowException(
            "Vote chain submit failed (HTTP ${res.statusCode}): ${res.body}",
          );
        }
      } else if (rejection.code > 0 || submittedHash.isEmpty) {
        throw AnyhowException(
          "Vote chain rejected the delegation: ${rejection.log}",
        );
      }
      txHash = submittedHash;

      await delegationMarkSubmitted(
        roundId: roundId,
        bundleIndex: bundleIndex,
        txHash: txHash,
        c: c,
      );
    } else {
      // poll-only path: the tx hash is already recorded
      final status = plan!
          .delegationStatuses
          .where((s) => s.bundleIndex == bundleIndex)
          .firstOrNull;
      txHash = status?.txHash;
      if (txHash == null || txHash.isEmpty) {
        throw AnyhowException(
          "Round $roundId bundle $bundleIndex is pending but has no recorded tx hash",
        );
      }
    }

    state = state.copyWith(stage: "confirming");
    final String recordedHash = txHash!;
    int? confirmHeight;
    try {
      final conf = await _pollTxConfirmationWithFallback(
        chainUrls: chainUrls,
        failover: failover,
        txHash: recordedHash,
        wireJson: () async =>
            await delegationWireJson(
              roundId: roundId,
              bundleIndex: bundleIndex,
              c: c,
            ) ??
            "",
        rebroadcast: () => failover.run(
          baseUrls: chainUrls,
          call: (u) async {
            final wire = await delegationWireJson(
              roundId: roundId,
              bundleIndex: bundleIndex,
              c: c,
            );
            if (wire == null || wire.isEmpty) {
              throw AnyhowException(
                "No delegation wire JSON to re-broadcast "
                "round $roundId bundle $bundleIndex",
              );
            }
            return votechainSubmitDelegation(
              baseUrl: u,
              submissionJson: wire,
              c: c,
            );
          },
        ),
        markSubmitted: (h) => delegationMarkSubmitted(
          roundId: roundId,
          bundleIndex: bundleIndex,
          txHash: h,
          c: c,
        ),
      );
      confirmHeight = conf.height;
      await delegationConfirm(
        roundId: roundId,
        bundleIndex: bundleIndex,
        txHash: recordedHash,
        eventsJson: conf.eventsJson,
        c: c,
      );
    } on DuplicateNullifierOnResubmitException {
      // The delegation committed on-chain but its hash was never recorded
      // locally; recover the VAN leaf position from the commitment tree.
      await _reconcileDelegation(
        chainUrls: chainUrls,
        bundleIndex: bundleIndex,
      );
    }
    state = state.copyWith(txHash: recordedHash, confirmHeight: confirmHeight);
    await ref.read(votingSessionProvider(roundId).notifier).refresh();
    // The done state must be backed by the fork's recorded confirmation:
    // verify the bundle reads back as confirmed (tx hash or a
    // commitment-tree-recovered VAN leaf) before claiming success — a stale
    // or partial state must not show "Delegation confirmed".
    final verified = await ref.read(votingSessionProvider(roundId).future);
    final status = verified.plan?.delegationStatuses
        .where((s) => s.bundleIndex == bundleIndex)
        .firstOrNull;
    if (status == null || status.phase != "confirmed") {
      throw AnyhowException(
        "Delegation confirmation was not recorded for "
        "round $roundId bundle $bundleIndex",
      );
    }
    return true;
  }

  /// Fills a missing PIR server URL / layout from the resolved voting config
  /// (the UI never passes them — both push sites send "" / null). Falls back
  /// to the passed values; when the config is unavailable the Rust side
  /// errors with a clear message.
  Future<(String, VotingPirLayout?)> _resolvePirConfig({
    required String pirServerUrl,
    required VotingPirLayout? pirLayout,
  }) async {
    if (pirServerUrl.isNotEmpty && pirLayout != null) {
      return (pirServerUrl, pirLayout);
    }
    try {
      final config = await ref.read(votingConfigProvider.future);
      final url = pirServerUrl.isNotEmpty
          ? pirServerUrl
          : (config?.pirServers.isNotEmpty ?? false)
              ? config!.pirServers.first.url
              : "";
      return (url, pirLayout ?? config?.pirLayout);
    } on Exception {
      return (pirServerUrl, pirLayout);
    }
  }

  /// Honest done label when a run performed no work: the plan may still have
  /// pending steps (e.g. helper shares scheduled near the vote window) even
  /// though nothing was due this run.
  Future<String> _remainingLabel() async {
    final session = await ref.read(votingSessionProvider(roundId).future);
    final pending = session.plan?.nextSteps ?? const <VotingNextStep>[];
    if (pending.isEmpty) return "All steps already confirmed";
    const shareSubmitKinds = {"submit_shares"};
    const shareConfirmKinds = {"confirm_share"};
    if (pending.every((s) => shareSubmitKinds.contains(s.kind))) {
      return "Waiting for the share window";
    }
    if (pending.every((s) => shareConfirmKinds.contains(s.kind))) {
      return "Waiting for share confirmations";
    }
    return "Waiting for the next step";
  }

  /// Resolves the round's ceremony start / vote end from the chain round
  /// status when the flow didn't provide them (the voting pages never pass
  /// them, and the share plan needs both to schedule submissions). Returns
  /// null when the fetch fails or the fields are missing.
  Future<({int ceremonyStart, int voteEnd})?> _roundShareTiming({
    required List<String> chainUrls,
    required VoteChainFailover failover,
  }) async {
    final c = coinContext.coin;
    final VotingChainResponse res;
    try {
      res = await failover.run(
        baseUrls: chainUrls,
        call: (u) => votechainRoundStatus(
          baseUrl: u,
          roundId: roundId,
          c: c,
        ),
      );
    } on TransientVoteChainException {
      return null;
    }
    if (res.statusCode < 200 || res.statusCode >= 300) return null;
    final body = jsonDecode(res.body) as Map<String, dynamic>;
    final round = body['round'] as Map<String, dynamic>? ?? {};
    final ceremony = round['ceremony_phase_start'];
    final end = round['vote_end_time'];
    if (ceremony is! int || end is! int) return null;
    return (ceremonyStart: ceremony, voteEnd: end);
  }

  /// The vote chain servers double as helper (share) servers. The voting
  /// flow never passes `shareServerUrls`, so fall back to the configured
  /// vote servers — mirrors vizor's `context.config.voteServers`.
  Future<List<String>> _effectiveShareServerUrls(
    List<String> shareServerUrls,
  ) async {
    if (shareServerUrls.isNotEmpty) return shareServerUrls;
    try {
      final config = await ref.read(votingConfigProvider.future);
      return config?.voteServers.map((s) => s.url).toList() ?? const [];
    } on Exception {
      return const [];
    }
  }

  /// Returns the app-configured lightwalletd URL (settings `lwd`) — the
  /// voting pages never pass one. Empty when unavailable; the fork then
  /// errors with a clear message.
  Future<String> _appLwdUrl() async {
    try {
      return (await ref.read(appSettingsProvider.future)).lwd;
    } on Exception {
      return "";
    }
  }

  /// Ensures a voting hotkey exists for this wallet. The caller must only
  /// invoke this when the round is not yet hotkey-bound: creating a new key
  /// for a bound round would mismatch the on-chain delegation.
  Future<void> _ensureVotingHotkey() async {
    final c = coinContext.coin;
    try {
      await votingHotkeyGet(c: c);
    } on AnyhowException {
      await votingHotkeyCreate(c: c);
    }
  }

  /// Runs the delegation build/prove stream with the fork's software path:
  /// the build re-runs `setup` internally (sampling fresh PCZT randomness),
  /// so the app must NOT call `delegationSetup` separately and must pass
  /// empty `pcztBytes` (skips the sighash consistency check). A restart
  /// after a partial run leaves a stored sighash that the internal setup
  /// refuses to overwrite — reset the unsigned setup and retry once
  /// (mirrors vizor's keystone-stale-setup recovery).
  Future<void> _buildDelegation({
    required String roundId,
    required int bundleIndex,
    required VotingPirLayout? pirLayout,
    required String pirServerUrl,
  }) async {
    final c = coinContext.coin;
    debugPrint(
      "Voting: delegation build starting for round $roundId bundle "
      "$bundleIndex (no separate setup, empty pczt bytes)",
    );
    for (var attempt = 0; attempt < 2; attempt++) {
      try {
        final stream = delegationBuildSubmission(
          roundId: roundId,
          bundleIndex: bundleIndex,
          pcztBytes: const [],
          pirLayout: pirLayout,
          pirServerUrl: pirServerUrl,
          c: c,
        );
        await for (final event in stream) {
          switch (event) {
            case VotingDelegationProgress_ProofProgress(:final progress):
              state = state.copyWith(progress: progress);
            default:
              break;
          }
        }
        return;
      } on AnyhowException catch (e) {
        if (!e.message.toLowerCase().contains("refusing to overwrite")) {
          rethrow;
        }
        if (attempt == 1) rethrow;
        debugPrint(
          "Voting: stale delegation setup during build for round $roundId "
          "bundle $bundleIndex; resetting and retrying",
        );
        await votingResetSessionState(roundId: roundId, c: c);
      }
    }
  }

  /// Polls the vote chain until the tx is included in a block (HTTP 200 with
  /// a positive `height`), returning the parsed confirmation.
  ///
  /// A recorded hash whose tx never confirms is re-broadcast with
  /// byte-identical wire and polling continues: the chain derives the tx hash
  /// from the payload, so the recorded hash stays valid, and server-side
  /// dedup makes the re-broadcast harmless when the tx is merely slow. This
  /// recovers a server crash that lost an accepted tx from its in-memory
  /// mempool. A duplicate-nullifier 422 on the re-broadcast means the tx
  /// committed successfully without the client learning its hash — the
  /// caller reconciles from the commitment tree via
  /// [DuplicateNullifierOnResubmitException].
  Future<VoteChainTxConfirmation> _pollTxConfirmationWithFallback({
    required List<String> chainUrls,
    required VoteChainFailover failover,
    required String txHash,
    required Future<String> Function() wireJson,
    required Future<VotingChainResponse> Function() rebroadcast,
    required Future<void> Function(String txHash) markSubmitted,
  }) async {
    final c = coinContext.coin;
    Future<VoteChainTxConfirmation?> pollWindow() async {
      for (var attempt = 0; attempt < 45; attempt++) {
        final res = await failover.run(
          baseUrls: chainUrls,
          call: (u) => votechainTxConfirmation(
            baseUrl: u,
            txHash: txHash,
            c: c,
          ),
        );
        if (res.statusCode == 200) {
          final conf = parseVoteChainTxConfirmation(res.body);
          if (conf != null) return conf;
        } else if (res.statusCode == 422) {
          // Included in a block but its execution failed — permanent.
          final rejection = parseVoteChainRejection(res.body);
          throw AnyhowException(
            "The vote chain included tx $txHash but its execution "
            "failed: ${rejection.log}",
          );
        }
        await Future<void>.delayed(const Duration(seconds: 2));
      }
      return null;
    }

    final first = await pollWindow();
    if (first != null) return first;

    final rebuilt = await wireJson();
    if (rebuilt.isEmpty) {
      throw AnyhowException(
        "No wire JSON available to re-broadcast tx $txHash",
      );
    }
    final res = await rebroadcast();
    if (res.statusCode == 422) {
      final rejection = parseVoteChainRejection(res.body);
      if (classifyVoteChainRejection(res.body) ==
          ChainRejectionKind.duplicateNullifier) {
        throw DuplicateNullifierOnResubmitException(
          "The vote chain says a nullifier in the re-broadcast of $txHash "
          "was already spent: the submission committed on-chain but its "
          "confirmation was never recorded locally. ${rejection.log}",
        );
      }
      throw AnyhowException(
        "Vote chain rejected the re-broadcast of $txHash: ${rejection.log}",
      );
    }
    if (res.statusCode < 200 || res.statusCode >= 300) {
      throw TransientVoteChainException(
        "Re-broadcast of $txHash failed (HTTP ${res.statusCode}): ${res.body}",
      );
    }
    final rebroadcastHash = txHashFromVoteChainBody(res.body) ?? "";
    final code = parseVoteChainRejection(res.body).code;
    if (code > 0) {
      throw AnyhowException(
        "Vote chain rejected the re-broadcast of $txHash: "
        "${parseVoteChainRejection(res.body).log}",
      );
    }
    if (rebroadcastHash.isNotEmpty && rebroadcastHash != txHash) {
      throw AnyhowException(
        "Vote chain returned tx hash $rebroadcastHash for a byte-identical "
        "re-broadcast of $txHash — refusing to record a mismatched hash",
      );
    }
    await markSubmitted(txHash);

    final second = await pollWindow();
    if (second != null) return second;
    throw TransientVoteChainException(
      "Timed out waiting for tx $txHash to confirm",
    );
  }

  /// Recovers a delegation whose nullifier is spent on-chain but whose tx
  /// hash was never recorded locally: locate the delegation's VAN commitment
  /// (`gov_comm`) in the commitment tree and record the confirmation from the
  /// recovered leaf position.
  Future<void> _reconcileDelegation({
    required List<String> chainUrls,
    required int bundleIndex,
  }) async {
    final c = coinContext.coin;
    final vanHex = await votingDelegationVanCommitmentHex(
      roundId: roundId,
      bundleIndex: bundleIndex,
      c: c,
    );
    if (vanHex == null || vanHex.isEmpty) {
      throw AnyhowException(
        "The delegation for round $roundId bundle $bundleIndex is on-chain "
        "but its commitment is not persisted — the wallet cannot locate it "
        "in the commitment tree. Contact support.",
      );
    }
    BigInt? position;
    for (final url in chainUrls) {
      try {
        position = await votingTreeFindLeaf(
          roundId: roundId,
          nodeUrl: url,
          targetHex: vanHex,
        );
      } on Exception {
        continue; // this server's tree is unavailable; try the next
      }
      if (position != null) break;
    }
    if (position == null) {
      throw AnyhowException(
        "The chain says the delegation nullifier for round $roundId bundle "
        "$bundleIndex was spent, but its commitment was not found in the "
        "commitment tree. Contact support.",
      );
    }
    await votingRecoverConfirmDelegationFromTree(
      roundId: roundId,
      bundleIndex: bundleIndex,
      vanLeafPosition: position.toInt(),
      c: c,
    );
  }

  /// Recovers a vote whose nullifier is spent on-chain but whose tx hash was
  /// never recorded locally: locate the vote commitment in the commitment
  /// tree and record the confirmation from the recovered leaf positions.
  Future<void> _reconcileVote({
    required List<String> chainUrls,
    required int bundleIndex,
    required int proposalId,
  }) async {
    final c = coinContext.coin;
    final targetHex = await votingVoteCommitmentHex(
      roundId: roundId,
      bundleIndex: bundleIndex,
      proposalId: proposalId,
      c: c,
    );
    BigInt? vcPosition;
    for (final url in chainUrls) {
      try {
        vcPosition = await votingTreeFindLeaf(
          roundId: roundId,
          nodeUrl: url,
          targetHex: targetHex,
        );
      } on Exception {
        continue;
      }
      if (vcPosition != null) break;
    }
    if (vcPosition == null) {
      throw AnyhowException(
        "The chain says the vote nullifier for proposal $proposalId was "
        "spent, but the vote commitment was not found in the commitment "
        "tree. Contact support.",
      );
    }
    // The cast-vote tx appends the VAN output commitment immediately before
    // the vote commitment (verified against the vote-sdk keeper), so the
    // vote's VAN position is the vote commitment position minus one.
    final van = vcPosition > BigInt.zero ? vcPosition - BigInt.one : null;
    await votingRecoverConfirmVoteFromTree(
      roundId: roundId,
      bundleIndex: bundleIndex,
      proposalId: proposalId,
      vcTreePosition: vcPosition,
      vanLeafPosition: van?.toInt(),
      c: c,
    );
  }

  /// Casts and confirms the remaining votes for a round, recovery-first:
  /// `cast_vote` steps commit (streamed), `submit_vote` steps broadcast and
  /// confirm, `poll_vote` steps only poll a previously recorded tx. Ballot
  /// intents are written from the drafts BEFORE the plan is read so a fresh
  /// round's cast steps appear (mirrors vizor's writeBallotIntents). Returns
  /// true when at least one vote was confirmed in this run.
  Future<bool> _runVotes({
    required List<String> chainUrls,
    required VoteChainFailover failover,
    required String voteNodeUrl,
  }) async {
    // An unset Vote Node URL falls back to the vote chain's REST API: the
    // chain server also serves the commitment tree the VAN witnesses sync
    // from (the polls page resolves the same default for the chain).
    final resolvedVoteNodeUrl = voteNodeUrl.isNotEmpty
        ? voteNodeUrl
        : (chainUrls.isNotEmpty ? chainUrls.first : "");
    final c = coinContext.coin;

    // Durable ballot intents first (mirrors vizor's writeBallotIntents):
    // recovery can resume from the right choice if the app dies mid-vote.
    // The round row exists by now (delegation prepared), so the FK holds.
    final draftsJson = await votingDraftsLoad(roundId: roundId, c: c);
    if (draftsJson != null && draftsJson.isNotEmpty) {
      final drafts = jsonDecode(draftsJson) as List<dynamic>;
      for (final d in drafts) {
        final map = d as Map<String, dynamic>;
        final proposalId = map['proposal_id'] as int? ?? 0;
        final choice = map['choice'] as int? ?? 0;
        final numOptions = map['num_options'] as int? ?? 2;
        final skipped = choice == numOptions;
        await votingSetBallotIntent(
          roundId: roundId,
          proposalId: proposalId,
          skipped: skipped,
          choice: skipped ? 0 : choice,
          numOptions: numOptions,
          c: c,
        );
      }
    }

    // Re-read the session: with intents written, the plan now carries the
    // cast_vote steps (bundles exist after delegation).
    await ref.read(votingSessionProvider(roundId).notifier).refresh();
    final session = await ref.read(votingSessionProvider(roundId).future);
    final plan = session.plan;
    final recovery = session.recovery;
    if (plan == null) return false;

    // Defer votes for bundles whose delegation still needs to run (a fresh
    // prepare creates rows for all policy bundles; each run advances one).
    final delegateBundles = plan.nextSteps
        .where((s) => s.kind == "delegate")
        .map((s) => s.bundleIndex)
        .toSet();
    final voteSteps = plan.nextSteps
        .where(
          (s) =>
              (s.kind == "cast_vote" ||
                  s.kind == "submit_vote" ||
                  s.kind == "poll_vote") &&
              !delegateBundles.contains(s.bundleIndex),
        )
        .toList();

    if (voteSteps.isEmpty) return false;
    var didWork = false;
    final byBundle = groupBy(voteSteps, (s) => s.bundleIndex);

    // The commit step deserializes drafts as fork DraftVote, which requires
    // vc_tree_position + single_share and rejects skipped choices and empty
    // batches — sanitize the UI drafts for the cast call.
    final commitDraftsJson = _sanitizedCommitDrafts(draftsJson);

    for (final entry in byBundle.entries) {
      final bundleIndex = entry.key;
      final steps = entry.value;

      final castSteps =
          steps.where((s) => s.kind == "cast_vote").toList();
      for (final step in castSteps) {
        if (commitDraftsJson == null || commitDraftsJson.isEmpty) {
          throw AnyhowException(
            "No draft ballot saved for round $roundId; "
            "open the ballot and review first",
          );
        }
        // Cast ONE proposal at a time and submit it before the next cast:
        // the fork derives the next VAN from the submission state (the
        // proposal-authority mask clears per recorded tx_hash), so the VAN
        // chaining only works when builds interleave with submissions
        // (mirrors vizor's per-draft build loop).
        final drafts = jsonDecode(commitDraftsJson) as List<dynamic>;
        final draft = drafts
            .where(
              (d) => (d as Map<String, dynamic>)['proposal_id'] ==
                  step.proposalId,
            )
            .firstOrNull;
        if (draft == null) {
          throw AnyhowException(
            "No draft for proposal ${step.proposalId} in round $roundId",
          );
        }
        state = state.copyWith(stage: "voting", progress: 0);
        final stream = votingCommitWithProgress(
          roundId: roundId,
          bundleIndex: bundleIndex,
          draftsJson: jsonEncode([draft]),
          voteNodeUrl: resolvedVoteNodeUrl,
          c: c,
        );
        await for (final event in stream) {
          switch (event) {
            case VotingVoteCommitStage_ProofProgress(:final progress):
              state = state.copyWith(progress: progress);
            default:
              break;
          }
        }
        await _submitVote(
          bundleIndex: bundleIndex,
          proposalId: step.proposalId,
          chainUrls: chainUrls,
          failover: failover,
        );
        didWork = true;
      }

      for (final step in steps.where((s) => s.kind == "submit_vote")) {
        await _submitVote(
          bundleIndex: bundleIndex,
          proposalId: step.proposalId,
          chainUrls: chainUrls,
          failover: failover,
        );
        didWork = true;
      }

      for (final step in steps.where((s) => s.kind == "poll_vote")) {
        final vote = recovery?.votes
            .where((v) =>
                v.bundleIndex == step.bundleIndex &&
                v.proposalId == step.proposalId)
            .firstOrNull;
        final txHash = vote?.txHash;
        if (txHash == null || txHash.isEmpty) {
          throw AnyhowException(
            "Vote for proposal ${step.proposalId} is pending but has no "
            "recorded tx hash",
          );
        }
        state = state.copyWith(stage: "confirming");
        final conf = await _pollVoteWithRecovery(
          chainUrls: chainUrls,
          failover: failover,
          bundleIndex: bundleIndex,
          proposalId: step.proposalId,
          txHash: txHash,
        );
        didWork = true;
        if (state.txHash == null) {
          state = state.copyWith(txHash: txHash, confirmHeight: conf?.height);
        }
      }
    }
    return didWork;
  }

  /// Polls a previously recorded vote tx and confirms it, with the
  /// re-broadcast and commitment-tree reconciliation fallbacks. Returns the
  /// confirmation, or null when the vote was reconciled from the tree (no tx
  /// events exist).
  Future<VoteChainTxConfirmation?> _pollVoteWithRecovery({
    required List<String> chainUrls,
    required VoteChainFailover failover,
    required int bundleIndex,
    required int proposalId,
    required String txHash,
  }) async {
    final c = coinContext.coin;
    try {
      final conf = await _pollTxConfirmationWithFallback(
        chainUrls: chainUrls,
        failover: failover,
        txHash: txHash,
        wireJson: () => votingVoteWireJson(
          roundId: roundId,
          bundleIndex: bundleIndex,
          proposalId: proposalId,
          c: c,
        ),
        rebroadcast: () => failover.run(
          baseUrls: chainUrls,
          call: (u) async {
            final wire = await votingVoteWireJson(
              roundId: roundId,
              bundleIndex: bundleIndex,
              proposalId: proposalId,
              c: c,
            );
            if (wire.isEmpty) {
              throw AnyhowException(
                "No vote wire JSON to re-broadcast proposal $proposalId",
              );
            }
            return votechainSubmitVote(
              baseUrl: u,
              submissionJson: wire,
              c: c,
            );
          },
        ),
        markSubmitted: (h) => votingMarkVoteSubmitted(
          roundId: roundId,
          bundleIndex: bundleIndex,
          proposalId: proposalId,
          txHash: h,
          c: c,
        ),
      );
      await votingConfirm(
        roundId: roundId,
        bundleIndex: bundleIndex,
        proposalId: proposalId,
        txHash: txHash,
        eventsJson: conf.eventsJson,
        c: c,
      );
      return conf;
    } on DuplicateNullifierOnResubmitException {
      await _reconcileVote(
        chainUrls: chainUrls,
        bundleIndex: bundleIndex,
        proposalId: proposalId,
      );
      return null;
    }
  }

  /// Broadcasts one committed vote and confirms it: rebuild the wire from
  /// the persisted commitment, submit to the vote chain, record the tx hash,
  /// poll until included in a block, and record the confirmation. Resubmits
  /// are byte-identical (the wire is persisted) and confirmation polling has
  /// the re-broadcast + commitment-tree reconciliation fallbacks.
  Future<void> _submitVote({
    required int bundleIndex,
    required int proposalId,
    required List<String> chainUrls,
    required VoteChainFailover failover,
  }) async {
    final c = coinContext.coin;
    state = state.copyWith(stage: "voting", progress: 0);
    final wireJson = await votingVoteWireJson(
      roundId: roundId,
      bundleIndex: bundleIndex,
      proposalId: proposalId,
      c: c,
    );
    final res = await failover.run(
      baseUrls: chainUrls,
      call: (u) => votechainSubmitVote(
        baseUrl: u,
        submissionJson: wireJson,
        c: c,
      ),
    );
    if (res.statusCode == 422) {
      throw AnyhowException(
        "Vote rejected by the vote chain: ${res.body}",
      );
    }
    final submittedHash = txHashFromVoteChainBody(res.body) ?? "";
    final rejection = parseVoteChainRejection(res.body);
    if (res.statusCode < 200 || res.statusCode >= 300) {
      // A 502 "broadcast outcome unknown" body carries the deterministic
      // tx hash: the tx may have landed — record it and let confirmation
      // polling learn the truth. Any other non-2xx without a hash fails.
      if (submittedHash.isEmpty) {
        throw AnyhowException(
          "Vote chain submit failed (HTTP ${res.statusCode}): ${res.body}",
        );
      }
    } else if (rejection.code > 0 || submittedHash.isEmpty) {
      throw AnyhowException(
        "Vote chain rejected the vote: ${rejection.log}",
      );
    }
    final txHash = submittedHash;

    await votingMarkVoteSubmitted(
      roundId: roundId,
      bundleIndex: bundleIndex,
      proposalId: proposalId,
      txHash: txHash,
      c: c,
    );
    state = state.copyWith(stage: "confirming");
    final conf = await _pollVoteWithRecovery(
      chainUrls: chainUrls,
      failover: failover,
      bundleIndex: bundleIndex,
      proposalId: proposalId,
      txHash: txHash,
    );
    if (state.txHash == null) {
      state = state.copyWith(txHash: txHash, confirmHeight: conf?.height);
    }
  }

  /// Converts UI drafts to the fork's DraftVote JSON for the commit step:
  /// drops skipped choices (validation rejects choice == num_options) and
  /// fills the fields the fork requires with no serde defaults. Returns null
  /// when there is nothing to cast (no drafts or all-skipped ballot).
  String? _sanitizedCommitDrafts(String? draftsJson) {
    if (draftsJson == null || draftsJson.isEmpty) return null;
    final drafts = jsonDecode(draftsJson) as List<dynamic>;
    final commit = <Map<String, dynamic>>[
      for (final d in drafts)
        if ((d as Map<String, dynamic>)['choice'] != d['num_options'])
          {
            'proposal_id': d['proposal_id'],
            'choice': d['choice'],
            'num_options': d['num_options'],
            'vc_tree_position': 0,
            'single_share': false,
          },
    ];
    return commit.isEmpty ? null : jsonEncode(commit);
  }

  /// Plans and submits helper shares for unconfirmed share rows. With no
  /// active vote window or no helper servers configured this is a no-op
  /// (the real inputs arrive with the dynamic config in a later phase).
  /// Returns true when at least one share was submitted and recorded.
  Future<bool> _submitShares({
    required VoteChainFailover failover,
    required int ceremonyStart,
    required int? voteEnd,
    required List<String> shareServerUrls,
    required bool singleShare,
  }) async {
    final c = coinContext.coin;
    state = state.copyWith(stage: "shares");
    var submitted = false;
    final voteEndValue = voteEnd;
    if (voteEndValue == null || shareServerUrls.isEmpty) {
      return false;
    }
    final now = DateTime.now().millisecondsSinceEpoch ~/ 1000;
    // First-pass source: the confirmed votes' share payloads — the share
    // delegation rows only exist after a submission records them, so the
    // old unconfirmed-row pairing could never start (mirrors vizor's
    // commitment-driven share submission).
    final payloads = await votingSharePayloads(roundId: roundId, c: c);
    if (payloads.isEmpty) {
      // All shares already recorded — a resume must not re-send them, but
      // the tracker still needs to run to poll the helpers for
      // confirmations.
      final unconfirmed = await votingShareUnconfirmed(roundId: roundId, c: c);
      if (unconfirmed.isNotEmpty) {
        ref.read(votingShareTrackerProvider.notifier).arm(
              roundId: roundId,
              delaySeconds: 60,
              ceremonyStart: ceremonyStart,
              voteEnd: voteEndValue,
              shareServerUrls: shareServerUrls,
              singleShare: singleShare,
            );
      }
      return false;
    }
    final plans = await votingSharePlans(
      shareCount: payloads.length,
      serverUrls: shareServerUrls,
      now: BigInt.from(now),
      voteEnd: BigInt.from(voteEndValue),
      ceremonyStart: BigInt.from(ceremonyStart),
      singleShare: singleShare,
      c: c,
    );
    for (var i = 0; i < payloads.length && i < plans.length; i++) {
      final payload = payloads[i];
      final plan = plans[i];
      final wireJson = await votingShareWireJson(
        roundId: roundId,
        bundleIndex: payload.bundleIndex,
        proposalId: payload.proposalId,
        shareIndex: payload.shareIndex,
        vcTreePosition: payload.vcTreePosition,
        submitAt: plan.submitAt,
        c: c,
      );
      final body =
          jsonEncode({...jsonDecode(wireJson), "vote_round_id": roundId});
      // The vote itself is already confirmed on-chain; a helper outage must
      // not fail the whole run. Share submits go through the failover
      // service: unreachable helpers rotate to the next target (the helper
      // fleet works as a cluster), and only an all-targets-down outage
      // raises TransientVoteChainException — which the job's auto-retry
      // picks up. Unrecorded shares stay in the payload list and the tracker
      // retries them while the round window is open.
      final res = await failover.run(
        baseUrls: plan.targetServers,
        call: (u) => votechainSubmitShare(
          serverUrl: u,
          payloadJson: body,
          c: c,
        ),
      );
      if (res.statusCode < 200 || res.statusCode >= 300) {
        // A definitive rejection from the first answering helper (e.g. the
        // round window closed): leave the share unrecorded for the tracker.
        debugPrint(
          "Voting: share submit rejected (HTTP ${res.statusCode}) — "
          "the tracker will retry",
        );
        continue;
      }
      await votingShareRecord(
        roundId: roundId,
        bundleIndex: payload.bundleIndex,
        proposalId: payload.proposalId,
        shareIndex: payload.shareIndex,
        sentToUrls: plan.targetServers,
        submitAt: plan.submitAt,
        c: c,
      );
      submitted = true;
    }

    // Background tracking until every share confirms (or the vote window
    // ends). The tracker is session-independent: it survives client restarts
    // via the polls page / splash re-arm hooks.
    final tracker = ref.read(votingShareTrackerProvider.notifier);
    tracker.arm(
      roundId: roundId,
      delaySeconds: voteChainShareTrackingDelay(plans, now),
      ceremonyStart: ceremonyStart,
      voteEnd: voteEndValue,
      shareServerUrls: shareServerUrls,
      singleShare: singleShare,
    );
    return submitted;
  }
}

/// Delay until the next planned share submission (min positive
/// submitAt − now), capped at an hour; 60s when nothing is pending.
int voteChainShareTrackingDelay(List<VotingSharePlanItem> plans, int now) {
  final nowBig = BigInt.from(now);
  var minDelta = BigInt.zero;
  var found = false;
  for (final p in plans) {
    final delta = p.submitAt - nowBig;
    if (delta > BigInt.zero && (!found || delta < minDelta)) {
      minDelta = delta;
      found = true;
    }
  }
  if (!found) return 60;
  return minDelta > BigInt.from(3600) ? 3600 : minDelta.toInt();
}

/// Session-independent helper-share tracking for one round.
///
/// Armed by the submission job, the voting polls page, and the post-wallet-
/// open hook, so a client restart does not strand share delivery while the
/// round window is open. Each tick submits unrecorded share payloads
/// (a helper outage during the foreground run leaves them unrecorded), polls
/// helper status for sent shares, and resubmits overdue ones — best-effort,
/// with the next tick retrying after any failure.
@Riverpod(keepAlive: true)
class VotingShareTracker extends _$VotingShareTracker {
  final Map<String, Timer> _timers = {};

  @override
  bool build() {
    ref.onDispose(() {
      for (final timer in _timers.values) {
        timer.cancel();
      }
      _timers.clear();
    });
    return false; // needsAttention flag
  }

  /// Sets the "share delivery pending" attention flag shown by the voting
  /// page banner.
  void markAttention(bool attention) {
    state = attention;
  }

  /// Arms (or re-arms) the tracking timer for a round.
  void arm({
    required String roundId,
    required int delaySeconds,
    required int ceremonyStart,
    required int? voteEnd,
    required List<String> shareServerUrls,
    required bool singleShare,
  }) {
    _timers[roundId]?.cancel();
    _timers[roundId] = Timer(Duration(seconds: delaySeconds), () async {
      _timers.remove(roundId);
      try {
        final pending = await tick(
          roundId: roundId,
          ceremonyStart: ceremonyStart,
          voteEnd: voteEnd,
          shareServerUrls: shareServerUrls,
          singleShare: singleShare,
        );
        state = pending;
      } on Exception {
        // The next tick retries; share tracking is best-effort.
      }
    });
  }

  /// One share-tracking tick: submit unrecorded payloads, poll helper status
  /// for sent shares, resubmit overdue shares, then re-arm if work remains.
  /// Returns whether work is still pending after the tick.
  Future<bool> tick({
    required String roundId,
    required int ceremonyStart,
    required int? voteEnd,
    required List<String> shareServerUrls,
    required bool singleShare,
  }) async {
    final c = coinContext.coin;
    final now = DateTime.now().millisecondsSinceEpoch ~/ 1000;
    var pending = false;
    // Per-tick failover: rotates across the target helpers and falls back to
    // the rest of the configured fleet when a target is unreachable.
    final failover = VoteChainFailover(allServers: shareServerUrls);

    // Submit shares whose rows were never recorded (all-target helper
    // failure in the foreground run, or a client crash before recording).
    // `votingSharePayloads` excludes recorded shares, so this never re-sends.
    final payloads = await votingSharePayloads(roundId: roundId, c: c);
    if (payloads.isNotEmpty && voteEnd != null) {
      final plans = await votingSharePlans(
        shareCount: payloads.length,
        serverUrls: shareServerUrls,
        now: BigInt.from(now),
        voteEnd: BigInt.from(voteEnd),
        ceremonyStart: BigInt.from(ceremonyStart),
        singleShare: singleShare,
        c: c,
      );
      for (var i = 0; i < payloads.length && i < plans.length; i++) {
        final payload = payloads[i];
        final plan = plans[i];
        final wireJson = await votingShareWireJson(
          roundId: roundId,
          bundleIndex: payload.bundleIndex,
          proposalId: payload.proposalId,
          shareIndex: payload.shareIndex,
          vcTreePosition: payload.vcTreePosition,
          submitAt: plan.submitAt,
          c: c,
        );
        final body =
            jsonEncode({...jsonDecode(wireJson), "vote_round_id": roundId});
        final VotingChainResponse res;
        try {
          // Same cluster semantics as the foreground run: rotate across the
          // target helpers, and mark the tick pending when the whole fleet
          // is unreachable — the next tick retries.
          res = await failover.run(
            baseUrls: plan.targetServers,
            call: (u) => votechainSubmitShare(
              serverUrl: u,
              payloadJson: body,
              c: c,
            ),
          );
        } on TransientVoteChainException {
          pending = true;
          continue;
        }
        if (res.statusCode < 200 || res.statusCode >= 300) {
          pending = true;
          continue;
        }
        await votingShareRecord(
          roundId: roundId,
          bundleIndex: payload.bundleIndex,
          proposalId: payload.proposalId,
          shareIndex: payload.shareIndex,
          sentToUrls: plan.targetServers,
          submitAt: plan.submitAt,
          c: c,
        );
      }
    }

    final plan = await votingSharePlan(
      roundId: roundId,
      now: BigInt.from(now),
      ceremonyStart: BigInt.from(ceremonyStart),
      voteEnd: voteEnd == null ? null : BigInt.from(voteEnd),
      serverUrls: shareServerUrls,
      singleShare: singleShare,
      c: c,
    );
    final unconfirmed = await votingShareUnconfirmed(roundId: roundId, c: c);
    if (unconfirmed.isEmpty && !pending) return false;

    for (final share in unconfirmed) {
      if (share.sentToUrls.isEmpty) continue;
      final shareId = hex.encode(share.nullifier);
      // Try every helper the share was sent to, not just the first: a dead
      // helper must not hide a confirmation another helper can report.
      var confirmed = false;
      for (final server in share.sentToUrls) {
        final res = await votechainShareStatus(
          serverUrl: server,
          roundId: roundId,
          shareId: shareId,
          c: c,
        );
        if (res.statusCode == 200) {
          await votingShareConfirm(
            roundId: roundId,
            bundleIndex: share.bundleIndex,
            proposalId: share.proposalId,
            shareIndex: share.shareIndex,
            c: c,
          );
          confirmed = true;
          break;
        }
      }
      if (!confirmed) pending = true;
    }

    if (plan.summary.overdue > BigInt.zero) {
      final count = min(plan.submissions.length, unconfirmed.length);
      for (var i = 0; i < count; i++) {
        final item = plan.submissions[i];
        final share = unconfirmed[i];
        final wireJson = await votingShareWireJson(
          roundId: roundId,
          bundleIndex: share.bundleIndex,
          proposalId: share.proposalId,
          shareIndex: share.shareIndex,
          vcTreePosition: null,
          submitAt: item.submitAt,
          c: c,
        );
        final body =
            jsonEncode({...jsonDecode(wireJson), "vote_round_id": roundId});
        final VotingChainResponse res;
        try {
          res = await failover.run(
            baseUrls: item.targetServers,
            call: (u) => votechainResubmitShare(
              serverUrl: u,
              payloadJson: body,
              c: c,
            ),
          );
        } on TransientVoteChainException {
          pending = true;
          continue;
        }
        if (res.statusCode < 200 || res.statusCode >= 300) {
          pending = true;
        }
        await votingShareAddServers(
          roundId: roundId,
          bundleIndex: share.bundleIndex,
          proposalId: share.proposalId,
          shareIndex: share.shareIndex,
          newUrls: item.targetServers,
          c: c,
        );
      }
    }

    if (voteEnd != null && plan.nextTrackingDelaySecs != null) {
      arm(
        roundId: roundId,
        delaySeconds: plan.nextTrackingDelaySecs!.toInt(),
        ceremonyStart: ceremonyStart,
        voteEnd: voteEnd,
        shareServerUrls: shareServerUrls,
        singleShare: singleShare,
      );
    }
    // Reflect the confirmations in the plan so the round tile and status
    // screens update ("Resume" -> "View results" once every share confirms).
    await ref.read(votingSessionProvider(roundId).notifier).refresh();
    return pending;
  }
}

/// Re-arms share tracking for every local round whose plan still has share
/// work (submissions or confirmations). Called when the voting page opens and
/// after the wallet unlocks, so a client restart does not strand helper-share
/// delivery while the round window is open.
///
/// Takes a provider-scoped [Ref] (see [ShareTrackingArm]) — never a widget
/// ref, which dies with its page and throws when read after unmount.
Future<void> armShareTrackingForPendingRounds(Ref ref) async {
  try {
    final c = coinContext.coin;
    final Map<String, VotingSessionState> sessions;
    try {
      sessions = await ref.read(votingSessionsAllProvider.future);
    } on Exception {
      return; // wallet/rounds unavailable
    }
    List<String> serverUrls;
    try {
      final config = await ref.read(votingConfigProvider.future);
      serverUrls = config?.voteServers.map((s) => s.url).toList() ?? const [];
    } on Exception {
      serverUrls = const [];
    }
    if (serverUrls.isEmpty) return;

    var anyPending = false;
    for (final entry in sessions.entries) {
      final roundId = entry.key;
      final steps = entry.value.plan?.nextSteps ?? const <VotingNextStep>[];
      final hasShareWork = steps.any(
        (s) => s.kind == "submit_shares" || s.kind == "confirm_share",
      );
      if (!hasShareWork) continue;
      // Resolve the round window from the chain so the share plan can
      // schedule.
      var ceremonyStart = 0;
      int? voteEnd;
      try {
        final res = await votechainRoundStatus(
          baseUrl: serverUrls.first,
          roundId: roundId,
          c: c,
        );
        if (res.statusCode >= 200 && res.statusCode < 300) {
          final body = jsonDecode(res.body) as Map<String, dynamic>;
          final round = body['round'] as Map<String, dynamic>? ?? {};
          final ceremony = round['ceremony_phase_start'];
          final end = round['vote_end_time'];
          if (ceremony is int) ceremonyStart = ceremony;
          if (end is int) voteEnd = end;
        }
      } on Exception {
        // Timing unavailable: arm anyway; the tick resolves it on retry.
      }
      ref.read(votingShareTrackerProvider.notifier).arm(
            roundId: roundId,
            delaySeconds: 60,
            ceremonyStart: ceremonyStart,
            voteEnd: voteEnd,
            shareServerUrls: serverUrls,
            singleShare: false,
          );
      anyPending = true;
    }
    if (anyPending) {
      ref.read(votingShareTrackerProvider.notifier).markAttention(true);
    }
  } on Exception {
    // Best-effort background scan: never surface errors from the re-arm.
  }
}

/// Re-runnable trigger for the share-tracking re-arm scan. Runs the scan on a
/// provider-scoped ref that outlives any page; callers invoke
/// `ref.read(shareTrackingArmProvider.notifier).run()` from initState, which
/// is safe even when the page unmounts before the scan completes.
@Riverpod(keepAlive: true)
class ShareTrackingArm extends _$ShareTrackingArm {
  @override
  void build() {}

  Future<void> run() => armShareTrackingForPendingRounds(ref);
}
