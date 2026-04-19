import 'dart:async';
import 'dart:convert';
import 'dart:io';
import 'dart:math';
import 'dart:ui';
import 'package:awesome_dialog/awesome_dialog.dart';
import 'package:convert/convert.dart';
import 'package:crypto/crypto.dart';
import 'package:decimal/intl.dart';
import 'package:file_picker/file_picker.dart';
import 'package:fixed/fixed.dart';
import 'package:flutter_passkey_service/flutter_passkey_service.dart';
import 'package:flutter_passkey_service/pigeons/messages.g.dart'
    show
        CreatePasskeyResponseData,
        GetPasskeyAuthenticationResponseData,
        PasskeyException,
        PasskeyErrorType;
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_form_builder/flutter_form_builder.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:form_builder_validators/form_builder_validators.dart';
import 'package:gap/gap.dart';
import 'package:go_router/go_router.dart';
import 'package:image_picker/image_picker.dart';
import 'package:intl/intl.dart';
import 'package:local_auth/error_codes.dart' as auth_error;
import 'package:path_provider/path_provider.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:showcaseview/showcaseview.dart';
import 'package:local_auth/local_auth.dart';
import 'package:password_strength_checker/password_strength_checker.dart';
import 'package:zkool/main.dart';
import 'package:zkool/router.dart';
import 'package:zkool/store.dart';

String initials(String name) => name.substring(0, min(2, name.length)).toUpperCase();

final locale = PlatformDispatcher.instance.locale.toString();
final formatter = NumberFormat.decimalPatternDigits(locale: locale, decimalDigits: 8);
final zatFormatter = DecimalFormatter(formatter);
final shortFormatter = NumberFormat.decimalPatternDigits(locale: locale, decimalDigits: 3);
final zatShortFormatter = DecimalFormatter(shortFormatter);
final fiatFormatter = NumberFormat.decimalPatternDigits(locale: locale, decimalDigits: 2);
final fiatCentFormatter = DecimalFormatter(fiatFormatter);
final invertSeparator = NumberFormat.decimalPattern(locale).symbols.DECIMAL_SEP != ".";

final int zatsPerZec = 100000000;

String doubleToString(double v, {required int decimals}) {
  final formatter = NumberFormat.decimalPatternDigits(locale: locale, decimalDigits: decimals);
  return formatter.format(v);
}

String zatToString(BigInt zat) {
  final z = Fixed.fromBigInt(zat, scale: 8);
  final s = zatFormatter.format(z.toDecimal());
  return s;
}

String zatToShortString(BigInt zat) {
  final z = Fixed.fromBigInt(zat, scale: 8);
  final s = zatShortFormatter.format(z.toDecimal());
  return s;
}

Widget zatToText(BigInt zat, {String prefix = "", TextStyle? style, Function()? onTap, required bool selectable, bool colored = false}) {
  style ??= Theme.of(navigatorKey.currentContext!).textTheme.bodyMedium!;
  if (colored && zat > BigInt.zero) {
    style = style.copyWith(color: Colors.green);
  }
  final s = zatToString(zat);
  final minorUnits = s.substring(s.length - 5, s.length);
  final majorUnits = s.substring(0, s.length - 5);
  return selectable
      ? Row(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.baseline,
          textBaseline: TextBaseline.alphabetic,
          children: [
            InkWell(onTap: onTap ?? () => copyToClipboard(s), child: Text(prefix)),
            SelectableText.rich(
              TextSpan(
                children: [
                  TextSpan(text: majorUnits, style: style.copyWith(fontWeight: FontWeight.bold)),
                  TextSpan(text: minorUnits, style: style.copyWith(fontSize: style.fontSize! * 0.5, color: Colors.grey)),
                ],
              ),
            ),
          ],
        )
      : Text.rich(
          TextSpan(
            children: [
              TextSpan(text: majorUnits, style: style),
              TextSpan(text: minorUnits, style: style.copyWith(fontSize: style.fontSize! * 0.6)),
            ],
          ),
        );
}

Fixed stringToDecimal(String s, {int? scale}) => Fixed.parse(s, scale: scale, invertSeparator: invertSeparator);

BigInt stringToZat(String s) {
  final z = Fixed.parse(s, scale: 8, invertSeparator: invertSeparator);
  return z.minorUnits;
}

