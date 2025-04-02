import 'package:flutter/material.dart';
import 'package:zkool/src/rust/api/account.dart';
import 'package:zkool/src/rust/frb_generated.dart';
import 'package:path_provider/path_provider.dart';

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();
  await RustLib.init();
  final dbDir = await getApplicationDocumentsDirectory();
  final dbFilepath = '${dbDir.path}/zkool.db';
  setDbFilepath(coin: 0, dbFilepath: dbFilepath);

  Setup.init();

  runApp(MaterialApp(home: const AccountList()));
}

class Setup {
  static void init() {
    final phrase =
        "destroy public fog slim about evolve traffic chef moment genius curtain spell genius mimic gravity around spot plug genre soldier warm basic anchor toddler";
    final extendedSecret =
        "secret-extended-key-main1qwquf4t7qqqqpqzsrxq0tgftjp75hrw0r03msyep2r8pw33uftmzcwkjadgqva27umk2y2zzewqrgj07cgj42gkx4pp3cfp255uhwh0930gfwexvsytqshfvuv24ygtd229sxdfc7dxtea36m0sx9lkejf3zatvy0wwa2uqvzaylq7ewxka6zka35282r20jhgh2ann4pk9hg0x23s32xasuy3kkt7t5nz7mysn9h9xyhrrajcdhwgp0cths6s680sqmqz29vpyh2lqpaefr6";
    final extendedViewing =
        "zxviews1qwquf4t7qqqqpqzsrxq0tgftjp75hrw0r03msyep2r8pw33uftmzcwkjadgqva27umaj5nx9lnxma45fay73gxwwj5kuk692zlc45jgjdusd0kg8chhtdf7v6fmamjt8fj3ym5wc6mtmpvr0j0ds4k2qem2zcgx5lcs3szk5zaylq7ewxka6zka35282r20jhgh2ann4pk9hg0x23s32xasuy3kkt7t5nz7mysn9h9xyhrrajcdhwgp0cths6s680sqmqz29vpyh2lq3kdkdt";
    final unifiedViewing =
        "uview1yhlefqrcqp34ra72vk2uezt9srcedengzyrf2gx4sp3dru97cfx6zju4ytpygks0cvc7fwtvwxvkmcanug75ge0l929mevcjrnckj0p3kaxmfsvh84a8rxtr5zt6gmwljwwhwkxsq3x0ffyyx59sk83cs58frhflwmpqy8h6luzzpkd3pahawnrr8wqhlhh2vngrqh4eg4cwzczmty56sqsfh69zglkzgz5zd94y20hf2rl8zclfqk9dmcy4qk62a9ppqra896gxxg936qg2djphh22tgxczehrlv3dr92ygm82v2kwg4ju3t9fm7l7ugrpx4ua8ee6dez5ht9acxjd993w9ve3xau5j40ydjxfurlr42cc3xuejzjdc2yvx9ec0v6jj5xrtw79xemewv4jy9mt3f5rejwa9gcnqx02f53uzxkpwnj3c0qwm3l9qtz32srz904qq30y6q6vd658x306ghnh6qxhjh57lav2dqnn0rcgnflgq";
    final tsk = "KxmMwLqraoufe6u6sh3siWtPa3po8k89jYw3PRsVzwV24Ggooaei";
    final account =
        putAccountMetadata(coin: 0, name: "Hanh", birth: 1, height: 10);
    putAccountSeed(coin: 0, id: account, phrase: phrase, aindex: 0);

    final account2 =
        putAccountMetadata(coin: 0, name: "Hanh2", birth: 1, height: 10);
    putAccountSaplingSecret(coin: 0, id: account2, esk: extendedSecret);

    final account3 =
        putAccountMetadata(coin: 0, name: "Hanh3", birth: 1, height: 10);
    putAccountSaplingViewing(
        coin: 0, id: account3, evk: extendedViewing);

    final account4 =
        putAccountMetadata(coin: 0, name: "Hanh3", birth: 1, height: 10);
    putAccountUnifiedViewing(coin: 0, id: account4, uvk: unifiedViewing);

    final account5 =
        putAccountMetadata(coin: 0, name: "Hanh3", birth: 1, height: 10);
    putAccountTransparentSecret(coin: 0, id: account5, sk: tsk);
  }
}

class MyApp extends StatefulWidget {
  const MyApp({super.key});

  @override
  State<MyApp> createState() => MyAppState();
}

class MyAppState extends State<MyApp> {
  @override
  Widget build(BuildContext context) {
    final ufvk = getAccountUfvk(coin: 0, id: 1);
    final ua = uaFromUfvk(coin: 0, ufvk: ufvk);
    final receivers = receiversFromUa(coin: 0, ua: ua);

    return MaterialApp(
      home: Scaffold(
        appBar: AppBar(title: const Text('ZKOOL 2')),
        body: Column(children: [
          ListTile(title: Text("UA"), subtitle: Text(ua)),
          ListTile(title: Text("T"), subtitle: Text(receivers.taddr ?? "")),
          ListTile(title: Text("S"), subtitle: Text(receivers.saddr ?? "")),
          ListTile(title: Text("O"), subtitle: Text(receivers.oaddr ?? "")),
        ]),
      ),
    );
  }
}
