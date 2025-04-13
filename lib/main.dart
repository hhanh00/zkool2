import 'dart:io';

import 'package:flutter/material.dart';
import 'package:logger/logger.dart';
import 'package:zkool/router.dart';
import 'package:zkool/src/rust/api/db.dart';
import 'package:zkool/src/rust/api/network.dart';
import 'package:zkool/src/rust/frb_generated.dart';
import 'package:path_provider/path_provider.dart';
import 'package:zkool/store.dart';

var logger = Logger();

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();
  await RustLib.init();
  final dbDir = await getApplicationDocumentsDirectory();
  final dbFilepath = '${dbDir.path}/zkool.db';
  if (!File(dbFilepath).existsSync()) {
    await createDatabase(coin: 0, dbFilepath: dbFilepath);
    logger.i("Database file created: $dbFilepath");
  }
  await openDatabase(dbFilepath: dbFilepath);
  // setLwd(lwd: "https://lwd4.zcash-infra.com:9067");
  setLwd(lwd: "https://zec.rocks");
  AppStoreBase.instance.init();

  runApp(MaterialApp.router(
      routerConfig: router, debugShowCheckedModeBanner: false));
}
