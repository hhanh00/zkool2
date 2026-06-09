import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated.dart';
import 'package:gap/gap.dart';
import 'package:go_router/go_router.dart';
import 'package:zkool/main.dart';
import 'package:zkool/src/rust/api/network.dart';
import 'package:zkool/src/rust/api/transaction.dart';
import 'package:zkool/store.dart';
import 'package:zkool/widgets/error_display.dart';

typedef CurrencyCallback = void Function(String newCurrency);

class CurrencyPage extends ConsumerStatefulWidget {
  final CurrencyCallback onClose;
  const CurrencyPage({required this.onClose, super.key});

  @override
  ConsumerState<CurrencyPage> createState() => _CurrencyPageState();
}

class _CurrencyPageState extends ConsumerState<CurrencyPage> {
  late final String _originalCurrency;
  late String _selectedCurrency;
  bool _handlingPop = false;

  @override
  void initState() {
    super.initState();
    final settings = ref.read(appSettingsProvider).requireValue;
    _originalCurrency = settings.currency;
    _selectedCurrency = _originalCurrency;
  }

  Future<bool> _onPopInvoked() async {
    if (_selectedCurrency == _originalCurrency) {
      return true; // no change, allow pop
    }

    if (_handlingPop) return false;
    _handlingPop = true;

    final settings = ref.read(appSettingsProvider).requireValue;
    final c = coinContext.coin;

    // Try to fetch the current exchange rate for pre-fill
    double? prefilledRate;
    try {
      final rate = await getExchangeRate(
        api: settings.coingecko,
        fromCurrency: _originalCurrency,
        toCurrency: _selectedCurrency,
      );
      prefilledRate = rate.toPrice / rate.fromPrice;
    } catch (_) {
      // API failed — user will enter manually
    }

    if (!mounted) {
      _handlingPop = false;
      return false;
    }

    final exchangeRate = await _showExchangeRateDialog(
      fromCurrency: _originalCurrency.toUpperCase(),
      toCurrency: _selectedCurrency.toUpperCase(),
      prefilledRate: prefilledRate,
    );

    if (!mounted) {
      _handlingPop = false;
      return false;
    }

    if (exchangeRate == null) {
      // User cancelled — restore original selection, stay on page
      setState(() {
        _selectedCurrency = _originalCurrency;
        _handlingPop = false;
      });
      return false;
    }

    // User confirmed — update historical prices
    try {
      await updateHistoricalPrices(
        currency: _selectedCurrency,
        exchangeRate: exchangeRate,
        c: c,
      );
    } on AnyhowException catch (e) {
      if (mounted) await showException(context, e.message);
      _handlingPop = false;
      return false;
    }

    // Persist and allow pop
    widget.onClose(_selectedCurrency);
    _handlingPop = false;
    return true;
  }

  Future<double?> _showExchangeRateDialog({
    required String fromCurrency,
    required String toCurrency,
    required double? prefilledRate,
  }) async {
    final controller = TextEditingController(
      text: prefilledRate?.toStringAsFixed(6) ?? '',
    );
    final formKey = GlobalKey<FormState>();

    return showDialog<double>(
      context: context,
      barrierDismissible: false,
      builder: (context) => AlertDialog(
        title: Text('Convert Prices'),
        content: Form(
          key: formKey,
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(
                'You are changing currency from $fromCurrency to $toCurrency. '
                'All historical transaction prices need to be converted.',
              ),
              const Gap(16),
              Text('How many $toCurrency per 1 $fromCurrency?'),
              const Gap(8),
              TextFormField(
                controller: controller,
                keyboardType: TextInputType.numberWithOptions(decimal: true),
                autofocus: true,
                decoration: InputDecoration(
                  labelText: 'Exchange Rate',
                  hintText: 'e.g. 0.92',
                  suffixText: '$toCurrency / $fromCurrency',
                ),
                validator: (v) {
                  if (v == null || v.isEmpty) return 'Required';
                  final r = double.tryParse(v);
                  if (r == null) return 'Must be a number';
                  if (r <= 0) return 'Must be greater than 0';
                  return null;
                },
              ),
              if (prefilledRate == null)
                Padding(
                  padding: const EdgeInsets.only(top: 8),
                  child: Text(
                    'Could not fetch current rate. Please enter manually.',
                    style: TextStyle(color: Colors.orange, fontSize: 12),
                  ),
                ),
            ],
          ),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(context).pop(null),
            child: const Text('Cancel'),
          ),
          FilledButton(
            onPressed: () {
              if (formKey.currentState!.validate()) {
                Navigator.of(context).pop(double.parse(controller.text));
              }
            },
            child: const Text('Convert'),
          ),
        ],
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final currenciesAV = ref.watch(supportedCurrenciesProvider);

    return PopScope(
      canPop: false,
      onPopInvokedWithResult: (didPop, _) async {
        if (didPop) return; // already popped
        final shouldPop = await _onPopInvoked();
        if (shouldPop && mounted) {
          GoRouter.of(context).pop();
        }
      },
      child: Scaffold(
        appBar: AppBar(
          title: const Text('Select Currency'),
        ),
        body: currenciesAV.when(
          loading: () => const Center(child: CircularProgressIndicator()),
          error: (error, stack) => Center(
            child: Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                Text('Failed to load currencies'),
                const Gap(8),
                Text(error.toString(), style: TextStyle(fontSize: 12, color: Colors.red)),
                const Gap(8),
                ElevatedButton(
                  onPressed: () => ref.invalidate(supportedCurrenciesProvider),
                  child: const Text('Retry'),
                ),
              ],
            ),
          ),
          data: (currencies) {
            // Sort alphabetically
            final sorted = List<String>.from(currencies)..sort();

            return ListView.builder(
              itemCount: sorted.length,
              itemBuilder: (context, index) {
                final currency = sorted[index];
                final isSelected = currency == _selectedCurrency;

                return ListTile(
                  title: Text(currency.toUpperCase()),
                  trailing: isSelected ? const Icon(Icons.check, color: Colors.green) : null,
                  selected: isSelected,
                  selectedTileColor: Theme.of(context).colorScheme.primaryContainer.withAlpha(100),
                  onTap: () {
                    setState(() => _selectedCurrency = currency);
                  },
                );
              },
            );
          },
        ),
      ),
    );
  }
}
