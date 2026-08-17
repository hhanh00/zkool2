import 'dart:typed_data';

import 'package:flutter/material.dart';
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
import 'package:zkool/pages/splash.dart';
import 'package:zkool/pages/tx.dart';
import 'package:zkool/pages/tx_view.dart';
import 'package:zkool/pages/migrate.dart';
import 'package:zkool/pages/voting_polls.dart';
import 'package:zkool/pages/voting_proposal.dart';
import 'package:zkool/pages/voting_review.dart';
import 'package:zkool/pages/voting_confirmation.dart';
import 'package:zkool/pages/voting_results.dart';
import 'package:zkool/pages/voting_status.dart';
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
final RouteObserver<ModalRoute<void>> routeObserver = RouteObserver<ModalRoute<void>>();

GoRouter router(bool disclaimerAccepted, bool recoveryMode) => GoRouter(
      initialLocation: !disclaimerAccepted
          ? '/disclaimer'
          : recoveryMode
              ? '/database_manager'
              : '/splash',
      observers: [routeObserver],
      navigatorKey: navigatorKey,
      routes: [
        GoRoute(
          path: '/',
          builder: (context, state) => AccountViewPage(),
        ),
        GoRoute(
          path: '/accounts',
          builder: (context, state) => AccountListPage(),
        ),
        GoRoute(
          path: '/account/edit',
          builder: (context, state) => AccountEditPage(state.extra as List<Account>),
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
          builder: (context, state) => ReceivePage(),
        ),
        GoRoute(
          path: '/addresses',
          builder: (context, state) {
            final extra = state.extra as Map<String, dynamic>;
            return AddressesPage(
              txCounts: extra['txCounts'] as List<TAddressTxCount>,
              availablePools: extra['availablePools'] as int,
              c: extra['c'] as Coin,
            );
          },
        ),
        GoRoute(
          path: '/send',
          builder: (context, state) => SendPage(),
        ),
        GoRoute(
          path: '/send2',
          builder: (context, state) {
            final (recipients, recipientPaysFee) = state.extra as (List<Recipient>, bool);
            return Send2Page(recipients, recipientPaysFee: recipientPaysFee);
          },
        ),
        GoRoute(path: '/tx', builder: (context, state) => TxPage(state.extra as PcztPackage)),
        GoRoute(path: '/tx_view', builder: (context, state) => TxViewPage(state.extra as int)),
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
        GoRoute(path: '/frost1', builder: (context, state) => FrostPage1(state.extra as PcztPackage)),
        GoRoute(path: '/frost2', builder: (context, state) => FrostPage2()),
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
        GoRoute(path: '/chart', builder: (context, state) => ChartPage()),
        GoRoute(path: '/lwd_select', builder: (context, state) => const LWDSelectPage()),
        GoRoute(path: '/show_animated_qr', builder: (context, state) => ShowAnimatedQRPage(state.extra as List<Uint8List>)),
        GoRoute(path: '/migrate', builder: (context, state) => const MigratePage()),
        GoRoute(path: '/voting', builder: (context, state) => const VotingPollsPage()),
        GoRoute(
          path: '/voting/proposal',
          builder: (context, state) {
            final args = state.extra as Map<String, dynamic>;
            return VotingProposalPage(
              roundId: args['roundId'] as String,
              chainUrl: args['chainUrl'] as String,
            );
          },
        ),
        GoRoute(
          path: '/voting/review',
          builder: (context, state) {
            final args = state.extra as Map<String, dynamic>;
            return VotingReviewPage(
              roundId: args['roundId'] as String,
              chainUrl: args['chainUrl'] as String,
              roundParamsJson: args['roundParamsJson'] as String?,
              roundName: args['roundName'] as String?,
              snapshotHeight: args['snapshotHeight'] as int?,
            );
          },
        ),
        GoRoute(
          path: '/voting/confirmation',
          builder: (context, state) {
            final args = state.extra as Map<String, dynamic>;
            return VotingConfirmationPage(
              roundId: args['roundId'] as String,
              roundName: args['roundName'] as String?,
              chainUrl: args['chainUrl'] as String? ?? "",
            );
          },
        ),
        GoRoute(
          path: '/voting/results',
          builder: (context, state) {
            final args = state.extra as Map<String, dynamic>;
            return VotingResultsPage(
              roundId: args['roundId'] as String,
              chainUrl: args['chainUrl'] as String,
            );
          },
        ),
        GoRoute(
          path: '/voting/status',
          builder: (context, state) {
            final args = state.extra as Map<String, dynamic>;
            return VotingStatusPage(
              roundId: args['roundId'] as String,
              chainUrl: args['chainUrl'] as String,
              pirServerUrl: args['pirServerUrl'] as String,
              pirLayout: args['pirLayout'] as VotingPirLayout?,
              roundParamsJson: args['roundParamsJson'] as String?,
              roundName: args['roundName'] as String?,
              maxRealNotesPerBundle: args['maxRealNotesPerBundle'] as int?,
              lightwalletdUrl: args['lightwalletdUrl'] as String?,
              voteNodeUrl: args['voteNodeUrl'] as String? ?? "",
              ceremonyStart: args['ceremonyStart'] as int? ?? 0,
              voteEnd: args['voteEnd'] as int?,
              shareServerUrls: (args['shareServerUrls'] as List<String>?) ?? const [],
              singleShare: args['singleShare'] as bool? ?? false,
            );
          },
        ),
        GoRoute(path: '/zsa', builder: (context, state) => const ZsaHoldingsPage()),
        GoRoute(path: '/zsa/issue', builder: (context, state) => IssueAssetPage(args: state.extra as IssuanceArgs?)),
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