String timeToString(int time) {
  if (time == 0) return "N/A";
  final date = DateTime.fromMillisecondsSinceEpoch(time * 1000);
  final dateString = DateFormat('yyyy-MM-dd').format(date);
  final timeAgo = compactBetween(date, DateTime.now());
  return '$dateString ($timeAgo)';
}

Widget timeToWidget(BuildContext context, int time) {
  if (time == 0) return SizedBox.shrink();
  final t = Theme.of(context).textTheme;
  final date = DateTime.fromMillisecondsSinceEpoch(time * 1000);
  final dateString = DateFormat('yyyy-MM-dd ').format(date);
  final timeAgo = compactBetween(date, DateTime.now());
  return Text.rich(TextSpan(children: [
    TextSpan(text: dateString),
    TextSpan(text: "($timeAgo)", style: t.labelSmall),
  ]));
}

String exactTimeToString(int time) {
  if (time == 0) return "N/A";
  final date = DateTime.fromMillisecondsSinceEpoch(time * 1000);
  return date.toString();
}

String compactBetween(DateTime from, DateTime to) {
  final parts = <String>[];

  int years  = to.year - from.year;
  int months = to.month - from.month;
  int days   = to.day - from.day;
  int hours  = to.hour - from.hour;
  int mins   = to.minute - from.minute;

  // Cascade borrows
  if (mins < 0)   { mins += 60;   hours -= 1; }
  if (hours < 0)  { hours += 24;  days -= 1; }
  if (days < 0) {
    final prevMonth = DateTime(to.year, to.month - 1);
    days += DateUtils.getDaysInMonth(prevMonth.year, prevMonth.month);
    months -= 1;
  }
  if (months < 0) { months += 12; years -= 1; }

  if (years  > 0) parts.add('${years}y');
  if (months > 0) parts.add('${months}mo');
  if (days   > 0) parts.add('${days}d');
  if (hours  > 0) parts.add('${hours}h');
  if (mins   > 0) parts.add('${mins}m');

  return parts.take(2).join('');
}

String txIdToString(Uint8List txid) {
  var reversed = txid.reversed.toList();
  final txId = hex.encode(reversed);
  return txId;
}

Uint8List stringToTxId(String txid) {
  var bytes = hex.decode(txid);
  return Uint8List.fromList(bytes.reversed.toList());
}

Future<String> getFullDatabasePath(String dbName) async {
  final dbDir = await getApplicationDocumentsDirectory();
  final dbFilepath = '${dbDir.path}/$dbName.db';
  return dbFilepath;
}

Future<void> showException(BuildContext context, String message) async {
  await AwesomeDialog(
    context: context,
    dialogType: DialogType.error,
    animType: AnimType.rightSlide,
    title: 'ERROR',
    desc: message,
    btnOkOnPress: () {},
    autoDismiss: true,
  ).show();
}

Future<AwesomeDialog> showMessage(BuildContext context, String message, {String? title, bool dismissable = true}) async {
  final dialog = AwesomeDialog(
    context: context,
    dialogType: DialogType.info,
    animType: AnimType.rightSlide,
    title: title,
    desc: message,
    btnOkOnPress: dismissable ? () {} : null,
    autoDismiss: true,
    dismissOnTouchOutside: dismissable,
    dismissOnBackKeyPress: dismissable,
  );
  final f = dialog.show();
  // if not dismissable, do not await because it should be dismissed
  // in code and we don't want to be hanging here
  if (dismissable) await f;
  return dialog;
}

Future<void> showSeed(BuildContext context, String message) async {
  final t = Theme.of(context).textTheme;
  await AwesomeDialog(
    context: context,
    dialogType: DialogType.warning,
    animType: AnimType.rightSlide,
    body: Column(
      children: [
        Text("SEED PHRASE - SAVE IT OR YOU CAN LOSE YOUR FUNDS", style: t.headlineSmall),
        Gap(16),
        CopyableText(
          message,
          textAlign: TextAlign.center,
        ),
      ],
    ),
    desc: message,
    btnOkOnPress: () {},
    autoDismiss: true,
  ).show();
}

void showSnackbar(String message) => ScaffoldMessenger.of(navigatorKey.currentContext!).showSnackBar(
      SnackBar(
        content: Text(message),
      ),
    );

