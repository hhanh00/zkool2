import 'dart:async';
import 'dart:io';
import 'dart:math';
import 'dart:typed_data';
import 'dart:ui' as ui;

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import 'package:mobile_scanner/mobile_scanner.dart';
import 'package:qr_flutter/qr_flutter.dart';
import 'package:zkool/src/rust/api/raptor.dart';
import 'package:zkool/store.dart';

class ShowAnimatedQRPage extends ConsumerStatefulWidget {
  final List<Uint8List> packets;
  const ShowAnimatedQRPage(this.packets, {super.key});

  @override
  ConsumerState<ShowAnimatedQRPage> createState() => ShowAnimatedQRPageState();
}

class ShowAnimatedQRPageState extends ConsumerState<ShowAnimatedQRPage> {
  List<Widget>? qrCodes;
  Timer? timer;
  int i = 0;

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) async {
      final appSettings = await ref.read(appSettingsProvider.future);
      final qrSettings = appSettings.qrSettings;
      final size = MediaQuery.of(context).size;
      final minSize = min(size.height, size.width);
      qrCodes = [];
      for (var (i, p) in widget.packets.indexed) {
        final qr = QrCode(qrSettings.size.toInt(), QrErrorCorrectLevel.levels[qrSettings.ecLevel])..addByteData(ByteData.sublistView(p));
        final img = await textToImageProvider(i.toString());
        qrCodes!.add(Center(
          child: QrImageView.withQr(
            qr: qr,
            size: minSize * 0.8,
            gapless: false,
            embeddedImage: img,
          ),
        ));
      }
      timer = Timer.periodic(Duration(milliseconds: qrSettings.delay), (_) {
        setState(() => i = (i + 1) % qrCodes!.length);
      });
    });
  }

  @override
  void dispose() {
    timer?.cancel();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(),
      body: (qrCodes != null) ? qrCodes![i] : SizedBox.shrink(),
    );
  }
}

Future<ImageProvider> textToImageProvider(String text, {double fontSize = 40}) async {
  final recorder = ui.PictureRecorder();
  final canvas = Canvas(recorder);

  final textStyle = TextStyle(
    color: const Color(0xFF000000),
    fontSize: fontSize,
  );

  final textSpan = TextSpan(
    text: text,
    style: textStyle,
  );

  final tp = TextPainter(
    text: textSpan,
    textDirection: TextDirection.ltr,
  );

  tp.layout();
  tp.paint(canvas, Offset.zero);

  final picture = recorder.endRecording();
  final img = await picture.toImage(tp.width.ceil(), tp.height.ceil());

  final byteData = await img.toByteData(format: ui.ImageByteFormat.png);
  return MemoryImage(byteData!.buffer.asUint8List());
}

Future<void> showAnimatedQR(BuildContext context, WidgetRef ref, String path) async {
  final appSettings = await ref.read(appSettingsProvider.future);
  final qrSettings = appSettings.qrSettings;

  final packets = await encode(
    path: path,
    params: RaptorQParams(
      version: qrSettings.size.toInt(),
      ecLevel: qrSettings.ecLevel,
      repair: qrSettings.repair,
    ),
  );
  GoRouter.of(context).push("/show_animated_qr", extra: packets);
}

class ScanAnimatedQRPage extends StatefulWidget {
  const ScanAnimatedQRPage({super.key});

  @override
  State<StatefulWidget> createState() => ScanAnimatedQRPageState();
}

class ScanAnimatedQRPageState extends State<ScanAnimatedQRPage> {
  Widget? scanner;
  final controller = MobileScannerController();
  Map<int, Uint8List> packets = {};

  @override
  void initState() {
    super.initState();

    Future(() async {
      final completed = Completer<Uint8List>();
      final sub = controller.barcodes.listen((qr) async {
        final barcode = qr.barcodes.first;
        var data = barcode.rawBytes!;
        if (Platform.isMacOS) data = getQrBytes(data: data);
        if (data.length < 16) return;
        final id = ByteData.sublistView(data).getUint32(12, Endian.big);

        if (!packets.containsKey(id)) {
          packets[id] = data;
          setState(() {});
          final result = await decode(packet: data);
          if (result != null) {
            completed.complete(result);
          }
        }
      });
      final data = await completed.future;
      sub.cancel();
      GoRouter.of(context).pop(data.toList());
    });
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: Stack(
        children: [
          MobileScanner(controller: controller),
          Positioned(
            bottom: 10,
            left: 10,
            child: Text(
              packets.length.toString(),
              style: Theme.of(context).textTheme.titleLarge!.copyWith(color: Colors.red),
            ),
          ),
        ],
      ),
    );
  }
}
