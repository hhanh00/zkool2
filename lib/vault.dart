import 'dart:convert';
import 'dart:typed_data';

import 'package:convert/convert.dart';
import 'package:zkool/main.dart';
import 'package:zkool/src/rust/api/vault.dart';

Future<DartVault> initializeVault() async {
  return await initVault(append: append, readLog: readLog);
}

Future<void> append(Uint8List entry) async {
  logger.i("append to log: ${hex.encode(entry)}");
}

Future<Uint8List> readLog() async {
  return Uint8List.fromList([]);
}