Future<bool> confirmDialog(BuildContext context, {required String title, required String message, Widget? body}) async {
  final confirmed = await AwesomeDialog(
        context: context,
        dialogType: DialogType.question,
        animType: AnimType.rightSlide,
        title: title,
        body: body ?? Text(message),
        btnCancelOnPress: () {},
        btnOkOnPress: () {},
        onDismissCallback: (type) {
          final res = (() {
            switch (type) {
              case DismissType.btnOk:
                return true;
              default:
                return false;
            }
          })();
          GoRouter.of(context).pop(res);
        },
        autoDismiss: false,
      ).show() ??
      false;
  return confirmed;
}

Future<String?> inputPassword(
  BuildContext context, {
  required String title,
  String? btnCancelText,
  String? message,
  bool repeated = false,
  bool required = false,
}) async {
  final formKey = GlobalKey<FormBuilderState>();
  final passStrengthNotifier = ValueNotifier<PasswordStrength?>(null);
  final password = await inputData<String?>(
    context,
    builder: (context) => FormBuilder(
      key: formKey,
      child: Column(
        children: [
          Text(title, style: Theme.of(context).textTheme.headlineSmall),
          Gap(8),
          FormBuilderTextField(
            name: "password",
            autofocus: true,
            decoration: InputDecoration(labelText: 'Password', hintText: message),
            obscureText: true,
            validator: required ? FormBuilderValidators.required() : null,
            onChanged: (v) {
              passStrengthNotifier.value = PasswordStrength.calculate(text: v ?? '');
            },
          ),
          Gap(4),
          PasswordStrengthChecker(
            strength: passStrengthNotifier,
          ),
          Gap(8),
          if (repeated)
            FormBuilderTextField(
              name: "repeated_password",
              autofocus: true,
              decoration: InputDecoration(labelText: 'Repeated Password', hintText: message),
              obscureText: true,
              validator: (v) {
                final password = formKey.currentState!.fields["password"]!.value as String?;
                if (password != v) return "Passwords do not match";
                return null;
              },
            ),
        ],
      ),
    ),
    validate: () => formKey.currentState!.validate(),
    onConfirmed: () => formKey.currentState!.fields["password"]!.value as String?,
  );
  return password;
}

Future<String?> inputText(BuildContext context, {required String title}) async {
  final controller = TextEditingController();
  return await inputData(
    context,
    builder: (context) => Column(
      children: [
        Text(title, style: Theme.of(context).textTheme.headlineSmall),
        Gap(8),
        TextField(
          autofocus: true,
          controller: controller,
        ),
      ],
    ),
    onConfirmed: () => controller.text,
  );
}

Future<T?> inputData<T>(
  BuildContext context, {
  required Widget Function(BuildContext) builder,
  required T Function() onConfirmed,
  bool Function()? validate,
}) async {
  bool validated = false;
  late final AwesomeDialog dialog;
  dialog = AwesomeDialog(
    context: context,
    dialogType: DialogType.question,
    animType: AnimType.rightSlide,
    body: builder(context),
    btnCancelOnPress: () {},
    btnOkOnPress: () {},
    btnOk: AnimatedButton(
      isFixedHeight: false,
      text: "Ok",
      color: const Color(0xFF00CA71),
      pressEvent: () {
        validated = validate?.call() ?? true;
        if (validated) {
          dialog.dismiss();
        }
      },
    ),
    onDismissCallback: (type) {
      GoRouter.of(context).pop(validated);
    },
    dismissOnTouchOutside: false,
    autoDismiss: false,
  );
  final confirmed = await dialog.show();
  if (confirmed) {
    return onConfirmed();
  }
  return null;
}

Future<void> resetTutorial(BuildContext context) async {
  final prefs = SharedPreferencesAsync();
  await prefs.remove("tutMain0");
  await prefs.remove("tutMain1");
  await prefs.remove("tutNew0");
  await prefs.remove("tutNew1");
  await prefs.remove("tutNew2");
  await prefs.remove("tutEdit0");
  await prefs.remove("tutAccount0");
  await prefs.remove("tutAccount1");
  await prefs.remove("tutReceive0");
  await prefs.remove("tutSend0");
  await prefs.remove("tutSend1");
  await prefs.remove("tutSend2");
  await prefs.remove("tutSend3");
  await prefs.remove("tutSend4");
  await prefs.remove("tutSettings0");
}

