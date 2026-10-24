import 'dart:math' as math;

import 'package:fl_chart/fl_chart.dart';
import 'package:flutter/material.dart';
import 'package:flutter_form_builder/flutter_form_builder.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:gap/gap.dart';
import 'package:intl/intl.dart';
import 'package:zkool/main.dart';
import 'package:zkool/src/rust/api/transaction.dart';
import 'package:zkool/store.dart';
import 'package:zkool/utils.dart';

/// The budget charts of an account: income and spending by category, and
/// the amounts of one category over time, within an optional date range.
class ChartPage extends ConsumerStatefulWidget {
  final int account;
  const ChartPage({super.key, required this.account});

  @override
  ConsumerState<ChartPage> createState() => ChartPageState();
}

class ChartPageState extends ConsumerState<ChartPage> with SingleTickerProviderStateMixin {
  final formKey = GlobalKey<FormBuilderState>();
  late final tabController = TabController(length: 2, vsync: this);
  DateTime? from;
  DateTime? to;

  @override
  void dispose() {
    tabController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final pinlock = ref.watch(lifecycleProvider);
    if (pinlock.value ?? false) return PinLock();

    return Scaffold(
      appBar: AppBar(title: Text("Charts")),
      body: Padding(
        padding: EdgeInsetsGeometry.symmetric(horizontal: 8),
        child: Column(
          children: [
            FormBuilder(
              key: formKey,
              child: Row(
                crossAxisAlignment: CrossAxisAlignment.end,
                children: [
                  SizedBox(
                    width: 100,
                    child: FormBuilderDateTimePicker(
                      name: "from",
                      decoration: InputDecoration(
                        label: Text("From"),
                      ),
                      inputType: InputType.date,
                      initialValue: from,
                      onChanged: (v) => setState(() => from = v),
                    ),
                  ),
                  Gap(8),
                  SizedBox(
                    width: 100,
                    child: FormBuilderDateTimePicker(
                      name: "to",
                      decoration: InputDecoration(
                        label: Text("To"),
                      ),
                      inputType: InputType.date,
                      initialValue: to,
                      onChanged: (v) => setState(() => to = v),
                    ),
                  ),
                ],
              ),
            ),
            Divider(),
            TabBar.secondary(
              controller: tabController,
              tabs: [
                Tab(text: "Income/Expenses"),
                Tab(text: "Category"),
              ],
            ),
            Expanded(
              child: TabBarView(
                controller: tabController,
                children: [
                  SpendingChart(account: widget.account, from: from, to: to),
                  CategoryChart(account: widget.account, from: from, to: to),
                ],
              ),
            ),
          ],
        ),
      ),
    );
  }
}

/// Income or spending by category, as a donut.
class SpendingChart extends ConsumerStatefulWidget {
  final DateTime? from;
  final DateTime? to;
  final int account;
  const SpendingChart({super.key, required this.account, this.from, this.to});

  @override
  ConsumerState<SpendingChart> createState() => SpendingChartState();
}

class SpendingChartState extends ConsumerState<SpendingChart> with AutomaticKeepAliveClientMixin {
  late final c = coinContext.coin;
  List<CategorySlice> income = [];
  List<CategorySlice> spending = [];
  bool isIncome = false;
  bool loaded = false;

  @override
  void didUpdateWidget(covariant SpendingChart oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (widget.from != oldWidget.from || widget.to != oldWidget.to) {
      Future(refresh);
    }
  }

  @override
  void initState() {
    super.initState();
    Future(refresh);
  }

  Future<void> refresh() async {
    final f = widget.from?.let((dt) => dt.millisecondsSinceEpoch ~/ 1000);
    final t = widget.to?.let((dt) => dt.millisecondsSinceEpoch ~/ 1000);
    final amounts = await fetchCategoryAmounts(from: f, to: t, account: widget.account, c: c);
    final inc = <CategorySlice>[];
    final spd = <CategorySlice>[];
    for (var (name, amount, isIncome) in amounts) {
      final slice = CategorySlice(name, (amount.abs() * 1000).ceilToDouble() * 0.001);
      (isIncome ? inc : spd).add(slice);
    }
    if (!mounted) return;
    setState(() {
      income = inc;
      spending = spd;
      loaded = true;
    });
  }

