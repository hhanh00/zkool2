import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';
import 'package:mobile_scanner/mobile_scanner.dart';
import 'package:qr_flutter/qr_flutter.dart';

class ScannerPage extends StatefulWidget {
  final String? Function(String? code) validator;
  const ScannerPage({super.key, required this.validator});

  @override
  State<ScannerPage> createState() => _ScannerPageState();
}

class _ScannerPageState extends State<ScannerPage> {
  bool scanned = false;

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text("QR Scanner"),
      ),
      body: Center(
        child: MobileScanner(
          onDetect: onDetect,
          fit: BoxFit.cover,
        ),
      ),
    );
  }

  void onDetect(BarcodeCapture? capture) {
    if (scanned || capture == null) return;
    final List<Barcode> barcodes = capture.barcodes;
    for (final barcode in barcodes) {
      final text = barcode.rawValue;
      if (text != null) {
        final error = widget.validator.call(text);
        if (error == null) {
          scanned = true;
          GoRouter.of(context).pop(text);
        }
      }
    }
  }
}

Future<String?> showScanner(BuildContext context,
    {required String? Function(String?) validator}) async {
  return await GoRouter.of(context).push(
    '/scanner',
    extra: validator,
  );
}

class QRPage extends StatelessWidget {
  final String title;
  final String text;
  const QRPage({super.key, required this.title, required this.text});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: Text(title),
      ),
      body: Center(
        child: QrImageView(
            data: text,
            size: 300,
            version: QrVersions.auto,
            errorCorrectionLevel: QrErrorCorrectLevel.L,
            gapless: false,
            padding: const EdgeInsets.all(16)), // Add your QR code widget here
      ),
    );
  }
}