void tutorialHelper(BuildContext context, String id, List<GlobalKey<State<StatefulWidget>>> ids) async {
  final prefs = SharedPreferencesAsync();
  final tutNew = await prefs.getBool(id) ?? true;
  if (tutNew) {
    if (!context.mounted) return;
    final scw = ShowCaseWidget.of(context);
    if (scw.isShowCaseCompleted) {
      scw.startShowCase(ids);
      await prefs.setBool(id, false);
    }
  }
}

Future<bool> authenticate({String? reason}) async {
  final LocalAuthentication auth = LocalAuthentication();
  try {
    final didAuthenticate =
        await auth.authenticate(localizedReason: reason ?? "Authenticate to continue", options: const AuthenticationOptions(useErrorDialogs: false));
    // if (didAuthenticate) runInAction(() => appStore.unlocked = DateTime.now());
    return didAuthenticate;
  } on PlatformException catch (e) {
    switch (e.code) {
      case auth_error.passcodeNotSet:
        return true; // no passcode set
      case auth_error.notEnrolled:
        return true; // no fingerprint enrolled
      case auth_error.notAvailable:
        return true; // don't require if the device doesn't support it
      default:
        showSnackbar("Authentication denied: ${e.code} - ${e.message}");
        return false;
    }
  } on MissingPluginException {
    // Fallback for platforms that do not support local authentication
    return true; // Assume authentication is successful
  }
}

Widget maybeShowcase(bool condition, {required GlobalKey key, required String description, required Widget child}) => condition
    ? Showcase(
        key: key,
        description: description,
        child: child,
      )
    : child;

void copyToClipboard(String text) {
  Clipboard.setData(ClipboardData(text: text));
  showSnackbar('Copied to clipboard');
}

class CopyableText extends StatelessWidget {
  final String text;
  final TextStyle? style;
  final TextAlign? textAlign;

  const CopyableText(this.text, {super.key, this.style, this.textAlign});

  @override
  Widget build(BuildContext context) {
    return SelectableText(
      text,
      style: style,
      textAlign: textAlign,
      onTap: () => copyToClipboard(text),
    );
  }
}

void lockApp(WidgetRef ref) {
  ref.read(lifecycleProvider.notifier).lock();
}

Future<bool> onUnlock(WidgetRef ref) async {
  final authenticated = await authenticate(reason: "Unlock the App");
  if (authenticated) {
    ref.read(lifecycleProvider.notifier).unlock();
  }
  return authenticated;
}

Widget blank(BuildContext context) => SizedBox.expand(child: Container(color: Theme.of(context).colorScheme.surface));

Widget showLoading(String area) =>
    Material(child: Padding(padding: EdgeInsetsGeometry.all(8), child: Text("Loading $area...", style: TextStyle(fontSize: 17))));

Widget showError(Object error) =>
    Material(child: Padding(padding: EdgeInsetsGeometry.all(8), child: Text("Error $error...", style: TextStyle(fontSize: 21, color: Colors.red))));

Future<Uint8List?> openFile({String? title}) async {
  final files = await FilePicker.platform.pickFiles(
    dialogTitle: title,
  );
  if (files != null) {
    final file = files.files.first;
    final encryptedFile = File(file.path!);
    final data = encryptedFile.readAsBytesSync();
    return data;
  }
  return null;
}

Future<XFile?> pickImage() async {
  final picker = ImagePicker();
  final icon = await picker.pickImage(source: ImageSource.gallery);
  return icon;
}

Future<String?> saveFile({String? title, String? fileName, required Uint8List data}) async {
  return await FilePicker.platform.saveFile(
    dialogTitle: title,
    fileName: fileName,
    bytes: data,
  );
}

T ensureAV<T>(BuildContext context, AsyncValue<T> av) {
  switch (av) {
    case AsyncLoading():
      throw blank(context);
    case AsyncError(:final error):
      throw showError(error);
    case AsyncData<T>(:final value):
      return value;
  }
}

extension ScopeFunctions<T> on T {
  R let<R>(R Function(T) block) => block(this);
}