  @override
  Widget build(BuildContext context) {
    super.build(context);
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        DropdownButton<bool>(
          items: [
            DropdownMenuItem(value: true, child: Text("Income")),
            DropdownMenuItem(value: false, child: Text("Spending")),
          ],
          value: isIncome,
          onChanged: (bool? v) {
            if (v != null) setState(() => isIncome = v);
          },
        ),
        Gap(8),
        Expanded(
          child: loaded
              ? CategoryDonut(slices: isIncome ? income : spending)
              : const Center(child: CircularProgressIndicator()),
        ),
      ],
    );
  }

  @override
  bool get wantKeepAlive => true;
}

/// The amounts of one category over time, each transaction a point, or
/// their running total as a step line.
class CategoryChart extends ConsumerStatefulWidget {
  final DateTime? from;
  final DateTime? to;
  final int account;
  const CategoryChart({super.key, required this.account, this.from, this.to});

  @override
  ConsumerState<CategoryChart> createState() => CategoryChartState();
}

class CategoryChartState extends ConsumerState<CategoryChart> with AutomaticKeepAliveClientMixin {
  late final c = coinContext.coin;
  List<DropdownMenuItem<int>> categoriesMenu = [];
  int? category;
  bool cumulative = false;
  bool loaded = false;
  List<(int, double)> amounts = [];

  @override
  void initState() {
    super.initState();
    Future(() async {
      final categories = await ref.read(getCategoriesProvider.future);
      if (!mounted) return;
      setState(() {
        categoriesMenu = categories.map((c) => DropdownMenuItem(value: c.id, child: Text(c.name))).toList();
        category ??= categories.firstOrNull?.id;
      });
      await refresh();
    });
  }

  @override
  void didUpdateWidget(covariant CategoryChart oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (widget.from != oldWidget.from || widget.to != oldWidget.to) {
      Future(refresh);
    }
  }

  Future<void> refresh() async {
    final cat = category;
    if (cat == null) {
      if (mounted) setState(() => loaded = true);
      return;
    }
    final f = widget.from?.let((dt) => dt.millisecondsSinceEpoch ~/ 1000);
    final t = widget.to?.let((dt) => dt.millisecondsSinceEpoch ~/ 1000);
    final a = await fetchAmounts(from: f, to: t, category: cat, account: widget.account, c: c);
    if (!mounted) return;
    setState(() {
      amounts = a;
      loaded = true;
    });
  }

  @override
  Widget build(BuildContext context) {
    super.build(context);
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Row(
          children: [
            SizedBox(width: 250, child: DropdownButton<int>(items: categoriesMenu, value: category, onChanged: onCategoryChanged)),
            Gap(8),
            Text("Cumulative"),
            Checkbox(value: cumulative, onChanged: (v) => setState(() => cumulative = v!)),
          ],
        ),
        Gap(8),
        Expanded(
          child: loaded
              ? CategoryTimeChart(points: amounts, cumulative: cumulative)
              : const Center(child: CircularProgressIndicator()),
        ),
      ],
    );
  }

  void onCategoryChanged(int? v) async {
    if (v == null) return;
    category = v;
    await refresh();
  }

  @override
  bool get wantKeepAlive => true;
}

// ── Pure chart widgets ─────────────────────────────────────────────────────

/// One slice of the donut: a category and its total.
class CategorySlice {
  final String name;
  final double value;
  const CategorySlice(this.name, this.value);

  @override
  bool operator ==(Object other) => other is CategorySlice && other.name == name && other.value == value;

  @override
  int get hashCode => Object.hash(name, value);

  @override
  String toString() => "$name: $value";
}

/// Categorical palette in a fixed order, one hue per slice, stepped for
/// each surface; colorblind-safe between neighbours. Slices beyond its
/// length fold into "Other" rather than reusing a hue.
const budgetChartColorsLight = [
  Color(0xFF2A78D6),
  Color(0xFFEB6834),
  Color(0xFF1BAF7A),
  Color(0xFFEDA100),
  Color(0xFFE87BA4),
  Color(0xFF008300),
  Color(0xFF6250D6),
  Color(0xFFE34948),
];
const budgetChartColorsDark = [
  Color(0xFF3987E5),
  Color(0xFFD95926),
  Color(0xFF199E70),
  Color(0xFFC98500),
  Color(0xFFD55181),
  Color(0xFF008300),
  Color(0xFF9085E9),
  Color(0xFFE66767),
];

List<Color> budgetChartColors(BuildContext context) =>
    Theme.of(context).brightness == Brightness.dark ? budgetChartColorsDark : budgetChartColorsLight;

