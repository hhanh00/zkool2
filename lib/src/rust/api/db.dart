// This file is automatically generated, so please do not edit it.
// @generated by `flutter_rust_bridge`@ 2.9.0.

// ignore_for_file: invalid_use_of_internal_member, unused_import, unnecessary_import

import '../frb_generated.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated.dart';

// These functions are ignored because they are not marked as `pub`: `get_connect_options`

Future<void> createDatabase(
        {required int coin, required String dbFilepath, String? password}) =>
    RustLib.instance.api.crateApiDbCreateDatabase(
        coin: coin, dbFilepath: dbFilepath, password: password);

Future<void> openDatabase({required String dbFilepath, String? password}) =>
    RustLib.instance.api
        .crateApiDbOpenDatabase(dbFilepath: dbFilepath, password: password);

Future<String?> getProp({required String key}) =>
    RustLib.instance.api.crateApiDbGetProp(key: key);

Future<void> putProp({required String key, required String value}) =>
    RustLib.instance.api.crateApiDbPutProp(key: key, value: value);
