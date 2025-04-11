import 'package:mobx/mobx.dart';
import 'package:zkool/src/rust/api/account.dart';

part 'store.g.dart';

var appStore = AppStore();

class AppStore = AppStoreBase with _$AppStore;

abstract class AppStoreBase with Store {
  @observable List<Account> accounts = [];
  @observable List<Tx> transactions = [];

  static Future<List<Account>> loadAccounts() async {
    final as = await listAccounts();
    appStore.accounts = as;
    return as;
  }

  static Future<void> loadTxHistory() async {
    final txs = await listTxHistory();
    appStore.transactions = txs;
  }
}
