// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'store.dart';

// **************************************************************************
// RiverpodGenerator
// **************************************************************************

// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint, type=warning

@ProviderFor(HasDb)
const hasDbProvider = HasDbProvider._();

final class HasDbProvider extends $NotifierProvider<HasDb, bool> {
  const HasDbProvider._()
      : super(
          from: null,
          argument: null,
          retry: null,
          name: r'hasDbProvider',
          isAutoDispose: true,
          dependencies: null,
          $allTransitiveDependencies: null,
        );

  @override
  String debugGetCreateSourceHash() => _$hasDbHash();

  @$internal
  @override
  HasDb create() => HasDb();

  /// {@macro riverpod.override_with_value}
  Override overrideWithValue(bool value) {
    return $ProviderOverride(
      origin: this,
      providerOverride: $SyncValueProvider<bool>(value),
    );
  }
}

String _$hasDbHash() => r'ef7efd1b03e4e711b6d25b8c20fd8c687ce2b5f0';

abstract class _$HasDb extends $Notifier<bool> {
  bool build();
  @$mustCallSuper
  @override
  void runBuild() {
    final created = build();
    final ref = this.ref as $Ref<bool, bool>;
    final element = ref.element as $ClassProviderElement<
        AnyNotifier<bool, bool>, bool, Object?, Object?>;
    element.handleValue(ref, created);
  }
}

@ProviderFor(SyncStateAccount)
const syncStateAccountProvider = SyncStateAccountFamily._();

final class SyncStateAccountProvider
    extends $AsyncNotifierProvider<SyncStateAccount, SyncProgressAccount> {
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

  @override
  bool operator ==(Object other) {
    return other is SyncStateAccountProvider && other.argument == argument;
  }

  @override
  int get hashCode {
    return argument.hashCode;
  }
}

String _$syncStateAccountHash() => r'47d643e6ee5cbb1805ef9fdd4db3efb3a85018ec';

final class SyncStateAccountFamily extends $Family
    with
        $ClassFamilyOverride<SyncStateAccount, AsyncValue<SyncProgressAccount>,
            SyncProgressAccount, FutureOr<SyncProgressAccount>, int> {
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

abstract class _$SyncStateAccount extends $AsyncNotifier<SyncProgressAccount> {
  late final _$args = ref.$arg as int;
  int get accountId => _$args;

  FutureOr<SyncProgressAccount> build(
    int accountId,
  );
  @$mustCallSuper
  @override
  void runBuild() {
    final created = build(
      _$args,
    );
    final ref =
        this.ref as $Ref<AsyncValue<SyncProgressAccount>, SyncProgressAccount>;
    final element = ref.element as $ClassProviderElement<
        AnyNotifier<AsyncValue<SyncProgressAccount>, SyncProgressAccount>,
        AsyncValue<SyncProgressAccount>,
        Object?,
        Object?>;
    element.handleValue(ref, created);
  }
}

@ProviderFor(SelectedAccount)
const selectedAccountProvider = SelectedAccountProvider._();

final class SelectedAccountProvider
    extends $AsyncNotifierProvider<SelectedAccount, Account?> {
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
}

String _$selectedAccountHash() => r'534a35d7a5729d455b88407867887a989fc5f13c';

abstract class _$SelectedAccount extends $AsyncNotifier<Account?> {
  FutureOr<Account?> build();
  @$mustCallSuper
  @override
  void runBuild() {
    final created = build();
    final ref = this.ref as $Ref<AsyncValue<Account?>, Account?>;
    final element = ref.element as $ClassProviderElement<
        AnyNotifier<AsyncValue<Account?>, Account?>,
        AsyncValue<Account?>,
        Object?,
        Object?>;
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

String _$getAccountsHash() => r'1a480c593e356312ff67118bbd325b51dd3b49f7';

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

String _$getFoldersHash() => r'f0e35928aa1c400a44939be7758e2306af958e10';

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

String _$getCategoriesHash() => r'45698dfd3290ba0ea7fd14508581e3d75b280f73';

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
    extends $AsyncNotifierProvider<AppSettingsNotifier, AppSettings> {
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
}

String _$appSettingsNotifierHash() =>
    r'de1f279d4215b7e76715812606cff477d0ebba29';

abstract class _$AppSettingsNotifier extends $AsyncNotifier<AppSettings> {
  FutureOr<AppSettings> build();
  @$mustCallSuper
  @override
  void runBuild() {
    final created = build();
    final ref = this.ref as $Ref<AsyncValue<AppSettings>, AppSettings>;
    final element = ref.element as $ClassProviderElement<
        AnyNotifier<AsyncValue<AppSettings>, AppSettings>,
        AsyncValue<AppSettings>,
        Object?,
        Object?>;
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
    r'9c2723ecee0ecd1178a1bd679782cbb7e7d40ea8';

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
