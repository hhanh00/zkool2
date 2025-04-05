// This file is automatically generated, so please do not edit it.
// @generated by `flutter_rust_bridge`@ 2.9.0.

// ignore_for_file: invalid_use_of_internal_member, unused_import, unnecessary_import

import '../frb_generated.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated.dart';

Stream<SyncProgress> synchronize(
        {required List<int> accounts, required int currentHeight}) =>
    RustLib.instance.api.crateApiSyncSynchronize(
        accounts: accounts, currentHeight: currentHeight);

Future<PoolBalance> balance() => RustLib.instance.api.crateApiSyncBalance();

class PoolBalance {
  final Uint64List balance;

  const PoolBalance({
    required this.balance,
  });

  @override
  int get hashCode => balance.hashCode;

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is PoolBalance &&
          runtimeType == other.runtimeType &&
          balance == other.balance;
}

class SyncProgress {
  final int height;
  final int time;

  const SyncProgress({
    required this.height,
    required this.time,
  });

  @override
  int get hashCode => height.hashCode ^ time.hashCode;

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is SyncProgress &&
          runtimeType == other.runtimeType &&
          height == other.height &&
          time == other.time;
}
