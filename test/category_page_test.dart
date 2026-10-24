import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:zkool/pages/category.dart';
import 'package:zkool/src/rust/api/account.dart';
import 'package:zkool/src/rust/api/coin.dart';
import 'package:zkool/src/rust/frb_generated.dart';

class _FakeCoin implements Coin {
  @override
  dynamic noSuchMethod(Invocation invocation) => super.noSuchMethod(invocation);
}

/// The default categories, as the wallet lists them: expenses, then income,
/// each by name.
const _expenses = [
  "Debt & Financial Obligations",
  "Education & Training",
  "Entertainment & Lifestyle",
  "Food & Groceries",
  "Health & Insurance",
  "Housing & Utilities",
  "Other Expenses",
  "Personal & Family Care",
  "Restaurants & Coffee",
  "Savings & Investments",
  "Transportation & Hotels",
];
const _income = ["Investment Income/Mining", "Other Income", "Rental/Property Income", "Salary"];

class _Api implements RustLibApi {
  @override
  Coin crateApiCoinCoinNew({int? defaultCoin}) => _FakeCoin();

  @override
  Future<List<Category>> crateApiAccountListCategories({required Coin c}) async => [
        for (final (i, name) in _expenses.indexed) Category(id: i + 1, name: name, isIncome: false),
        for (final (i, name) in _income.indexed) Category(id: _expenses.length + i + 1, name: name, isIncome: true),
      ];

  @override
  dynamic noSuchMethod(Invocation invocation) => super.noSuchMethod(invocation);
}

void main() {
  setUpAll(() => RustLib.initMock(api: _Api()));
  tearDownAll(RustLib.dispose);

  testWidgets('the list lays out every category, with one divider between the groups', (tester) async {
    // Tall enough for every row: a list builds only what is in view, and
    // the separator after the first row is what used to break the layout.
    tester.view.physicalSize = const Size(800, 1400);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.reset);

    await tester.pumpWidget(const ProviderScope(child: MaterialApp(home: CategoryPage())));
    await tester.pumpAndSettle();

    expect(find.byType(ListTile), findsNWidgets(_expenses.length + _income.length));
    expect(find.byType(Divider), findsOneWidget);
    final divider = tester.getRect(find.byType(Divider));
    expect(tester.getRect(find.text(_expenses.last)).bottom, lessThanOrEqualTo(divider.top));
    expect(tester.getRect(find.text(_income.first)).top, greaterThanOrEqualTo(divider.bottom));
  });
}
