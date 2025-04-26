import 'package:convert/convert.dart';
import 'package:fixed/fixed.dart';
import 'package:zkool/src/rust/api/key.dart';
import 'package:zkool/src/rust/api/pay.dart';

String? validKey(String? key, {bool restore = false}) {
  if ((key == null || key.isEmpty)) {
    return restore ? "Key is required" : null;
  }
  if (!isValidKey(key: key)) {
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
  final checkAddress = validAddress(s);
  if (checkAddress == null) return null;
  final checkURI = validPaymentURI(s);
  if (checkURI == null) return null;
  return "Invalid Address or Payment URI";
}

String? validAmount(String? amount) {
  if ((amount == null || amount.isEmpty)) {
    return "Amount is required";
  }
  final a = Fixed.tryParse(amount);
  if (a == null) {
    return "Invalid Amount";
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
