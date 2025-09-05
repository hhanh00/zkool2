import 'dart:math';
import 'dart:ui';
import 'package:awesome_dialog/awesome_dialog.dart';
import 'package:convert/convert.dart';
import 'package:decimal/intl.dart';
import 'package:fixed/fixed.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:gap/gap.dart';
import 'package:go_router/go_router.dart';
import 'package:intl/intl.dart';
import 'package:local_auth/error_codes.dart' as auth_error;
import 'package:mobx/mobx.dart';
import 'package:path_provider/path_provider.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:showcaseview/showcaseview.dart';
import 'package:timeago/timeago.dart' as timeago;
import 'package:local_auth/local_auth.dart';
import 'package:zkool/router.dart';
import 'package:zkool/store.dart';

String initials(String name) => name.substring(0, min(2, name.length)).toUpperCase();

final locale = PlatformDispatcher.instance.locale.toString();
final formatter = NumberFormat.decimalPatternDigits(locale: locale, decimalDigits: 8);
final zatFormatter = DecimalFormatter(formatter);
final invertSeparator = NumberFormat.decimalPattern(locale).symbols.DECIMAL_SEP != ".";

String zatToString(BigInt zat) {
  final z = Fixed.fromBigInt(zat, scale: 8);
  final s = zatFormatter.format(z.toDecimal());
  return s;
}

Widget zatToText(BigInt zat, {String prefix = "", TextStyle? style, Function()? onTap, required bool selectable, bool colored = false}) {
  style ??= Theme.of(navigatorKey.currentContext!).textTheme.bodyMedium!;
  if (colored && zat > BigInt.zero) {
    style = style.copyWith(color: Colors.green);
  }
  final s = zatToString(zat);
  final minorUnits = s.substring(s.length - 5, s.length);
  final majorUnits = s.substring(0, s.length - 5);
  return selectable
      ? Row(mainAxisSize: MainAxisSize.min, crossAxisAlignment: CrossAxisAlignment.baseline, textBaseline: TextBaseline.alphabetic, children: [
          InkWell(onTap: onTap ?? () => copyToClipboard(s), child: Text(prefix)),
          SelectableText.rich(TextSpan(children: [
            TextSpan(text: majorUnits, style: style),
            TextSpan(text: minorUnits, style: style.copyWith(fontSize: style.fontSize! * 0.6)),
          ]))
        ])
      : Text.rich(TextSpan(children: [
          TextSpan(text: majorUnits, style: style),
          TextSpan(text: minorUnits, style: style.copyWith(fontSize: style.fontSize! * 0.6)),
        ]));
}

BigInt stringToZat(String s) {
  final z = Fixed.parse(s, scale: 8, invertSeparator: invertSeparator);
  return z.minorUnits;
}

String timeToString(int time) {
  if (time == 0) return "N/A";
  final date = DateTime.fromMillisecondsSinceEpoch(time * 1000);
  final dateString = DateFormat('yyyy-MM-dd').format(date);
  final timeAgoStr = timeago.format(date);
  return '$dateString ($timeAgoStr)';
}

String exactTimeToString(int time) {
  if (time == 0) return "N/A";
  final date = DateTime.fromMillisecondsSinceEpoch(time * 1000);
  return date.toString();
}

String txIdToString(Uint8List txid) {
  var reversed = txid.reversed.toList();
  final txId = hex.encode(reversed);
  return txId;
}

Uint8List stringToTxId(String txid) {
  var bytes = hex.decode(txid);
  return Uint8List.fromList(bytes.reversed.toList());
}

Future<String> getFullDatabasePath(String dbName) async {
  final dbDir = await getApplicationDocumentsDirectory();
  final dbFilepath = '${dbDir.path}/$dbName.db';
  return dbFilepath;
}

Future<void> showException(BuildContext context, String message) async {
  await AwesomeDialog(
    context: context,
    dialogType: DialogType.error,
    animType: AnimType.rightSlide,
    title: 'ERROR',
    desc: message,
    btnOkOnPress: () {},
    autoDismiss: true,
  ).show();
}

Future<void> showMessage(BuildContext context, String message, {String? title}) async {
  await AwesomeDialog(
    context: context,
    dialogType: DialogType.info,
    animType: AnimType.rightSlide,
    title: title,
    desc: message,
    btnOkOnPress: () {},
    autoDismiss: true,
  ).show();
}

Future<void> showSeed(BuildContext context, String message) async {
  final t = Theme.of(context).textTheme;
  await AwesomeDialog(
    context: context,
    dialogType: DialogType.warning,
    animType: AnimType.rightSlide,
    body: Column(children: [
      Text("SEED PHRASE - SAVE IT OR YOU CAN LOSE YOUR FUNDS", style: t.headlineSmall),
      Gap(16),
      CopyableText(
        message,
        textAlign: TextAlign.center,
      ),
    ]),
    desc: message,
    btnOkOnPress: () {},
    autoDismiss: true,
  ).show();
}

void showSnackbar(String message) => ScaffoldMessenger.of(navigatorKey.currentContext!).showSnackBar(
      SnackBar(
        content: Text(message),
      ),
    );

Future<bool> confirmDialog(BuildContext context, {required String title, required String message}) async {
  final confirmed = await AwesomeDialog(
        context: context,
        dialogType: DialogType.question,
        animType: AnimType.rightSlide,
        title: title,
        body: Text(message),
        btnCancelOnPress: () {},
        btnOkOnPress: () {},
        onDismissCallback: (type) {
          final res = (() {
            switch (type) {
              case DismissType.btnOk:
                return true;
              default:
                return false;
            }
          })();
          GoRouter.of(context).pop(res);
        },
        autoDismiss: false,
      ).show() ??
      false;
  return confirmed;
}

