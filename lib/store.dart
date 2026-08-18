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
import 'package:zkool/src/rust/api/voting_workflow.dart';
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

  Future<void> autoSync({bool now = false}) async {
    final settings = await ref.read(appSettingsProvider.future);
    final interval = int.tryParse(settings.syncInterval) ?? 0;

    if (settings.offline || interval <= 0) {
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
    required String stage, // idle|preparing|proving|submitting|confirming|voting|shares|done|error
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
  }) = _VotingSubmissionJobState;
}

/// Durable-workflow monitor for one round. All orchestration lives in the
/// Rust `vote_round` workflow (durare, see docs/voting-workflow.md); this
/// monitor only starts it, polls its status every second, and offers retry
/// on terminal error. The workflow survives app restarts and resumes from its
/// checkpoints; re-entering this screen attaches to the existing run
/// (votingWorkflowStart is idempotent).
@Riverpod(keepAlive: true)
class VotingSubmissionMonitor extends _$VotingSubmissionMonitor {
  Timer? _statusTimer;

  @override
  VotingSubmissionJobState build(String roundId) {
    ref.onDispose(() => _statusTimer?.cancel());
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
      return; // already running
    }
    state = state.copyWith(stage: "running", progress: 0, error: null);
    ref.read(votingSubmissionGuardProvider.notifier).setActive(true);
    try {
      final c = coinContext.coin;
      // The vote chain servers double as helper (share) servers; the voting
      // flow never passes shareServerUrls, so fall back to the configured
      // vote servers (mirrors vizor's context.config.voteServers).
      final shareUrls = await _effectiveShareServerUrls(shareServerUrls);
      // The voting pages never pass a lightwalletd URL — use the app's
      // configured one for the fresh prepare (the workflow's prepare step
      // needs it to fetch the snapshot anchor tree state).
      final lwdUrl = (lightwalletdUrl == null || lightwalletdUrl.isEmpty)
          ? await _appLwdUrl()
          : lightwalletdUrl;
      await votingWorkflowStart(
        input: VoteRoundInput(
          roundId: roundId,
          coin: c.coin,
          account: c.account,
          dbFilepath: c.dbFilepath,
          url: c.url,
          serverType: c.serverType,
          transport: c.transport,
          proxy: c.proxy,
          chainUrl: chainUrl,
          pirServerUrl: pirServerUrl,
          pirLayout: pirLayout,
          roundParamsJson: roundParamsJson,
          roundName: roundName,
          maxRealNotesPerBundle: maxRealNotesPerBundle,
          lightwalletdUrl: lwdUrl,
          voteNodeUrl: voteNodeUrl,
          ceremonyStart: BigInt.from(ceremonyStart),
          voteEnd: voteEnd == null ? null : BigInt.from(voteEnd),
          shareServerUrls: shareUrls,
          singleShare: singleShare,
        ),
        c: c,
      );
      _startPolling();
    } on Exception catch (e) {
      state = state.copyWith(stage: "error", error: e.toString());
      ref.read(votingSubmissionGuardProvider.notifier).setActive(false);
    }
  }

  /// Re-runs a terminal-error workflow from its last checkpoint via the
  /// durable retry (the fork's artifacts persist; steps skip completed work).
  Future<void> retry() async {
    state = state.copyWith(stage: "running", progress: 0, error: null);
    ref.read(votingSubmissionGuardProvider.notifier).setActive(true);
    try {
      await votingWorkflowRetry(roundId: roundId, c: coinContext.coin);
      _startPolling();
    } on Exception catch (e) {
      state = state.copyWith(stage: "error", error: e.toString());
      ref.read(votingSubmissionGuardProvider.notifier).setActive(false);
    }
  }

  void reset() {
    _statusTimer?.cancel();
    state = VotingSubmissionJobState(stage: "idle", progress: 0);
  }

  void _startPolling() {
    _statusTimer?.cancel();
    _statusTimer = Timer.periodic(const Duration(seconds: 1), (_) async {
      try {
        await _poll();
      } on Exception catch (e) {
        // Transient poll failure; the next tick retries.
        debugPrint("Voting: workflow status poll failed: $e");
      }
    });
    Future(_poll);
  }

  Future<void> _poll() async {
    final status =
        await votingWorkflowStatus(roundId: roundId, c: coinContext.coin);
    if (status == null) return; // not started yet; keep polling
    switch (status.status) {
      case "success":
        _statusTimer?.cancel();
        state = state.copyWith(
          stage: "done",
          progress: 1,
          doneLabel: status.doneLabel,
          txHash: status.txHash,
          confirmHeight: status.confirmHeight?.toInt(),
          eligibleWeightZatoshi: status.eligibleWeightZatoshi,
        );
        ref.read(votingSubmissionGuardProvider.notifier).setActive(false);
        // The round's plan advanced; refresh the voting page so the tile
        // leaves "Join" and shows the real status without a manual refresh.
        ref.invalidate(votingRoundListProvider);
        ref.invalidate(votingSessionProvider(roundId));
      case "error":
        _statusTimer?.cancel();
        state = state.copyWith(
          stage: "error",
          error: status.error ?? "Voting workflow failed",
        );
        ref.read(votingSubmissionGuardProvider.notifier).setActive(false);
      default:
        state = state.copyWith(
          stage: status.stage,
          progress: status.progress ?? state.progress,
          txHash: status.txHash,
          confirmHeight: status.confirmHeight?.toInt(),
          eligibleWeightZatoshi: status.eligibleWeightZatoshi,
          doneLabel: status.doneLabel,
        );
    }
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
}