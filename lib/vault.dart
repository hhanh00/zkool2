import 'dart:io';

import 'package:convert/convert.dart';
import 'package:device_info_plus/device_info_plus.dart';
import 'package:flutter/services.dart';
import 'package:extension_google_sign_in_as_googleapis_auth/extension_google_sign_in_as_googleapis_auth.dart';
import 'package:google_sign_in/google_sign_in.dart';
import 'package:googleapis/drive/v3.dart' as drive;
import 'package:path_provider/path_provider.dart';
import 'package:zkool/main.dart';
import 'package:zkool/src/rust/api/vault.dart' as rust;
export 'package:zkool/src/rust/api/vault.dart' show RestoredAccount;

enum VaultFile { masterPart, devicePart }

class Vault {
  late final String deviceId;
  GoogleSignIn? googleSignIn;
  late final rust.DartVault rustVault;
  bool _hasDownloadedDevicePart = false;

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
    final vault = Vault._(id);
    vault.rustVault = await rust.initVault(
      append: (entry) => vault.append(entry),
    );
    return vault;
  }

  Future<void> signIn({bool silent = true}) async {
    logger.i("Signing in (silent=$silent)");
    googleSignIn = GoogleSignIn(
      scopes: ['https://www.googleapis.com/auth/drive.appdata'],
    );
    GoogleSignInAccount? account;
    if (silent) {
      account = await googleSignIn!.signInSilently();
    }
    account ??= await googleSignIn!.signIn();
    logger.i("Signed in ${account!.displayName} ${account.email}");
  }

  Future<bool> hasVault() async {
    logger.i("hasVault: checking local file");
    final file = await _localMasterFile;
    if (await file.exists()) {
      logger.i("hasVault: local file exists");
      return true;
    }
    // try downloading from Google Drive
    try {
      logger.i("hasVault: trying Google Drive download");
      final bytes = await _download(VaultFile.masterPart);
      if (bytes.isNotEmpty) {
        logger.i("hasVault: downloaded ${bytes.length} bytes, saving locally");
        await file.writeAsBytes(bytes);
        return true;
      }
      logger.i("hasVault: no master file on Drive");
    } catch (e) {
      logger.w("hasVault: download failed: $e");
    }
    return false;
  }

  /// Read pk from vault-mp.bin (the Init LogEntry).
  /// Returns null if no vault exists. Asserts the entry is an Init.
  Future<Uint8List?> get masterPk async {
    final file = await _localMasterFile;
    if (!await file.exists()) return null;
    final bytes = await file.readAsBytes();
    assert(bytes[0] == 0, "Expected Init tag 0, got ${bytes[0]}");
    return bytes.sublist(1, 33);
  }

  Future<void> initialize(String password) async {
    final file = await _localMasterFile;
    if (await file.exists()) {
      throw StateError('Vault already initialized');
    }
    await setMasterPassword(null, password);
  }

  Future<void> deleteLocalVault() async {
    final masterFile = await _localMasterFile;
    if (await masterFile.exists()) await masterFile.delete();
    final localFile = await _localFile;
    if (await localFile.exists()) await localFile.delete();
  }

  Future<void> registerDevice({required String password, required Uint8List prf}) async {
    final masterFile = await _localMasterFile;
    final initBytes = await masterFile.readAsBytes();
    logger.i('[PRF] registerDevice: prf=${hex.encode(prf.sublist(0, 4))}..., deviceId=$deviceId');
    await rustVault.registerDevice(
      initBytes: initBytes,
      masterPassword: password,
      deviceIdStr: deviceId,
      prfOutput: prf,
    );
  }

  Future<void> storeAccount({required String name, required String seed, required int aindex, required bool useInternal, required int birthHeight, required Uint8List pk}) async {
    final timestamp = DateTime.now().millisecondsSinceEpoch ~/ 1000;
    await rustVault.storeAccount(timestamp: timestamp, name: name, seed: seed, aindex: aindex, useInternal: useInternal, birthHeight: birthHeight, pk: pk);
  }

  Future<void> setMasterPassword(
    String? oldPassword,
    String newPassword,
  ) async {
    final oldBytes = oldPassword != null ? await _download(VaultFile.masterPart) : null;

    final bytes = await rustVault.setMasterPassword(
      newPassword: newPassword,
      oldPassword: oldPassword,
      oldBytes: oldBytes != null && oldBytes.isNotEmpty ? oldBytes : null,
    );

    final localFile = await _localMasterFile;
    await localFile.writeAsBytes(bytes);
    await _upload(bytes, VaultFile.masterPart, createOnly: oldPassword == null);
  }

  Future<Uint8List> downloadVaultBytes() async {
    logger.i("downloadVaultBytes: starting");
    final bytes = await _download(VaultFile.devicePart);
    logger.i("downloadVaultBytes: got ${bytes.length} bytes");
    return bytes;
  }

  Future<List<rust.RestoredAccount>> recoverWithPrf({required Uint8List vaultBytes, required Uint8List prf}) async {
    logger.i('[PRF] recoverWithPrf: vaultBytes=${vaultBytes.length} bytes, prf=${hex.encode(prf.sublist(0, 4))}..., deviceId=$deviceId');
    final result = await rustVault.recoverWithPrf(vaultBytes: vaultBytes, deviceIdStr: deviceId, prfOutput: prf);
    logger.i('[PRF] recoverWithPrf: recovered ${result.length} accounts');
    return result;
  }

  Future<List<rust.RestoredAccount>> recoverVault({required Uint8List vaultBytes, required String masterPassword}) async {
    logger.i("recoverVault: vaultBytes=${vaultBytes.length} bytes");
    final result = await rustVault.recover(vaultBytes: vaultBytes, masterPassword: masterPassword);
    logger.i("recoverVault: recovered ${result.length} accounts");
    return result;
  }

  Future<void> append(Uint8List entry) async {
    logger.i("append to log: ${hex.encode(entry)}");

    final localFile = await _localFile;

    // Lazy download: download once from cloud to ensure we have complete data
    if (!_hasDownloadedDevicePart) {
      try {
        logger.i("append: downloading from cloud first");
        final cloudBytes = await _download(VaultFile.devicePart);
        await localFile.writeAsBytes(cloudBytes);
        logger.i("append: downloaded ${cloudBytes.length} bytes from cloud");
      } catch (e) {
        logger.w("append: download from cloud failed: $e");
      }
      _hasDownloadedDevicePart = true;
    }

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

  static const String spaces = "appDataFolder";

  // --- Private ---

  Future<drive.DriveApi> get _driveApi async {
    if (googleSignIn == null) await signIn();
    logger.i("_driveApi: getting authenticated client");
    final httpClient = await googleSignIn!.authenticatedClient();
    if (httpClient == null) {
      logger.e("_driveApi: authenticatedClient returned null");
      throw StateError("Failed to get authenticated HTTP client");
    }
    logger.i("_driveApi: client obtained, creating DriveApi");
    return drive.DriveApi(httpClient);
  }

  Future<T> _withReauth<T>(Future<T> Function(drive.DriveApi) fn) async {
    try {
      return await fn(await _driveApi);
    } catch (e) {
      if (_isAuthError(e)) {
        logger.w("Auth error, disconnecting account and re-authenticating");
        await googleSignIn?.disconnect();
        try {
          await signIn(silent: false);
          return await fn(await _driveApi);
        } catch (e2) {
          logger.e("Re-authentication failed: $e2");
          rethrow;
        }
      }
      rethrow;
    }
  }

  bool _isAuthError(Object e) {
    logger.e(e.toString());
    final msg = e.toString().toLowerCase();
    return msg.contains('401') || msg.contains('invalid_token') || msg.contains('access was denied');
  }

  String _fileName(VaultFile file) => switch (file) {
        VaultFile.masterPart => "vault-mp.bin",
        VaultFile.devicePart => "vault-dp-$deviceId.bin",
      };

  Future<String?> _findFileId(drive.DriveApi driveApi, String filename) async {
    final fileList = await driveApi.files.list(
      spaces: 'appDataFolder',
      q: "name = '$filename' and trashed = false",
      $fields: 'files(id)',
    );
    return fileList.files?.isNotEmpty == true ? fileList.files!.first.id : null;
  }

  Future<Uint8List> _download(VaultFile file) async {
    logger.i("_download: ${file.name}");
    return _withReauth((driveApi) async {
      if (file == VaultFile.devicePart) {
        // Download all files matching vault-* and aggregate
        final fileList = await driveApi.files.list(
          spaces: 'appDataFolder',
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
    });
  }

  Future<void> _upload(Uint8List bytes, VaultFile file, {bool createOnly = false}) async {
    await _withReauth((driveApi) async {
      final filename = _fileName(file);
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
    });
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
