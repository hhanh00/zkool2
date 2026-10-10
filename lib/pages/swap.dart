import 'package:flutter/material.dart';
import 'package:flutter_form_builder/flutter_form_builder.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:gap/gap.dart';
import 'package:zkool/main.dart';
import 'package:zkool/validators.dart';
import 'package:zkool/store.dart';
import 'package:zkool/services/swap_quote.dart';
import 'package:zkool/widgets/error_display.dart';
import 'package:zkool/src/rust/api/near_intents.dart';
import 'package:zkool/src/rust/near_intents.dart' as ni;
import 'package:zkool/pages/swaps.dart';
import 'package:zkool/pages/swap_summary.dart';
import 'package:zkool/utils.dart';
import 'package:zkool/widgets/input_amount.dart';

/// Destination networks offered by the initial frontend.
enum SwapNetwork {
  solana('Solana'),
  ethereum('Ethereum');

  final String label;
  const SwapNetwork(this.label);
}

// TODO: Load supported assets from 1Click and add search and network filtering.
enum SwapAsset {
  usdt('USDT', 6, [SwapNetwork.solana, SwapNetwork.ethereum]),
  usdc('USDC', 6, [SwapNetwork.solana, SwapNetwork.ethereum]),
  sol('SOL', 9, [SwapNetwork.solana]),
  eth('ETH', 18, [SwapNetwork.ethereum]);

  final String symbol;
  final int decimals;
  final List<SwapNetwork> networks;
  const SwapAsset(this.symbol, this.decimals, this.networks);
}

enum SwapAmountMode { send, receive }

class SwapDraft {
  final SwapAsset asset;
  final SwapNetwork network;
  final SwapAmountMode amountMode;
  final String amount;
  final String recipient;
  final int slippageTolerance;

  const SwapDraft(
      {required this.asset, required this.network, required this.amountMode, required this.amount, required this.recipient, this.slippageTolerance = 100});
}

/// A displayed quote contains decimal strings, never floating-point amounts.
class SwapReviewQuote {
  final ni.SwapRequest? request;
  final int? destinationDecimals;
  final String amountIn;
  final String amountOut;
  final String minimumReceived;
  final String? estimatedCostUsd;
  final DateTime expiresAt;

  const SwapReviewQuote(
      {required this.amountIn,
      required this.amountOut,
      required this.minimumReceived,
      this.estimatedCostUsd,
      required this.expiresAt,
      this.request,
      this.destinationDecimals});
}

typedef SwapQuoteLoader = Future<SwapReviewQuote> Function(SwapDraft draft);

class SwapPage extends ConsumerStatefulWidget {
  final SwapQuoteLoader? quoteLoader;
  const SwapPage({this.quoteLoader, super.key});

  @override
  ConsumerState<SwapPage> createState() => _SwapPageState();
}

class _SwapPageState extends ConsumerState<SwapPage> {
  final _formKey = GlobalKey<FormBuilderState>();
  SwapAsset _asset = SwapAsset.usdt;
  SwapNetwork _network = SwapNetwork.solana;
  SwapAmountMode _amountMode = SwapAmountMode.send;

