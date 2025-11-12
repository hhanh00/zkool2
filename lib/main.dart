import 'dart:io';
import 'dart:typed_data';

import 'package:file_picker/file_picker.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
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
late final LifecycleWatcher appWatcher;

const String appName = "zkool";

final appKey = GlobalKey();

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();

  await RustLib.init();
  final dataDir = await getApplicationDocumentsDirectory();
  await initDatadir(directory: dataDir.path);
  // await appStore.init();
  // await appStore.loadAppSettings();

  // Future<AppSettings> initApp(WidgetRef ref, AppSettings s) async {
  //   appWatcher = LifecycleWatcher(ref);
  //   return s;
  // }

  runApp(
    ToastificationWrapper(
      child: ShowCaseWidget(
        globalTooltipActions: [
          const TooltipActionButton(type: TooltipDefaultActionType.skip, textStyle: TextStyle(color: Colors.red), backgroundColor: Colors.transparent),
          const TooltipActionButton(type: TooltipDefaultActionType.next, backgroundColor: Colors.transparent),
        ],
        builder: (context) => ProviderScope(
          child: Consumer(
            builder: (context, ref, _) {
              final settings = ref.read(appSettingsProvider.future);
              return FutureBuilder(
                future: settings, // .then((s) => initApp(ref, s)),
                builder: (context, snapshot) {
                  if (snapshot.hasError) return Text(snapshot.error!.toString());
                  if (!snapshot.hasData) return SizedBox.expand();
                  //   appWatcher.init();
                  final settings = snapshot.data!;
                  final r = router(settings.recovery);
                  return MaterialApp.router(
                    key: appKey,
                    routerConfig: r,
                    themeMode: ThemeMode.system,
                    theme: ThemeData.light(),
                    darkTheme: ThemeData.dark(),
                    debugShowCheckedModeBanner: false,
                  );
                },
              );
            },
          ),
        ),
      ),
    ),
  );
}

class LifecycleWatcher with WidgetsBindingObserver {
  bool disabled = false;
  final WidgetRef ref;
  LifecycleWatcher(this.ref);

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
    // TODO
    // if (state == AppLifecycleState.resumed) {
    //   cancelMempoolListener();
    //   if (disabled) {
    //     disabled = false;
    //     return;
    //   }

    //   if (appStore.needPin && appStore.unlocked != null && DateTime.now().difference(appStore.unlocked!).inSeconds >= 5) {
    //     lockApp();
    //   }
    // }
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

  Future<String?> saveFile({String? title, String? fileName, required Uint8List data}) async {
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
        child: InkWell(
          onTap: onUnlock,
          child: Image.asset("misc/icon.png", width: 200),
        ),
      ),
    );
  }
}
