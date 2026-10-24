import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:gap/gap.dart';
import 'package:zkool/src/rust/api/network.dart';
import 'package:zkool/store.dart';
import 'package:zkool/widgets/market_chart.dart';

/// The intervals offered, as Binance kline intervals.
const marketIntervals = [("1H", "1h"), ("4H", "4h"), ("1D", "1d"), ("1W", "1w")];

/// Binance's per-request cap; about four years of daily bars.
const _historyBars = 1500;
const _pollEvery = Duration(seconds: 10);

/// The market price page: ZEC/USDC perpetual from Binance, drawn natively.
/// The interval and style are remembered in the app settings.
class MarketPrice extends ConsumerStatefulWidget {
  const MarketPrice({super.key});

  @override
  ConsumerState<MarketPrice> createState() => MarketPriceState();
}

class MarketPriceState extends ConsumerState<MarketPrice> {
  final chartKey = GlobalKey<MarketChartState>();
  String interval = "1d";

  /// The interval of [candles], which lags [interval] while a load runs.
  String loadedInterval = "1d";
  MarketChartStyle style = MarketChartStyle.candles;
  bool showVolume = true;
  bool logScale = false;
  List<Candle> candles = const [];
  Candle? hovered;
  bool atLatest = true;
  bool loading = true;
  String? error;
  Timer? poll;
  int epoch = 0;

  @override
  void initState() {
    super.initState();
    final saved = ref.read(appSettingsProvider).value;
    if (saved != null) {
      if (marketIntervals.any((i) => i.$2 == saved.marketInterval)) interval = saved.marketInterval;
      style = MarketChartStyle.values.firstWhere((s) => s.name == saved.marketStyle, orElse: () => style);
    }
    Future(load);
  }

  void remember() => ref.read(appSettingsProvider.notifier).setMarketChart(interval: interval, style: style.name);

  @override
  void dispose() {
    poll?.cancel();
    super.dispose();
  }

  /// Loads the history of the current interval, then polls the last two bars.
  Future<void> load() async {
    poll?.cancel();
    final e = ++epoch;
    setState(() {
      loading = true;
      error = null;
      hovered = null;
    });
    try {
      final c = await fetchCandles(interval: interval, limit: _historyBars, c: coinContext.coin);
      if (!mounted || e != epoch) return;
      setState(() {
        candles = c;
        loadedInterval = interval;
        loading = false;
      });
      poll = Timer.periodic(_pollEvery, (_) => refresh());
    } catch (err) {
      if (!mounted || e != epoch) return;
      setState(() {
        loading = false;
        error = err.toString();
      });
    }
  }

  /// Fetches the forming bar and the one before it and merges them in.
  Future<void> refresh() async {
    final e = epoch;
    try {
      final tail = await fetchCandles(interval: interval, limit: 2, c: coinContext.coin);
      if (!mounted || e != epoch || candles.isEmpty) return;
      final merged = List<Candle>.of(candles);
      for (final c in tail) {
        final i = merged.indexWhere((m) => m.time == c.time, merged.length - 2 < 0 ? 0 : merged.length - 2);
        if (i >= 0) {
          merged[i] = c;
        } else if (c.time > merged.last.time) {
          merged.add(c);
        }
      }
      setState(() => candles = merged);
    } catch (_) {
      // A failed poll keeps the last good chart; the next tick retries.
    }
  }

  void onInterval(String v) {
    if (v == interval) return;
    interval = v;
    remember();
    load();
  }

  void onStyle(MarketChartStyle s) {
    setState(() => style = s);
    remember();
  }

