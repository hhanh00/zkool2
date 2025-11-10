// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'store.dart';

// **************************************************************************
// RiverpodGenerator
// **************************************************************************

// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint, type=warning

@ProviderFor(SyncStateAccount)
const syncStateAccountProvider = SyncStateAccountFamily._();

final class SyncStateAccountProvider
    extends $NotifierProvider<SyncStateAccount, SyncProgressAccount> {
  const SyncStateAccountProvider._(
      {required SyncStateAccountFamily super.from, required int super.argument})
      : super(
          retry: null,
          name: r'syncStateAccountProvider',
          isAutoDispose: true,
          dependencies: null,
          $allTransitiveDependencies: null,
        );

  @override
  String debugGetCreateSourceHash() => _$syncStateAccountHash();

  @override
  String toString() {
    return r'syncStateAccountProvider'
        ''
        '($argument)';
  }

  @$internal
  @override
  SyncStateAccount create() => SyncStateAccount();

  /// {@macro riverpod.override_with_value}
  Override overrideWithValue(SyncProgressAccount value) {
    return $ProviderOverride(
      origin: this,
      providerOverride: $SyncValueProvider<SyncProgressAccount>(value),
    );
  }

  @override
  bool operator ==(Object other) {
    return other is SyncStateAccountProvider && other.argument == argument;
  }

  @override
  int get hashCode {
    return argument.hashCode;
  }
}

String _$syncStateAccountHash() => r'f40094cf4bf2fe9500a9003d49eb5e8700ef10d8';

final class SyncStateAccountFamily extends $Family
    with
        $ClassFamilyOverride<SyncStateAccount, SyncProgressAccount,
            SyncProgressAccount, SyncProgressAccount, int> {
  const SyncStateAccountFamily._()
      : super(
          retry: null,
          name: r'syncStateAccountProvider',
          dependencies: null,
          $allTransitiveDependencies: null,
          isAutoDispose: true,
        );

  SyncStateAccountProvider call(
    int accountId,
  ) =>
      SyncStateAccountProvider._(argument: accountId, from: this);

  @override
  String toString() => r'syncStateAccountProvider';
}

abstract class _$SyncStateAccount extends $Notifier<SyncProgressAccount> {
  late final _$args = ref.$arg as int;
  int get accountId => _$args;

  SyncProgressAccount build(
    int accountId,
  );
  @$mustCallSuper
  @override
  void runBuild() {
    final created = build(
      _$args,
    );
    final ref = this.ref as $Ref<SyncProgressAccount, SyncProgressAccount>;
    final element = ref.element as $ClassProviderElement<
        AnyNotifier<SyncProgressAccount, SyncProgressAccount>,
        SyncProgressAccount,
        Object?,
        Object?>;
    element.handleValue(ref, created);
  }
}

@ProviderFor(SelectedAccount)
const selectedAccountProvider = SelectedAccountProvider._();

final class SelectedAccountProvider
    extends $NotifierProvider<SelectedAccount, Account?> {
  const SelectedAccountProvider._()
      : super(
          from: null,
          argument: null,
          retry: null,
          name: r'selectedAccountProvider',
          isAutoDispose: true,
          dependencies: null,
          $allTransitiveDependencies: null,
        );

  @override
  String debugGetCreateSourceHash() => _$selectedAccountHash();

  @$internal
  @override
  SelectedAccount create() => SelectedAccount();

  /// {@macro riverpod.override_with_value}
  Override overrideWithValue(Account? value) {
    return $ProviderOverride(
      origin: this,
      providerOverride: $SyncValueProvider<Account?>(value),
    );
  }
}

String _$selectedAccountHash() => r'b37697d30f5675cfe3c5e83772a1d9e72cf9f475';

abstract class _$SelectedAccount extends $Notifier<Account?> {
  Account? build();
  @$mustCallSuper
  @override
  void runBuild() {
    final created = build();
    final ref = this.ref as $Ref<Account?, Account?>;
    final element = ref.element as $ClassProviderElement<
        AnyNotifier<Account?, Account?>, Account?, Object?, Object?>;
    element.handleValue(ref, created);
  }
}

@ProviderFor(SelectedFolder)
const selectedFolderProvider = SelectedFolderProvider._();

