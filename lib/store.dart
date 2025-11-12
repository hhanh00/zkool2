import 'dart:async';
import 'dart:math';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:gap/gap.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:toastification/toastification.dart';
import 'package:flutter/material.dart';
import 'package:zkool/main.dart';
import 'package:zkool/src/rust/account.dart';
import 'package:zkool/src/rust/api/account.dart';
import 'package:zkool/src/rust/api/db.dart';
import 'package:zkool/src/rust/api/init.dart';
import 'package:zkool/src/rust/api/mempool.dart';
import 'package:zkool/src/rust/api/network.dart';
import 'package:zkool/src/rust/api/sweep.dart';
import 'package:zkool/src/rust/api/sync.dart';
import 'package:zkool/utils.dart';

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
  final Widget Function(BuildContext context, SyncProgressAccount status, TextStyle? style) builder;
  const ProgressWidget(
    this.account, {
    super.key,
    this.width,
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
    final style = old ? TextStyle(color: Colors.red) : null;

    return SizedBox(
      width: width,
      height: 80,
      child: Stack(
        children: [
          if (ss.start != ss.end)
            SizedBox.expand(
              child: LinearProgressIndicator(
                color: t.colorScheme.primary.withAlpha(128),
                value: ss.progress(),
              ),
            ),
          Center(child: builder(context, ss, style)),
        ],
      ),
    );
  }
}

class SmallProgressWidget extends StatelessWidget {
  final Account account;
  const SmallProgressWidget(this.account, {super.key});
  @override
  Widget build(BuildContext context) => ProgressWidget(account, width: 80, builder: (context, status, style) => Text("${status.height}", style: style));
}

class HeroProgressWidget extends StatelessWidget {
  final Account account;
  const HeroProgressWidget(this.account, {super.key});

  @override
  Widget build(BuildContext context) => ProgressWidget(
        account,
        builder: (context, status, style) {
          final t = Theme.of(context).textTheme;
          return Column(
            mainAxisAlignment: MainAxisAlignment.center,
            children: [
              Text.rich(
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
              ),
              Gap(8),
              Text(timeToString(status.time), style: t.bodySmall),
            ],
          );
        },
      );
}

// AppStore get appStore => AppStoreBase.instance;

@Riverpod(keepAlive: true)
class SelectedAccount extends _$SelectedAccount {
  @override
  Future<Account?> build() async {
    final accounts = await ref.read(getAccountsProvider.future);
    final s = await getProp(key: "selected_account");
    if (s == null || s == "") return null;
    final id = int.parse(s);
    return accounts.firstWhere((a) => a.id == id);
  }

  void selectAccount(Account account) async {
    await putProp(key: "selected_account", value: "${account.id}");
    state = AsyncData(account);
  }

  void unselect() {
    state = AsyncData(null);
  }
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

@riverpod
Future<List<Account>> getAccounts(Ref ref) async {
  return await listAccounts();
}

@riverpod
Future<List<Folder>> getFolders(Ref ref) async {
  return await listFolders();
}

@riverpod
Future<List<Category>> getCategories(Ref ref) async {
  return await listCategories();
}

@riverpod
Future<AccountData> account(Ref ref, int id) async {
  final accounts = await ref.watch(getAccountsProvider.future);
  final account = accounts.firstWhere((a) => a.id == id);
  final poolBalance = await balance();
  final pool = await getAccountPools(account: id);
  final frostParams = await getAccountFrostParams();
  final transactions = await listTxHistory();
  final memos = await listMemos();
  final notes = await listNotes();

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
    final hasDb = ref.watch(hasDbProvider);
    final prefs = SharedPreferencesAsync();
    String dbName = await prefs.getString("database") ?? appName;
    final disclaimerAccepted = await prefs.getBool("disclaimer_accepted") ?? false;
    final needPin = await prefs.getBool("pin_lock") ?? false;
    final offline = await prefs.getBool("offline") ?? false;
    final useTor = await prefs.getBool("use_tor") ?? false;
    final recovery = await prefs.getBool("recovery") ?? false;

    final net = hasDb ? await getNetworkName() : "mainnet";
    final isLightNode = hasDb ? await getProp(key: "is_light_node") : "true";
    final lwd = hasDb ? await getProp(key: "lwd") : "https://zec.rocks";
    final syncInterval = hasDb ? await getProp(key: "sync_interval") : "30";
    final actionsPerSync = hasDb ? await getProp(key: "actions_per_sync") : "10000";
    final blockExplorer = hasDb ? await getProp(key: "block_explorer") : "https://{net}.zcashexplorer.app/transactions/{txid}";
    return AppSettings(
      dbName: dbName,
      net: net,
      isLightNode: isLightNode! == "true",
      lwd: lwd!,
      disclaimerAccepted: disclaimerAccepted,
      needPin: needPin,
      pinUnlockedAt: DateTime.now(),
      offline: offline,
      useTor: useTor,
      recovery: recovery,
      syncInterval: syncInterval!,
      actionsPerSync: actionsPerSync!,
      blockExplorer: blockExplorer!,
    );
  }

