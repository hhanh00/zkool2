import 'dart:async';
import 'dart:math';

import 'package:toastification/toastification.dart';
import 'package:flutter/material.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge.dart';
import 'package:mobx/mobx.dart';
import 'package:zkool/main.dart';
import 'package:zkool/router.dart';
import 'package:zkool/src/rust/api/account.dart';
import 'package:zkool/src/rust/api/db.dart';
import 'package:zkool/src/rust/api/init.dart';
import 'package:zkool/src/rust/api/network.dart';
import 'package:zkool/src/rust/api/sync.dart';

part 'store.g.dart';

class AppStore = AppStoreBase with _$AppStore;

abstract class AppStoreBase with Store {
  bool loaded = false;
  @observable
  String accountName = "";
  @observable
  List<Account> accounts = [];
  @observable
  List<Tx> transactions = [];
  @observable
  List<Memo> memos = [];
  ObservableMap<int, int> heights = ObservableMap.of({});

  String dbName = "zkool";
  String lwd = "https://zec.rocks";

  ObservableList<String> log = ObservableList.of([]);

  void init()  {
    final stream = setLogStream();
    stream.listen((m) {
      logger.i(m);
      log.add(m.message);
      if (m.span == "transaction_plan") {
        toastification.show(
          description: Text(m.message),
          margin: EdgeInsets.all(8),
          borderRadius: BorderRadius.circular(8),
          animationDuration: Durations.long1,
          autoCloseDuration: Duration(seconds: 3)
        );
      }
    });
  }

  Future<void> loadSettings() async {
    lwd = await getProp(key: "lwd") ?? lwd;
  }

  Future<List<Account>> loadAccounts() async {
    final as = await listAccounts();
    accounts = as;
    return as;
  }

  Future<void> loadTxHistory() async {
    final txs = await listTxHistory();
    transactions = txs;
  }

  Future<void> loadMemos() async {
    final mems = await listMemos();
    memos = mems;
  }

  bool syncInProgress = false;
  int retryCount = 0;
  StreamSubscription<SyncProgress>? syncProgressSubscription;
  Timer? retrySyncTimer;

  Future<Stream<SyncProgress>?> startSynchronize(List<int> accounts) async {
    if (syncInProgress) {
      return null;
    }

    try {
      ScaffoldMessenger.of(navigatorKey.currentContext!).showSnackBar(
        SnackBar(
          content: Text("Starting Synchronization"),
        ),
      );
      syncInProgress = true;
      retrySyncTimer?.cancel();
      retrySyncTimer = null;
      final currentHeight = await getCurrentHeight();
      final progress =
          synchronize(accounts: accounts, currentHeight: currentHeight,
          transparentLimit: 10) // TODO: Make this configurable
              .asBroadcastStream();
      for (var id in accounts) {
        syncs[id] = progress;
      }
      await syncProgressSubscription?.cancel();
      syncProgressSubscription =
          progress.listen((_) => retryCount = 0, onError: (_) {
        retry(accounts);
      }, onDone: () {
        syncInProgress = false;
        syncs.clear();
        syncProgressSubscription?.cancel();
        syncProgressSubscription = null;
      ScaffoldMessenger.of(navigatorKey.currentContext!).showSnackBar(
        SnackBar(
          content: Text("Synchronization Completed"),
        ),
      );
      });
      return progress;
    } on AnyhowException {
      retry(accounts);
    }
    return null;
  }

  void retry(List<int> accounts) {
    syncInProgress = false;
    retryCount++;
    final maxDelay = pow(2, min(retryCount, 10)).toInt(); // up to 1024s = 17min
    final delay = Random().nextInt(maxDelay); // randomize delay
    final message =
        "Sync error, $retryCount retries, retrying in $delay seconds";
    logger.e(message);
    ScaffoldMessenger.of(navigatorKey.currentContext!).showSnackBar(
      SnackBar(
        content: Text(message),
      ),
    );
    retrySyncTimer?.cancel();
    retrySyncTimer = Timer(Duration(seconds: delay), () {
      startSynchronize(accounts);
    });
  }

  Map<int, Stream<SyncProgress>> syncs = {};

  static AppStore instance = AppStore();
}