final class SelectedFolderProvider
    extends $NotifierProvider<SelectedFolder, Folder?> {
  const SelectedFolderProvider._()
      : super(
          from: null,
          argument: null,
          retry: null,
          name: r'selectedFolderProvider',
          isAutoDispose: true,
          dependencies: null,
          $allTransitiveDependencies: null,
        );

  @override
  String debugGetCreateSourceHash() => _$selectedFolderHash();

  @$internal
  @override
  SelectedFolder create() => SelectedFolder();

  /// {@macro riverpod.override_with_value}
  Override overrideWithValue(Folder? value) {
    return $ProviderOverride(
      origin: this,
      providerOverride: $SyncValueProvider<Folder?>(value),
    );
  }
}

String _$selectedFolderHash() => r'745eadd2ecba8f4f49e125e11c2255b2d1949317';

abstract class _$SelectedFolder extends $Notifier<Folder?> {
  Folder? build();
  @$mustCallSuper
  @override
  void runBuild() {
    final created = build();
    final ref = this.ref as $Ref<Folder?, Folder?>;
    final element = ref.element as $ClassProviderElement<
        AnyNotifier<Folder?, Folder?>, Folder?, Object?, Object?>;
    element.handleValue(ref, created);
  }
}

@ProviderFor(getAccounts)
const getAccountsProvider = GetAccountsProvider._();

final class GetAccountsProvider extends $FunctionalProvider<
        AsyncValue<List<Account>>, List<Account>, FutureOr<List<Account>>>
    with $FutureModifier<List<Account>>, $FutureProvider<List<Account>> {
  const GetAccountsProvider._()
      : super(
          from: null,
          argument: null,
          retry: null,
          name: r'getAccountsProvider',
          isAutoDispose: true,
          dependencies: null,
          $allTransitiveDependencies: null,
        );

  @override
  String debugGetCreateSourceHash() => _$getAccountsHash();

  @$internal
  @override
  $FutureProviderElement<List<Account>> $createElement(
          $ProviderPointer pointer) =>
      $FutureProviderElement(pointer);

  @override
  FutureOr<List<Account>> create(Ref ref) {
    return getAccounts(ref);
  }
}

String _$getAccountsHash() => r'a5009cb946095d5d165abfe91b152a86d03b2c27';

@ProviderFor(getFolders)
const getFoldersProvider = GetFoldersProvider._();

final class GetFoldersProvider extends $FunctionalProvider<
        AsyncValue<List<Folder>>, List<Folder>, FutureOr<List<Folder>>>
    with $FutureModifier<List<Folder>>, $FutureProvider<List<Folder>> {
  const GetFoldersProvider._()
      : super(
          from: null,
          argument: null,
          retry: null,
          name: r'getFoldersProvider',
          isAutoDispose: true,
          dependencies: null,
          $allTransitiveDependencies: null,
        );

  @override
  String debugGetCreateSourceHash() => _$getFoldersHash();

  @$internal
  @override
  $FutureProviderElement<List<Folder>> $createElement(
          $ProviderPointer pointer) =>
      $FutureProviderElement(pointer);

  @override
  FutureOr<List<Folder>> create(Ref ref) {
    return getFolders(ref);
  }
}

String _$getFoldersHash() => r'ed216df049823b1c8da1e1bc08cc6f650f520fc3';

@ProviderFor(getCategories)
const getCategoriesProvider = GetCategoriesProvider._();

final class GetCategoriesProvider extends $FunctionalProvider<
        AsyncValue<List<Category>>, List<Category>, FutureOr<List<Category>>>
    with $FutureModifier<List<Category>>, $FutureProvider<List<Category>> {
  const GetCategoriesProvider._()
      : super(
          from: null,
          argument: null,
          retry: null,
          name: r'getCategoriesProvider',
          isAutoDispose: true,
          dependencies: null,
          $allTransitiveDependencies: null,
        );

  @override
  String debugGetCreateSourceHash() => _$getCategoriesHash();

  @$internal
  @override
  $FutureProviderElement<List<Category>> $createElement(
          $ProviderPointer pointer) =>
      $FutureProviderElement(pointer);

  @override
  FutureOr<List<Category>> create(Ref ref) {
    return getCategories(ref);
  }
}

String _$getCategoriesHash() => r'f5873059077115662988cf08580baf23b13e3681';

@ProviderFor(account)
const accountProvider = AccountFamily._();

