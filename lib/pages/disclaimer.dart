import 'package:flutter/material.dart';
import 'package:flutter_form_builder/flutter_form_builder.dart';
import 'package:gap/gap.dart';
import 'package:go_router/go_router.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:zkool/store.dart';

class DisclaimerPage extends StatefulWidget {
  const DisclaimerPage({super.key});

  @override
  State<DisclaimerPage> createState() => _DisclaimerPageState();
}

class _DisclaimerPageState extends State<DisclaimerPage> {
  final formKey = GlobalKey<FormBuilderState>();
  bool agree = false;

  @override
  Widget build(BuildContext context) {
    final t = Theme.of(context).textTheme;
    return Scaffold(
      appBar: AppBar(
        title: const Text("Disclaimer"),
      ),
      body: SingleChildScrollView(
          child: Padding(
              padding: EdgeInsets.symmetric(horizontal: 16.0),
              child: FormBuilder(
                key: formKey,
                child: Column(
                  children: [
                    Text("Self-Custody Responsibility", style: t.headlineSmall),
                    Gap(8),
                    Text(
                      "This wallet is a self-custody solution for Zcash (ZEC). You are solely responsible for the security and management of your private keys and recovery phrases. If you lose your private keys or recovery phrase, your funds cannot be recovered. There is no central authority or support service that can access or restore your wallet.",
                      style: t.bodyMedium,
                    ),
                    Gap(16),
                    Text("No Financial Advice", style: t.headlineSmall),
                    Gap(8),
                    Text(
                      "This application does not provide financial, investment, or tax advice. Always conduct your own research and consult with a qualified professional before making financial decisions.",
                      style: t.bodyMedium,
                    ),
                    Gap(16),
                    Text("Risk of Loss", style: t.headlineSmall),
                    Gap(8),
                    Text(
                        "Cryptocurrencies, including Zcash, are subject to high market volatility. By using this wallet, you acknowledge and accept the risks associated with holding, transacting, or storing Zcash. We are not responsible for any loss of funds due to user error, software bugs, third-party attacks, or network-related issues.",
                        style: t.bodyMedium),
                    Gap(16),
                    Text("No Liability", style: t.headlineSmall),
                    Gap(8),
                    Text(
                      "In no event shall we be liable for any direct, indirect, incidental, special, consequential, or punitive damages arising from your use of this wallet or any related services. This includes but is not limited to loss of profits, data, or other intangible losses.",
                      style: t.bodyMedium,
                    ),
                    Gap(16),
                    Text("Use at Your Own Risk", style: t.headlineSmall),
                    Gap(8),
                    Text(
                      "By using this wallet, you acknowledge that you have read and understood this disclaimer and agree to its terms. If you do not agree with any part of this disclaimer, you should not use this wallet.",
                      style: t.bodyMedium,
                    ),
                    Gap(16),
                    FormBuilderSwitch(
                        name: "agree",
                        title: Text("I Agree"),
                        initialValue: agree,
                        onChanged: (v) => setState(() => agree = v!)),
                    Gap(16),
                    ElevatedButton(
                      onPressed: agree
                          ? onContinue
                          : null,
                      child: Text("Continue"),
                    ),
                    Gap(16),
                  ],
                ),
              ))),
    );
  }

  void onContinue() async {
    final prefs = SharedPreferencesAsync();
    await prefs.setBool("disclaimer_accepted", true);
    appStore.disclaimerAccepted = true;
    if (!mounted) return;
    GoRouter.of(context).pop();
  }
}
