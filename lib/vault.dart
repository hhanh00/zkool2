import 'dart:io';

import 'package:convert/convert.dart';
import 'package:device_info_plus/device_info_plus.dart';
import 'package:flutter/services.dart';
import 'package:extension_google_sign_in_as_googleapis_auth/extension_google_sign_in_as_googleapis_auth.dart';
import 'package:google_sign_in/google_sign_in.dart';
import 'package:googleapis/drive/v3.dart' as drive;
import 'package:zkool/main.dart';
import 'package:zkool/src/rust/api/vault.dart' as rust;
import 'package:zkool/utils.dart';
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

  Future<void> signOut() async {
    logger.i("signOut: disconnecting Google account");
    await googleSignIn?.disconnect();
    googleSignIn = null;
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

  /// Reset the device part so the next append starts fresh (no cloud download).
  Future<void> resetDevicePart() async {
    final localFile = await _localFile;
    if (await localFile.exists()) await localFile.delete();
    _hasDownloadedDevicePart = true; // skip cloud download, start empty
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

  Future<void> storeAccount(
      {required String name, required String seed, required int aindex, required bool useInternal, required int birthHeight, required Uint8List pk}) async {
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

    // Once per session, make the local file the union of this device's
    // cloud part and whatever is local: the cloud part can be ahead (a
    // reinstall) or behind (an upload that failed), and parts written by
    // earlier versions hold copies of other parts, which the dedupe drops.
    if (!_hasDownloadedDevicePart) {
      try {
        logger.i("append: downloading this device's part from cloud first");
        final cloudBytes = await _downloadOwnPart();
        final localBytes = await localFile.exists() ? await localFile.readAsBytes() : Uint8List(0);
        final merged = rust.dedupeVaultLog(bytes: Uint8List.fromList([...cloudBytes, ...localBytes]));
        await localFile.writeAsBytes(merged);
        logger.i("append: cloud ${cloudBytes.length} + local ${localBytes.length} bytes -> ${merged.length}");
      } catch (e) {
        logger.w("append: download from cloud failed: $e");
        rethrow;
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

  bool _isAuthError(Object? e) {
    if (e == null) return false;
    try {
      final msg = e.toString().toLowerCase();
      logger.e('Auth error check: $msg');
      return msg.contains('401') || msg.contains('invalid_token') || msg.contains('access was denied');
    } catch (_) {
      return false;
    }
  }

  String _fileName(VaultFile file) => switch (file) {
        VaultFile.masterPart => "vault-mp.bin",
        VaultFile.devicePart => "vault-dp-$deviceId.bin",
      };

  /// Every non-trashed file in the app folder matching [q], across pages.
  Future<List<drive.File>> _listFiles(drive.DriveApi driveApi, String q) async {
    final files = <drive.File>[];
    String? pageToken;
    do {
      final page = await driveApi.files.list(
        spaces: 'appDataFolder',
        q: "$q and trashed = false",
        $fields: 'nextPageToken, files(id, name)',
        pageToken: pageToken,
      );
      files.addAll(page.files ?? []);
      pageToken = page.nextPageToken;
    } while (pageToken != null);
    return files;
  }

  Future<String?> _findFileId(drive.DriveApi driveApi, String filename) async {
    final files = await _listFiles(driveApi, "name = '$filename'");
    return files.isNotEmpty ? files.first.id : null;
  }

  Future<Uint8List> _downloadFile(drive.DriveApi driveApi, drive.File f) async {
    final media = await driveApi.files.get(
      f.id!,
      downloadOptions: drive.DownloadOptions.fullMedia,
    ) as drive.Media;
    final bytes = await media.stream.expand((chunk) => chunk).toList();
    logger.i("Downloaded ${f.name} (${bytes.length} bytes)");
    return Uint8List.fromList(bytes);
  }

  /// One file by name, or empty bytes when it is not on Drive.
  Future<Uint8List> _downloadNamed(drive.DriveApi driveApi, String filename) async {
    final files = await _listFiles(driveApi, "name = '$filename'");
    if (files.isEmpty) {
      logger.i("File $filename not found on Drive");
      return Uint8List(0);
    }
    return _downloadFile(driveApi, files.first);
  }

  /// This device's own part, to seed its local log. Recovery is the other
  /// reader of the device parts and takes them all; see [_download].
  Future<Uint8List> _downloadOwnPart() {
    final filename = _fileName(VaultFile.devicePart);
    return _withReauth((driveApi) => _downloadNamed(driveApi, filename));
  }

  /// The master part alone, or, for the device part, every vault file on
  /// the drive joined: the master entry and all devices' parts, which is
  /// what recovery decrypts.
  Future<Uint8List> _download(VaultFile file) async {
    logger.i("_download: ${file.name}");
    return _withReauth((driveApi) async {
      if (file == VaultFile.devicePart) {
        final files = await _listFiles(driveApi, "name contains 'vault-'");
        if (files.isEmpty) {
          logger.i("No vault files found on Drive");
          return Uint8List(0);
        }
        final allBytes = <int>[];
        for (final f in files) {
          allBytes.addAll(await _downloadFile(driveApi, f));
        }
        return Uint8List.fromList(allBytes);
      }
      return _downloadNamed(driveApi, _fileName(file));
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
    final dir = await getAppDirectory();
    return File('${dir.path}/${_fileName(VaultFile.devicePart)}');
  }

  Future<File> get _localMasterFile async {
    final dir = await getAppDirectory();
    return File('${dir.path}/${_fileName(VaultFile.masterPart)}');
  }
}
