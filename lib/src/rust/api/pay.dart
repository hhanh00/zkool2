// This file is automatically generated, so please do not edit it.
// @generated by `flutter_rust_bridge`@ 2.9.0.

// ignore_for_file: invalid_use_of_internal_member, unused_import, unnecessary_import

import '../frb_generated.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated.dart';

Future<void> prepare(
        {required int account,
        required bool senderPayFees,
        required int srcPools}) =>
    RustLib.instance.api.crateApiPayPrepare(
        account: account, senderPayFees: senderPayFees, srcPools: srcPools);
