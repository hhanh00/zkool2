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

ThemeMode _parseThemeMode(String? s) {
  switch (s) {
    case "light":
      return ThemeMode.light;
    case "system":
      return ThemeMode.system;
    case "dark":
    default:
      // Default to dark when no preference is stored (requested default).
      return ThemeMode.dark;
  }
}

String themeModeToString(ThemeMode m) {
  switch (m) {
    case ThemeMode.light:
      return "light";
    case ThemeMode.system:
      return "system";
    case ThemeMode.dark:
      return "dark";
  }
}

/// Theme mode preference, defaulting to dark and persisted across launches.
class ThemeModeNotifier extends Notifier<ThemeMode> {
  @override
  ThemeMode build() {
    // Default synchronously to dark, then load the saved preference async.
    _load();
    return ThemeMode.dark;
  }

  Future<void> _load() async {
    final prefs = SharedPreferencesAsync();
    final saved = await prefs.getString(_kThemeModePref);
    if (saved != null) {
      final m = _parseThemeMode(saved);
      if (m != state) state = m;
    }
  }

  Future<void> set(ThemeMode mode) async {
    state = mode;
    final prefs = SharedPreferencesAsync();
    await prefs.setString(_kThemeModePref, themeModeToString(mode));
  }
}

final themeModeProvider = NotifierProvider<ThemeModeNotifier, ThemeMode>(ThemeModeNotifier.new);

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
