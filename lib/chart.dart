import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:flutter_form_builder/flutter_form_builder.dart';
import 'package:flutter_inappwebview/flutter_inappwebview.dart';
import 'package:gap/gap.dart';
import 'package:zkool/main.dart';
import 'package:zkool/src/rust/api/transaction.dart';

class ChartPage extends StatefulWidget {
  const ChartPage({super.key});

  @override
  State<StatefulWidget> createState() => ChartPageState();
}

class ChartPageState extends State<ChartPage> {
  final formKey = GlobalKey<FormBuilderState>();
  final List<Map<String, dynamic>> data = [];

  @override
  Widget build(BuildContext context) {
    final now = DateTime.now();

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
                      initialValue: now.subtract(Duration(days: 30)),
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
                      initialValue: now,
                    ),
                  ),
                  Gap(8),
                  SizedBox(
                    width: 100,
                    child: FormBuilderDropdown(
                      name: "income",
                      initialValue: false,
                      items: [
                        DropdownMenuItem(value: true, child: Text("Income")),
                        DropdownMenuItem(value: false, child: Text("Spending")),
                      ],
                    ),
                  ),
                  Gap(16),
                  IconButton(onPressed: onRefresh, icon: Icon(Icons.check))
                ],
              ),
            ),
            Divider(),
            Expanded(child: chart(context)),
          ],
        ),
      ),
    );
  }

  void onRefresh() async {
    final fields = formKey.currentState!.fields;
    final from = fields["from"]!.value as DateTime;
    final to = fields["to"]!.value as DateTime;
    final income = fields["income"]!.value as bool;
    final f = from.millisecondsSinceEpoch ~/ 1000;
    final t = to.millisecondsSinceEpoch ~/ 1000;
    final amounts = await fetchCategoryAmounts(from: f, to: t, income: income);
    data.clear();
    for (var (c, a) in amounts) {
      data.add({
        "name": c,
        "value": (a.abs() * 1000).ceilToDouble() * 0.001,
      });
    }
    setState(() {});
  }

  Widget chart(BuildContext context) {
    return Center(
      key: UniqueKey(),
      child: InAppWebView(
        onLoadStop: (c, uri) {
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
      max-width: 600px;
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
      console.log(data);

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
    );
  }
}