Future<String?> inputPassword(BuildContext context, {required String title, String? btnCancelText, String? message}) async {
  final password = TextEditingController();
  bool confirmed = await AwesomeDialog(
        context: context,
        dialogType: DialogType.question,
        animType: AnimType.rightSlide,
        body: Column(children: [
          Text(title, style: Theme.of(context).textTheme.headlineSmall),
          Gap(8),
          TextField(
            autofocus: true,
            decoration: InputDecoration(labelText: 'Password', hintText: message),
            obscureText: true,
            controller: password,
          )
        ]),
        btnCancelText: btnCancelText,
        btnCancelOnPress: () {},
        btnOkOnPress: () {},
        onDismissCallback: (type) {
          final res = (() {
            switch (type) {
              case DismissType.btnOk:
                return true;
              default:
                return false;
            }
          })();
          GoRouter.of(context).pop(res);
        },
        dismissOnTouchOutside: false,
        autoDismiss: false,
      ).show() ??
      false;
  if (confirmed) {
    final p = password.text;
    return p;
  }
  return null;
}

Future<String?> inputText(BuildContext context, {required String title}) async {
  final controller = TextEditingController();
  bool confirmed = await AwesomeDialog(
        context: context,
        dialogType: DialogType.question,
        animType: AnimType.rightSlide,
        body: Column(children: [
          Text(title, style: Theme.of(context).textTheme.headlineSmall),
          Gap(8),
          TextField(
            autofocus: true,
            controller: controller,
          )
        ]),
        btnCancelOnPress: () {},
        btnOkOnPress: () {},
        onDismissCallback: (type) {
          final res = (() {
            switch (type) {
              case DismissType.btnOk:
                return true;
              default:
                return false;
            }
          })();
          GoRouter.of(context).pop(res);
        },
        dismissOnTouchOutside: false,
        autoDismiss: false,
      ).show() ??
      false;
  if (confirmed) {
    final p = controller.text;
    return p;
  }
  return null;
}

Future<void> resetTutorial(BuildContext context) async {
  final prefs = SharedPreferencesAsync();
  await prefs.remove("tutMain0");
  await prefs.remove("tutMain1");
  await prefs.remove("tutNew0");
  await prefs.remove("tutNew1");
  await prefs.remove("tutNew2");
  await prefs.remove("tutEdit0");
  await prefs.remove("tutAccount0");
  await prefs.remove("tutAccount1");
  await prefs.remove("tutReceive0");
  await prefs.remove("tutSend0");
  await prefs.remove("tutSend1");
  await prefs.remove("tutSend2");
  await prefs.remove("tutSend3");
  await prefs.remove("tutSend4");
  await prefs.remove("tutSettings0");
  await showMessage(context, "Tutorial tooltips will be shown again.");
}

void tutorialHelper(BuildContext context, String id, List<GlobalKey<State<StatefulWidget>>> ids) async {
  final prefs = SharedPreferencesAsync();
  final tutNew = await prefs.getBool(id) ?? true;
  if (tutNew) {
    if (!context.mounted) return;
    final scw = ShowCaseWidget.of(context);
    if (scw.isShowCaseCompleted) {
      scw.startShowCase(ids);
      await prefs.setBool(id, false);
    }
  }
}

Future<bool> authenticate({String? reason}) async {
  final LocalAuthentication auth = LocalAuthentication();
  try {
    final didAuthenticate =
        await auth.authenticate(localizedReason: reason ?? "Authenticate to continue", options: const AuthenticationOptions(useErrorDialogs: false));
    if (didAuthenticate) runInAction(() => appStore.unlocked = DateTime.now());
    return didAuthenticate;
  } on PlatformException catch (e) {
    switch (e.code) {
      case auth_error.passcodeNotSet:
        return true; // no passcode set
      case auth_error.notEnrolled:
        return true; // no fingerprint enrolled
      default:
        showSnackbar("Authentication denied: ${e.code} - ${e.message}");
        return false;
    }
  } on MissingPluginException {
    // Fallback for platforms that do not support local authentication
    return true; // Assume authentication is successful
  }
}

Widget maybeShowcase(bool condition, {required GlobalKey key, required String description, required Widget child}) => condition
    ? Showcase(
        key: key,
        description: description,
        child: child,
      )
    : child;

void copyToClipboard(String text) {
  Clipboard.setData(ClipboardData(text: text));
  showSnackbar('Copied to clipboard');
}

class CopyableText extends StatelessWidget {
  final String text;
  final TextStyle? style;
  final TextAlign? textAlign;

  const CopyableText(this.text, {super.key, this.style, this.textAlign});

  @override
  Widget build(BuildContext context) {
    return SelectableText(
      text,
      style: style,
      textAlign: textAlign,
      onTap: () => copyToClipboard(text),
    );
  }
}

void lockApp() {
  runInAction(() {
    appStore.needPin = true;
    appStore.unlocked = null;
  });
}

Future<bool> onUnlock() async {
  final authenticated = await authenticate(reason: "Unlock the App");
  if (authenticated) {
    await runInAction(() async {
      final prefs = SharedPreferencesAsync();
      appStore.needPin = await prefs.getBool("pin_lock") ?? appStore.needPin;
      appStore.unlocked = DateTime.now();
    });
  }
  return authenticated;
}
