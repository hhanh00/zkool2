import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:flutter_form_builder/flutter_form_builder.dart';
import 'package:flutter_inappwebview/flutter_inappwebview.dart';
import 'package:gap/gap.dart';
import 'package:zkool/src/rust/api/transaction.dart';
import 'package:zkool/store.dart';
import 'package:zkool/utils.dart';

class ChartPage extends StatefulWidget {
  const ChartPage({super.key});

  @override
  State<StatefulWidget> createState() => ChartPageState();
}

class ChartPageState extends State<ChartPage> with SingleTickerProviderStateMixin {
  final formKey = GlobalKey<FormBuilderState>();
  final List<Map<String, dynamic>> data = [];
  late final tabController = TabController(length: 2, vsync: this);
  DateTime? from;
  DateTime? to;

  @override
  Widget build(BuildContext context) {
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
                  SpendingChart(from: from, to: to),
                  CategoryChart(from: from, to: to),
                ],
              ),
            ),
          ],
        ),
      ),
    );
  }
}

class SpendingChart extends StatefulWidget {
  final DateTime? from;
  final DateTime? to;
  const SpendingChart({super.key, this.from, this.to});

  @override
  State<StatefulWidget> createState() => SpendingChartState();
}

class SpendingChartState extends State<SpendingChart> with AutomaticKeepAliveClientMixin {
  final List<Map<String, dynamic>> income = [];
  final List<Map<String, dynamic>> spending = [];
  bool isIncome = false;
  int epoch = 0;

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
    final amounts = await fetchCategoryAmounts(from: f, to: t);
    income.clear();
    spending.clear();
    for (var (c, a, i) in amounts) {
      final datum = {
        "name": c,
        "value": (a.abs() * 1000).ceilToDouble() * 0.001,
      };
      if (i) {
        income.add(datum);
      } else {
        spending.add(datum);
      }
    }
    if (mounted)
      setState(() {
        epoch += 1;
      });
  }

  @override
  Widget build(BuildContext context) {
    super.build(context);
    return Center(
      child: Column(
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
            child: InAppWebView(
              key: ValueKey((isIncome, epoch)),
              onLoadStop: (c, uri) {
                final data = isIncome ? income : spending;
                final json = jsonEncode(data);
                c.evaluateJavascript(source: "window.dispatchEvent(new CustomEvent('flutter-data', { detail: $json }))");
              },
              initialData: InAppWebViewInitialData(
                data: r"""
<html lang="en">
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width,initial-scale=1" />
  <style>
    body {
      font-family: system-ui, -apple-system, "Segoe UI", Roboto, "Helvetica Neue", Arial;
      margin: 8px;
      background: #fafafa;
      color: #222;
    }
    #chart {
      width: 100%;
      height: 550px;
      margin: auto;
    }
  </style>
</head>
<body>
  <div id="chart"></div>
  <script src="https://cdn.jsdelivr.net/npm/echarts@5/dist/echarts.min.js"></script>
  <script>
    function initChart(data) {
      const chart = echarts.init(document.getElementById('chart'));

      const option = {
        tooltip: {
          trigger: 'item',
          formatter: function(params) {
            return params.name + ': ' + Math.round(params.value) + ' (' + Math.round(params.percent) + '%)';
          }
        },
        legend: {
          top: '1%',
          left: 'center'
        },
        series: [
          {
            name: 'Income/Expense by Category',
            type: 'pie',
            radius: ['40%', '70%'],
            center: ['50%', '60%'],
            avoidLabelOverlap: false,
            itemStyle: { borderRadius: 6, borderColor: '#fff', borderWidth: 2 },
            label: {
              show: true,
              formatter: function(params) {
                return params.name + ': ' + Math.round(params.value) + ' (' + Math.round(params.percent) + '%)';
              }
            },
            emphasis: {
              label: { show: true, fontSize: 18, fontWeight: 'bold' }
            },
            data
          }
        ]
      };

      chart.setOption(option);
      window.addEventListener('resize', () => chart.resize());
    }

    window.addEventListener('flutter-data', (e) => initChart(e.detail));
  </script>
</body>
</html>
""",
              ),
            ),
          ),
        ],
      ),
    );
  }

  @override
  bool get wantKeepAlive => true;
}

