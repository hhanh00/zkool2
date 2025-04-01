// This file is automatically generated, so please do not edit it.
// @generated by `flutter_rust_bridge`@ 2.9.0.

// ignore_for_file: invalid_use_of_internal_member, unused_import, unnecessary_import

import '../frb_generated.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated.dart';

String newSeed({required String phrase}) =>
    RustLib.instance.api.crateApiAccountNewSeed(phrase: phrase);

int putAccountMetadata(
        {required int coin,
        required String name,
        Uint8List? icon,
        required int birth,
        required int height}) =>
    RustLib.instance.api.crateApiAccountPutAccountMetadata(
        coin: coin, name: name, icon: icon, birth: birth, height: height);

int putAccountSeed(
        {required int coin,
        required int id,
        required String phrase,
        required int aindex}) =>
    RustLib.instance.api.crateApiAccountPutAccountSeed(
        coin: coin, id: id, phrase: phrase, aindex: aindex);

int putAccountSaplingSecret(
        {required int coin, required int id, required String esk}) =>
    RustLib.instance.api
        .crateApiAccountPutAccountSaplingSecret(coin: coin, id: id, esk: esk);

int putAccountSaplingViewing(
        {required int coin, required int id, required String evk}) =>
    RustLib.instance.api
        .crateApiAccountPutAccountSaplingViewing(coin: coin, id: id, evk: evk);

int putAccountUnifiedViewing(
        {required int coin, required int id, required String uvk}) =>
    RustLib.instance.api
        .crateApiAccountPutAccountUnifiedViewing(coin: coin, id: id, uvk: uvk);

Future<int> putAccountTransparentSecret(
        {required int coin, required int id, required String sk}) =>
    RustLib.instance.api
        .crateApiAccountPutAccountTransparentSecret(coin: coin, id: id, sk: sk);

void setDbFilepath({required int coin, required String dbFilepath}) =>
    RustLib.instance.api
        .crateApiAccountSetDbFilepath(coin: coin, dbFilepath: dbFilepath);
