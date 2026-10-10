import 'dart:convert';

import 'package:decimal/decimal.dart';
import 'package:zkool/pages/swap.dart';
import 'package:zkool/src/rust/api/account.dart';
import 'package:zkool/src/rust/api/coin.dart';
import 'package:zkool/src/rust/api/near_intents.dart';
import 'package:zkool/src/rust/near_intents.dart' as ni;

DateTime swapDeadline(ni.SavedSwap swap) {
  final response = jsonDecode(swap.quoteResponse) as Map<String, dynamic>;
  final request = response['quoteRequest'] as Map<String, dynamic>?;
  return DateTime.parse(request?['deadline'] as String? ?? swap.deadline);
}

/// Preview only: requesting a quote does not create a deposit or send funds.
Future<SwapReviewQuote> loadSwapQuote(SwapDraft draft, Coin coin) async {
  if (coin.coin != 0) throw StateError('Swaps require a Zcash mainnet wallet.');
  final assets = await nearIntentsAssets(c: coin);
  final networks = draft.network == SwapNetwork.solana ? {'sol', 'solana'} : {'eth', 'ethereum'};
  final matches = assets.where((asset) => asset.symbol.toUpperCase() == draft.asset.symbol && networks.contains(asset.blockchain.toLowerCase())).toList();
  if (matches.length != 1) throw StateError('Unable to identify ${draft.asset.symbol} on ${draft.network.label}.');
  final asset = matches.single;
  final addresses = await getAddresses(uaPools: 0, c: coin);
  final walletAddress = addresses.taddr;
  if (walletAddress == null || walletAddress.isEmpty) throw StateError('This account needs a transparent Zcash address.');
  final deadline = DateTime.now().toUtc().add(const Duration(minutes: 10));
  final request = buildSwapRequest(draft, asset.assetId, asset.decimals, walletAddress, deadline);
  final response = await nearIntentsQuote(request: request, c: coin);
  final quote = response.quote;
  final outgoing = draft.direction == SwapDirection.sendZec;
  final originDecimals = outgoing ? 8 : asset.decimals;
  final destinationDecimals = outgoing ? asset.decimals : 8;
  return SwapReviewQuote(
    request: request,
    destinationDecimals: destinationDecimals,
    originDecimals: originDecimals,
    amountIn: swapDecimalAmount(quote.amountIn, originDecimals),
    amountOut: swapDecimalAmount(quote.amountOut, destinationDecimals),
    minimumReceived: quote.minAmountOut == null ? 'Unavailable' : swapDecimalAmount(quote.minAmountOut!, destinationDecimals),
    estimatedCostUsd: swapEstimatedCostUsd(quote.amountInUsd, quote.amountOutUsd),
    expiresAt: deadline,
  );
}

ni.SwapRequest buildSwapRequest(SwapDraft draft, String assetId, int assetDecimals, String walletAddress, DateTime deadline) {
  final outgoing = draft.direction == SwapDirection.sendZec;
  final originDecimals = outgoing ? 8 : assetDecimals;
  final destinationDecimals = outgoing ? assetDecimals : 8;
  if (!outgoing && (draft.refundTo == null || draft.refundTo!.isEmpty)) throw StateError('Enter a refund address.');
  return ni.SwapRequest(
    dry: true,
    swapType: draft.amountMode == SwapAmountMode.send ? ni.SwapType.exactInput : ni.SwapType.exactOutput,
    originAsset: outgoing ? 'nep141:zec.omft.near' : assetId,
    destinationAsset: outgoing ? assetId : 'nep141:zec.omft.near',
    amount: swapBaseUnits(draft.amount, draft.amountMode == SwapAmountMode.send ? originDecimals : destinationDecimals),
    slippageTolerance: draft.slippageTolerance,
    recipient: outgoing ? draft.recipient : walletAddress,
    refundTo: outgoing ? walletAddress : draft.refundTo!,
    deadline: deadline.toUtc().toIso8601String(),
  );
}

String? swapEstimatedCostUsd(String? amountInUsd, String? amountOutUsd) {
  if (amountInUsd == null || amountOutUsd == null) return null;
  final input = Decimal.tryParse(amountInUsd);
  final output = Decimal.tryParse(amountOutUsd);
  if (input == null || output == null || input < Decimal.zero || output < Decimal.zero) return null;
  return (input - output).toString();
}

int swapSlippageBasisPoints(String value) {
  final percent = value.trim();
  if (!RegExp(r'^\d+(\.\d{1,2})?$').hasMatch(percent)) throw const FormatException('Invalid slippage');
  final parts = percent.split('.');
  final fraction = parts.length == 2 ? parts[1] : '';
  final bps = BigInt.parse('${parts[0]}${fraction.padRight(2, '0')}');
  if (bps > BigInt.from(10000)) throw const FormatException('Invalid slippage');
  return bps.toInt();
}

String swapBaseUnits(String amount, int decimals) {
  if (decimals < 0 || decimals > 255 || !RegExp(r'^\d+(\.\d+)?$').hasMatch(amount)) throw FormatException('Invalid amount');
  final parts = amount.split('.');
  final fraction = parts.length == 2 ? parts[1] : '';
  if (fraction.length > decimals) throw FormatException('Amount has too many decimal places');
  final units = BigInt.parse('${parts[0]}${fraction.padRight(decimals, '0')}');
  if (units <= BigInt.zero) throw FormatException('Amount must be positive');
  return units.toString();
}

String swapDecimalAmount(String units, int decimals) {
  if (decimals < 0 || decimals > 255 || !RegExp(r'^\d+$').hasMatch(units)) throw FormatException('Invalid quote amount');
  final digits = BigInt.parse(units).toString().padLeft(decimals + 1, '0');
  if (decimals == 0) return digits;
  final split = digits.length - decimals;
  final fraction = digits.substring(split).replaceFirst(RegExp(r'0+$'), '');
  return fraction.isEmpty ? digits.substring(0, split) : '${digits.substring(0, split)}.$fraction';
}
