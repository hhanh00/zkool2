import 'package:fixed/fixed.dart';
import 'package:zkool/src/rust/api/key.dart';

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
    return "Address is required";
  }
  if (!isValidAddress(address: address)) {
    return "Invalid Address";
  }
  return null;
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