// flutter_passkey_service only supports Android, iOS, and macOS
bool get passkeySupported => Platform.isAndroid || Platform.isIOS || Platform.isMacOS;

// domain associated with zkool,
// ie the author's github account
const rpId = 'hhanh00.github.io';
const rpName = 'zkool';

// Credential ID storage removed - always show picker, user selects key each time

Future<CreatePasskeyResponseData?> registerPasskey() async {
  if (!passkeySupported) throw UnsupportedError('Passkey is not supported on this platform');
  logger.i("[Passkey] registerPasskey: starting");

  final challenge = Uint8List.fromList(List<int>.generate(32, (_) => Random.secure().nextInt(256)));
  final options = FlutterPasskeyService.createRegistrationOptions(
    challenge: base64Url.encode(challenge),
    rpName: rpName,
    rpId: rpId,
    userId: base64Url.encode(utf8.encode(rpName)),
    username: rpName,
    displayName: rpName,
    enablePrf: true,
    residentKey: 'required', // GPM requires this
    requireResidentKey: true,
  );
  try {
    logger.i("[Passkey] registerPasskey: calling FlutterPasskeyService.register");
    final response = await FlutterPasskeyService.register(options);
    logger.i("[Passkey] registerPasskey: registration succeeded, id=${response.id}");

    // Check if PRF is actually supported by the authenticator
    final prfEnabled = response.clientExtensionResults.prf?.enabled ?? false;
    if (!prfEnabled) {
      logger.e("[Passkey] registerPasskey: PRF not enabled by authenticator");
      throw PasskeyException(
        errorType: PasskeyErrorType.operationNotSupported,
        message: "This authenticator does not support PRF (Pseudo-Random Function). PRF is required for passkey authentication in this app.",
        details: "Please use a different authenticator that supports PRF, such as Google Password Manager, Touch ID, or a compatible security key.",
      );
    }

    return response;
  } on PasskeyException catch (e) {
    logger.e("[Passkey] registerPasskey: registration failed (${e.errorType}): ${e.message}");
    rethrow;
  } catch (e) {
    logger.e("[Passkey] registerPasskey: registration failed: $e");
    rethrow;
  }
}

final _prfSalt = base64Url.encode(sha256.convert(utf8.encode('cc.methyl.zkool-vault-v1')).bytes);

/// Authenticate with the user's passkey. Does NOT set preferImmediatelyAvailableCredentials
/// — so the platform is free to offer "use a passkey from another device" (hybrid / CDA)
/// when no local credential is present. This always shows the picker to let the user
/// select which key to use.
Future<Uint8List> authenticatePasskey() async {
  if (!passkeySupported) throw UnsupportedError('Passkey is not supported on this platform');
  logger.i("[Passkey] authenticatePasskey: starting");
  final challenge = Uint8List.fromList(List<int>.generate(32, (_) => Random.secure().nextInt(256)));
  final options = FlutterPasskeyService.createAuthenticationOptions(
    challenge: base64Url.encode(challenge),
    rpId: rpId,
    prfEval: {'first': _prfSalt},
    // preferImmediatelyAvailableCredentials intentionally omitted — let the
    // platform offer hybrid/CDA if no local credential is found.
  );
  logger.i("[Passkey] authenticatePasskey: calling FlutterPasskeyService.authenticate");
  final GetPasskeyAuthenticationResponseData response;
  try {
    response = await FlutterPasskeyService.authenticate(options);
  } on PasskeyException catch (e) {
    logger.e("[Passkey] authenticatePasskey: authentication failed (${e.errorType}): ${e.message}");
    rethrow;
  }
  logger.i("[Passkey] authenticatePasskey: got response, checking PRF result");
  final derivedKey = response.clientExtensionResults?.prf?.results?['first'];
  if (derivedKey == null || derivedKey.isEmpty) {
    logger.e("[Passkey] authenticatePasskey: PRF derivation failed, derivedKey is null or empty");
    throw StateError('PRF derivation failed');
  }
  final prfBytes = base64Url.decode(base64Url.normalize(derivedKey));
  logger.i('[Passkey] authenticatePasskey: salt=$_prfSalt, prf=${hex.encode(prfBytes.sublist(0, 4))}...');
  return prfBytes;
}
