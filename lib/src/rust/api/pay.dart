// This file is automatically generated, so please do not edit it.
// @generated by `flutter_rust_bridge`@ 2.9.0.

// ignore_for_file: invalid_use_of_internal_member, unused_import, unnecessary_import

import '../frb_generated.dart';
import '../pay.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated.dart';

Future<TxPlan> prepare(
        {required int account,
        required int srcPools,
        required List<Recipient> recipients,
        required bool recipientPaysFee}) =>
    RustLib.instance.api.crateApiPayPrepare(
        account: account,
        srcPools: srcPools,
        recipients: recipients,
        recipientPaysFee: recipientPaysFee);
