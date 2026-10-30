// Platform fakes for widget tests of pages that keep state in the shared
// preferences and in the documents directory.

import 'dart:async';
import 'dart:io';

import 'package:path_provider_platform_interface/path_provider_platform_interface.dart';
import 'package:rive/rive.dart';
import 'package:plugin_platform_interface/plugin_platform_interface.dart';
import 'package:shared_preferences_platform_interface/in_memory_shared_preferences_async.dart';
import 'package:shared_preferences_platform_interface/shared_preferences_async_platform_interface.dart';

/// Points the documents directory (and the app directory with it) at [dir].
class FakePathProvider extends PathProviderPlatform with MockPlatformInterfaceMixin {
  final Directory dir;
  FakePathProvider(this.dir);

  @override
  Future<String?> getApplicationDocumentsPath() async => dir.path;

  @override
  Future<String?> getApplicationSupportPath() async => dir.path;
}

/// Installs in-memory shared preferences holding [values].
void useInMemoryPrefs(Map<String, Object> values) {
  SharedPreferencesAsyncPlatform.instance = InMemorySharedPreferencesAsync.withData(values);
}

/// AwesomeDialog draws its header with a Rive animation, and Rive's text
/// engine looks up a native symbol the host test runner does not have. The
/// lookup throws once, from an unawaited callback inside Rive, and after that
/// Rive's initializer never completes, so the dialogs render without a header
/// and work. Triggering that first failure here, in a guarded zone, keeps it
/// out of the test that happens to open the first dialog.
void silenceRiveHeaders() {
  runZonedGuarded(() {
    unawaited(RiveFile.initialize());
  }, (_, __) {});
}
