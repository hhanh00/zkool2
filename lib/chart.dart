import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:flutter_form_builder/flutter_form_builder.dart';
import 'package:flutter_inappwebview/flutter_inappwebview.dart';
import 'package:gap/gap.dart';
import 'package:zkool/main.dart';
import 'package:zkool/src/rust/api/transaction.dart';
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
      appBar: AppBar(),
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
            Expanded(child: SpendingChart(key: ValueKey((from, to)), from: from, to: to)),
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

class SpendingChartState extends State<SpendingChart> {
  final List<Map<String, dynamic>> income = [];
  final List<Map<String, dynamic>> spending = [];
  bool isIncome = false;
  int epoch = 0;

  @override
  void didUpdateWidget(covariant SpendingChart oldWidget) {
    super.didUpdateWidget(oldWidget);
    epoch += 1;
  }

  @override
  void initState() {
    super.initState();
    logger.i("---> ${widget.from} ${widget.to}");

    Future(() async {
      final f = widget.from?.let((dt) => dt.millisecondsSinceEpoch ~/ 1000);
      final t = widget.to?.let((dt) => dt.millisecondsSinceEpoch ~/ 1000);
      final amounts = await fetchCategoryAmounts(from: f, to: t);
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
      if (mounted) setState(() {});
    });
  }

  @override
  Widget build(BuildContext context) {
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
              key: ValueKey((epoch, isIncome)),
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
      margin: 20px;
      background: #fafafa;
      color: #222;
    }
    #chart {
      width: 100%;
      max-width: 800px;
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
          formatter: '{b}: {c} ({d}%)'
        },
        legend: {
          top: '5%',
          left: 'center'
        },
        series: [
          {
            name: 'Income/Expense by Category',
            type: 'pie',
            radius: ['40%', '70%'],
            avoidLabelOverlap: false,
            itemStyle: { borderRadius: 6, borderColor: '#fff', borderWidth: 2 },
            label: {
              show: true,
              formatter: '{b}: {c}'
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
}