  @override
  Widget build(BuildContext context) {
    if (ref.watch(lifecycleProvider).value ?? false) return PinLock();
    return Scaffold(
      appBar: AppBar(
        title: const Text('Swap'),
        actions: [IconButton(tooltip: 'Next', onPressed: _review, icon: const Icon(Icons.arrow_forward))],
      ),
      body: SingleChildScrollView(
        padding: const EdgeInsets.all(16),
        child: FormBuilder(
          key: _formKey,
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              const ListTile(contentPadding: EdgeInsets.zero, leading: Icon(Icons.currency_exchange), title: Text('From ZEC'), subtitle: Text('Zcash')),
              const Gap(16),
              FormBuilderDropdown<SwapAsset>(
                name: 'asset',
                initialValue: _asset,
                decoration: const InputDecoration(labelText: 'Receive asset'),
                items: SwapAsset.values.map((asset) => DropdownMenuItem(value: asset, child: Text(asset.symbol))).toList(),
                onChanged: (asset) {
                  if (asset == null || asset == _asset) return;
                  setState(() {
                    _asset = asset;
                    if (!asset.networks.contains(_network)) _network = asset.networks.first;
                  });
                  _formKey.currentState?.fields['recipient']?.validate();
                },
              ),
              const Gap(8),
              FormBuilderDropdown<SwapNetwork>(
                key: ValueKey(('network', _asset)),
                name: 'network',
                initialValue: _network,
                decoration: const InputDecoration(labelText: 'Receiving network'),
                items: _asset.networks.map((network) => DropdownMenuItem(value: network, child: Text(network.label))).toList(),
                onChanged: (network) {
                  if (network == null) return;
                  setState(() => _network = network);
                  _formKey.currentState?.fields['recipient']?.validate();
                },
              ),
              const Gap(16),
              FormBuilderDropdown<SwapAmountMode>(
                name: 'amountMode',
                initialValue: _amountMode,
                decoration: const InputDecoration(labelText: 'Set amount to'),
                items: [
                  const DropdownMenuItem(value: SwapAmountMode.send, child: Text('Send ZEC')),
                  DropdownMenuItem(value: SwapAmountMode.receive, child: Text('Receive ${_asset.symbol}')),
                ],
                onChanged: (mode) {
                  if (mode == null || mode == _amountMode) return;
                  setState(() => _amountMode = mode);
                },
              ),
              const Gap(16),
              if (_amountMode == SwapAmountMode.send)
                InputAmount(
                  key: const ValueKey(SwapAmountMode.send),
                  name: 'amount',
                  showFx: false,
                  label: 'Amount in ZEC',
                )
              else
                FormBuilderTextField(
                  key: ValueKey(('amount', _asset)),
                  name: 'amount',
                  decoration: InputDecoration(labelText: 'Amount in ${_asset.symbol}'),
                  keyboardType: const TextInputType.numberWithOptions(decimal: true),
                ),
              const Gap(16),
              FormBuilderTextField(
                name: 'recipient',
                decoration: InputDecoration(labelText: '${_asset.symbol} receiving address', helperText: 'Address on ${_network.label}'),
                autocorrect: false,
                enableSuggestions: false,
                validator: _validateRecipient,
              ),
              const Gap(16),
              FormBuilderTextField(
                name: 'slippage',
                initialValue: '1',
                decoration: const InputDecoration(labelText: 'Slippage tolerance', suffixText: '%'),
                keyboardType: const TextInputType.numberWithOptions(decimal: true),
                validator: (value) {
                  try {
                    swapSlippageBasisPoints(value ?? '');
                    return null;
                  } on FormatException {
                    return 'Enter 0–100% with at most 2 decimals';
                  }
                },
              ),
              const Gap(24),
              ElevatedButton.icon(onPressed: _review, label: const Text('Next'), icon: const Icon(Icons.arrow_forward)),
            ],
          ),
        ),
      ),
    );
  }

  String? _validateRecipient(String? value) {
    final address = value?.trim() ?? '';
    if (address.isEmpty) return 'Enter a receiving address';
    final pattern = _network == SwapNetwork.ethereum ? RegExp(r'^0x[0-9a-fA-F]{40}$') : RegExp(r'^[1-9A-HJ-NP-Za-km-z]{32,44}$');
    if (!pattern.hasMatch(address)) return 'Enter a ${_network.label} address';
    return null;
  }

  void _review() {
    final form = _formKey.currentState!;
    if (!form.saveAndValidate()) return;
    final amount = (form.value['amount'] as String?)?.trim() ?? '';
    // Keep parsing local and exact. No money is sent from this frontend step.
    final decimals = _amountMode == SwapAmountMode.send ? 8 : _asset.decimals;
    final normalized = normalizeCryptoAmount(amount, decimals: decimals);
    if (normalized == null) {
      form.fields['amount']?.invalidate('Enter a positive amount with at most $decimals decimals');
      return;
    }
    final draft = SwapDraft(
        asset: _asset,
        network: _network,
        amountMode: _amountMode,
        amount: normalized,
        recipient: (form.value['recipient'] as String).trim(),
        slippageTolerance: swapSlippageBasisPoints(form.value['slippage'] as String));
    Navigator.of(context).push(MaterialPageRoute<void>(builder: (_) => SwapReviewPage(draft: draft, quoteLoader: widget.quoteLoader)));
  }
}

class SwapReviewPage extends ConsumerStatefulWidget {
  final SwapDraft draft;
  final SwapQuoteLoader? quoteLoader;
  const SwapReviewPage({required this.draft, this.quoteLoader, super.key});

  @override
  ConsumerState<SwapReviewPage> createState() => _SwapReviewPageState();
}

class _SwapReviewPageState extends ConsumerState<SwapReviewPage> {
  Future<SwapReviewQuote>? _quote;
  SwapReviewQuote? _displayedQuote;
  bool _submitting = false;
  String _submitStep = '';
  late final _coin = coinContext.coin;

