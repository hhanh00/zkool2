import 'dart:math' as math;

import 'package:flutter/material.dart';
import 'package:swm_chart/swm_chart.dart';
import 'package:zkool/src/rust/api/network.dart';

/// The styles of the market chart, the subset of TradingView's style menu
/// that derives from OHLCV data alone.
enum MarketChartStyle {
  candles("Candles"),
  hollowCandles("Hollow candles"),
  heikinAshi("Heikin Ashi"),
  bars("Bars"),
  line("Line"),
  area("Area");

  final String label;
  const MarketChartStyle(this.label);
}

/// TradingView's default palette.
class MarketChartColors {
  static const up = Color(0xFF089981);
  static const down = Color(0xFFF23645);
  static const line = Color(0xFF2962FF);

  final Color background;
  final Color text;
  final Color grid;
  final Color border;
  final Color crosshairLabel;

  const MarketChartColors._({
    required this.background,
    required this.text,
    required this.grid,
    required this.border,
    required this.crosshairLabel,
  });

  static const dark = MarketChartColors._(
    background: Color(0xFF131722),
    text: Color(0xFFB2B5BE),
    grid: Color(0xFF2A2E39),
    border: Color(0xFF2A2E39),
    crosshairLabel: Color(0xFF363A45),
  );

  static const light = MarketChartColors._(
    background: Color(0xFFFFFFFF),
    text: Color(0xFF131722),
    grid: Color(0xFFF0F3FA),
    border: Color(0xFFE0E3EB),
    crosshairLabel: Color(0xFF131722),
  );

  static MarketChartColors of(BuildContext context) =>
      Theme.of(context).brightness == Brightness.dark ? dark : light;
}

/// Heikin Ashi bars computed from [candles]: the close is the bar's average
/// price, the open the midpoint of the previous Heikin Ashi bar, and the high
/// and low extend to them. Volume is unchanged.
List<Candle> heikinAshi(List<Candle> candles) {
  final out = <Candle>[];
  for (final c in candles) {
    final close = (c.open + c.high + c.low + c.close) / 4;
    final open = out.isEmpty ? (c.open + c.close) / 2 : (out.last.open + out.last.close) / 2;
    out.add(Candle(
      time: c.time,
      open: open,
      high: math.max(c.high, math.max(open, close)),
      low: math.min(c.low, math.min(open, close)),
      close: close,
      volume: c.volume,
    ));
  }
  return out;
}

/// The market chart: a TradingView-style chart of [candles], oldest first.
///
/// [candles] may grow at the end or have its last bar replaced between
/// builds, which updates the chart in place; any other change reloads it.
/// [onHover] reports the candle under the crosshair, or null when the
/// crosshair leaves. [onAtLatestChanged] reports whether the last bar is in
/// view, for a "go to latest" button that calls [MarketChartState.scrollToLatest].
class MarketChart extends StatefulWidget {
  final List<Candle> candles;
  final MarketChartStyle style;
  final bool showVolume;
  final bool logScale;
  final ValueChanged<Candle?>? onHover;
  final ValueChanged<bool>? onAtLatestChanged;

  const MarketChart({
    super.key,
    required this.candles,
    this.style = MarketChartStyle.candles,
    this.showVolume = true,
    this.logScale = false,
    this.onHover,
    this.onAtLatestChanged,
  });

  @override
  State<MarketChart> createState() => MarketChartState();
}

class MarketChartState extends State<MarketChart> {
  late final ChartController chart;
  late SeriesApi main;
  late final HistogramSeriesApi volume;
  List<Candle> shown = const [];
  Map<int, Candle> byTime = const {};
  MarketChartColors? colors;
  bool atLatest = true;

