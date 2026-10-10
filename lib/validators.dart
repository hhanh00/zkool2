import 'package:convert/convert.dart';
import 'package:fixed/fixed.dart';
import 'package:zkool/src/rust/api/coin.dart';
import 'package:zkool/src/rust/api/key.dart';
import 'package:zkool/src/rust/api/pay.dart';
import 'package:zkool/utils.dart';

String? validKey(String? key, {required Coin c, bool restore = false}) {
  if ((key == null || key.isEmpty)) {
    return restore ? "Key is required" : null;
  }
  if (!isValidKey(key: key, c: c)) {
    return "Invalid Key";
  }
  return null;
}

String? validAddress(String? address) {
  if ((address == null || address.isEmpty)) {
    return null;
  }
  if (!isValidAddress(address: address)) {
    return "Invalid Address";
  }
  return null;
}

String? validPaymentURI(String? uri) {
  if ((uri == null || uri.isEmpty)) {
    return null;
  }
  final recipient = parsePaymentUri(uri: uri);
  if (recipient == null) {
    return "Invalid Payment URI";
  }
  return null;
}

String? validAddressOrPaymentURI(String? s) {
  if ((s == null || s.isEmpty)) {
    return null;
  }
  // Allow !accountname references — resolution happens later
  if (s.startsWith('!') && s.length > 1) {
    return null;
  }
  // Allow @contactname references — resolution happens later
  if (s.startsWith('@') && s.length > 1) {
    return null;
  }
  // Allow #alias OpenAlias references — resolution happens via DNS
  if (s.startsWith('#') && s.length > 1) {
    return null;
  }
  final checkAddress = validAddress(s);
  if (checkAddress == null) return null;
  final checkURI = validPaymentURI(s);
  if (checkURI == null) return null;
  return "Invalid Address or Payment URI";
}

class ParsedAmount {
  final String normalized;
  final BigInt units;

  const ParsedAmount({required this.normalized, required this.units});
}

/// Parses a nonnegative localized amount using the wallet's Fixed library.
/// Check precision before rescaling so extra decimals are never rounded away.
ParsedAmount? parseAmount(String? amount, {required int decimals}) {
  assert(decimals >= 0);
  if (amount == null || amount.trim().isEmpty) return null;
  try {
    final parsed = Fixed.parse(amount.trim(), invertSeparator: invertSeparator);
    if (parsed.minorUnits < BigInt.zero || parsed.decimalDigits > decimals) return null;
    final scaled = parsed.copyWith(decimalDigits: decimals);
    return ParsedAmount(normalized: parsed.toString(), units: scaled.minorUnits);
  } on FixedException {
    return null;
  } on FormatException {
    return null;
  } on RangeError {
    return null;
  }
}

/// Requires a positive amount and returns its decimal-point form.
String? normalizeCryptoAmount(String? amount, {required int decimals}) {
  final parsed = parseAmount(amount, decimals: decimals);
  return parsed != null && parsed.units > BigInt.zero ? parsed.normalized : null;
}

/// Optional ZEC amount, with an optional maximum expressed in zatoshis.
String? validAmount(String? amount, {BigInt? max}) {
  if (amount == null || amount.isEmpty) return null;
  final parsed = parseAmount(amount, decimals: 8);
  if (parsed == null) return "Invalid Amount";
  if (max != null && parsed.units > max) {
    return "Amount ${parsed.units} exceeds maximum of $max";
  }
  return null;
}

String? validHexString(String? s, int lenth) {
  if (s == null) return null;
  try {
    final bytes = hex.decode(s);
    if (bytes.length != lenth) return "Invalid length";
  } on FormatException {
    return "Not a valid hex string";
  }
  return null;
}
