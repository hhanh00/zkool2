import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated.dart';
import 'package:flutter_svg/flutter_svg.dart';
import 'package:gap/gap.dart';
import 'package:go_router/go_router.dart';
import 'package:zkool/network.dart';
import 'package:zkool/store.dart';
import 'package:zkool/utils.dart';
import 'package:zkool/widgets/error_display.dart';

/// Network selection page. Lets the user switch the active Zcash network
/// (mainnet / testnet / regtest). Each network keeps its own accounts in a
/// dedicated database, so switching swaps the entire account list.
class NetworksPage extends ConsumerStatefulWidget {
  const NetworksPage({super.key});

  @override
  ConsumerState<NetworksPage> createState() => NetworksPageState();
}

class NetworksPageState extends ConsumerState<NetworksPage> {
  ZNetwork? selected;
  bool switching = false;

  @override
  Widget build(BuildContext context) {
    final settingsAV = ref.watch(appSettingsProvider);
    return Scaffold(
      appBar: AppBar(title: const Text("Switch Network")),
      body: settingsAV.when(
        loading: () => blank(context),
        error: (error, stack) => showError(error),
        data: (settings) {
          final current = networkForName(settings.net);
          final active = selected ?? current;
          return SingleChildScrollView(
            child: Padding(
              padding: const EdgeInsets.all(16),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.stretch,
                children: [
                  Text(
                    "Select the network to use. Accounts are not shared between networks.",
                    style: Theme.of(context).textTheme.bodyMedium,
                  ),
                  const Gap(16),
                  // RadioGroup manages the selected value + change callback for the
                  // RadioListTiles below (the non-deprecated API; the per-tile
                  // groupValue/onChanged were deprecated after Flutter 3.32).
                  RadioGroup<ZNetwork>(
                    groupValue: active,
                    onChanged: (v) {
                      if (switching) return;
                      onSelect(v, current);
                    },
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.stretch,
                      children: ZNetwork.values.map((net) => _networkTile(net, current)).toList(),
                    ),
                  ),
                  if (switching) ...[
                    const Gap(24),
                    const Center(child: CircularProgressIndicator()),
                  ],
                ],
              ),
            ),
          );
        },
      ),
    );
  }

  Widget _networkTile(ZNetwork net, ZNetwork current) {
    final info = networkInfo(net);
    final isCurrent = net == current;
    // Tint the Zcash logo with the per-network accent so the three options stay
    // visually distinct on any theme (the SVG disc uses currentColor).
    final accent = networkAccent(net);
    return Card(
      elevation: 1,
      margin: const EdgeInsets.symmetric(vertical: 4),
      child: RadioListTile<ZNetwork>(
        value: net,
        secondary: SvgPicture.asset(
          kNetworkIconAsset,
          width: 32,
          height: 32,
          colorFilter: ColorFilter.mode(accent, BlendMode.srcIn),
        ),
        title: Text(info.label),
        subtitle: isCurrent ? const Text("Current network") : null,
      ),
    );
  }

  void onSelect(ZNetwork? net, ZNetwork current) async {
    if (net == null || net == current || switching) return;
    final info = networkInfo(net);
    setState(() {
      selected = net;
      switching = true;
    });
    try {
      final ok = await switchNetwork(
        ref,
        net,
        askPassword: () => inputPassword(
          context,
          title: "Enter Database Password for ${info.label}",
        ),
      );
      if (!mounted) return;
      if (ok) {
        showSnackbar("Switched to ${info.label}");
        GoRouter.of(context).pop();
      } else {
        setState(() {
          switching = false;
          selected = null;
        });
      }
    } on AnyhowException catch (e) {
      if (mounted) {
        await showException(context, e.message);
        setState(() {
          switching = false;
          selected = null;
        });
      }
    }
  }
}
