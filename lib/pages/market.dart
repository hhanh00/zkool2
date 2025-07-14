import 'package:flutter/material.dart';
import 'package:flutter_inappwebview/flutter_inappwebview.dart';

class MarketPrice extends StatefulWidget {
  const MarketPrice({super.key});

  @override
  State<MarketPrice> createState() => MarketPriceState();
}

class MarketPriceState extends State<MarketPrice> {
  @override
  Widget build(BuildContext context) {
    final t = Theme.of(context).textTheme;

    return Scaffold(
      appBar: AppBar(title: Text("Market Price", style: t.titleLarge)),
      body: Center(
        child: InAppWebView(
          initialData: InAppWebViewInitialData(data: r"""
          <html>
            <head>
              <meta name='viewport' content='width=device-width, initial-scale=1.0'>
              <title>Market Price</title>
            </head>
            <body>
            <script type="text/javascript" src="https://s3.tradingview.com/external-embedding/embed-widget-advanced-chart.js" async>
            {
              "allow_symbol_change": false,
              "calendar": false,
              "details": false,
              "hide_side_toolbar": true,
              "hide_top_toolbar": false,
              "hide_legend": false,
              "hide_volume": false,
              "hotlist": false,
              "interval": "D",
              "locale": "en",
              "save_image": false,
              "style": "1",
              "symbol": "ZECUSDC.P",
              "withdateranges": true,
              "autosize": true
            }
            </script>
          </body>"""),
        ),
      ),
    );
  }
}