  @override
  void initState() {
    super.initState();
    chart = ChartController(
      options: const ChartOptions(
        timeScale: TimeScaleOptions(timeVisible: true, secondsVisible: false, rightOffset: 5, barSpacing: 6),
        crosshair: CrosshairOptions(mode: CrosshairMode.normal),
      ),
    );
    main = _createMain(widget.style);
    volume = chart.addHistogramSeries(
      const HistogramSeriesOptions(
        priceScaleId: "volume",
        priceFormat: PriceFormat.volume(),
        lastValueVisible: false,
        priceLineVisible: false,
      ),
    );
    // Volume lives in the bottom fifth of the pane, as on TradingView.
    chart.priceScale("volume").applyOptions(const PriceScaleOptions(scaleMargins: ScaleMargins(top: 0.8, bottom: 0)));
    _applyVolumeVisible();
    _applyLogScale();
    chart.subscribeCrosshairMove(_onCrosshairMove);
    chart.timeScale().subscribeVisibleLogicalRangeChange(_onRangeChange);
    _setData(widget.candles);
  }

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    final c = MarketChartColors.of(context);
    if (colors != c) {
      colors = c;
      _applyTheme(c);
    }
  }

  @override
  void didUpdateWidget(covariant MarketChart oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (widget.style != oldWidget.style) {
      chart.removeSeries(main);
      main = _createMain(widget.style);
      _setData(widget.candles);
    } else if (!identical(widget.candles, oldWidget.candles)) {
      _update(widget.candles);
    }
    if (widget.showVolume != oldWidget.showVolume) _applyVolumeVisible();
    if (widget.logScale != oldWidget.logScale) _applyLogScale();
  }

  @override
  void dispose() {
    chart.dispose();
    super.dispose();
  }

  /// Scrolls so the last bar is in view, as TradingView's arrow button does.
  void scrollToLatest() => chart.timeScale().scrollToRealTime();

  SeriesApi _createMain(MarketChartStyle style) {
    const up = MarketChartColors.up;
    const down = MarketChartColors.down;
    return switch (style) {
      MarketChartStyle.candles || MarketChartStyle.heikinAshi => chart.addCandlestickSeries(
          const CandlestickSeriesOptions(
            upColor: up,
            downColor: down,
            borderUpColor: up,
            borderDownColor: down,
            wickUpColor: up,
            wickDownColor: down,
          ),
        ),
      MarketChartStyle.hollowCandles => chart.addCandlestickSeries(
          const CandlestickSeriesOptions(
            upColor: Colors.transparent,
            downColor: down,
            borderUpColor: up,
            borderDownColor: down,
            wickUpColor: up,
            wickDownColor: down,
          ),
        ),
      MarketChartStyle.bars => chart.addBarSeries(const BarSeriesOptions(upColor: up, downColor: down)),
      MarketChartStyle.line => chart.addLineSeries(
          const LineSeriesOptions(color: MarketChartColors.line, lineWidth: 2),
        ),
      MarketChartStyle.area => chart.addAreaSeries(
          const AreaSeriesOptions(
            lineColor: MarketChartColors.line,
            topColor: Color(0x662962FF),
            bottomColor: Color(0x002962FF),
            lineWidth: 2,
          ),
        ),
    };
  }

  /// The bars drawn for [candles] in the current style.
  List<Candle> _styled(List<Candle> candles) =>
      widget.style == MarketChartStyle.heikinAshi ? heikinAshi(candles) : candles;

  SeriesData _mainItem(Candle c) => switch (widget.style) {
        MarketChartStyle.candles ||
        MarketChartStyle.hollowCandles ||
        MarketChartStyle.heikinAshi =>
          CandlestickData(time: c.time, open: c.open, high: c.high, low: c.low, close: c.close),
        MarketChartStyle.bars => BarData(time: c.time, open: c.open, high: c.high, low: c.low, close: c.close),
        MarketChartStyle.line => LineData(time: c.time, value: c.close),
        MarketChartStyle.area => AreaData(time: c.time, value: c.close),
      };

  HistogramData _volumeItem(Candle c) => HistogramData(
        time: c.time,
        value: c.volume,
        color: (c.close >= c.open ? MarketChartColors.up : MarketChartColors.down).withValues(alpha: 0.5),
      );

  void _setData(List<Candle> candles) {
    shown = candles;
    byTime = {for (final c in candles) c.time: c};
    final bars = _styled(candles);
    main.setData(bars.map(_mainItem).toList());
    volume.setData(candles.map(_volumeItem).toList());
  }

  /// Applies [candles] in place when only the tail changed: the last bar
  /// replaced and/or one new bar appended. Anything else reloads the chart.
  void _update(List<Candle> candles) {
    final old = shown;
    final added = candles.length - old.length;
    final tailOnly = old.isNotEmpty &&
        (added == 0 || added == 1) &&
        candles.first.time == old.first.time &&
        candles.length > 1 &&
        candles[old.length - 1].time == old.last.time &&
        _samePrefix(old, candles, old.length - 1);
    if (!tailOnly) {
      _setData(candles);
      return;
    }
    shown = candles;
    byTime = {for (final c in candles) c.time: c};
    final bars = _styled(candles);
    for (var i = old.length - 1; i < candles.length; i++) {
      main.update(_mainItem(bars[i]));
      volume.update(_volumeItem(candles[i]));
    }
  }

  static bool _samePrefix(List<Candle> a, List<Candle> b, int n) {
    for (var i = 0; i < n; i++) {
      if (a[i] != b[i]) return false;
    }
    return true;
  }

  void _applyVolumeVisible() => volume.applyOptions(volume.options.copyWith(visible: widget.showVolume));

  void _applyLogScale() {
    final scale = chart.priceScale();
    scale.applyOptions(
      scale.options.copyWith(mode: widget.logScale ? PriceScaleMode.logarithmic : PriceScaleMode.normal),
    );
  }

  void _applyTheme(MarketChartColors c) {
    final o = chart.options;
    chart.applyOptions(
      o.copyWith(
        layout: o.layout.copyWith(
          backgroundColor: c.background,
          textColor: c.text,
        ),
        crosshair: o.crosshair.copyWith(
          vertLine: o.crosshair.vertLine.copyWith(labelBackgroundColor: c.crosshairLabel),
          horzLine: o.crosshair.horzLine.copyWith(labelBackgroundColor: c.crosshairLabel),
        ),
        grid: GridOptions(
          vertLines: GridLineOptions(color: c.grid),
          horzLines: GridLineOptions(color: c.grid),
        ),
        rightPriceScale: o.rightPriceScale.copyWith(borderColor: c.border),
        timeScale: o.timeScale.copyWith(borderColor: c.border),
      ),
    );
    // applyOptions resets the price scales; put the volume margins back.
    chart.priceScale("volume").applyOptions(const PriceScaleOptions(scaleMargins: ScaleMargins(top: 0.8, bottom: 0)));
    _applyLogScale();
  }

  void _onCrosshairMove(ChartEventParams p) {
    final t = p.time;
    widget.onHover?.call(t == null ? null : byTime[t.toInt()]);
  }

  void _onRangeChange(LogicalRange? range) {
    if (range == null || shown.isEmpty) return;
    final latest = range.to >= shown.length - 1;
    if (latest != atLatest) {
      atLatest = latest;
      widget.onAtLatestChanged?.call(latest);
    }
  }

  @override
  Widget build(BuildContext context) => SwmChart(controller: chart, semanticsLabel: "Market chart");
}
