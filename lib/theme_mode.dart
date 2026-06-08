import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:shared_preferences/shared_preferences.dart';

// Zcash brand gold. Used as the accent / "line" color of the dark theme.
const zcashGold = Color(0xFFF4B728);

// Dark surfaces for the Zcash dark-yellow look.
const _zcashDarkScaffold = Color(0xFF121212);
const _zcashDarkSurface = Color(0xFF1C1C1C);
const _zcashDarkSurfaceHigh = Color(0xFF262626);

const _kThemeModePref = "theme_mode";

/// App theme selection. In addition to the standard light/dark/system modes,
/// "zkool" restores the original pink/Material light look that existed before
/// the gold Zcash theme (pre-commit 8fe8f35e).
enum AppTheme { zkool, dark, light, system }

AppTheme _parseAppTheme(String? s) {
  switch (s) {
    case "zkool":
      return AppTheme.zkool;
    case "light":
      return AppTheme.light;
    case "system":
      return AppTheme.system;
    case "dark":
    default:
      // Default to dark when no preference is stored (requested default).
      return AppTheme.dark;
  }
}

String appThemeToString(AppTheme t) {
  switch (t) {
    case AppTheme.zkool:
      return "zkool";
    case AppTheme.light:
      return "light";
    case AppTheme.system:
      return "system";
    case AppTheme.dark:
      return "dark";
  }
}

/// The Flutter [ThemeMode] implied by an [AppTheme]. "Zkool" forces the light
/// slot (where the pink theme is installed by main.dart).
ThemeMode themeModeFor(AppTheme t) {
  switch (t) {
    case AppTheme.zkool:
    case AppTheme.light:
      return ThemeMode.light;
    case AppTheme.dark:
      return ThemeMode.dark;
    case AppTheme.system:
      return ThemeMode.system;
  }
}

/// App theme preference, defaulting to dark and persisted across launches.
class ThemeModeNotifier extends Notifier<AppTheme> {
  @override
  AppTheme build() {
    // Default synchronously to dark, then load the saved preference async.
    _load();
    return AppTheme.dark;
  }

  Future<void> _load() async {
    final prefs = SharedPreferencesAsync();
    final saved = await prefs.getString(_kThemeModePref);
    if (saved != null) {
      final t = _parseAppTheme(saved);
      if (t != state) state = t;
    }
  }

  Future<void> set(AppTheme t) async {
    state = t;
    final prefs = SharedPreferencesAsync();
    await prefs.setString(_kThemeModePref, appThemeToString(t));
  }
}

final themeModeProvider = NotifierProvider<ThemeModeNotifier, AppTheme>(ThemeModeNotifier.new);

/// Zcash dark theme: charcoal surfaces with the gold accent for lines/highlights.
final ThemeData zcashDarkTheme = () {
  final base = ColorScheme.fromSeed(
    seedColor: zcashGold,
    brightness: Brightness.dark,
  );
  final cs = base.copyWith(
    primary: zcashGold,
    onPrimary: Colors.black,
    secondary: zcashGold,
    onSecondary: Colors.black,
    surface: _zcashDarkSurface,
    onSurface: const Color(0xFFEDEDED),
  );

  return ThemeData(
    useMaterial3: true,
    brightness: Brightness.dark,
    colorScheme: cs,
    scaffoldBackgroundColor: _zcashDarkScaffold,
    appBarTheme: const AppBarTheme(
      backgroundColor: _zcashDarkScaffold,
      foregroundColor: zcashGold,
      elevation: 0,
    ),
    dividerColor: zcashGold.withAlpha(60),
    dividerTheme: DividerThemeData(color: zcashGold.withAlpha(60)),
    progressIndicatorTheme: const ProgressIndicatorThemeData(color: zcashGold),
    tabBarTheme: const TabBarThemeData(
      labelColor: zcashGold,
      indicatorColor: zcashGold,
    ),
    elevatedButtonTheme: ElevatedButtonThemeData(
      style: ElevatedButton.styleFrom(
        backgroundColor: _zcashDarkSurfaceHigh,
        foregroundColor: zcashGold,
      ),
    ),
    iconTheme: const IconThemeData(color: zcashGold),
    switchTheme: SwitchThemeData(
      thumbColor: WidgetStateProperty.resolveWith(
        (s) => s.contains(WidgetState.selected) ? zcashGold : null,
      ),
      trackColor: WidgetStateProperty.resolveWith(
        (s) => s.contains(WidgetState.selected) ? zcashGold.withAlpha(120) : null,
      ),
    ),
  );
}();

/// Zcash light theme: brand gold seed, light surfaces.
final ThemeData zcashLightTheme = () {
  final cs = ColorScheme.fromSeed(
    seedColor: zcashGold,
    brightness: Brightness.light,
  );
  return ThemeData(
    useMaterial3: true,
    brightness: Brightness.light,
    colorScheme: cs,
  );
}();

/// "Zkool" pink light theme: restores the original pink/Material light look
/// that existed before the gold Zcash theme (pre-commit 8fe8f35e).
final ThemeData zkoolPinkTheme = () {
  final cs = ColorScheme.fromSeed(
    seedColor: const Color(0xFFE91E63), // Material Pink 500
    brightness: Brightness.light,
  );
  return ThemeData(
    useMaterial3: true,
    brightness: Brightness.light,
    colorScheme: cs,
  );
}();

/// The light [ThemeData] to install for a given [AppTheme]. Dark/System are
/// handled separately via [zcashDarkTheme] in the darkTheme slot.
ThemeData lightThemeFor(AppTheme t) {
  switch (t) {
    case AppTheme.zkool:
      return zkoolPinkTheme;
    case AppTheme.light:
    case AppTheme.dark:
    case AppTheme.system:
      return zcashLightTheme;
  }
}
