import 'dart:async';
import 'dart:math';

import 'package:mobx/mobx.dart';
import 'package:zkool/main.dart';
import 'package:zkool/src/rust/api/account.dart';
import 'package:zkool/src/rust/api/init.dart';
import 'package:zkool/src/rust/api/network.dart';
import 'package:zkool/src/rust/api/sync.dart';

part 'store.g.dart';

class AppStore = AppStoreBase with _$AppStore;

abstract class AppStoreBase with Store {
  @observable
  String accountName = "";
  @observable
  List<Account> accounts = [];
  @observable
  List<Tx> transactions = [];
  @observable
  List<Memo> memos = [];

  bool includeHidden = false;
  ObservableList<String> log = ObservableList.of([]);

  void init() {
    final stream = setLogStream();
    stream.listen((m) {
      logger.i(m);
      log.add(m.message);
    });
  }

  Future<List<Account>> loadAccounts() async {
    final as = await listAccounts(includeHidden: includeHidden);
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
  Timer? retrySyncTimer;

  Future<Stream<SyncProgress>> startSynchronize(List<int> accounts) async {
    if (syncInProgress) {
      return Stream.empty();
    }

    syncInProgress = true;
    retrySyncTimer?.cancel();
    retrySyncTimer = null;
    final currentHeight = await getCurrentHeight();
    final progress =
        synchronize(accounts: accounts, currentHeight: currentHeight)
            .asBroadcastStream();
    progress.listen((_) => retryCount = 0, onError: (_) {
      syncInProgress = false;
      retryCount++;
      final maxDelay = pow(2, min(retryCount, 10)).toInt(); // up to 1024s = 17min
      final delay = Random().nextInt(maxDelay); // randomize delay
      logger.i("Sync error, retrying in $delay seconds");
      retrySyncTimer?.cancel();
      retrySyncTimer = Timer(Duration(seconds: delay), () {
        startSynchronize(accounts);
      });
    }, onDone: () {
      syncInProgress = false;
    });
    return progress;
  }

  static AppStore instance = AppStore();
}