final class AccountProvider extends $FunctionalProvider<AsyncValue<AccountData>,
        AccountData, FutureOr<AccountData>>
    with $FutureModifier<AccountData>, $FutureProvider<AccountData> {
  const AccountProvider._(
      {required AccountFamily super.from, required int super.argument})
      : super(
          retry: null,
          name: r'accountProvider',
          isAutoDispose: true,
          dependencies: null,
          $allTransitiveDependencies: null,
        );

  @override
  String debugGetCreateSourceHash() => _$accountHash();

  @override
  String toString() {
    return r'accountProvider'
        ''
        '($argument)';
  }

  @$internal
  @override
  $FutureProviderElement<AccountData> $createElement(
          $ProviderPointer pointer) =>
      $FutureProviderElement(pointer);

  @override
  FutureOr<AccountData> create(Ref ref) {
    final argument = this.argument as int;
    return account(
      ref,
      argument,
    );
  }

  @override
  bool operator ==(Object other) {
    return other is AccountProvider && other.argument == argument;
  }

  @override
  int get hashCode {
    return argument.hashCode;
  }
}

String _$accountHash() => r'7102f2ab7e2a987605f8add47baae238a48aa07c';

final class AccountFamily extends $Family
    with $FunctionalFamilyOverride<FutureOr<AccountData>, int> {
  const AccountFamily._()
      : super(
          retry: null,
          name: r'accountProvider',
          dependencies: null,
          $allTransitiveDependencies: null,
          isAutoDispose: true,
        );

  AccountProvider call(
    int id,
  ) =>
      AccountProvider._(argument: id, from: this);

  @override
  String toString() => r'accountProvider';
}

@ProviderFor(AppSettingsNotifier)
const appSettingsProvider = AppSettingsNotifierProvider._();

final class AppSettingsNotifierProvider
    extends $NotifierProvider<AppSettingsNotifier, AppSettings> {
  const AppSettingsNotifierProvider._()
      : super(
          from: null,
          argument: null,
          retry: null,
          name: r'appSettingsProvider',
          isAutoDispose: true,
          dependencies: null,
          $allTransitiveDependencies: null,
        );

  @override
  String debugGetCreateSourceHash() => _$appSettingsNotifierHash();

  @$internal
  @override
  AppSettingsNotifier create() => AppSettingsNotifier();

  /// {@macro riverpod.override_with_value}
  Override overrideWithValue(AppSettings value) {
    return $ProviderOverride(
      origin: this,
      providerOverride: $SyncValueProvider<AppSettings>(value),
    );
  }
}

String _$appSettingsNotifierHash() =>
    r'96283353cfea5b370a8152d24f609b4da0329679';

abstract class _$AppSettingsNotifier extends $Notifier<AppSettings> {
  AppSettings build();
  @$mustCallSuper
  @override
  void runBuild() {
    final created = build();
    final ref = this.ref as $Ref<AppSettings, AppSettings>;
    final element = ref.element as $ClassProviderElement<
        AnyNotifier<AppSettings, AppSettings>, AppSettings, Object?, Object?>;
    element.handleValue(ref, created);
  }
}

@ProviderFor(PriceNotifier)
const priceProvider = PriceNotifierProvider._();

final class PriceNotifierProvider
    extends $NotifierProvider<PriceNotifier, double?> {
  const PriceNotifierProvider._()
      : super(
          from: null,
          argument: null,
          retry: null,
          name: r'priceProvider',
          isAutoDispose: true,
          dependencies: null,
          $allTransitiveDependencies: null,
        );

  @override
  String debugGetCreateSourceHash() => _$priceNotifierHash();

  @$internal
  @override
  PriceNotifier create() => PriceNotifier();

  /// {@macro riverpod.override_with_value}
  Override overrideWithValue(double? value) {
    return $ProviderOverride(
      origin: this,
      providerOverride: $SyncValueProvider<double?>(value),
    );
  }
}

String _$priceNotifierHash() => r'b7a89807a11e82312888aa1c78dc319609ab355e';

abstract class _$PriceNotifier extends $Notifier<double?> {
  double? build();
  @$mustCallSuper
  @override
  void runBuild() {
    final created = build();
    final ref = this.ref as $Ref<double?, double?>;
    final element = ref.element as $ClassProviderElement<
        AnyNotifier<double?, double?>, double?, Object?, Object?>;
    element.handleValue(ref, created);
  }
}

@ProviderFor(LogNotifier)
const logProvider = LogNotifierProvider._();

