import 'package:zkool/services/continuation_context.dart';
import 'dart:typed_data';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import 'package:zkool/chart.dart';
import 'package:zkool/pages/category.dart';
import 'package:zkool/pages/contact_editor.dart';
import 'package:zkool/pages/contacts.dart';
import 'package:zkool/pages/folder.dart';
import 'package:zkool/pages/account.dart';
import 'package:zkool/pages/accounts.dart';
import 'package:zkool/pages/currency.dart';
import 'package:zkool/pages/db.dart';
import 'package:zkool/pages/disclaimer.dart';
import 'package:zkool/pages/dkg.dart';
import 'package:zkool/pages/frost.dart';
import 'package:zkool/pages/log.dart';
import 'package:zkool/pages/market.dart';
import 'package:zkool/pages/new_account.dart';
import 'package:zkool/pages/raptor.dart';
import 'package:zkool/pages/receive.dart';
import 'package:zkool/pages/send.dart';
import 'package:zkool/pages/swap.dart';
import 'package:zkool/pages/swap_summary.dart';
import 'package:zkool/pages/swaps.dart';
import 'package:zkool/pages/splash.dart';
import 'package:zkool/pages/tx.dart';
import 'package:zkool/pages/tx_view.dart';
import 'package:zkool/pages/migrate.dart';
import 'package:zkool/pages/voting.dart';
import 'package:zkool/pages/voting_round.dart';
import 'package:zkool/pages/zsa.dart';
import 'package:zkool/pages/lwd_select.dart';
import 'package:zkool/pages/plugin_manager.dart';
import 'package:zkool/settings.dart';
import 'package:zkool/src/rust/api/account.dart';
import 'package:zkool/src/rust/api/coin.dart';
import 'package:zkool/src/rust/api/contacts.dart';
import 'package:zkool/src/rust/api/pay.dart';
import 'package:zkool/src/rust/api/voting.dart';
import 'package:zkool/src/rust/pay.dart';
import 'package:zkool/store.dart';
import 'package:zkool/widgets/scanner.dart';

final navigatorKey = GlobalKey<NavigatorState>();
final accountEditPageKey = GlobalKey<AccountEditPageState>();
final RouteObserver<ModalRoute<void>> routeObserver = RouteObserver<ModalRoute<void>>();

/// Location of an account-scoped page. The account travels as a query
/// parameter so it is visible in the location; `extra` stays for data that
/// cannot be encoded in a URL.
String accountRoute(String path, int account) =>
    Uri(path: path, queryParameters: {'account': account.toString()}).toString();

extension AccountRouteState on GoRouterState {
  /// The account this route was opened for, when it carries one.
  int? get accountIdOrNull => int.tryParse(uri.queryParameters['account'] ?? '');

  /// The account this route was opened for. Account-scoped routes are always
  /// opened through [accountRoute], so a missing parameter is a bug.
  int get accountId {
    final id = accountIdOrNull;
    if (id == null) throw StateError('route $uri has no account');
    return id;
  }
}