  Future<void> _confirmAndSubmit() async {
    final quote = _displayedQuote!;
    final confirmed =
        await confirmDialog(context, title: 'Confirm swap', message: 'Are you sure you want to send this ZEC → ${widget.draft.asset.symbol} swap?');
    if (!confirmed) return;
    if (!DateTime.now().isBefore(quote.expiresAt)) {
      if (mounted) setState(_refresh);
      return;
    }
    if (mounted) setState(() => _submitStep = 'Creating swap...');
    final saved = await nearIntentsCreateSwap(request: quote.request!, c: _coin);
    // TODO: Prepare, sign and broadcast the deposit, then notify 1Click.
    // This dialog is the final confirmation, as in transaction sending.
    if (mounted) {
      ref.invalidate(savedSwapsProvider);
      Navigator.of(context).pushReplacement(MaterialPageRoute<void>(
        builder: (_) => SwapSummaryPage(swap: saved, symbol: widget.draft.asset.symbol, decimals: quote.destinationDecimals!),
      ));
    } else {
      showSnackbar('Swap created successfully (background)');
    }
  }

  Future<void> _confirm() async {
    if (_submitting || _displayedQuote?.request == null) return;
    if (!DateTime.now().isBefore(_displayedQuote!.expiresAt)) {
      setState(_refresh);
      return;
    }
    setState(() {
      _submitting = true;
      _submitStep = 'Preparing...';
    });
    try {
      await _confirmAndSubmit();
    } catch (error) {
      if (mounted) {
        await showException(context, error.toString());
      } else {
        showSnackbar('Swap submission failed: $error');
      }
    } finally {
      if (mounted)
        setState(() {
          _submitting = false;
          _submitStep = '';
        });
    }
  }

  @override
  void initState() {
    super.initState();
    _refresh();
  }

  void _refresh() {
    _displayedQuote = null;
    final loader = widget.quoteLoader;
    _quote = Future.sync(() => loader != null ? loader(widget.draft) : loadSwapQuote(widget.draft, _coin));
  }

  @override
  Widget build(BuildContext context) {
    if (ref.watch(lifecycleProvider).value ?? false) return PinLock();
    final draft = widget.draft;
    return Scaffold(
      appBar: AppBar(
        title: const Text('Review swap'),
      ),
      body: SingleChildScrollView(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            ListTile(contentPadding: EdgeInsets.zero, title: Text('ZEC → ${draft.asset.symbol}')),
            _row('From', 'Zcash'),
            _row('To', draft.network.label),
            _row('Slippage tolerance', '${swapDecimalAmount(draft.slippageTolerance.toString(), 2)}%'),
            _row(draft.amountMode == SwapAmountMode.send ? 'You send' : 'You receive',
                '${draft.amount} ${draft.amountMode == SwapAmountMode.send ? 'ZEC' : draft.asset.symbol}'),
            const Gap(16),
            Text('Receiving address', style: Theme.of(context).textTheme.labelLarge),
            const Gap(8),
            SelectableText(draft.recipient),
            const Gap(24),
            FutureBuilder<SwapReviewQuote>(
              future: _quote,
              builder: (context, snapshot) {
                if (_quote == null) return const Text('A quote is required before this swap can be confirmed.');
                if (snapshot.connectionState != ConnectionState.done) return const Center(child: CircularProgressIndicator());
                if (snapshot.hasError) {
                  return ErrorCard(error: snapshot.error!, onRetry: () => setState(_refresh));
                }
                final quote = snapshot.requireData;
                _displayedQuote = quote;
                return Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
                  _row('You send', '${quote.amountIn} ZEC'),
                  _row('You receive', '${quote.amountOut} ${draft.asset.symbol}'),
                  _row('Minimum received', quote.minimumReceived == 'Unavailable' ? 'Unavailable' : '${quote.minimumReceived} ${draft.asset.symbol}'),
                  _row('Estimated swap cost', quote.estimatedCostUsd == null ? 'Unavailable' : '${quote.estimatedCostUsd} USD'),
                  const Text(
                      'Based on quoted USD values. Includes fees, spread and price impact; may include a refundable slippage buffer. Zcash network fee is extra.'),
                  _row('Quote expires', exactTimeToString(quote.expiresAt.millisecondsSinceEpoch ~/ 1000)),
                  TextButton(onPressed: _submitting ? null : () => setState(_refresh), child: const Text('Refresh quote')),
                  ElevatedButton(onPressed: _submitting || quote.request == null ? null : _confirm, child: const Text('Confirm swap')),
                ]);
              },
            ),
            const Gap(24),
            if (_submitting) ...[
              const LinearProgressIndicator(),
              const Gap(8),
              Text(_submitStep),
              const Text('Submission continues in the background if this page is closed.'),
            ],
          ],
        ),
      ),
    );
  }

  Widget _row(String label, String value) => Padding(
        padding: const EdgeInsets.symmetric(vertical: 8),
        child: Row(children: [Expanded(child: Text(label)), Flexible(child: Text(value, textAlign: TextAlign.end))]),
      );
}
