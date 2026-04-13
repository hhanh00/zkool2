import 'dart:io';

import 'package:convert/convert.dart';
import 'package:device_info_plus/device_info_plus.dart';
import 'package:flutter/services.dart';
import 'package:extension_google_sign_in_as_googleapis_auth/extension_google_sign_in_as_googleapis_auth.dart';
import 'package:google_sign_in/google_sign_in.dart';
import 'package:googleapis/drive/v3.dart' as drive;
import 'package:path_provider/path_provider.dart';
import 'package:zkool/main.dart';
import 'package:zkool/src/rust/api/vault.dart';

// TODO For prod
// vault should go to appdata folder (hidden)
// scope should be changed in the consent page (google dev console)
class Vault {
  late final String deviceId;
  GoogleSignIn? googleSignIn;
  String? fileId;

  Vault._(this.deviceId);

  static Future<Vault> create() async {
    final deviceInfo = DeviceInfoPlugin();
    String id;
    if (Platform.isAndroid) {
      id = (await deviceInfo.androidInfo).id;
    } else if (Platform.isIOS) {
      id = (await deviceInfo.iosInfo).identifierForVendor ?? '';
    } else if (Platform.isMacOS) {
      id = (await deviceInfo.macOsInfo).systemGUID ?? '';
    } else if (Platform.isLinux) {
      id = (await deviceInfo.linuxInfo).machineId ?? '';
    } else if (Platform.isWindows) {
      id = (await deviceInfo.windowsInfo).deviceId;
    } else {
      throw PlatformException(code: "Unsupported");
    }
    return Vault._(id);
  }

  Future<void> signIn() async {
    logger.i("Signing in");
    googleSignIn = GoogleSignIn(
      // TODO: Update to appdata
      scopes: ['https://www.googleapis.com/auth/drive'],
    );
    var account = await googleSignIn!.signInSilently();
    account ??= await googleSignIn!.signIn();
    logger.i("Signed in ${account!.displayName}");
  }

  Future<void> download() async {
    if (googleSignIn == null) await signIn();
    final httpClient = await googleSignIn!.authenticatedClient();
    final driveApi = drive.DriveApi(httpClient!);

    final localFile = await _localFile;
    await _findFileId(driveApi);

    if (fileId != null) {
      final media = await driveApi.files.get(
        fileId!,
        downloadOptions: drive.DownloadOptions.fullMedia,
      ) as drive.Media;

      final bytes = await media.stream.expand((chunk) => chunk).toList();
      await localFile.writeAsBytes(bytes);
      logger.i("Downloaded vault file (${bytes.length} bytes)");
    } else {
      // No remote vault — create empty local file
      await localFile.writeAsBytes([]);
      logger.i("No remote vault found, created empty local file");
    }
  }

  Future<void> upload() async {
    if (googleSignIn == null) await signIn();
    final httpClient = await googleSignIn!.authenticatedClient();
    final driveApi = drive.DriveApi(httpClient!);

    final localFile = await _localFile;
    final bytes = await localFile.readAsBytes();

    await _findFileId(driveApi);

    final media = drive.Media(
      Stream.value(bytes.toList()),
      bytes.length,
      contentType: 'application/octet-stream',
    );

    if (fileId != null) {
      // Update existing file
      await driveApi.files.update(
        drive.File(),
        fileId!,
        uploadMedia: media,
      );
      logger.i("Updated vault file (${bytes.length} bytes)");
    } else {
      // Create new file
      final file = drive.File()
        ..name = vaultFilename
        ..parents = [spaces];
      final created = await driveApi.files.create(file, uploadMedia: media);
      fileId = created.id;
      logger.i("Created vault file (${bytes.length} bytes)");
    }
  }

  Future<void> _findFileId(drive.DriveApi driveApi) async {
    if (fileId != null) return;
    final fileList = await driveApi.files.list(
      // TODO: Update to appdata
      spaces: 'drive',
      q: "name = '$vaultFilename'",
      $fields: 'files(id)',
    );
    fileId = fileList.files?.isNotEmpty == true ? fileList.files!.first.id : null;
  }

  Future<File> get _localFile async {
    final dir = await getApplicationDocumentsDirectory();
    return File('${dir.path}/$vaultFilename');
  }

  String get vaultFilename => "vault-$deviceId.bin";

  static Future<Uint8List> readLog() async {
    final vault = instance;
    if (vault == null) return Uint8List(0);

    await vault.download();
    final localFile = await vault._localFile;
    if (await localFile.exists()) {
      return localFile.readAsBytes();
    }
    return Uint8List(0);
  }

  static Future<void> append(Uint8List entry) async {
    logger.i("append to log: ${hex.encode(entry)}");
    final vault = instance;
    if (vault == null) return;

    final localFile = await vault._localFile;
    await localFile.writeAsBytes(
      entry,
      mode: FileMode.append,
    );

    try {
      await vault.upload();
    } catch (e) {
      logger.w("upload failed: $e");
    }
  }

  static Vault? instance;
  // TODO: Update to appdata
  static const String spaces = "root";
}

late Vault vault;

Future<DartVault> initializeVault() async {
  return await initVault(append: Vault.append, readLog: Vault.readLog);
}
