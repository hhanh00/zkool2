import 'dart:async';
import 'dart:math';
import 'package:collection/collection.dart';
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
import 'package:zkool/src/rust/api/account.dart';
import 'package:zkool/src/rust/api/coin.dart';
import 'package:zkool/src/rust/api/db.dart';
import 'package:zkool/src/rust/api/init.dart';
import 'package:zkool/src/rust/api/mempool.dart';
import 'package:zkool/src/rust/api/network.dart';
import 'package:zkool/src/rust/api/sweep.dart';
import 'package:zkool/src/rust/api/sync.dart';
import 'package:zkool/src/rust/api/vote.dart';
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

@riverpod
class SelectedAccountId extends _$SelectedAccountId {
  @override
  int build() => 0;

  void set(int account) {
    state = account;
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
  final acc = accounts.firstWhere((a) => a.id == accountId);
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

  return AccountData(
    account: account,
    balance: poolBalance,
    pool: pool,
    transactions: transactions,
    memos: memos,
    notes: notes,
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
    final useTor = await prefs.getBool("use_tor") ?? false;
    final getFx = await prefs.getBool("get_fx") ?? false;
    final coingecko = await prefs.getString("coingecko") ?? "";
    final recovery = await prefs.getBool("recovery") ?? false;
    final net = (hasDb ? await getNetworkName(c: c) : null) ?? "mainnet";
    final isLightNode = (hasDb ? await getProp(key: "is_light_node", c: c) : null) ?? "true";
    final lwd = (hasDb ? await getProp(key: "lwd", c: c) : null) ?? "https://zec.rocks";
    final syncInterval = (hasDb ? await getProp(key: "sync_interval", c: c) : null) ?? "30";
    final actionsPerSync = (hasDb ? await getProp(key: "actions_per_sync", c: c) : null) ?? "10000";
    final blockExplorer = (hasDb ? await getProp(key: "block_explorer", c: c) : null) ?? "https://{net}.zcashexplorer.app/transactions/{txid}";
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
    final price = ref.watch(priceProvider.notifier);
    price.setAutoFetchFx(getFx, coingecko);
    final vault = await prefs.getBool("vault") ?? false;
    final expertMode = await prefs.getBool("expert_mode") ?? false;

    return AppSettings(
      dbName: dbName,
      net: net,
      isLightNode: isLightNode == "true",
      lwd: lwd,
      needPin: needPin,
      pinUnlockedAt: DateTime.now(),
      offline: offline,
      useTor: useTor,
      getFx: getFx,
      coingecko: coingecko,
      recovery: recovery,
      syncInterval: syncInterval,
      actionsPerSync: actionsPerSync,
      blockExplorer: blockExplorer,
      qrSettings: qrSettings,
      vault: vault,
      expertMode: expertMode,
    );
  }

  void unlock() {
    state = state.whenData((s) => s.copyWith(
          pinUnlockedAt: DateTime.now(),
        ));
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
  void setAutoFetchFx(bool autoGetFx, String api) async {
    if (autoGetFx) {
      await fetch(api);
      fetchFxTimer = Timer.periodic(Duration(minutes: 1), (_) async {
        await fetch(api);
      });
    } else {
      fetchFxTimer?.cancel();
      fetchFxTimer = null;
    }
  }

  Future<double?> fetch(String api) async {
    try {
      final p = await getCoingeckoPrice(api: api);
      setPrice(p);
      return p;
    } catch (_) {
      return null;
    }
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
    required bool useTor,
    required String coingecko,
    required bool recovery,
    required bool needPin,
    required DateTime pinUnlockedAt,
    required bool offline,
    required bool getFx,
    required QRSettings qrSettings,
    required bool vault,
    required bool expertMode,
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
class CurrentHeightNotifier extends _$CurrentHeightNotifier {
  @override
  int? build() => null;

  bool setHeight(int height) {
    if (state == height) return false;
    state = height;
    return true;
  }
}

@freezed
sealed class ElectionData with _$ElectionData {
  factory ElectionData({
    required ElectionPropsPub? election,
    required String url,
    required int account,
  }) = _ElectionData;
}

@Riverpod(keepAlive: true)
class ElectionNotifier extends _$ElectionNotifier {
  @override
  ElectionData? build() => null;

  Future<void> fetch(int account, String url) async {
    final c = coinContext.coin;
    final election = await fetchElection(account: account, url: url, c: c);
    state = ElectionData(election: election, account: account, url: url);
  }

  Future<ElectionPropsPub?> init() async {
    final c = coinContext.coin;
    final (account, url, election) = await getElection(c: c);
    state = ElectionData(election: election, account: account, url: url);
    return election;
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
  Timer? retrySyncTimer;
  StreamSubscription<SyncProgress>? syncProgressSubscription;
  int retryCount = 0;

  @override
  SyncState build() {
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

  Future<void> startSynchronize(List<Account> accounts) async {
    if (syncInProgress) {
      return;
    }

    final c = coinContext.coin;
    final settings = ref.read(appSettingsProvider).requireValue;
    if (settings.offline) return;

    final completer = Completer<void>();
    try {
      logger.i("Starting Synchronization");
      if (WidgetsBinding.instance.lifecycleState == AppLifecycleState.resumed) showSnackbar("Starting Synchronization");
      syncInProgress = true;
      retrySyncTimer?.cancel();
      retrySyncTimer = null;
      final currentHeight = await getCurrentHeight(c: c);

      begin(accounts, currentHeight);

      final progress = synchronize(
        accounts: accounts.map((a) => a.id).toList(),
        currentHeight: currentHeight,
        actionsPerSync: int.parse(settings.actionsPerSync),
        transparentLimit: 100, // scan the last 100 known transparent addresses
        checkpointAge: 500_000, // a year worth of checkpoints in case we have to rewind for voting
        fast: true,
        c: c,
      ); // trim checkpoints older than 200 blocks
      await syncProgressSubscription?.cancel();
      syncProgressSubscription = progress.listen(
        (p) {
          retryCount = 0;
          update(p.height, p.time);
        },
        onError: (e) {
          retry(accounts, e);
        },
        onDone: () {
          end();
          syncInProgress = false;
          syncProgressSubscription?.cancel();
          syncProgressSubscription = null;
          Timer.run(() async {
            ref.invalidate(getAccountsProvider);
            ref.invalidate(accountProvider);
            if (WidgetsBinding.instance.lifecycleState == AppLifecycleState.resumed) showSnackbar("Synchronization Completed");
            logger.i("Synchronization Completed");
            // Fetch tx details in the background
            unawaited(Future(() async {
              await fetchTxDetails(c: c);
              ref.invalidate(accountProvider);
            }));
            completer.complete();
          });
        },
      );
    } on AnyhowException catch (e) {
      retry(accounts, e);
    }
    return completer.future;
  }

  void retry(List<Account> accounts, AnyhowException e) {
    syncInProgress = false;
    retryCount++;
    final maxDelay = pow(2, min(retryCount, 10)).toInt(); // up to 1024s = 17min
    final delay = 30 + Random().nextInt(maxDelay); // randomize delay
    final message = "Sync error: $e\n\nRetrying in $delay seconds (attempt $retryCount)";
    logger.e(message);

    final context = navigatorKey.currentContext;
    if (context != null) {
      ErrorDialog.show(
        context,
        error: e,
        customMessage: "Sync error (attempt $retryCount of ~10). Retrying in $delay seconds...",
      );
    }

    retrySyncTimer?.cancel();
    retrySyncTimer = Timer(Duration(seconds: delay), () async {
      await startSynchronize(
        accounts,
      );
      retryCount = 0;
    });
  }

  void autoSync({bool now = false}) async {
    final settings = await ref.read(appSettingsProvider.future);
    final interval = int.tryParse(settings.syncInterval) ?? 0;

    if (settings.offline || interval <= 0) {
      return;
    }
    try {
      final c = coinContext.coin;
      final currentHeight = await getCurrentHeight(c: c);
      final h = ref.read(currentHeightProvider.notifier);
      if (h.setHeight(currentHeight)) {
        await checkSyncNeeded(currentHeight, now: now);
      }
    } on AnyhowException catch (e) {
      logger.e(e);
      // ignore
    } finally {
      if (interval > 0) Timer(Duration(seconds: 15), () => autoSync());
    }
  }

  Future<void> checkSyncNeeded(int currentHeight, {required bool now}) async {
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
      await startSynchronize(
        accountsToSync,
      );
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
  final accountId = basicData.currentAccount?.account.id ?? 0;
  final syncState = await ref.watch(syncStateAccountProvider(accountId).future);

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
}
