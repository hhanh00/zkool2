import 'dart:math';
import 'dart:typed_data';
import 'package:awesome_dialog/awesome_dialog.dart';
import 'package:convert/convert.dart';
import 'package:fixed/fixed.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_form_builder/flutter_form_builder.dart';
import 'package:gap/gap.dart';
import 'package:go_router/go_router.dart';
import 'package:intl/intl.dart';
import 'package:local_auth/error_codes.dart' as auth_error;
import 'package:path_provider/path_provider.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:showcaseview/showcaseview.dart';
import 'package:timeago/timeago.dart' as timeago;
import 'package:local_auth/local_auth.dart';
import 'package:zkool/router.dart';

String initials(String name) =>
    name.substring(0, min(2, name.length)).toUpperCase();

String zatToString(BigInt zat) {
  final z = Fixed.fromBigInt(zat, scale: 8);
  return z.toString();
}

Widget zatToText(BigInt zat,
    {String prefix = "", TextStyle? style,
    Function()? onTap,
    bool colored = false}) {
  style ??= Theme.of(navigatorKey.currentContext!).textTheme.bodyMedium!;
  if (colored && zat > BigInt.zero) {
    style = style.copyWith(color: Colors.green);
  }
  final s = zatToString(zat);
  final minorUnits = s.substring(s.length - 5, s.length);
  final majorUnits = s.substring(0, s.length - 5);
  return Row(
    mainAxisSize: MainAxisSize.min,
    crossAxisAlignment: CrossAxisAlignment.baseline,
    textBaseline: TextBaseline.alphabetic,
    children: [
      InkWell(onTap: onTap, child: Text(prefix)),
      SelectableText.rich(TextSpan(children: [
      TextSpan(text: majorUnits, style: style),
      TextSpan(
          text: minorUnits,
          style: style.copyWith(fontSize: style.fontSize! * 0.6)),
    ],))]
  );
}

BigInt stringToZat(String s) {
  final z = Fixed.parse(s, scale: 8);
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

Future<void> showMessage(BuildContext context, String message,
    {String? title}) async {
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
      Text("SEED PHRASE - SAVE IT OR YOU CAN LOSE YOUR FUNDS",
          style: t.headlineSmall),
      Gap(16),
      SelectableText(
        message,
        textAlign: TextAlign.center,
      ),
    ]),
    desc: message,
    btnOkOnPress: () {},
    autoDismiss: true,
  ).show();
}

void showSnackbar(String message) =>
    ScaffoldMessenger.of(navigatorKey.currentContext!).showSnackBar(
      SnackBar(
        content: Text(message),
      ),
    );

Future<bool> confirmDialog(BuildContext context,
    {required String title, required String message}) async {
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

Future<String?> inputPassword(BuildContext context,
    {required String title, String? message}) async {
  final password = TextEditingController();
  bool confirmed = await AwesomeDialog(
        context: context,
        dialogType: DialogType.question,
        animType: AnimType.rightSlide,
        title: title,
        body: FormBuilder(
            child: Column(children: [
          Text(title, style: Theme.of(context).textTheme.headlineSmall),
          Gap(8),
          FormBuilderTextField(
            name: 'password',
            decoration:
                InputDecoration(labelText: 'Password', hintText: message),
            obscureText: true,
            controller: password,
          )
        ])),
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

Future<void> resetTutorial() async {
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
}

void tutorialHelper(BuildContext context, String id,
    List<GlobalKey<State<StatefulWidget>>> ids) async {
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
    final didAuthenticate = await auth.authenticate(
        localizedReason: reason ?? "Authenticate to continue",
        options: const AuthenticationOptions(useErrorDialogs: false));
    return didAuthenticate;
  } on PlatformException catch (e) {
    switch (e.code) {
      case auth_error.passcodeNotSet:
        return true; // no passcode set
      case auth_error.notEnrolled:
        return true; // no fingerprint enrolled
      default:
        return false;
    }
  } on MissingPluginException {
    // Fallback for platforms that do not support local authentication
    return true; // Assume authentication is successful
  }
}

Widget maybeShowcase(bool condition,
        {required GlobalKey key,
        required String description,
        required Widget child}) =>
    condition
        ? Showcase(
            key: key,
            description: description,
            child: child,
          )
        : child;
