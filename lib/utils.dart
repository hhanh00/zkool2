import 'dart:math';
import 'dart:typed_data';
import 'package:awesome_dialog/awesome_dialog.dart';
import 'package:convert/convert.dart';
import 'package:fixed/fixed.dart';
import 'package:flutter/material.dart';
import 'package:flutter_form_builder/flutter_form_builder.dart';
import 'package:gap/gap.dart';
import 'package:go_router/go_router.dart';
import 'package:timeago/timeago.dart' as timeago;

String initials(String name) =>
    name.substring(0, min(2, name.length)).toUpperCase();

String zatToString(BigInt zat) {
  final z = Fixed.fromBigInt(zat, scale: 8);
  return z.toString();
}

BigInt stringToZat(String s) {
  final z = Fixed.parse(s, scale: 8);
  return z.minorUnits;
}

String timeToString(int time) {
  final date = DateTime.fromMillisecondsSinceEpoch(time * 1000);
  return timeago.format(date);
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

Future<void> showSeed(BuildContext context, String message) async {
  await AwesomeDialog(
    context: context,
    dialogType: DialogType.warning,
    animType: AnimType.rightSlide,
    title: 'SEED PHRASE - SAVE IT OR YOU CAN LOSE YOUR FUNDS',
    desc: message,
    btnOkOnPress: () {},
    autoDismiss: true,
  ).show();
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
        autoDismiss: false,
      ).show() ??
      false;
  if (confirmed) {
    final p = password.text;
    return p;
  }
  return null;
}
