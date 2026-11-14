import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:zkool/pages/key_tree.dart';
import 'package:zkool/src/rust/api/account.dart';
import 'package:zkool/src/rust/api/coin.dart';
import 'package:zkool/src/rust/frb_generated.dart';
import 'package:zkool/store.dart';
import 'package:zkool/utils.dart';

class _FakeCoin implements Coin {
  @override
  dynamic noSuchMethod(Invocation invocation) => super.noSuchMethod(invocation);
}

class _Api implements RustLibApi {
  KeyNode? tree;

  @override
  Coin crateApiCoinCoinNew({int? defaultCoin}) => _FakeCoin();

  @override
  Future<KeyNode?> crateApiAccountGetAccountKeyTree({required int account, required Coin c}) async => tree;

  @override
  dynamic noSuchMethod(Invocation invocation) => super.noSuchMethod(invocation);
}

class _Unlocked extends Lifecycle {
  @override
  Future<bool> build() async => false;
}

KeyFace _face(String label, String? value, {String? note}) => KeyFace(label: label, value: value, note: note);

KeyNode _node(String name, KeyFace secret, KeyFace viewing, {String? detail, List<KeyNode> children = const []}) =>
    KeyNode(name: name, summary: false, secret: secret, viewing: viewing, detail: detail, children: children);

/// A seed account's tree, as the Rust side shapes it.
final _seedTree = KeyNode(
  name: 'Seed',
  summary: true,
  secret: _face('Seed phrase', 'word1 word2'),
  viewing: _face('Unified full viewing key', 'uview-root'),
  detail: null,
  children: [
    _node('Account index', _face('Account index', '0'), _face('Account index', null)),
    _node(
      'Transparent',
      _face('Transparent xprv', 'xprv-key'),
      _face('Transparent xpub', 'xpub-key'),
      children: [
        _node('External #0', _face('Transparent WIF', 'wif-0'), _face('Transparent public key', 'zpk-0'), detail: 't1addr'),
      ],
    ),
    _node(
      'Orchard',
      _face('Orchard spending key', null, note: 'orchard-secret-note'),
      _face('Orchard viewing key', null, note: 'orchard-viewing-note'),
    ),
  ],
);

/// A viewing-only account: a root with no secret face.
final _viewingTree = KeyNode(
  name: 'Sapling',
  summary: true,
  secret: _face('Sapling spending key', null),
  viewing: _face('Sapling viewing key', 'zxviews-key'),
  detail: null,
  children: const [],
);

Finder _copyable(String text) => find.byWidgetPredicate((w) => w is CopyableText && w.text == text);

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();
  final api = _Api();
  setUpAll(() => RustLib.initMock(api: api));
  tearDownAll(RustLib.dispose);

  Future<void> open(WidgetTester tester, KeyNode tree) async {
    api.tree = tree;
    await tester.pumpWidget(ProviderScope(
      overrides: [lifecycleProvider.overrideWith(_Unlocked.new)],
      child: const MaterialApp(home: KeyTreePage(1)),
    ));
    await tester.pumpAndSettle();
  }

  testWidgets('the switch flips every face and keeps the expansion', (tester) async {
    await open(tester, _seedTree);

    // Secret mode: the root open one level, the address list closed.
    expect(_copyable('word1 word2'), findsOneWidget);
    expect(_copyable('0'), findsOneWidget);
    expect(_copyable('xprv-key'), findsOneWidget);
    expect(_copyable('wif-0'), findsNothing, reason: 'addresses start closed');
    expect(find.text('N/A'), findsOneWidget, reason: 'Orchard has no secret face');
    expect(find.text('orchard-secret-note'), findsOneWidget);
    expect(find.byIcon(Icons.warning_amber), findsOneWidget);

    // Open the addresses.
    await tester.tap(find.byIcon(Icons.chevron_right));
    await tester.pumpAndSettle();
    expect(_copyable('wif-0'), findsOneWidget);
    expect(find.text('t1addr'), findsOneWidget);

    // Viewing mode: same rows, other faces, the addresses still open.
    await tester.tap(find.byType(Switch));
    await tester.pumpAndSettle();
    expect(_copyable('uview-root'), findsOneWidget);
    expect(_copyable('word1 word2'), findsNothing);
    expect(_copyable('xpub-key'), findsOneWidget);
    expect(_copyable('zpk-0'), findsOneWidget, reason: 'the expansion survives the flip');
    expect(_copyable('wif-0'), findsNothing);
    expect(find.text('t1addr'), findsOneWidget, reason: 'the address is on both faces');
    expect(find.text('N/A'), findsNWidgets(2), reason: 'the index and Orchard have no viewing face');
    expect(find.text('orchard-viewing-note'), findsOneWidget);
    expect(find.byIcon(Icons.info_outline), findsOneWidget);
    expect(find.text('External #0'), findsOneWidget);
  });

  // #1642: rows were laid out at heights guessed from the text, which a key
  // filling its last line outgrew, and the indented rows ran past the edge.
  // The test font draws every glyph as a square of the font size, so at
  // each width below some key fills its line to within a cursor's width.
  testWidgets('a row fits its key at every width and stays inside the list', (tester) async {
    final rootKey = 'r' * 28;
    final wif = 'w' * 70;
    final tree = KeyNode(
      name: 'Seed',
      summary: true,
      secret: _face('Seed phrase', rootKey),
      viewing: _face('Unified full viewing key', 'uview-root'),
      detail: null,
      children: [
        _node(
          'Transparent',
          _face('Transparent xprv', 'x' * 45),
          _face('Transparent xpub', 'xpub-key'),
          children: [
            _node('External #0', _face('Transparent WIF', wif), _face('Transparent public key', 'zpk-0'), detail: 't1addr'),
          ],
        ),
      ],
    );
    addTearDown(tester.view.reset);
    tester.view.devicePixelRatio = 1;
    for (var width = 440.0; width <= 480; width += 1) {
      tester.view.physicalSize = Size(width, 1200);
      // A fresh page each time, with the addresses closed again.
      await tester.pumpWidget(const SizedBox());
      await open(tester, tree);
      await tester.tap(find.byIcon(Icons.chevron_right));
      await tester.pumpAndSettle();
      expect(tester.takeException(), isNull, reason: 'no row overflows at width $width');
      for (final key in [rootKey, wif]) {
        final rect = tester.getRect(_copyable(key));
        expect(rect.right, lessThanOrEqualTo(width - 16), reason: '$key stays inside the list at width $width');
      }
    }
  });

  testWidgets('a viewing-only account opens on its viewing face without a switch', (tester) async {
    await open(tester, _viewingTree);
    expect(find.byType(Switch), findsNothing);
    expect(_copyable('zxviews-key'), findsOneWidget);
    expect(find.text('N/A'), findsNothing);
  });
}