  @override
  Widget build(BuildContext context) {
    final t = Theme.of(context).textTheme;
    final colors = MarketChartColors.of(context);

    return Scaffold(
      appBar: AppBar(
        title: Text("Market Price", style: t.titleLarge),
        actions: [
          PopupMenuButton<MarketChartStyle>(
            tooltip: "Chart style",
            icon: const Icon(Icons.candlestick_chart),
            initialValue: style,
            onSelected: onStyle,
            itemBuilder: (_) => [for (final s in MarketChartStyle.values) PopupMenuItem(value: s, child: Text(s.label))],
          ),
          IconButton(
            tooltip: "Volume",
            isSelected: showVolume,
            icon: const Icon(Icons.bar_chart),
            onPressed: () => setState(() => showVolume = !showVolume),
          ),
          IconButton(
            tooltip: "Logarithmic scale",
            isSelected: logScale,
            icon: const Icon(Icons.linear_scale),
            onPressed: () => setState(() => logScale = !logScale),
          ),
        ],
      ),
      body: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Padding(
            padding: const EdgeInsets.fromLTRB(8, 8, 8, 0),
            child: Row(
              children: [
                SegmentedButton<String>(
                  segments: [for (final (label, v) in marketIntervals) ButtonSegment(value: v, label: Text(label))],
                  selected: {interval},
                  showSelectedIcon: false,
                  style: const ButtonStyle(visualDensity: VisualDensity.compact),
                  onSelectionChanged: (s) => onInterval(s.first),
                ),
              ],
            ),
          ),
          Padding(
            padding: const EdgeInsets.fromLTRB(12, 8, 12, 4),
            child: MarketLegend(candle: hovered ?? (candles.isEmpty ? null : candles.last), interval: loadedInterval),
          ),
          Expanded(
            child: Container(
              color: colors.background,
              child: Stack(
                children: [
                  if (candles.isNotEmpty)
                    MarketChart(
                      key: chartKey,
                      candles: candles,
                      style: style,
                      showVolume: showVolume,
                      logScale: logScale,
                      onHover: (c) => setState(() => hovered = c),
                      onAtLatestChanged: (v) => setState(() => atLatest = v),
                    ),
                  if (loading) const Center(child: CircularProgressIndicator()),
                  if (error != null)
                    Center(
                      child: Column(
                        mainAxisSize: MainAxisSize.min,
                        children: [
                          Text("Market data unavailable", style: t.titleMedium),
                          const Gap(8),
                          Padding(
                            padding: const EdgeInsets.symmetric(horizontal: 24),
                            child: Text(error!, style: t.bodySmall, textAlign: TextAlign.center),
                          ),
                          const Gap(16),
                          FilledButton.tonal(onPressed: load, child: const Text("Retry")),
                        ],
                      ),
                    ),
                  if (!atLatest && candles.isNotEmpty)
                    Positioned(
                      right: 72,
                      bottom: 40,
                      child: IconButton.filledTonal(
                        tooltip: "Go to latest",
                        icon: const Icon(Icons.keyboard_double_arrow_right),
                        onPressed: () => chartKey.currentState?.scrollToLatest(),
                      ),
                    ),
                ],
              ),
            ),
          ),
        ],
      ),
    );
  }
}

/// TradingView's legend line: symbol, interval, source, O H L C, the change
/// of the bar and its volume, coloured by the bar's direction.
class MarketLegend extends StatelessWidget {
  final Candle? candle;
  final String interval;
  const MarketLegend({super.key, required this.candle, required this.interval});

  @override
  Widget build(BuildContext context) {
    final t = Theme.of(context).textTheme;
    final c = candle;
    final label = marketIntervals.firstWhere((i) => i.$2 == interval).$1;
    final head = TextSpan(text: "ZECUSDC · $label · Binance  ", style: t.bodyMedium);
    if (c == null) return Text.rich(head);
    final up = c.close >= c.open;
    final color = up ? MarketChartColors.up : MarketChartColors.down;
    final change = c.close - c.open;
    final pct = c.open == 0 ? 0.0 : change / c.open * 100;
    final valueStyle = t.bodyMedium?.copyWith(color: color, fontFeatures: const [FontFeature.tabularFigures()]);
    final keyStyle = t.bodySmall;
    TextSpan kv(String k, String v) => TextSpan(children: [
          TextSpan(text: "$k ", style: keyStyle),
          TextSpan(text: "$v  ", style: valueStyle),
        ]);
    return Text.rich(
      TextSpan(children: [
        head,
        kv("O", formatPrice(c.open)),
        kv("H", formatPrice(c.high)),
        kv("L", formatPrice(c.low)),
        kv("C", formatPrice(c.close)),
        TextSpan(
          text: "${change >= 0 ? "+" : ""}${formatPrice(change)} (${pct >= 0 ? "+" : ""}${pct.toStringAsFixed(2)}%)  ",
          style: valueStyle,
        ),
        kv("Vol", formatVolume(c.volume)),
      ]),
      maxLines: 2,
      overflow: TextOverflow.ellipsis,
    );
  }
}

String formatPrice(double v) => v.toStringAsFixed(2);

/// 12.3K, 4.56M: TradingView's compact volume.
String formatVolume(double v) {
  if (v >= 1e9) return "${(v / 1e9).toStringAsFixed(2)}B";
  if (v >= 1e6) return "${(v / 1e6).toStringAsFixed(2)}M";
  if (v >= 1e3) return "${(v / 1e3).toStringAsFixed(2)}K";
  return v.toStringAsFixed(2);
}
