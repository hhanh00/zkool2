import 'dart:async';
import 'dart:math';
import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated.dart';
import 'package:package_info_plus/package_info_plus.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:toastification/toastification.dart';
import 'package:flutter/material.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge.dart';
import 'package:mobx/mobx.dart';
import 'package:zkool/main.dart';
import 'package:zkool/src/rust/account.dart';
import 'package:zkool/src/rust/api/account.dart';
import 'package:zkool/src/rust/api/db.dart';
import 'package:zkool/src/rust/api/init.dart';
import 'package:zkool/src/rust/api/mempool.dart';
import 'package:zkool/src/rust/api/network.dart';
import 'package:zkool/src/rust/api/sync.dart';
import 'package:zkool/utils.dart';

part 'store.g.dart';

class AppStore = AppStoreBase with _$AppStore;

abstract class AppStoreBase with Store {
  bool loaded = false;
  @observable
  Account? selectedAccount;
  @observable
  List<Account> accounts = [];
  @observable
  List<Tx> transactions = [];
  @observable
  List<Memo> memos = [];
  @observable
  List<TxNote> notes = [];
  ObservableMap<int, int> heights = ObservableMap.of({});
  @observable
  int currentHeight = 0;

  String dbName = appName;
  String dbFilepath = "";
  bool isLightNode = true;
  String lwd = "https://zec.rocks";
  String syncInterval = "30"; // in blocks
  String actionsPerSync = "10000";
  bool disclaimerAccepted = false;
  String? versionString;

  ObservableList<String> log = ObservableList.of([]);
  @observable
  bool mempoolRunning = false;
  ObservableMap<int, int> mempoolAccounts = ObservableMap.of({});
  ObservableList<(String, String, int)> mempoolTxIds = ObservableList.of([]);

  FrostParams? frostParams;
  Mempool mempool = Mempool();

  Future<void> init() async {
    final prefs = SharedPreferencesAsync();
    dbName = await prefs.getString("database") ?? appName;
    disclaimerAccepted =
        await prefs.getBool("disclaimer_accepted") ?? disclaimerAccepted;
    final packageInfo = await PackageInfo.fromPlatform();
    final version = packageInfo.version;
    final buildNumber = packageInfo.buildNumber;
    versionString = "$version+$buildNumber";

    final stream = setLogStream();
    stream.listen((m) {
      logger.i(m);
      log.add(m.message);
      if (m.span == "transaction") {
        toastification.show(
            description: Text(m.message),
            margin: EdgeInsets.all(8),
            borderRadius: BorderRadius.circular(8),
            animationDuration: Durations.long1,
            autoCloseDuration: Duration(seconds: 3));
      }
    });
  }

  Future<void> loadSettings() async {
    lwd = await getProp(key: "lwd") ?? lwd;
    final isLightNodeProp = await getProp(key: "is_light_node");
    if (isLightNodeProp != null) {
      isLightNode = isLightNodeProp == "true";
    }
    syncInterval = await getProp(key: "sync_interval") ?? syncInterval;
    actionsPerSync = await getProp(key: "actions_per_sync") ?? actionsPerSync;
  }

  Future<List<Account>> loadAccounts() async {
    final as = await listAccounts();
    accounts = as;
    for (var a in as) {
      heights[a.id] = a.height;
    }
    return as;
  }

  Future<void> loadOtherData() async {
    frostParams = await getAccountFrostParams();
  }

  @action
  Future<void> loadTxHistory() async {
    transactions = await listTxHistory();
  }

  @action
  Future<void> loadMemos() async {
    memos = await listMemos();
  }

  @action
  Future<void> loadNotes() async {
    notes = await listNotes();
  }

  bool syncInProgress = false;
  int retryCount = 0;
  StreamSubscription<SyncProgress>? syncProgressSubscription;
  Timer? retrySyncTimer;

