// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'store.dart';

// **************************************************************************
// StoreGenerator
// **************************************************************************

// ignore_for_file: non_constant_identifier_names, unnecessary_brace_in_string_interps, unnecessary_lambdas, prefer_expression_function_bodies, lines_longer_than_80_chars, avoid_as, avoid_annotating_with_dynamic, no_leading_underscores_for_local_identifiers

mixin _$ObservableHeight on ObservableHeightBase, Store {
  Computed<double>? _$progressComputed;

  @override
  double get progress =>
      (_$progressComputed ??= Computed<double>(() => super.progress,
              name: 'ObservableHeightBase.progress'))
          .value;

  late final _$heightAtom =
      Atom(name: 'ObservableHeightBase.height', context: context);

  @override
  int get height {
    _$heightAtom.reportRead();
    return super.height;
  }

  @override
  set height(int value) {
    _$heightAtom.reportWrite(value, super.height, () {
      super.height = value;
    });
  }

  late final _$startAtom =
      Atom(name: 'ObservableHeightBase.start', context: context);

  @override
  int get start {
    _$startAtom.reportRead();
    return super.start;
  }

  @override
  set start(int value) {
    _$startAtom.reportWrite(value, super.start, () {
      super.start = value;
    });
  }

  late final _$rangeAtom =
      Atom(name: 'ObservableHeightBase.range', context: context);

  @override
  int get range {
    _$rangeAtom.reportRead();
    return super.range;
  }

  @override
  set range(int value) {
    _$rangeAtom.reportWrite(value, super.range, () {
      super.range = value;
    });
  }

  late final _$ObservableHeightBaseActionController =
      ActionController(name: 'ObservableHeightBase', context: context);

  @override
  void begin(int endHeight) {
    final _$actionInfo = _$ObservableHeightBaseActionController.startAction(
        name: 'ObservableHeightBase.begin');
    try {
      return super.begin(endHeight);
    } finally {
      _$ObservableHeightBaseActionController.endAction(_$actionInfo);
    }
  }

  @override
  void set(int h, int t) {
    final _$actionInfo = _$ObservableHeightBaseActionController.startAction(
        name: 'ObservableHeightBase.set');
    try {
      return super.set(h, t);
    } finally {
      _$ObservableHeightBaseActionController.endAction(_$actionInfo);
    }
  }

  @override
  void done(int endHeight) {
    final _$actionInfo = _$ObservableHeightBaseActionController.startAction(
        name: 'ObservableHeightBase.done');
    try {
      return super.done(endHeight);
    } finally {
      _$ObservableHeightBaseActionController.endAction(_$actionInfo);
    }
  }

  @override
  String toString() {
    return '''
height: ${height},
start: ${start},
range: ${range},
progress: ${progress}
    ''';
  }
}

