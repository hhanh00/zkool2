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

AppStore get appStore => AppStoreBase.instance;

class ObservableHeight = ObservableHeightBase with _$ObservableHeight;

abstract class ObservableHeightBase with Store {
  @observable
  int height = 0;
  @observable
  int start = 0;
  @observable
  int range = 0;

  @computed
  double get progress => range > 0 ? (height - start) / range : 0.0;

  @action
  void init(int h) {
    height = h;
    start = h;
    range = 0;
  }

  @action
  void begin(int endHeight) {
    range = endHeight - start;
  }

  @action
  void set(int h) {
    height = h;
  }

  @action
  void done(int endHeight) {
    height = endHeight;
    range = 0;
  }

  Widget build(BuildContext context) {
    return ProgressWidget(
        height: this, width: 80, child: Text(height.toString()));
  }

  Widget buildHero(BuildContext context) {
    final t = Theme.of(context).textTheme;
    final currentHeight = appStore.currentHeight;
    return ProgressWidget(
        height: this,
        child: Text.rich(TextSpan(children: [
          TextSpan(text: "$height", style: t.bodyLarge),
          if (currentHeight - height > 0)
            TextSpan(
                text: " tip-${currentHeight - height}", style: t.labelSmall)
        ])));
  }
}

class ProgressWidget extends StatelessWidget {
  final ObservableHeightBase height;
  final double? width;
  final Widget child;
  const ProgressWidget(
      {super.key, required this.height, this.width, required this.child});

  @override
  Widget build(BuildContext context) {
    final t = Theme.of(context);
    final p = height.progress;

    return SizedBox(
        width: width,
        height: 40,
        child: Stack(children: [
          if (height.range != 0)
            SizedBox.expand(
                child: LinearProgressIndicator(
                    color: t.colorScheme.primary.withAlpha(128), value: p)),
          Center(child: child)
        ]));
  }
}

class AppStore = AppStoreBase with _$AppStore;

abstract class AppStoreBase with Store {
  bool loaded = false;
  @observable
  Account? selectedAccount;
  @observable
  List<Account> accounts = [];
  @observable
  int pools = 7;
  @observable
  int seqno = 0;
  @observable
  PoolBalance? poolBalance;
  @observable
  List<Tx> transactions = [];
  @observable
  List<Memo> memos = [];
  @observable
  List<TxNote> notes = [];
  Map<int, ObservableHeight> heights = {};
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
  bool pinLock = true;

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

  // Only settings from SharedPreferences
  // This is called before getting the database
  Future<void> loadAppSettings() async {
    final prefs = SharedPreferencesAsync();
    isLightNode = await prefs.getBool("is_light_node") ?? isLightNode;
    pinLock = await prefs.getBool("pin_lock") ?? pinLock;
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
      final h = ObservableHeight();
      h.init(a.height);
      heights[a.id] = h;
    }
    return as;
  }

  Future<void> loadOtherData() async {
    poolBalance = await balance();
    pools = await getAccountPools(account: selectedAccount!.id);
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

  @action
  Future<void> refresh() async {
    logger.i("Refresh account data");
    await appStore.loadAccounts();
    if (selectedAccount != null) {
      await appStore.loadTxHistory();
      await appStore.loadMemos();
      await appStore.loadNotes();
      await appStore.loadOtherData();
    }
    incSeqno();
  }

  @action
  void incSeqno() {
    seqno += 1;
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

      for (var a in accounts) {
        heights[a]!.begin(currentHeight);
      }

      final progress = synchronize(
          accounts: accounts,
          currentHeight: currentHeight,
          actionsPerSync: actionsPerSync,
          transparentLimit:
              100, // scan the last 100 known transparent addresses
          checkpointAge: 200); // trim checkpoints older than 200 blocks
      await syncProgressSubscription?.cancel();
      syncProgressSubscription = progress.listen((p) {
        retryCount = 0;
        runInAction(() {
          for (var a in accounts) {
            if (p.height > heights[a]!.height)
              heights[a]!
                  .set(p.height); // propagate progress to all account streams
          }
        });
      }, onError: (e) {
        retry(accounts, e);
      }, onDone: () {
        runInAction(() {
          for (var a in accounts) {
            heights[a]?.done(currentHeight);
          }
        });
        syncInProgress = false;
        syncProgressSubscription?.cancel();
        syncProgressSubscription = null;
        Timer.run(() async {
          await refresh();
          showSnackbar("Synchronization Completed");
          logger.i("Synchronization Completed");
          completer.complete();
        });
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
        int.parse(appStore.actionsPerSync),
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
        final height = heights[account.id]?.height ?? 0;
        if (now || currentHeight - height >= int.parse(syncInterval)) {
          logger.i("Sync needed for ${account.name}");
          accountsToSync.add(account.id);
        }
      }
    }
    if (accountsToSync.isNotEmpty) {
      await startSynchronize(
          accountsToSync, int.parse(appStore.actionsPerSync));
    }
  }

  static AppStore instance = AppStore();
}

void runMempoolListener() async {
  final mp = appStore.mempool;
  while (true) {
    try {
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
  await appStore.mempool.cancel();
}

Future<void> selectAccount(Account? account) async {
  if (account != null) {
    await setAccount(account: account.id);
    await putProp(key: "selected_account", value: account.id.toString());
    appStore.selectedAccount = account;
  } else {
    await putProp(key: "selected_account", value: "");
  }
}

Future<int?> getSelectedAccount() async {
  final s = await getProp(key: "selected_account");
  if (s == null || s == "") return null;
  return int.parse(s);
}
