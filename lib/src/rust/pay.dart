// This file is automatically generated, so please do not edit it.
// @generated by `flutter_rust_bridge`@ 2.9.0.

// ignore_for_file: invalid_use_of_internal_member, unused_import, unnecessary_import

import 'frb_generated.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated.dart';

class Recipient {
  final String address;
  final BigInt amount;
  final int? pools;
  final String? userMemo;
  final Uint8List? memoBytes;

  const Recipient({
    required this.address,
    required this.amount,
    this.pools,
    this.userMemo,
    this.memoBytes,
  });

  @override
  int get hashCode =>
      address.hashCode ^
      amount.hashCode ^
      pools.hashCode ^
      userMemo.hashCode ^
      memoBytes.hashCode;

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is Recipient &&
          runtimeType == other.runtimeType &&
          address == other.address &&
          amount == other.amount &&
          pools == other.pools &&
          userMemo == other.userMemo &&
          memoBytes == other.memoBytes;
}
