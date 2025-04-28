// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'store.dart';

// **************************************************************************
// StoreGenerator
// **************************************************************************

// ignore_for_file: non_constant_identifier_names, unnecessary_brace_in_string_interps, unnecessary_lambdas, prefer_expression_function_bodies, lines_longer_than_80_chars, avoid_as, avoid_annotating_with_dynamic, no_leading_underscores_for_local_identifiers

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

  @override
  String toString() {
    return '''
selectedAccount: ${selectedAccount},
accounts: ${accounts},
transactions: ${transactions},
memos: ${memos},
notes: ${notes},
currentHeight: ${currentHeight}
    ''';
  }
}