final class LogNotifierProvider
    extends $NotifierProvider<LogNotifier, List<String>> {
  const LogNotifierProvider._()
      : super(
          from: null,
          argument: null,
          retry: null,
          name: r'logProvider',
          isAutoDispose: true,
          dependencies: null,
          $allTransitiveDependencies: null,
        );

  @override
  String debugGetCreateSourceHash() => _$logNotifierHash();

  @$internal
  @override
  LogNotifier create() => LogNotifier();

  /// {@macro riverpod.override_with_value}
  Override overrideWithValue(List<String> value) {
    return $ProviderOverride(
      origin: this,
      providerOverride: $SyncValueProvider<List<String>>(value),
    );
  }
}

String _$logNotifierHash() => r'1fbcc88f9bce49713c3ab734695e8bd1869f452c';

abstract class _$LogNotifier extends $Notifier<List<String>> {
  List<String> build();
  @$mustCallSuper
  @override
  void runBuild() {
    final created = build();
    final ref = this.ref as $Ref<List<String>, List<String>>;
    final element = ref.element as $ClassProviderElement<
        AnyNotifier<List<String>, List<String>>,
        List<String>,
        Object?,
        Object?>;
    element.handleValue(ref, created);
  }
}

@ProviderFor(CurrentHeightNotifier)
const currentHeightProvider = CurrentHeightNotifierProvider._();

final class CurrentHeightNotifierProvider
    extends $NotifierProvider<CurrentHeightNotifier, int?> {
  const CurrentHeightNotifierProvider._()
      : super(
          from: null,
          argument: null,
          retry: null,
          name: r'currentHeightProvider',
          isAutoDispose: true,
          dependencies: null,
          $allTransitiveDependencies: null,
        );

  @override
  String debugGetCreateSourceHash() => _$currentHeightNotifierHash();

  @$internal
  @override
  CurrentHeightNotifier create() => CurrentHeightNotifier();

  /// {@macro riverpod.override_with_value}
  Override overrideWithValue(int? value) {
    return $ProviderOverride(
      origin: this,
      providerOverride: $SyncValueProvider<int?>(value),
    );
  }
}

String _$currentHeightNotifierHash() =>
    r'f52275d8bff2395996b80041883a8ec0f2df6d0e';

abstract class _$CurrentHeightNotifier extends $Notifier<int?> {
  int? build();
  @$mustCallSuper
  @override
  void runBuild() {
    final created = build();
    final ref = this.ref as $Ref<int?, int?>;
    final element = ref.element as $ClassProviderElement<
        AnyNotifier<int?, int?>, int?, Object?, Object?>;
    element.handleValue(ref, created);
  }
}

@ProviderFor(MempoolNotifier)
const mempoolProvider = MempoolNotifierProvider._();

final class MempoolNotifierProvider
    extends $NotifierProvider<MempoolNotifier, MempoolState> {
  const MempoolNotifierProvider._()
      : super(
          from: null,
          argument: null,
          retry: null,
          name: r'mempoolProvider',
          isAutoDispose: true,
          dependencies: null,
          $allTransitiveDependencies: null,
        );

  @override
  String debugGetCreateSourceHash() => _$mempoolNotifierHash();

  @$internal
  @override
  MempoolNotifier create() => MempoolNotifier();

  /// {@macro riverpod.override_with_value}
  Override overrideWithValue(MempoolState value) {
    return $ProviderOverride(
      origin: this,
      providerOverride: $SyncValueProvider<MempoolState>(value),
    );
  }
}

String _$mempoolNotifierHash() => r'ef30b8d13903eecbdd48ca9a1c7426534fef1bde';

abstract class _$MempoolNotifier extends $Notifier<MempoolState> {
  MempoolState build();
  @$mustCallSuper
  @override
  void runBuild() {
    final created = build();
    final ref = this.ref as $Ref<MempoolState, MempoolState>;
    final element = ref.element as $ClassProviderElement<
        AnyNotifier<MempoolState, MempoolState>,
        MempoolState,
        Object?,
        Object?>;
    element.handleValue(ref, created);
  }
}

@ProviderFor(SynchronizerNotifier)
const synchronizerProvider = SynchronizerNotifierProvider._();

final class SynchronizerNotifierProvider
    extends $NotifierProvider<SynchronizerNotifier, SyncState> {
  const SynchronizerNotifierProvider._()
      : super(
          from: null,
          argument: null,
          retry: null,
          name: r'synchronizerProvider',
          isAutoDispose: true,
          dependencies: null,
          $allTransitiveDependencies: null,
        );

  @override
  String debugGetCreateSourceHash() => _$synchronizerNotifierHash();

  @$internal
  @override
  SynchronizerNotifier create() => SynchronizerNotifier();

  /// {@macro riverpod.override_with_value}
  Override overrideWithValue(SyncState value) {
    return $ProviderOverride(
      origin: this,
      providerOverride: $SyncValueProvider<SyncState>(value),
    );
  }
}

