import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:flutter_inappwebview/flutter_inappwebview.dart';

class ChartPage extends StatefulWidget {
  @override
  State<StatefulWidget> createState() => ChartPageState();
}

class ChartPageState extends State<ChartPage> {
  late final Map<String, int> data;
  @override
  void initState() {
    super.initState();

    data = {
      "totalIncome": 7600,
      "totalExpenses": 5900,
    };
  }


  @override
  Widget build(BuildContext context) {
    return Scaffold(appBar: AppBar(title: Text("Chart")), body: Center(child: InAppWebView(
      onLoadStop: (c, uri) {
        final json = jsonEncode(data);
        c.evaluateJavascript(source: "window.dispatchEvent(new CustomEvent('flutter-data', { detail: $json }))");
        },
      onConsoleMessage: (c, m) => print(m.message),
      initialData: InAppWebViewInitialData(
        data: r"""
<html lang="en">
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width,initial-scale=1" />
  <title>ECharts — Income vs Expense</title>
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
  <h1>Income vs Expense</h1>
  <div id="chart"></div>

  <!-- ECharts CDN -->
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
            name: 'Income vs Expense',
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
            data: [
              {name: 'Total Income', value: data.totalIncome},
              {name: 'Total Expenses', value: data.totalExpenses}
            ]
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
"""))));
  }
}
