import 'package:flutter_test/flutter_test.dart';
import 'package:zkool/src/rust/api/network.dart';
import 'package:zkool/store.dart';

// The server list page returns a ServerChoice and changes nothing itself;
// the settings page applies it to its draft (#1581, V12 F-199).

LWDInfo server(String url, {bool isTor = false}) => LWDInfo(
      url: url,
      isTor: isTor,
      height: 100,
      status: 'online',
      uptime: 99,
      version: '',
      ping: 10,
    );

AppSettings draft({int transport = 3, bool isLightNode = false}) => AppSettings(
      dbName: '',
      net: '',
      isLightNode: isLightNode,
      lwd: 'https://old.example',
      blockExplorer: '',
      syncInterval: '',
      actionsPerSync: '',
      transport: transport,
      proxy: 'socks5h://127.0.0.1:9050',
      coingecko: '',
      recovery: false,
      needPin: false,
      pinUnlockedAt: DateTime(2026),
      offline: false,
      getFx: false,
      qrSettings: QRSettings(enabled: false, size: 100, ecLevel: 0, delay: 0, repair: 0),
      vault: false,
      expertMode: false,
      paletteName: '',
      darkMode: false,
      transactionTableMode: false,
      collapsePoolBalances: false,
      currency: '',
      votingConfigUrl: '',
      marketInterval: '1d',
      marketStyle: 'candles',
    );

void main() {
  group('ServerChoice.forServer', () {
    test('a Tor server gets the http scheme and the Tor transport', () {
      final choice = ServerChoice.forServer(server('abc.onion:9067', isTor: true));
      expect(choice.url, 'http://abc.onion:9067');
      expect(choice.transport, 1);
    });

    test('an ordinary server gets https and leaves the transport alone', () {
      final choice = ServerChoice.forServer(server('zec.rocks:443'));
      expect(choice.url, 'https://zec.rocks:443');
      expect(choice.transport, isNull);
    });

    test('a URL that already has a scheme is kept', () {
      expect(ServerChoice.forServer(server('http://plain.example:9067')).url, 'http://plain.example:9067');
    });
  });

  test('a Nym endpoint carries the Direct transport', () {
    final choice = ServerChoice.nym('nym://id.enc@gateway');
    expect(choice.url, 'nym://id.enc@gateway');
    expect(choice.transport, 0);
  });

  group('applyTo', () {
    test('sets the URL and keeps the draft transport when the choice has none', () {
      final applied = ServerChoice.forServer(server('zec.rocks:443')).applyTo(draft(transport: 3));
      expect(applied.lwd, 'https://zec.rocks:443');
      expect(applied.transport, 3);
      expect(applied.proxy, 'socks5h://127.0.0.1:9050', reason: 'other settings are untouched');
    });

    test('sets the transport the choice carries', () {
      expect(ServerChoice.forServer(server('abc.onion', isTor: true)).applyTo(draft(transport: 3)).transport, 1);
      expect(ServerChoice.nym('nym://id.enc@gateway').applyTo(draft(transport: 1)).transport, 0);
    });
  });
}
