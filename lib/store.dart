import 'dart:async';
import 'dart:math';
import 'package:collection/collection.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:gap/gap.dart';
import 'package:package_info_plus/package_info_plus.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:toastification/toastification.dart';
import 'package:flutter/material.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge.dart';
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
  SyncProgressAccount build(int accountId) {
    final accounts = ref.read(getAccountsProvider).requireValue;
    final account = accounts.firstWhere((a) => a.id == accountId);
    final ss = ref.read(synchronizerProvider);
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
    state = state.copyWith(height: height, time: time);
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
    final ss = ref.watch(syncStateAccountProvider(account.id));
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
  Widget build(BuildContext context) => ProgressWidget(account, builder: (context, status, style) => Text("${status.height}", style: style));
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

@riverpod
class SelectedAccount extends _$SelectedAccount {
  @override
  Account? build() {
    return null;
  }

  void selectAccount(Account account) {
    state = account;
  }

  void unselect() {
    state = null;
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
  return <Account>[];
}

@riverpod
Future<List<Folder>> getFolders(Ref ref) async {
  return <Folder>[];
}

@riverpod
Future<List<Category>> getCategories(Ref ref) async {
  return <Category>[];
}

@riverpod
Future<AccountData> account(Ref ref, int id) async {
  return AccountData(
    account: Account(
      coin: 0,
      id: 0,
      name: '',
      aindex: 0,
      birth: 0,
      folder: Folder(id: 0, name: ''),
      position: 1,
      hidden: true,
      saved: true,
      enabled: true,
      internal: false,
      hw: 0,
      height: 1,
      time: 0,
      balance: BigInt.zero,
    ),
    balance: PoolBalance(field0: Uint64List.fromList([0, 0, 0])),
    pool: 7,
    transactions: [],
    memos: [],
    notes: [],
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

@riverpod
class AppSettingsNotifier extends _$AppSettingsNotifier {
  @override
  AppSettings build() {
    return AppSettings(needPin: false, pinUnlockedAt: DateTime.now(), offline: false);
  }

  void acceptDisclaimer() {
    state = state.copyWith(disclaimerAccepted: true);
  }

  void unlock() {
    state = state.copyWith(
      pinUnlockedAt: DateTime.now(),
    );
  }

  void setNeedPin(bool needPin) {
    state = state.copyWith(needPin: needPin);
  }

  void setOffline(bool offline) {
    state = state.copyWith(offline: offline);
  }

  void setDbName(String dbName) {
    state = state.copyWith(dbName: dbName);
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

// return ProgressWidget(
//   status: status,
//   width: 80,
//   child: Text(status.height.toString(), style: style),
// );

// class AppStore = AppStoreBase with _$AppStore;

// abstract class AppStoreBase with Store {
//   bool loaded = false;
//   String net = "mainnet";
//   @observable
//   Account? selectedAccount;
//   @observable
//   Folder? selectedFolder;
//   @observable
//   List<Account> accounts = [];
//   @observable
//   List<Folder> folders = [];
//   @observable
//   List<Category> categories = [];
//   @observable
//   int pools = 7;
//   @observable
//   int seqno = 0;
//   @observable
//   PoolBalance poolBalance = PoolBalance(field0: Uint64List.fromList([0, 0, 0]));
//   @observable
//   List<Tx> transactions = [];
//   @observable
//   List<Memo> memos = [];
//   @observable
//   List<TxNote> notes = [];
//   Map<int, ObservableHeight> heights = {};
//   @observable
//   int currentHeight = 0;

@freezed
sealed class AppSettings with _$AppSettings {
  factory AppSettings({
    @Default(appName) String dbName,
    @Default("") String dbFilepath,
    @Default("mainnet") String net,
    @Default(true) bool isLightNode,
    @Default("https://zec.rocks") String lwd,
    @Default("https://{net}.zcashexplorer.app/transactions/{txid}") String blockExplorer,
    @Default("30") String syncInterval, // in blocks
    @Default("10000") String actionsPerSync,
    @Default(false) bool disclaimerAccepted,
    String? versionString,
    @Default(false) bool useTor,
    @Default(false) bool recovery,
    required bool needPin,
    required DateTime pinUnlockedAt,
    required bool offline,
  }) = _AppSettings;

  // TransparentScannerStore transparentScanner = TransparentScannerStore();

  // ObservableList<String> log = ObservableList.of([]);
  // @observable
  // bool mempoolRunning = false;
  // ObservableMap<int, int> mempoolAccounts = ObservableMap.of({});
  // ObservableList<(String, String, int)> mempoolTxIds = ObservableList.of([]);

  // FrostParams? frostParams;
}

@riverpod
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
    required Map<int, int> unconfirmedFunds,
    required List<(String, String, int)> unconfirmedTx,
  }) = _MempoolState;
}

@riverpod
class MempoolNotifier extends _$MempoolNotifier {
  @override
  MempoolState build() {
    return MempoolState(unconfirmedFunds: {}, unconfirmedTx: []);
  }
}

// Need a mempool provider to inform accounts
// that their balance may have changed due to
// new txs in the mempool

//   Future<void> init() async {
//     final prefs = SharedPreferencesAsync();
//     dbName = await prefs.getString("database") ?? appName;
//     disclaimerAccepted = await prefs.getBool("disclaimer_accepted") ?? disclaimerAccepted;
//     final packageInfo = await PackageInfo.fromPlatform();
//     final version = packageInfo.version;
//     final buildNumber = packageInfo.buildNumber;
//     versionString = "$version+$buildNumber";

//     final stream = setLogStream();
//     stream.listen((m) {
//       log.add(m.message);
//       if (m.span == "transaction") {
//         toastification.show(
//           description: Text(m.message),
//           margin: EdgeInsets.all(8),
//           borderRadius: BorderRadius.circular(8),
//           animationDuration: Durations.long1,
//           autoCloseDuration: Duration(seconds: 3),
//         );
//       }
//     });
//   }

//   // Only settings from SharedPreferences
//   // This is called before getting the database
//   Future<void> loadAppSettings() async {
//     final prefs = SharedPreferencesAsync();
//     isLightNode = await prefs.getBool("is_light_node") ?? isLightNode;
//     needPin = await prefs.getBool("pin_lock") ?? needPin;
//     offline = await prefs.getBool("offline") ?? offline;
//     useTor = await prefs.getBool("use_tor") ?? useTor;
//     recovery = await prefs.getBool("recovery") ?? recovery;
//   }

//   Future<void> loadSettings() async {
//     net = await getNetworkName();
//     lwd = await getProp(key: "lwd") ?? lwd;
//     syncInterval = await getProp(key: "sync_interval") ?? syncInterval;
//     actionsPerSync = await getProp(key: "actions_per_sync") ?? actionsPerSync;
//     blockExplorer = await getProp(key: "block_explorer") ?? blockExplorer;
//   }

//   Future<List<Account>> loadAccounts() async {
//     final as = await listAccounts();
//     accounts = as;
//     for (var a in as) {
//       final h = heights.putIfAbsent(a.id, () => ObservableHeight());
//       h.set(a.height, a.time);
//     }
//     return as;
//   }

//   Future<void> loadOtherData() async {
//     poolBalance = await balance();
//     pools = await getAccountPools(account: selectedAccount!.id);
//     frostParams = await getAccountFrostParams();
//   }

//   @action
//   Future<void> loadTxHistory() async {
//     transactions = await listTxHistory();
//   }

//   @action
//   Future<void> loadMemos() async {
//     memos = await listMemos();
//   }

//   @action
//   Future<void> loadNotes() async {
//     notes = await listNotes();
//   }

//   @action
//   Future<void> refresh() async {
//     await loadAccounts();
//     await loadFolders();
//     await loadCategories();
//     if (selectedAccount != null) {
//       await loadTxHistory();
//       await loadMemos();
//       await loadNotes();
//       await loadOtherData();
//     }
//     incSeqno();
//   }

//   @action
//   void incSeqno() {
//     seqno += 1;
//   }

//   @action
//   Future<void> loadFolders() async {
//     folders = await listFolders();
//   }

//   @action
//   Future<void> loadCategories() async {
//     categories = await listCategories();
//   }

//   bool syncInProgress = false;

@riverpod
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

  Future<void> startSynchronize(WidgetRef ref, List<Account> accounts) async {
    if (syncInProgress) {
      return;
    }

    final settings = ref.read(appSettingsProvider);
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
          retry(ref, accounts, e);
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
      retry(ref, accounts, e);
    }
    return completer.future;
  }

  void retry(WidgetRef ref, List<Account> accounts, AnyhowException e) {
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
        ref,
        accounts,
      );
      retryCount = 0;
    });
  }

  void autoSync(WidgetRef ref, {bool now = false}) async {
    final settings = ref.read(appSettingsProvider);
    final interval = int.tryParse(settings.syncInterval) ?? 0;

    if (settings.offline || interval <= 0) {
      return;
    }
    try {
      final currentHeight = await getCurrentHeight();
      final h = ref.read(currentHeightProvider.notifier);
      if (h.setHeight(currentHeight)) {
        await checkSyncNeeded(ref, currentHeight, now: now);
      }
    } on AnyhowException catch (e) {
      logger.i(e);
      // ignore
    } finally {
      if (interval > 0) Timer(Duration(seconds: 15), () => autoSync(ref));
    }
  }

  Future<void> checkSyncNeeded(WidgetRef ref, int currentHeight, {required bool now}) async {
    final settings = ref.read(appSettingsProvider);
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
        ref,
        accountsToSync,
      );
    }
  }
}

