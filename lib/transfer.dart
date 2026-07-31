import 'dart:typed_data';

import 'package:flutter_rust_bridge/flutter_rust_bridge.dart';

import 'package:zkool/src/rust/api/account.dart';
import 'package:zkool/src/rust/api/coin.dart';
import 'package:zkool/src/rust/api/pay.dart';
import 'package:zkool/src/rust/pay.dart';

Future<PcztPackage> transferAllBetweenPools({
  required Coin c,
  required int sourcePools,
  required String destinationAddress,
  int? destinationPools,
  bool smartTransparent = false,
}) async {
  if (destinationAddress.isEmpty) {
    throw AnyhowException("Destination address is unavailable.");
  }

  final notes = await listNotes(c: c);
  final amount = notes
      .where(
        (note) =>
            (sourcePools & (1 << note.pool)) != 0 &&
            !note.locked &&
            note.idAsset == null &&
            note.value >= BigInt.from(5000),
      )
      .fold(BigInt.zero, (total, note) => total + note.value);
  if (amount == BigInt.zero) {
    throw AnyhowException("No economically spendable ZEC notes are available.");
  }

  return prepare(
    recipients: [
      Recipient(
        address: destinationAddress,
        amount: amount,
        pools: destinationPools,
        assetBase: Uint8List(32),
      ),
    ],
    options: PaymentOptions(
      srcPools: sourcePools,
      recipientPaysFee: true,
      smartTransparent: smartTransparent,
    ),
    c: c,
  );
}
