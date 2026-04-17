import 'dart:io';

import 'package:convert/convert.dart';
import 'package:device_info_plus/device_info_plus.dart';
import 'package:flutter/services.dart';
import 'package:extension_google_sign_in_as_googleapis_auth/extension_google_sign_in_as_googleapis_auth.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:google_sign_in/google_sign_in.dart';
import 'package:googleapis/drive/v3.dart' as drive;
import 'package:path_provider/path_provider.dart';
import 'package:zkool/main.dart';
import 'package:zkool/src/rust/api/vault.dart';
import 'package:zkool/store.dart';

// TODO For prod
// vault should go to appdata folder (hidden)
// scope should be changed in the consent page (google dev console)
enum VaultFile { masterPart, devicePart }

class Vault {
  late final String deviceId;
  GoogleSignIn? googleSignIn;
  static Vault? instance;

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

  Future<DartVault> toDartVault() async {
    return await initVault(
      append: (entry) => append(entry),
      readLog: () => readLog(),
    );
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

  Future<bool> hasVault() async {
    final file = await _localMasterFile;
    return file.exists();
  }

  Future<void> initialize(WidgetRef ref, String password) async {
    final file = await _localMasterFile;
    if (await file.exists()) {
      throw StateError('Vault already initialized');
    }
    await setMasterPassword(ref, null, password);
  }

  Future<void> setMasterPassword(
    WidgetRef ref,
    String? oldPassword,
    String newPassword,
  ) async {
    final dartVault = await ref.read(vaultProvider.future);
    final oldBytes = oldPassword != null ? await _download(VaultFile.masterPart) : null;

    final bytes = await dartVault.setMasterPassword(
      newPassword: newPassword,
      oldPassword: oldPassword,
      oldBytes: oldBytes != null && oldBytes.isNotEmpty ? oldBytes : null,
    );

    final localFile = await _localMasterFile;
    await localFile.writeAsBytes(bytes);
    await _upload(bytes, VaultFile.masterPart, createOnly: oldPassword == null);
  }

  Future<Uint8List> readLog() async {
    return _download(VaultFile.devicePart);
  }

  Future<void> append(Uint8List entry) async {
    logger.i("append to log: ${hex.encode(entry)}");

    final localFile = await vault._localFile;
    await localFile.writeAsBytes(
      entry,
      mode: FileMode.append,
    );

    try {
      await _upload(await localFile.readAsBytes(), VaultFile.devicePart);
    } catch (e) {
      logger.w("upload failed: $e");
    }
  }

  // TODO: Update to appdata
  static const String spaces = "root";

  // --- Private ---

  Future<drive.DriveApi> get _driveApi async {
    if (googleSignIn == null) await signIn();
    final httpClient = await googleSignIn!.authenticatedClient();
    return drive.DriveApi(httpClient!);
  }

  String _fileName(VaultFile file) => switch (file) {
        VaultFile.masterPart => "vault-mp.bin",
        VaultFile.devicePart => "vault-dp-$deviceId.bin",
      };

  Future<String?> _findFileId(drive.DriveApi driveApi, String filename) async {
    final fileList = await driveApi.files.list(
      // TODO: Update to appdata
      spaces: 'drive',
      q: "name = '$filename'",
      $fields: 'files(id)',
    );
    return fileList.files?.isNotEmpty == true ? fileList.files!.first.id : null;
  }

  Future<Uint8List> _download(VaultFile file) async {
    final driveApi = await _driveApi;

    if (file == VaultFile.devicePart) {
      // Download all files matching vault-* and aggregate
      final fileList = await driveApi.files.list(
        spaces: 'drive',
        q: "name contains 'vault-'",
        $fields: 'files(id, name)',
      );
      final files = fileList.files ?? [];
      if (files.isEmpty) {
        logger.i("No vault device files found on Drive");
        return Uint8List(0);
      }
      final allBytes = <int>[];
      for (final f in files) {
        final media = await driveApi.files.get(
          f.id!,
          downloadOptions: drive.DownloadOptions.fullMedia,
        ) as drive.Media;
        final bytes = await media.stream.expand((chunk) => chunk).toList();
        logger.i("Downloaded ${f.name} (${bytes.length} bytes)");
        allBytes.addAll(bytes);
      }
      return Uint8List.fromList(allBytes);
    } else {
      // Single file download (masterPart)
      final filename = _fileName(file);
      final id = await _findFileId(driveApi, filename);
      if (id != null) {
        final media = await driveApi.files.get(
          id,
          downloadOptions: drive.DownloadOptions.fullMedia,
        ) as drive.Media;
        final bytes = await media.stream.expand((chunk) => chunk).toList();
        logger.i("Downloaded $filename (${bytes.length} bytes)");
        return Uint8List.fromList(bytes);
      }
      logger.i("File $filename not found on Drive");
      return Uint8List(0);
    }
  }

  Future<void> _upload(Uint8List bytes, VaultFile file, {bool createOnly = false}) async {
    final filename = _fileName(file);
    final driveApi = await _driveApi;
    final id = await _findFileId(driveApi, filename);

    final media = drive.Media(
      Stream.value(bytes.toList()),
      bytes.length,
      contentType: 'application/octet-stream',
    );

    if (id != null) {
      if (createOnly) throw StateError('File $filename already exists');
      await driveApi.files.update(
        drive.File(),
        id,
        uploadMedia: media,
      );
      logger.i("Updated $filename (${bytes.length} bytes)");
    } else {
      final driveFile = drive.File()
        ..name = filename
        ..parents = [spaces];
      await driveApi.files.create(driveFile, uploadMedia: media);
      logger.i("Created $filename (${bytes.length} bytes)");
    }
  }

  Future<File> get _localFile async {
    final dir = await getApplicationDocumentsDirectory();
    return File('${dir.path}/${_fileName(VaultFile.devicePart)}');
  }

  Future<File> get _localMasterFile async {
    final dir = await getApplicationDocumentsDirectory();
    return File('${dir.path}/${_fileName(VaultFile.masterPart)}');
  }
}

late Vault vault;
