// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'store.dart';

// **************************************************************************
// StoreGenerator
// **************************************************************************

// ignore_for_file: non_constant_identifier_names, unnecessary_brace_in_string_interps, unnecessary_lambdas, prefer_expression_function_bodies, lines_longer_than_80_chars, avoid_as, avoid_annotating_with_dynamic, no_leading_underscores_for_local_identifiers

mixin _$AppStore on AppStoreBase, Store {
  late final _$accountNameAtom =
      Atom(name: 'AppStoreBase.accountName', context: context);

  @override
  String get accountName {
    _$accountNameAtom.reportRead();
    return super.accountName;
  }

  @override
  set accountName(String value) {
    _$accountNameAtom.reportWrite(value, super.accountName, () {
      super.accountName = value;
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

  @override
  String toString() {
    return '''
accountName: ${accountName},
accounts: ${accounts},
transactions: ${transactions},
memos: ${memos}
    ''';
  }
}