  Future<void> startSynchronize(List<int> accounts, int actionsPerSync) async {
    if (syncInProgress) {
      return;
    }

    final completer = Completer<void>();
    try {
      logger.i("Starting Synchronization");
      showSnackbar("Starting Synchronization");
      syncInProgress = true;
      retrySyncTimer?.cancel();
      retrySyncTimer = null;
      final currentHeight = await getCurrentHeight();
      final progress = synchronize(
          accounts: accounts,
          currentHeight: currentHeight,
          actionsPerSync: actionsPerSync,
          transparentLimit: 10, // scan the last 10 known transparent addresses
          checkpointAge: 200); // trim checkpoints older than 200 blocks
      await syncProgressSubscription?.cancel();
      syncProgressSubscription = progress.listen((p) {
        retryCount = 0;
        runInAction(() {
          for (var a in accounts) {
            heights[a] = p.height; // propagate progress to all account streams
          }
        });
      }, onError: (e) {
        retry(accounts, e);
      }, onDone: () {
        runInAction(() {
          for (var a in accounts) {
            heights[a] = currentHeight;
          }
        });
        syncInProgress = false;
        syncProgressSubscription?.cancel();
        syncProgressSubscription = null;
        Future(loadAccounts);
        showSnackbar("Synchronization Completed");
        logger.i("Synchronization Completed");
        completer.complete();
      });
    } on AnyhowException catch (e) {
      retry(accounts, e);
    }
    return completer.future;
  }

  void retry(List<int> accounts, AnyhowException e) {
    syncInProgress = false;
    retryCount++;
    final maxDelay = pow(2, min(retryCount, 10)).toInt(); // up to 1024s = 17min
    final delay = 30 + Random().nextInt(maxDelay); // randomize delay
    final message =
        "Sync error $e, $retryCount retries, retrying in $delay seconds";
    logger.e(message);
    showSnackbar(message);
    retrySyncTimer?.cancel();
    retrySyncTimer = Timer(Duration(seconds: delay), () async {
      await startSynchronize(
        accounts,
        int.parse(AppStoreBase.instance.actionsPerSync),
      );
      retryCount = 0;
    });
  }

  void autoSync({bool now = false}) async {
    final interval = int.tryParse(syncInterval) ?? 0;

    if (interval <= 0) {
      return;
    }
    try {
      final height = await getCurrentHeight();
      if (now || height > currentHeight) {
        runInAction(() => currentHeight = height);
        await checkSyncNeeded(now: now);
      }
    } on AnyhowException catch (e) {
      logger.i(e);
      // ignore
    } finally {
      if (interval > 0) Timer(Duration(seconds: 15), autoSync);
    }
  }

  Future<void> checkSyncNeeded({required bool now}) async {
    List<int> accountsToSync = [];
    for (var account in accounts) {
      if (account.enabled) {
        final height = heights[account.id] ?? 0;
        if (now || currentHeight - height >= int.parse(syncInterval)) {
          logger.i("Sync needed for ${account.name}");
          accountsToSync.add(account.id);
        }
      }
    }
    if (accountsToSync.isNotEmpty) {
      await startSynchronize(
          accountsToSync, int.parse(AppStoreBase.instance.actionsPerSync));
    }
  }

  static AppStore instance = AppStore();
}

void runMempoolListener() async {
  final mp = AppStoreBase.instance.mempool;
  while (true) {
    try {
      final appStore = AppStoreBase.instance;
      runInAction(() => appStore.mempoolRunning = true);
      appStore.mempoolAccounts.clear();
      appStore.mempoolTxIds.clear();

      final height = await getCurrentHeight();
      final c = Completer();
      mp.run(height: height).listen(
          (msg) {
            if (msg is MempoolMsg_TxId) {
              final txId = msg.field0;
              final amounts = msg.field1
                  .map((a) => "${a.$2} ${zatToString(BigInt.from(a.$3))}")
                  .join(", ");
              final size = msg.field2;
              appStore.mempoolTxIds.add((txId, amounts, size));
              for (var (account, _, amount) in msg.field1) {
                appStore.mempoolAccounts.update(
                  account,
                  (value) => value + amount,
                  ifAbsent: () => amount,
                );
              }
            }
          },
          onDone: c.complete,
          onError: (e) {
            c.complete();
          });
      await c.future; // wait for the stream to complete
      await Future.delayed(Duration(seconds: 5));
    } catch (_) {}
  }
}

void cancelMempoolListener() async {
  final appStore = AppStoreBase.instance;
  await appStore.mempool.cancel();
}
