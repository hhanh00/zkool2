import 'dart:math';
import 'package:fixed/fixed.dart';

String initials(String name) => name.substring(0, min(2, name.length)).toUpperCase();

String zatToString(BigInt zat) {
  final z = Fixed.fromBigInt(zat, scale: 8);
  return z.toString();
}
