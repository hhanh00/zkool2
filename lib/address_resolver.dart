import 'package:zkool/src/rust/api/account.dart';
import 'package:zkool/src/rust/api/coin.dart';
import 'package:zkool/src/rust/api/contacts.dart';

/// Names are exact: `@alice` is the contact named `alice`, not `Alice`.
/// Only the suggestion list, while typing, matches case-insensitively, and
/// picking from it resolves by the item picked, not by its name.
///
/// Every item of [items] whose name is exactly [name].
List<T> allNamed<T>(String name, Iterable<T> items, String Function(T) nameOf) =>
    items.where((item) => nameOf(item) == name).toList();

/// The one item of [items] named exactly [name], or null when there is
/// none or more than one.
T? uniqueNamed<T>(String name, Iterable<T> items, String Function(T) nameOf) {
  final matches = allNamed(name, items, nameOf);
  return matches.length == 1 ? matches.single : null;
}

/// The name an `!account` or `@contact` input refers to, or null when the
/// input has no such prefix or nothing after it.
String? referencedName(String input, String prefix) {
  if (!input.startsWith(prefix) || input.length < 2) return null;
  final name = input.substring(1).trim();
  return name.isEmpty ? null : name;
}

/// The address to pay [account] at: UA → Orchard → Sapling → Transparent.
Future<String?> accountAddress(Account account, Coin c) async {
  final addresses = await getAccountAddresses(
    account: account.id,
    uaPools: 7, // all pools
    c: c,
  );
  return addresses.ua ?? addresses.oaddr ?? addresses.saddr ?? addresses.taddr;
}

/// The address to pay [contact] at: its first one.
String? contactAddress(Contact contact) =>
    contact.addresses.isEmpty ? null : contact.addresses.first;

/// Resolves an input string that starts with `!` to a Zcash address.
///
/// The remainder is an account name, matched exactly against [accounts].
/// Returns `null` if the input is not an account reference, if no account
/// has that name or several do (see [allNamed] to tell the two apart), or
/// if the account has no addresses.
Future<String?> resolveAccountName(
  String input,
  List<Account> accounts,
  Coin c,
) async {
  final name = referencedName(input, '!');
  if (name == null) return null;
  final account = uniqueNamed(name, accounts, (a) => a.name);
  if (account == null) return null;
  return accountAddress(account, c);
}

/// Resolves an input string that starts with `@` to a Zcash address.
///
/// The remainder is a contact name, matched exactly against [contacts].
/// Returns `null` if the input is not a contact reference, if no contact
/// has that name, or if the contact has no addresses.
String? resolveContactName(
  String input,
  List<Contact> contacts,
) {
  final name = referencedName(input, '@');
  if (name == null) return null;
  final contact = uniqueNamed(name, contacts, (c) => c.name);
  if (contact == null) return null;
  return contactAddress(contact);
}