class CategoryChart extends StatefulWidget {
  final DateTime? from;
  final DateTime? to;
  const CategoryChart({super.key, this.from, this.to});

  @override
  State<StatefulWidget> createState() => CategoryChartState();
}

class CategoryChartState extends State<CategoryChart> with AutomaticKeepAliveClientMixin {
  late final List<DropdownMenuItem<int>> categories;
  int? category = 1;
  bool cumulative = false;
  int epoch = 0;
  List<(int, double)> amounts = [];

  @override
  void initState() {
    super.initState();
    categories = appStore.categories
        .map(
          (c) => DropdownMenuItem(
            value: c.id,
            child: Text(c.name),
          ),
        )
        .toList();
    Future(refresh);
  }

  @override
  void didUpdateWidget(covariant CategoryChart oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (widget.from != oldWidget.from || widget.to != oldWidget.to) {
      Future(refresh);
    }
  }

  Future<void> refresh() async {
    final f = widget.from?.let((dt) => dt.millisecondsSinceEpoch ~/ 1000);
    final t = widget.to?.let((dt) => dt.millisecondsSinceEpoch ~/ 1000);
    amounts = await fetchAmounts(from: f, to: t, category: category!);
    setState(() {
      epoch += 1;
    });
  }

  @override
  Widget build(BuildContext context) {
    super.build(context);

    final data = amounts.map((a) => [a.$1 * 1000.0, a.$2.abs()]).toList();
    if (cumulative) {
      var agg = 0.0;
      for (var i = 0; i < data.length; i++) {
        data[i][1] = agg + data[i][1];
        agg = data[i][1];
      }
    }

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Row(children: [
          SizedBox(width: 250, child: DropdownButton<int>(items: categories, value: category, onChanged: onCategoryChanged)),
          Gap(8),
          Text("Cumulative"),
          Checkbox(value: cumulative, onChanged: (v) => setState(() => cumulative = v!)),
        ]),
        Gap(8),
        Expanded(
          child: InAppWebView(
            key: ValueKey((category, cumulative, epoch)),
            onLoadStop: (c, uri) {
              final json = jsonEncode({"type": cumulative ? "line" : "scatter", "data": data});
              c.evaluateJavascript(source: "window.dispatchEvent(new CustomEvent('flutter-data', { detail: $json }))");
            },
            initialData: InAppWebViewInitialData(
              data: r"""
<html lang="en">
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width,initial-scale=1" />
  <style>
    body {
      font-family: system-ui, -apple-system, "Segoe UI", Roboto, "Helvetica Neue", Arial;
      margin: 8px;
      background: #fafafa;
      color: #222;
    }
    #chart {
      width: 100%;
      height: 400px;
      margin: auto;
    }
  </style>
</head>
<body>
  <div id="chart"></div>
  <script src="https://cdn.jsdelivr.net/npm/echarts@5/dist/echarts.min.js"></script>
  <script>
    function initChart(data) {
      const chart = echarts.init(document.getElementById('chart'));

      const option = {
        tooltip: {
          trigger: 'item',
          formatter: function (params) {
            const date = new Date(params.value[0]);
            return date.toLocaleString() + '<br/>Amount: ' + params.value[1];
          }
        },
        xAxis: {
          type: 'time',
          name: 'Date'
        },
        yAxis: {
          type: 'value',
          name: 'Amount'
        },
        series: [
          {
            type: data.type,
            step: 'start',
            symbolSize: 10,
            data: data.data,
          }
        ]
      };

      chart.setOption(option);
      window.addEventListener('resize', () => chart.resize());
    }

    window.addEventListener('flutter-data', (e) => initChart(e.detail));
  </script>
</body>
</html>
""",
            ),
          ),
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
