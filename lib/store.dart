import 'package:mobx/mobx.dart';
import 'package:zkool/main.dart';
import 'package:zkool/src/rust/api/account.dart';
import 'package:zkool/src/rust/api/init.dart';

part 'store.g.dart';

class AppStore = AppStoreBase with _$AppStore;

abstract class AppStoreBase with Store {
  @observable String accountName = "";
  @observable List<Account> accounts = [];
  @observable List<Tx> transactions = [];
  @observable List<Memo> memos = [];

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

  static AppStore instance = AppStore();
}
