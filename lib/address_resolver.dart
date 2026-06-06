import 'package:zkool/src/rust/api/account.dart';
import 'package:zkool/src/rust/api/coin.dart';

/// Resolves an input string that starts with `@` to a Zcash address.
///
/// If [input] starts with `@`, the remainder is treated as an account name.
/// The function looks up the account by name (case-insensitive) in [accounts]
/// and returns its address. Returns `null` if the input is not an account
/// reference (no `@` prefix), if no matching account is found, or if the
/// account has no addresses.
///
/// Address priority: UA → Orchard → Sapling → Transparent
Future<String?> resolveAccountName(
  String input,
  List<Account> accounts,
  Coin c,
) async {
  if (!input.startsWith('@') || input.length < 2) {
    return null;
  }

  final name = input.substring(1).trim().toLowerCase();
  if (name.isEmpty) return null;

  final account = accounts.cast<Account?>().firstWhere(
    (a) => a!.name.toLowerCase() == name,
    orElse: () => null,
  );

  if (account == null) return null;

  final addresses = await getAccountAddresses(
    account: account.id,
    uaPools: 7, // all pools
    c: c,
  );

  // Return the best available address
  return addresses.ua ??
      addresses.oaddr ??
      addresses.saddr ??
      addresses.taddr;
}