/// The slices sorted by value, the largest first; when there are more than
/// [max], the smallest are summed into one "Other" slice.
List<CategorySlice> foldSlices(List<CategorySlice> slices, {int max = 8}) {
  final sorted = [...slices.where((s) => s.value > 0)]..sort((a, b) => b.value.compareTo(a.value));
  if (sorted.length <= max) return sorted;
  final kept = sorted.sublist(0, max - 1);
  final other = sorted.sublist(max - 1).fold(0.0, (sum, s) => sum + s.value);
  return [...kept, CategorySlice("Other", other)];
}

/// The running totals of [points], in order.
List<(int, double)> cumulativeSums(List<(int, double)> points) {
  var total = 0.0;
  return [
    for (final (t, v) in points) (t, total += v.abs()),
  ];
}

String _formatAmount(double v) => NumberFormat("#,##0.00").format(v);
String _formatDate(int seconds) => DateFormat("yyyy-MM-dd").format(DateTime.fromMillisecondsSinceEpoch(seconds * 1000));

/// A donut of [slices] with a legend. Tapping a slice brings it forward and
/// highlights its legend entry.
class CategoryDonut extends StatefulWidget {
  final List<CategorySlice> slices;
  const CategoryDonut({super.key, required this.slices});

  @override
  State<CategoryDonut> createState() => _CategoryDonutState();
}

class _CategoryDonutState extends State<CategoryDonut> {
  int touched = -1;

  @override
  Widget build(BuildContext context) {
    final t = Theme.of(context).textTheme;
    final colors = budgetChartColors(context);
    final slices = foldSlices(widget.slices, max: colors.length);
    if (slices.isEmpty) {
      return Center(child: Text("No amounts in this period", style: t.bodyMedium));
    }
    final total = slices.fold(0.0, (sum, s) => sum + s.value);

    return Column(
      children: [
        Expanded(
          child: LayoutBuilder(builder: (context, constraints) {
            final size = math.min(constraints.maxWidth, constraints.maxHeight);
            final outer = size * 0.35;
            final inner = size * 0.2;
            return PieChart(
              PieChartData(
                centerSpaceRadius: inner,
                sectionsSpace: 2,
                startDegreeOffset: -90,
                pieTouchData: PieTouchData(
                  touchCallback: (event, response) {
                    if (!event.isInterestedForInteractions) return;
                    final i = response?.touchedSection?.touchedSectionIndex ?? -1;
                    if (i != touched) setState(() => touched = i);
                  },
                ),
                sections: [
                  for (var i = 0; i < slices.length; i++)
                    PieChartSectionData(
                      value: slices[i].value,
                      color: colors[i],
                      radius: (outer - inner) + (i == touched ? 8 : 0),
                      title: slices[i].value / total >= 0.05 ? "${(slices[i].value / total * 100).round()}%" : "",
                      titleStyle: t.labelSmall?.copyWith(color: Colors.white, fontWeight: FontWeight.bold),
                      titlePositionPercentageOffset: 0.6,
                    ),
                ],
              ),
            );
          }),
        ),
        Gap(8),
        Wrap(
          spacing: 16,
          runSpacing: 4,
          alignment: WrapAlignment.center,
          children: [
            for (var i = 0; i < slices.length; i++)
              DonutLegendEntry(
                color: colors[i],
                label: slices[i].name,
                value: "${_formatAmount(slices[i].value)} (${(slices[i].value / total * 100).round()}%)",
                highlighted: i == touched,
                onTap: () => setState(() => touched = touched == i ? -1 : i),
              ),
          ],
        ),
        Gap(8),
      ],
    );
  }
}

class DonutLegendEntry extends StatelessWidget {
  final Color color;
  final String label;
  final String value;
  final bool highlighted;
  final VoidCallback? onTap;
  const DonutLegendEntry({
    super.key,
    required this.color,
    required this.label,
    required this.value,
    this.highlighted = false,
    this.onTap,
  });

  @override
  Widget build(BuildContext context) {
    final t = Theme.of(context).textTheme;
    final weight = highlighted ? FontWeight.bold : null;
    return InkWell(
      onTap: onTap,
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          Container(width: 10, height: 10, decoration: BoxDecoration(color: color, shape: BoxShape.circle)),
          Gap(6),
          Text(label, style: t.bodySmall?.copyWith(fontWeight: weight)),
          Gap(4),
          Text(value, style: t.bodySmall?.copyWith(fontWeight: weight, fontFeatures: const [FontFeature.tabularFigures()])),
        ],
      ),
    );
  }
}

/// The amounts of [points] over time: each one a dot, or the running total
/// as a step line when [cumulative]. Touching a point shows its date and
/// amount.
class CategoryTimeChart extends StatelessWidget {
  final List<(int, double)> points;
  final bool cumulative;
  const CategoryTimeChart({super.key, required this.points, this.cumulative = false});

