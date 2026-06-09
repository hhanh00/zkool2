import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:zkool/store.dart';

/// A discreet outlined button that displays the ZEC-to-fiat exchange rate.
/// Tapping refreshes the rate from CoinGecko.
/// Auto-scales: "1 ZEC = 445 USD" when rate >= 1, or "100 ZEC = 1 BTC" when rate < 1.
class ExchangeRateButton extends ConsumerWidget {
  const ExchangeRateButton({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final price = ref.watch(priceProvider);
    final settingsAV = ref.watch(appSettingsProvider);
    final currency = settingsAV.value?.currency.toUpperCase() ?? 'USD';

    if (price == null) {
      return const SizedBox.shrink();
    }

    final label = _formatRate(price, currency);

    return OutlinedButton(
      onPressed: () => _refresh(ref),
      style: OutlinedButton.styleFrom(
        elevation: 0,
        shadowColor: Colors.transparent,
        surfaceTintColor: Colors.transparent,
        side: BorderSide(color: Theme.of(context).colorScheme.outline),
        padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 6),
      ),
      child: Text(label, style: Theme.of(context).textTheme.bodyMedium),
    );
  }

  String _formatRate(double rate, String currency) {
    if (rate >= 1.0) {
      final formatted = rate.toStringAsFixed(2);
      return '1 ZEC = $formatted $currency';
    } else {
      // Scale up: find how many ZEC per 1 unit of currency
      final scale = (1.0 / rate).ceil();
      return '$scale ZEC = 1 $currency';
    }
  }

  void _refresh(WidgetRef ref) async {
    final settings = ref.read(appSettingsProvider).value;
    if (settings == null) return;
    final priceNotifier = ref.read(priceProvider.notifier);
    await priceNotifier.fetch(settings.coingecko, settings.currency);
  }
}