mixin _$AppStore on AppStoreBase, Store {
  late final _$selectedAccountAtom =
      Atom(name: 'AppStoreBase.selectedAccount', context: context);

  @override
  Account? get selectedAccount {
    _$selectedAccountAtom.reportRead();
    return super.selectedAccount;
  }

  @override
  set selectedAccount(Account? value) {
    _$selectedAccountAtom.reportWrite(value, super.selectedAccount, () {
      super.selectedAccount = value;
    });
  }

  late final _$selectedFolderAtom =
      Atom(name: 'AppStoreBase.selectedFolder', context: context);

  @override
  Folder? get selectedFolder {
    _$selectedFolderAtom.reportRead();
    return super.selectedFolder;
  }

  @override
  set selectedFolder(Folder? value) {
    _$selectedFolderAtom.reportWrite(value, super.selectedFolder, () {
      super.selectedFolder = value;
    });
  }

  late final _$accountsAtom =
      Atom(name: 'AppStoreBase.accounts', context: context);

  @override
  List<Account> get accounts {
    _$accountsAtom.reportRead();
    return super.accounts;
  }

  @override
  set accounts(List<Account> value) {
    _$accountsAtom.reportWrite(value, super.accounts, () {
      super.accounts = value;
    });
  }

  late final _$foldersAtom =
      Atom(name: 'AppStoreBase.folders', context: context);

  @override
  List<Folder> get folders {
    _$foldersAtom.reportRead();
    return super.folders;
  }

  @override
  set folders(List<Folder> value) {
    _$foldersAtom.reportWrite(value, super.folders, () {
      super.folders = value;
    });
  }

  late final _$poolsAtom = Atom(name: 'AppStoreBase.pools', context: context);

  @override
  int get pools {
    _$poolsAtom.reportRead();
    return super.pools;
  }

  @override
  set pools(int value) {
    _$poolsAtom.reportWrite(value, super.pools, () {
      super.pools = value;
    });
  }

  late final _$seqnoAtom = Atom(name: 'AppStoreBase.seqno', context: context);

  @override
  int get seqno {
    _$seqnoAtom.reportRead();
    return super.seqno;
  }

  @override
  set seqno(int value) {
    _$seqnoAtom.reportWrite(value, super.seqno, () {
      super.seqno = value;
    });
  }

  late final _$poolBalanceAtom =
      Atom(name: 'AppStoreBase.poolBalance', context: context);

  @override
  PoolBalance? get poolBalance {
    _$poolBalanceAtom.reportRead();
    return super.poolBalance;
  }

  @override
  set poolBalance(PoolBalance? value) {
    _$poolBalanceAtom.reportWrite(value, super.poolBalance, () {
      super.poolBalance = value;
    });
  }

  late final _$transactionsAtom =
      Atom(name: 'AppStoreBase.transactions', context: context);

  @override
  List<Tx> get transactions {
    _$transactionsAtom.reportRead();
    return super.transactions;
  }

  @override
  set transactions(List<Tx> value) {
    _$transactionsAtom.reportWrite(value, super.transactions, () {
      super.transactions = value;
    });
  }

  late final _$memosAtom = Atom(name: 'AppStoreBase.memos', context: context);

  @override
  List<Memo> get memos {
    _$memosAtom.reportRead();
    return super.memos;
  }

  @override
  set memos(List<Memo> value) {
    _$memosAtom.reportWrite(value, super.memos, () {
      super.memos = value;
    });
  }

  late final _$notesAtom = Atom(name: 'AppStoreBase.notes', context: context);

  @override
  List<TxNote> get notes {
    _$notesAtom.reportRead();
    return super.notes;
  }

  @override
  set notes(List<TxNote> value) {
    _$notesAtom.reportWrite(value, super.notes, () {
      super.notes = value;
    });
  }

  late final _$currentHeightAtom =
      Atom(name: 'AppStoreBase.currentHeight', context: context);

  @override
  int get currentHeight {
    _$currentHeightAtom.reportRead();
    return super.currentHeight;
  }

  @override
  set currentHeight(int value) {
    _$currentHeightAtom.reportWrite(value, super.currentHeight, () {
      super.currentHeight = value;
    });
  }

  late final _$needPinAtom =
      Atom(name: 'AppStoreBase.needPin', context: context);

  @override
  bool get needPin {
    _$needPinAtom.reportRead();
    return super.needPin;
  }

  @override
  set needPin(bool value) {
    _$needPinAtom.reportWrite(value, super.needPin, () {
      super.needPin = value;
    });
  }

  late final _$unlockedAtom =
      Atom(name: 'AppStoreBase.unlocked', context: context);

  @override
  DateTime? get unlocked {
    _$unlockedAtom.reportRead();
    return super.unlocked;
  }

  @override
  set unlocked(DateTime? value) {
    _$unlockedAtom.reportWrite(value, super.unlocked, () {
      super.unlocked = value;
    });
  }

  late final _$offlineAtom =
      Atom(name: 'AppStoreBase.offline', context: context);

  @override
  bool get offline {
    _$offlineAtom.reportRead();
    return super.offline;
  }

  @override
  set offline(bool value) {
    _$offlineAtom.reportWrite(value, super.offline, () {
      super.offline = value;
    });
  }

  late final _$mempoolRunningAtom =
      Atom(name: 'AppStoreBase.mempoolRunning', context: context);

  @override
  bool get mempoolRunning {
    _$mempoolRunningAtom.reportRead();
    return super.mempoolRunning;
  }

  @override
  set mempoolRunning(bool value) {
    _$mempoolRunningAtom.reportWrite(value, super.mempoolRunning, () {
      super.mempoolRunning = value;
    });
  }

  late final _$loadTxHistoryAsyncAction =
      AsyncAction('AppStoreBase.loadTxHistory', context: context);

  @override
  Future<void> loadTxHistory() {
    return _$loadTxHistoryAsyncAction.run(() => super.loadTxHistory());
  }

  late final _$loadMemosAsyncAction =
      AsyncAction('AppStoreBase.loadMemos', context: context);

  @override
  Future<void> loadMemos() {
    return _$loadMemosAsyncAction.run(() => super.loadMemos());
  }

  late final _$loadNotesAsyncAction =
      AsyncAction('AppStoreBase.loadNotes', context: context);

  @override
  Future<void> loadNotes() {
    return _$loadNotesAsyncAction.run(() => super.loadNotes());
  }

  late final _$refreshAsyncAction =
      AsyncAction('AppStoreBase.refresh', context: context);

  @override
  Future<void> refresh() {
    return _$refreshAsyncAction.run(() => super.refresh());
  }

  late final _$loadFoldersAsyncAction =
      AsyncAction('AppStoreBase.loadFolders', context: context);

  @override
  Future<void> loadFolders() {
    return _$loadFoldersAsyncAction.run(() => super.loadFolders());
  }

  late final _$AppStoreBaseActionController =
      ActionController(name: 'AppStoreBase', context: context);

  @override
  void incSeqno() {
    final _$actionInfo = _$AppStoreBaseActionController.startAction(
        name: 'AppStoreBase.incSeqno');
    try {
      return super.incSeqno();
    } finally {
      _$AppStoreBaseActionController.endAction(_$actionInfo);
    }
  }

  @override
  String toString() {
    return '''
selectedAccount: ${selectedAccount},
selectedFolder: ${selectedFolder},
accounts: ${accounts},
folders: ${folders},
pools: ${pools},
seqno: ${seqno},
poolBalance: ${poolBalance},
transactions: ${transactions},
memos: ${memos},
notes: ${notes},
currentHeight: ${currentHeight},
needPin: ${needPin},
unlocked: ${unlocked},
offline: ${offline},
mempoolRunning: ${mempoolRunning}
    ''';
  }
}
