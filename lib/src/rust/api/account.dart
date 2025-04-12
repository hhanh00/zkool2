// This file is automatically generated, so please do not edit it.
// @generated by `flutter_rust_bridge`@ 2.9.0.

// ignore_for_file: invalid_use_of_internal_member, unused_import, unnecessary_import

import '../frb_generated.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated.dart';
import 'package:freezed_annotation/freezed_annotation.dart' hide protected;
part 'account.freezed.dart';

String newSeed({required String phrase}) =>
    RustLib.instance.api.crateApiAccountNewSeed(phrase: phrase);

Future<String> getAccountUfvk() =>
    RustLib.instance.api.crateApiAccountGetAccountUfvk();

String uaFromUfvk({required String ufvk, int? di}) =>
    RustLib.instance.api.crateApiAccountUaFromUfvk(ufvk: ufvk, di: di);

Receivers receiversFromUa({required String ua}) =>
    RustLib.instance.api.crateApiAccountReceiversFromUa(ua: ua);

Future<List<Account>> listAccounts() =>
    RustLib.instance.api.crateApiAccountListAccounts();

Future<void> updateAccount({required AccountUpdate update}) =>
    RustLib.instance.api.crateApiAccountUpdateAccount(update: update);

Future<void> dropSchema() => RustLib.instance.api.crateApiAccountDropSchema();

Future<void> deleteAccount({required Account account}) =>
    RustLib.instance.api.crateApiAccountDeleteAccount(account: account);

Future<void> reorderAccount(
        {required int oldPosition, required int newPosition}) =>
    RustLib.instance.api.crateApiAccountReorderAccount(
        oldPosition: oldPosition, newPosition: newPosition);

Future<void> setAccount({required int id}) =>
    RustLib.instance.api.crateApiAccountSetAccount(id: id);

Future<void> newAccount({required NewAccount na}) =>
    RustLib.instance.api.crateApiAccountNewAccount(na: na);

Future<int> generateNextDindex() =>
    RustLib.instance.api.crateApiAccountGenerateNextDindex();

Future<void> generateNextChangeAddress() =>
    RustLib.instance.api.crateApiAccountGenerateNextChangeAddress();

Future<List<Account>> getAllAccounts() =>
    RustLib.instance.api.crateApiAccountGetAllAccounts();

Future<void> removeAccount({required int accountId}) =>
    RustLib.instance.api.crateApiAccountRemoveAccount(accountId: accountId);

Future<void> moveAccount(
        {required int oldPosition, required int newPosition}) =>
    RustLib.instance.api.crateApiAccountMoveAccount(
        oldPosition: oldPosition, newPosition: newPosition);

Future<List<Tx>> listTxHistory() =>
    RustLib.instance.api.crateApiAccountListTxHistory();

Future<List<Memo>> listMemos() =>
    RustLib.instance.api.crateApiAccountListMemos();

Future<Addresses> getAddresses() =>
    RustLib.instance.api.crateApiAccountGetAddresses();

@freezed
class Account with _$Account {
  const factory Account({
    required int coin,
    required int id,
    required String name,
    String? seed,
    required int aindex,
    Uint8List? icon,
    required int birth,
    required int position,
    required bool hidden,
    required bool saved,
    required bool enabled,
    required int height,
  }) = _Account;
}

@freezed
class AccountUpdate with _$AccountUpdate {
  const factory AccountUpdate({
    required int coin,
    required int id,
    String? name,
    Uint8List? icon,
    int? birth,
    bool? hidden,
    bool? enabled,
  }) = _AccountUpdate;
}

class Addresses {
  final String? taddr;
  final String? saddr;
  final String? oaddr;
  final String? ua;

  const Addresses({
    this.taddr,
    this.saddr,
    this.oaddr,
    this.ua,
  });

  @override
  int get hashCode =>
      taddr.hashCode ^ saddr.hashCode ^ oaddr.hashCode ^ ua.hashCode;

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is Addresses &&
          runtimeType == other.runtimeType &&
          taddr == other.taddr &&
          saddr == other.saddr &&
          oaddr == other.oaddr &&
          ua == other.ua;
}

@freezed
class Memo with _$Memo {
  const factory Memo({
    required int id,
    required int idTx,
    int? idNote,
    required int pool,
    required int height,
    required int vout,
    required int time,
    required Uint8List memoBytes,
    String? memo,
  }) = _Memo;
}

@freezed
class NewAccount with _$NewAccount {
  const factory NewAccount({
    Uint8List? icon,
    required String name,
    required bool restore,
    required String key,
    required int aindex,
    int? birth,
  }) = _NewAccount;
}

class Receivers {
  final String? taddr;
  final String? saddr;
  final String? oaddr;

  const Receivers({
    this.taddr,
    this.saddr,
    this.oaddr,
  });

  static Future<Receivers> default_() =>
      RustLib.instance.api.crateApiAccountReceiversDefault();

  @override
  int get hashCode => taddr.hashCode ^ saddr.hashCode ^ oaddr.hashCode;

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is Receivers &&
          runtimeType == other.runtimeType &&
          taddr == other.taddr &&
          saddr == other.saddr &&
          oaddr == other.oaddr;
}

@freezed
class Tx with _$Tx {
  const factory Tx({
    required int id,
    required Uint8List txid,
    required int height,
    required int time,
    required PlatformInt64 value,
  }) = _Tx;
}
