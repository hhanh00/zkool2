import 'dart:async';
import 'dart:math';
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
  String lwd = "https://zec.rocks";
  String syncInterval = "30"; // in blocks
  String actionsPerSync = "10000";
  bool disclaimerAccepted = false;
  String? versionString;

  ObservableList<String> log = ObservableList.of([]);

  FrostParams? frostParams;

  Future<void> init() async {
    final prefs = SharedPreferencesAsync();
    dbName = await prefs.getString("database") ?? appName;
    disclaimerAccepted = await prefs.getBool("disclaimer_accepted") ?? disclaimerAccepted;
    final packageInfo = await PackageInfo.fromPlatform();
    final version = packageInfo.version;
    final buildNumber = packageInfo.buildNumber;
    versionString = "$version+$buildNumber";

    final stream = setLogStream();
    stream.listen((m) {
      logger.i(m);
      log.add(m.message);
      if (m.span == "transaction_plan" ||
          m.span == "sign_transaction" ||
          m.span == "extract_transaction") {
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
      showSnackbar("Starting Synchronization");
      syncInProgress = true;
      retrySyncTimer?.cancel();
      retrySyncTimer = null;
      final currentHeight = await getCurrentHeight();
      logger.i(accounts);
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
          accounts, int.parse(AppStoreBase.instance.actionsPerSync),
      );
      retryCount = 0;
    });
  }

  Timer? autosyncTimer;

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