GoRouter router(bool disclaimerAccepted, bool recoveryMode, {String? initialLocation}) => GoRouter(
      initialLocation: initialLocation ??
          (!disclaimerAccepted
              ? '/disclaimer'
              : recoveryMode
                  ? '/database_manager'
                  : '/splash'),
      observers: [routeObserver],
      navigatorKey: navigatorKey,
      routes: [
        GoRoute(
          path: '/',
          builder: (context, state) => AccountViewPage(
            account: state.accountIdOrNull ?? ProviderScope.containerOf(context).read(selectedAccountIdProvider),
          ),
        ),
        GoRoute(
          path: '/accounts',
          builder: (context, state) => AccountListPage(),
        ),
        GoRoute(
          path: '/account/edit',
          builder: (context, state) => AccountEditPage(state.extra as List<Account>, key: accountEditPageKey),
          onExit: (context, state) => accountEditPageKey.currentState?.confirmLeave() ?? true,
        ),
        GoRoute(
          path: '/account/new',
          builder: (context, state) => NewAccountPage(),
        ),
        GoRoute(
          path: '/viewing_keys',
          builder: (context, state) => ViewingKeysPage(state.extra as int),
        ),
        GoRoute(
          path: '/receive',
          builder: (context, state) => ReceivePage(account: state.accountId),
        ),
        GoRoute(
          path: '/addresses',
          builder: (context, state) {
            final extra = state.extra as Map<String, dynamic>;
            return AddressesPage(
              txCounts: extra['txCounts'] as List<TAddressTxCount>,
              availablePools: extra['availablePools'] as int,
              c: extra['c'] as Coin,
              account: state.accountId,
            );
          },
        ),
        GoRoute(
          path: '/send',
          builder: (context, state) => SendPage(account: state.accountId),
        ),
        GoRoute(
          path: '/send2',
          builder: (context, state) {
            final (recipients, recipientPaysFee) = state.extra as (List<Recipient>, bool);
            return Send2Page(recipients, account: state.accountId, recipientPaysFee: recipientPaysFee);
          },
        ),
        GoRoute(
            path: '/tx',
            builder: (context, state) {
              final extra = state.extra;
              return extra is TxPageArgs
                  ? TxPage(extra.pczt, continuation: extra.continuation, account: state.accountId)
                  : TxPage(extra as PcztPackage, account: state.accountId);
            }),
        GoRoute(path: '/tx_view', builder: (context, state) => TxViewPage(state.extra as int, account: state.accountId)),
        GoRoute(path: '/log', builder: (context, state) => LogviewPage()),
        GoRoute(path: '/scanner', builder: (context, state) => ScannerPage(validator: state.extra as String? Function(String?))),
        GoRoute(
          path: '/qr',
          builder: (context, state) {
            final args = state.extra as Map<String, dynamic>;
            return QRPage(text: args["text"], title: args["title"]);
          },
        ),
        GoRoute(path: '/splash', builder: (context, state) => SplashPage()),
        GoRoute(path: '/swap', builder: (context, state) => SwapPage(account: state.accountId)),
        GoRoute(
          path: '/swap/review',
          builder: (context, state) {
            final (draft, quoteLoader) = state.extra as (SwapDraft, SwapQuoteLoader?);
            return SwapReviewPage(draft: draft, quoteLoader: quoteLoader, account: state.accountId);
          },
        ),
        GoRoute(
          path: '/swap/summary',
          builder: (context, state) {
            final args = state.extra as SwapSummaryArgs;
            return SwapSummaryPage(
              account: state.accountId,
              swap: args.swap,
              symbol: args.symbol,
              decimals: args.decimals,
              originSymbol: args.originSymbol,
              originDecimals: args.originDecimals,
              depositNetwork: args.depositNetwork,
            );
          },
        ),
        GoRoute(path: '/swaps', builder: (context, state) => SwapsPage(account: state.accountId)),
        GoRoute(path: '/market', builder: (context, state) => MarketPrice()),
        GoRoute(path: '/mempool', builder: (context, state) => MempoolPage()),
        GoRoute(path: '/mempool_view', builder: (context, state) => MempoolTxViewPage(state.extra as Uint8List)),
        GoRoute(path: '/folders', builder: (context, state) => FolderPage()),
        GoRoute(path: '/categories', builder: (context, state) => CategoryPage()),
        GoRoute(path: '/contacts', builder: (context, state) => ContactListPage()),
        GoRoute(path: '/contact/edit', builder: (context, state) => ContactEditPage(contact: state.extra as Contact?)),
        GoRoute(path: '/dkg1', builder: (context, state) => DKGPage1()),
        GoRoute(path: '/dkg2', builder: (context, state) => DKGPage2()),
        GoRoute(path: '/dkg3', builder: (context, state) => DKGPage3()),
        GoRoute(path: '/frost1', builder: (context, state) => FrostPage1(state.extra as PcztPackage, account: state.accountId)),
        GoRoute(path: '/frost2', builder: (context, state) => FrostPage2(account: state.accountId)),
        GoRoute(
            path: '/settings',
            routes: [
              GoRoute(path: 'qr', builder: (context, state) => SettingsQRPage(onClose: state.extra as VoidFunction<QRSettings>)),
              GoRoute(
                  path: 'theme',
                  builder: (context, state) {
                    final onClose = state.extra as void Function((String, bool));
                    return SettingsThemePage(onClose: onClose);
                  }),
              GoRoute(
                  path: 'currency',
                  builder: (context, state) {
                    final onClose = state.extra as void Function(String);
                    return CurrencyPage(onClose: onClose);
                  }),
              GoRoute(path: 'plugins', builder: (context, state) => const PluginManagerPage()),
            ],
            builder: (context, state) => SettingsPage()),
        GoRoute(path: '/database_manager', builder: (context, state) => DatabaseManagerPage()),
        GoRoute(path: '/disclaimer', builder: (context, state) => DisclaimerPage()),
        GoRoute(path: '/chart', builder: (context, state) => ChartPage(account: state.accountId)),
        GoRoute(path: '/lwd_select', builder: (context, state) => const LWDSelectPage()),
        GoRoute(path: '/show_animated_qr', builder: (context, state) => ShowAnimatedQRPage(state.extra as List<Uint8List>)),
        GoRoute(path: '/migrate', builder: (context, state) => MigratePage(account: state.accountId)),
        GoRoute(path: '/voting', builder: (context, state) => VotingPage(account: state.accountId)),
        GoRoute(
          path: '/voting/round',
          builder: (context, state) => VotingRoundPage(
            account: state.accountId,
            round: state.extra as VotingRoundListItem,
          ),
        ),
        GoRoute(path: '/zsa', builder: (context, state) => ZsaHoldingsPage(account: state.accountId)),
        GoRoute(path: '/zsa/issue', builder: (context, state) => IssueAssetPage(args: state.extra as IssuanceArgs?, account: state.accountId)),
        GoRoute(path: '/scan_animated_qr', builder: (context, state) => ScanAnimatedQRPage()),
      ],
    );

// @riverpod
// class PinLocked extends _$PinLocked {
//   @override
//   Future<bool> build() async {
//     final settings = await ref.watch(appSettingsProvider.future);
//     return settings.needPin;
//   }

//   void unlock() {
//     // state = state.whenData();
//     Future(() {
//       relock();
//     });
//   }

//   void relock() {
//     final settings = ref.read(appSettingsProvider).requireValue;
//     state = state.whenData((s) => settings.needPin);
//   }
// }