//   bool checkOffline() {
//     if (offline) {
//       showSnackbar("Zkool is in Offline mode");
//       return true;
//     }
//     return false;
//   }

//   static AppStore instance = AppStore();
// }

// void runMempoolListener() async {
//   final mp = appStore.mempool;
//   while (true) {
//     if (appStore.offline) return;
//     try {
//       runInAction(() => appStore.mempoolRunning = true);
//       appStore.mempoolAccounts.clear();
//       appStore.mempoolTxIds.clear();

//       final height = await getCurrentHeight();
//       final c = Completer();
//       mp.run(height: height).listen(
//             (msg) {
//               if (msg is MempoolMsg_TxId) {
//                 final txId = msg.field0;
//                 final amounts = msg.field1.map((a) => "${a.$2} ${zatToString(BigInt.from(a.$3))}").join(", ");
//                 final size = msg.field2;
//                 appStore.mempoolTxIds.add((txId, amounts, size));
//                 for (var (account, _, amount) in msg.field1) {
//                   appStore.mempoolAccounts.update(
//                     account,
//                     (value) => value + amount,
//                     ifAbsent: () => amount,
//                   );
//                 }
//               }
//             },
//             onDone: c.complete,
//             onError: (e) {
//               c.complete();
//             },
//           );
//       await c.future; // wait for the stream to complete
//       await Future.delayed(Duration(seconds: 5));
//     } catch (_) {}
//   }
// }

// void cancelMempoolListener() async {
//   await appStore.mempool.cancel();
// }

// Future<void> selectAccount(Account? account) async {
//   if (account != null) {
//     await setAccount(account: account.id);
//     await putProp(key: "selected_account", value: account.id.toString());
//     appStore.selectedAccount = account;
//   } else {
//     await putProp(key: "selected_account", value: "");
//   }
// }

// Future<int?> getSelectedAccount() async {
//   final s = await getProp(key: "selected_account");
//   if (s == null || s == "") return null;
//   return int.parse(s);
// }

// class TransparentScannerStore = _TransparentScannerStore with _$TransparentScannerStore;

@riverpod
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
