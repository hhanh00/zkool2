import 'package:flutter/material.dart';
import 'package:zkool/src/rust/api/simple.dart';
import 'package:zkool/src/rust/frb_generated.dart';
import 'package:path_provider/path_provider.dart';

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();
  await RustLib.init();
  final dbDir = await getApplicationDocumentsDirectory();
  final dbFilepath = '${dbDir.path}/zkool.db';
  setDbFilepath(coin: 0, dbFilepath: dbFilepath);
  
  runApp(const MyApp());
}

class MyApp extends StatefulWidget {
  final phrase = "destroy public fog slim about evolve traffic chef moment genius curtain spell genius mimic gravity around spot plug genre soldier warm basic anchor toddler";
  const MyApp({super.key});

  @override
  State<MyApp> createState() => MyAppState();
}

class MyAppState extends State<MyApp> {
  @override
  void initState() {
    final account = storeAccountMetadata(coin: 0, name: "Hanh", birth: 1, height: 10);
    storeAccountSeed(coin: 0, id: account, phrase: widget.phrase, aindex: 0);
    print(account);
    super.initState();
  }

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      home: Scaffold(
        appBar: AppBar(title: const Text('flutter_rust_bridge quickstart')),
        body: Center(
          child: Text(
              'Action: Call Rust `greet("Tom")`\nResult: `${newSeed(phrase: widget.phrase)}`'),
        ),
      ),
    );
  }
}