  @override
  Widget build(BuildContext context) {
    final t = Theme.of(context).textTheme;
    final scheme = Theme.of(context).colorScheme;
    if (points.isEmpty) {
      return Center(child: Text("No amounts in this period", style: t.bodyMedium));
    }
    final data = cumulative ? cumulativeSums(points) : [for (final (x, y) in points) (x, y.abs())];
    final xs = data.map((p) => p.$1.toDouble());
    final ys = data.map((p) => p.$2);
    var minX = xs.reduce(math.min);
    var maxX = xs.reduce(math.max);
    if (maxX == minX) {
      minX -= 86400;
      maxX += 86400;
    }
    final spanX = maxX - minX;
    minX -= spanX * 0.05;
    maxX += spanX * 0.05;
    final maxY = ys.reduce(math.max);
    final color = budgetChartColors(context).first;
    final labelStyle = t.labelSmall;
    final gridColor = scheme.outlineVariant.withValues(alpha: 0.5);

    final titles = FlTitlesData(
      topTitles: const AxisTitles(),
      rightTitles: const AxisTitles(),
      leftTitles: AxisTitles(
        sideTitles: SideTitles(
          showTitles: true,
          reservedSize: 56,
          getTitlesWidget: (v, meta) => v == meta.max || v == meta.min
              ? const SizedBox.shrink()
              : Text(NumberFormat.compact().format(v), style: labelStyle),
        ),
      ),
      bottomTitles: AxisTitles(
        sideTitles: SideTitles(
          showTitles: true,
          reservedSize: 28,
          interval: (maxX - minX) / 4,
          getTitlesWidget: (v, meta) => v == meta.max || v == meta.min
              ? const SizedBox.shrink()
              : Padding(padding: const EdgeInsets.only(top: 6), child: Text(_formatDate(v.round()), style: labelStyle)),
        ),
      ),
    );
    final grid = FlGridData(
      drawVerticalLine: false,
      getDrawingHorizontalLine: (_) => FlLine(color: gridColor, strokeWidth: 1),
    );
    final border = FlBorderData(show: true, border: Border(bottom: BorderSide(color: gridColor), left: BorderSide(color: gridColor)));
    final tooltipStyle = t.bodySmall?.copyWith(color: scheme.onInverseSurface);
    String tooltip(double x, double y) => "${_formatDate(x.round())}\n${_formatAmount(y)}";

    final chart = cumulative
        ? LineChart(
            LineChartData(
              minX: minX,
              maxX: maxX,
              minY: 0,
              maxY: maxY * 1.1,
              titlesData: titles,
              gridData: grid,
              borderData: border,
              lineBarsData: [
                LineChartBarData(
                  spots: [for (final (x, y) in data) FlSpot(x.toDouble(), y)],
                  isStepLineChart: true,
                  lineChartStepData: const LineChartStepData(stepDirection: LineChartStepData.stepDirectionForward),
                  color: color,
                  barWidth: 2,
                  dotData: FlDotData(
                    getDotPainter: (_, __, ___, ____) => FlDotCirclePainter(radius: 4, color: color, strokeColor: scheme.surface, strokeWidth: 1),
                  ),
                ),
              ],
              lineTouchData: LineTouchData(
                touchTooltipData: LineTouchTooltipData(
                  getTooltipColor: (_) => scheme.inverseSurface,
                  getTooltipItems: (spots) => [for (final s in spots) LineTooltipItem(tooltip(s.x, s.y), tooltipStyle ?? const TextStyle())],
                ),
              ),
            ),
          )
        : ScatterChart(
            ScatterChartData(
              minX: minX,
              maxX: maxX,
              minY: 0,
              maxY: maxY * 1.1,
              titlesData: titles,
              gridData: grid,
              borderData: border,
              scatterSpots: [
                for (final (x, y) in data)
                  ScatterSpot(x.toDouble(), y, dotPainter: FlDotCirclePainter(radius: 5, color: color, strokeColor: scheme.surface, strokeWidth: 1)),
              ],
              scatterTouchData: ScatterTouchData(
                touchTooltipData: ScatterTouchTooltipData(
                  getTooltipColor: (_) => scheme.inverseSurface,
                  getTooltipItems: (spot) => ScatterTooltipItem(tooltip(spot.x, spot.y), textStyle: tooltipStyle),
                ),
              ),
            ),
          );
    return Padding(padding: const EdgeInsets.fromLTRB(0, 8, 16, 0), child: chart);
  }
}