  void unlock() {
    state = state.whenData((s) => s.copyWith(
          pinUnlockedAt: DateTime.now(),
        ));
  }
}

@riverpod
class PriceNotifier extends _$PriceNotifier {
  @override
  double? build() => null;

  void setPrice(double price) {
    state = price;
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
    required bool disclaimerAccepted,
    required bool useTor,
    required bool recovery,
    required bool needPin,
    required DateTime pinUnlockedAt,
    required bool offline,
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

@riverpod
class CurrentHeightNotifier extends _$CurrentHeightNotifier {
  @override
  int? build() => null;

  bool setHeight(int height) {
    if (state == height) return false;
    state = height;
    return true;
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
    final settings = await ref.read(appSettingsProvider.future);
    if (settings.offline) return;

    while (true) {
      try {
        if (settings.offline) return;
        state = MempoolState(running: true, unconfirmedFunds: {}, unconfirmedTx: []);

        final height = await getCurrentHeight();
        final c = Completer();
        mempool.run(height: height).listen(
              (msg) {
                if (msg is MempoolMsg_TxId) {
                  final txId = msg.field0; // txid hash
                  final amounts = msg.field1; // list of (account id, name, value unconfirmed)
                  final size = msg.field2; // size in bytes of the tx
                  addTx(txId, amounts, size);
                }
              },
              onDone: c.complete,
              onError: (e) {
                c.complete();
              },
            );
        await c.future; // wait for the stream to complete
        await Future.delayed(Duration(seconds: 5));
      } catch (_) {}
    }
  }

  void addTx(String txId, List<(int, String, int)> unconfirmedValues, int size) {
    final unconfirmed = unconfirmedValues.map((a) => "${a.$2} ${zatToString(BigInt.from(a.$3))}").join(", ");
    final unconfirmedTx = state.unconfirmedTx;
    unconfirmedTx.add((txId, unconfirmed, size));

    final unconfirmedFunds = state.unconfirmedFunds;
    for (var (account, _, amount) in unconfirmedValues) {
      unconfirmedFunds.update(
        account,
        (value) => value + amount,
        ifAbsent: () => amount,
      );
    }
    state = state.copyWith(unconfirmedTx: unconfirmedTx, unconfirmedFunds: unconfirmedFunds);
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

    final settings = ref.read(appSettingsProvider).requireValue;
    if (settings.offline) return;

    final completer = Completer<void>();
    try {
      logger.i("Starting Synchronization");
      showSnackbar("Starting Synchronization");
      syncInProgress = true;
      retrySyncTimer?.cancel();
      retrySyncTimer = null;
      final currentHeight = await getCurrentHeight();

      begin(accounts, currentHeight);

      final progress = synchronize(
        accounts: accounts.map((a) => a.id).toList(),
        currentHeight: currentHeight,
        actionsPerSync: int.parse(settings.actionsPerSync),
        transparentLimit: 100, // scan the last 100 known transparent addresses
        checkpointAge: 200,
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
            showSnackbar("Synchronization Completed");
            logger.i("Synchronization Completed");
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
    final message = "Sync error $e, $retryCount retries, retrying in $delay seconds";
    logger.e(message);
    showSnackbar(message);
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
      final currentHeight = await getCurrentHeight();
      final h = ref.read(currentHeightProvider.notifier);
      if (h.setHeight(currentHeight)) {
        await checkSyncNeeded(currentHeight, now: now);
      }
    } on AnyhowException catch (e) {
      logger.i(e);
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
    final sc = await TransparentScanner.newInstance();
    scanner = sc;
    final endHeight = await getCurrentHeight();
    final sub = sc.run(endHeight: endHeight, gapLimit: gapLimit);
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
    return await getTxDetails(idTx: id);
  }
}
