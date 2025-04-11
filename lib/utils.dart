import 'dart:math';
import 'dart:typed_data';
import 'package:convert/convert.dart';
import 'package:fixed/fixed.dart';
import 'package:timeago/timeago.dart' as timeago;

String initials(String name) => name.substring(0, min(2, name.length)).toUpperCase();

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
