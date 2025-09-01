import 'dart:io';
import 'dart:typed_data';

import 'package:file_picker/file_picker.dart';
import 'package:flutter/material.dart';
import 'package:image_picker/image_picker.dart';
import 'package:logger/logger.dart';
import 'package:path_provider/path_provider.dart';
import 'package:showcaseview/showcaseview.dart';
import 'package:toastification/toastification.dart';
import 'package:zkool/router.dart';
import 'package:zkool/src/rust/api/network.dart';
import 'package:zkool/src/rust/frb_generated.dart';
import 'package:zkool/store.dart';
import 'package:zkool/utils.dart';

final logger = Logger(filter: ProductionFilter());
final appWatcher = LifecycleWatcher();

const String appName = "zkool";

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();

  await RustLib.init();
  await appStore.init();
  await appStore.loadAppSettings();
  final dataDir = await getApplicationDocumentsDirectory();
  await initDatadir(directory: dataDir.path);

  appWatcher.init();
  final r = router(appStore.recovery);

  runApp(ToastificationWrapper(
      child: ShowCaseWidget(
          globalTooltipActions: [
        const TooltipActionButton(
            type: TooltipDefaultActionType.skip,
            textStyle: TextStyle(color: Colors.red),
            backgroundColor: Colors.transparent),
        const TooltipActionButton(
            type: TooltipDefaultActionType.next,
            backgroundColor: Colors.transparent),
      ],
          builder: (context) => MaterialApp.router(
              routerConfig: r,
              themeMode: ThemeMode.system,
              theme: ThemeData.light(),
              darkTheme: ThemeData.dark(),
              debugShowCheckedModeBanner: false))));
}

class LifecycleWatcher with WidgetsBindingObserver {
  bool disabled = false;

  void init() {
    WidgetsBinding.instance.addObserver(this);
  }

  // Disables the pin lock on the next resume
  // Used when the app opens a platform dialog (open/save file)
  // to prevent asking for the pin when the dialog closes
  void temporaryDisableLock() {
    disabled = true;
  }

  @override
  void didChangeAppLifecycleState(AppLifecycleState state) {
    if (state == AppLifecycleState.resumed) {
      cancelMempoolListener();
      if (disabled) {
        disabled = false;
        return;
      }

      if (appStore.needPin && appStore.unlocked != null &&
          DateTime.now().difference(appStore.unlocked!).inSeconds >= 5) {
        lockApp();
      }
    }
  }

  Future<Uint8List?> openFile({String? title}) async {
    temporaryDisableLock();
    final files = await FilePicker.platform.pickFiles(
      dialogTitle: title,
    );
    if (files != null) {
      final file = files.files.first;
      final encryptedFile = File(file.path!);
      final data = encryptedFile.readAsBytesSync();
      return data;
    }
    return null;
  }

  Future<XFile?> pickImage() async {
    temporaryDisableLock();
    final picker = ImagePicker();
    final icon = await picker.pickImage(source: ImageSource.gallery);
    return icon;
  }

  Future<String?> saveFile(
      {String? title, String? fileName, required Uint8List data}) async {
    temporaryDisableLock();
    return await FilePicker.platform.saveFile(
      dialogTitle: title,
      fileName: fileName,
      bytes: data,
    );
  }
}

class PinLock extends StatefulWidget {
  const PinLock({
    super.key,
  });

  @override
  State<StatefulWidget> createState() => PinLockState();
}

class PinLockState extends State<PinLock> {
  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) => onUnlock());
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
        appBar: AppBar(title: Text("Locked")),
        body: Center(
            child: IconButton(
          onPressed: onUnlock,
          icon: Icon(
            Icons.lock,
            size: 200,
          ),
        )));
  }
}