String _$synchronizerNotifierHash() =>
    r'2463400832975227eae9567852f16ca00c39cf47';

abstract class _$SynchronizerNotifier extends $Notifier<SyncState> {
  SyncState build();
  @$mustCallSuper
  @override
  void runBuild() {
    final created = build();
    final ref = this.ref as $Ref<SyncState, SyncState>;
    final element = ref.element as $ClassProviderElement<
        AnyNotifier<SyncState, SyncState>, SyncState, Object?, Object?>;
    element.handleValue(ref, created);
  }
}

@ProviderFor(TransparentScan)
const transparentScanProvider = TransparentScanProvider._();

final class TransparentScanProvider
    extends $NotifierProvider<TransparentScan, String> {
  const TransparentScanProvider._()
      : super(
          from: null,
          argument: null,
          retry: null,
          name: r'transparentScanProvider',
          isAutoDispose: true,
          dependencies: null,
          $allTransitiveDependencies: null,
        );

  @override
  String debugGetCreateSourceHash() => _$transparentScanHash();

  @$internal
  @override
  TransparentScan create() => TransparentScan();

  /// {@macro riverpod.override_with_value}
  Override overrideWithValue(String value) {
    return $ProviderOverride(
      origin: this,
      providerOverride: $SyncValueProvider<String>(value),
    );
  }
}

String _$transparentScanHash() => r'bb673d2aff5694408b86415ee471f9eb5252114c';

abstract class _$TransparentScan extends $Notifier<String> {
  String build();
  @$mustCallSuper
  @override
  void runBuild() {
    final created = build();
    final ref = this.ref as $Ref<String, String>;
    final element = ref.element as $ClassProviderElement<
        AnyNotifier<String, String>, String, Object?, Object?>;
    element.handleValue(ref, created);
  }
}

@ProviderFor(GetTxDetails)
const getTxDetailsProvider = GetTxDetailsFamily._();

final class GetTxDetailsProvider
    extends $AsyncNotifierProvider<GetTxDetails, TxAccount> {
  const GetTxDetailsProvider._(
      {required GetTxDetailsFamily super.from, required int super.argument})
      : super(
          retry: null,
          name: r'getTxDetailsProvider',
          isAutoDispose: true,
          dependencies: null,
          $allTransitiveDependencies: null,
        );

  @override
  String debugGetCreateSourceHash() => _$getTxDetailsHash();

  @override
  String toString() {
    return r'getTxDetailsProvider'
        ''
        '($argument)';
  }

  @$internal
  @override
  GetTxDetails create() => GetTxDetails();

  @override
  bool operator ==(Object other) {
    return other is GetTxDetailsProvider && other.argument == argument;
  }

  @override
  int get hashCode {
    return argument.hashCode;
  }
}

String _$getTxDetailsHash() => r'228df08f9ccc94e81142f57f41b31ab6f1731805';

final class GetTxDetailsFamily extends $Family
    with
        $ClassFamilyOverride<GetTxDetails, AsyncValue<TxAccount>, TxAccount,
            FutureOr<TxAccount>, int> {
  const GetTxDetailsFamily._()
      : super(
          retry: null,
          name: r'getTxDetailsProvider',
          dependencies: null,
          $allTransitiveDependencies: null,
          isAutoDispose: true,
        );

  GetTxDetailsProvider call(
    int id,
  ) =>
      GetTxDetailsProvider._(argument: id, from: this);

  @override
  String toString() => r'getTxDetailsProvider';
}

abstract class _$GetTxDetails extends $AsyncNotifier<TxAccount> {
  late final _$args = ref.$arg as int;
  int get id => _$args;

  FutureOr<TxAccount> build(
    int id,
  );
  @$mustCallSuper
  @override
  void runBuild() {
    final created = build(
      _$args,
    );
    final ref = this.ref as $Ref<AsyncValue<TxAccount>, TxAccount>;
    final element = ref.element as $ClassProviderElement<
        AnyNotifier<AsyncValue<TxAccount>, TxAccount>,
        AsyncValue<TxAccount>,
        Object?,
        Object?>;
    element.handleValue(ref, created);
  }
}
