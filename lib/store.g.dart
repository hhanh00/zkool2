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

@ProviderFor(SelectedAccountId)
const selectedAccountIdProvider = SelectedAccountIdProvider._();

final class SelectedAccountIdProvider
    extends $NotifierProvider<SelectedAccountId, int> {
  const SelectedAccountIdProvider._()
      : super(
          from: null,
          argument: null,
          retry: null,
          name: r'selectedAccountIdProvider',
          isAutoDispose: true,
          dependencies: null,
          $allTransitiveDependencies: null,
        );

  @override
  String debugGetCreateSourceHash() => _$selectedAccountIdHash();

  @$internal
  @override
  SelectedAccountId create() => SelectedAccountId();

  /// {@macro riverpod.override_with_value}
  Override overrideWithValue(int value) {
    return $ProviderOverride(
      origin: this,
      providerOverride: $SyncValueProvider<int>(value),
    );
  }
}

String _$selectedAccountIdHash() => r'833aad0e4b19e6812674eedc0418e83387d5ee59';

abstract class _$SelectedAccountId extends $Notifier<int> {
  int build();
  @$mustCallSuper
  @override
  void runBuild() {
    final created = build();
    final ref = this.ref as $Ref<int, int>;
    final element = ref.element
        as $ClassProviderElement<AnyNotifier<int, int>, int, Object?, Object?>;
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

String _$syncStateAccountHash() => r'cb3d58d81b59192492c0aab60de138055b823f7f';

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

@ProviderFor(selectedAccount)
const selectedAccountProvider = SelectedAccountProvider._();

final class SelectedAccountProvider extends $FunctionalProvider<
        AsyncValue<Account?>, Account?, FutureOr<Account?>>
    with $FutureModifier<Account?>, $FutureProvider<Account?> {
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
  $FutureProviderElement<Account?> $createElement($ProviderPointer pointer) =>
      $FutureProviderElement(pointer);

  @override
  FutureOr<Account?> create(Ref ref) {
    return selectedAccount(ref);
  }
}

String _$selectedAccountHash() => r'749b6e2d1f8d9a677d054da6da362642a8a4198d';

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
          isAutoDispose: false,
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

String _$getAccountsHash() => r'4628dce465555f59311a5f3232bb00fbfb6e428c';

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

String _$getFoldersHash() => r'2458237b23db05d19a7b49856e9987542680249e';

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

String _$getCategoriesHash() => r'b936c571d89ff2ede483f5239881ba90219af321';

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

String _$accountHash() => r'0c9f9dc4a92e89a15e9f89d71949c1c84c1a5817';

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

@ProviderFor(getCurrentAccount)
const getCurrentAccountProvider = GetCurrentAccountProvider._();

final class GetCurrentAccountProvider extends $FunctionalProvider<
        AsyncValue<AccountData?>, AccountData?, FutureOr<AccountData?>>
    with $FutureModifier<AccountData?>, $FutureProvider<AccountData?> {
  const GetCurrentAccountProvider._()
      : super(
          from: null,
          argument: null,
          retry: null,
          name: r'getCurrentAccountProvider',
          isAutoDispose: false,
          dependencies: null,
          $allTransitiveDependencies: null,
        );

  @override
  String debugGetCreateSourceHash() => _$getCurrentAccountHash();

  @$internal
  @override
  $FutureProviderElement<AccountData?> $createElement(
          $ProviderPointer pointer) =>
      $FutureProviderElement(pointer);

  @override
  FutureOr<AccountData?> create(Ref ref) {
    return getCurrentAccount(ref);
  }
}

String _$getCurrentAccountHash() => r'fb9e03f8c767fe77e0f33e71495c3bf0c167c7a1';

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
          isAutoDispose: false,
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
    r'd9771f9ad0db346653ce1f2f9a22f28d21ea76d2';

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
          isAutoDispose: false,
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

String _$priceNotifierHash() => r'9c6f7b369e9fef6348fa6fdda446294b8c8dac98';

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
          isAutoDispose: false,
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

String _$logNotifierHash() => r'6b5eec2c62a37ad6e6eff3b197288e47e9ea932d';

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
          isAutoDispose: false,
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
    r'16daa9bce5d88e0c013ac53051b9c0479659ff6d';

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

@ProviderFor(ElectionNotifier)
const electionProvider = ElectionNotifierProvider._();

final class ElectionNotifierProvider
    extends $NotifierProvider<ElectionNotifier, ElectionData?> {
  const ElectionNotifierProvider._()
      : super(
          from: null,
          argument: null,
          retry: null,
          name: r'electionProvider',
          isAutoDispose: false,
          dependencies: null,
          $allTransitiveDependencies: null,
        );

  @override
  String debugGetCreateSourceHash() => _$electionNotifierHash();

  @$internal
  @override
  ElectionNotifier create() => ElectionNotifier();

  /// {@macro riverpod.override_with_value}
  Override overrideWithValue(ElectionData? value) {
    return $ProviderOverride(
      origin: this,
      providerOverride: $SyncValueProvider<ElectionData?>(value),
    );
  }
}

String _$electionNotifierHash() => r'7ab19af17d5ec65bbf88cd66f2cd7b912022bfe0';

abstract class _$ElectionNotifier extends $Notifier<ElectionData?> {
  ElectionData? build();
  @$mustCallSuper
  @override
  void runBuild() {
    final created = build();
    final ref = this.ref as $Ref<ElectionData?, ElectionData?>;
    final element = ref.element as $ClassProviderElement<
        AnyNotifier<ElectionData?, ElectionData?>,
        ElectionData?,
        Object?,
        Object?>;
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
          isAutoDispose: false,
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

String _$mempoolNotifierHash() => r'e30275d3d759d1bee1dc3b4f5eb18fa1bd0cedc4';

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
          isAutoDispose: false,
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
    r'a5f47097b8bcb7187b66b4e7ca657931f08b443e';

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
          isAutoDispose: false,
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

String _$transparentScanHash() => r'9a8d590be183064d275865b9fa96668cf104996c';

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

String _$getTxDetailsHash() => r'67175e914e53d2de8944db85e0f9225374cba276';

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

@ProviderFor(Lifecycle)
const lifecycleProvider = LifecycleProvider._();

final class LifecycleProvider extends $AsyncNotifierProvider<Lifecycle, bool> {
  const LifecycleProvider._()
      : super(
          from: null,
          argument: null,
          retry: null,
          name: r'lifecycleProvider',
          isAutoDispose: false,
          dependencies: null,
          $allTransitiveDependencies: null,
        );

  @override
  String debugGetCreateSourceHash() => _$lifecycleHash();

  @$internal
  @override
  Lifecycle create() => Lifecycle();
}

String _$lifecycleHash() => r'84c4bc8b08d2a464e0b67519730300ec766a48bb';

abstract class _$Lifecycle extends $AsyncNotifier<bool> {
  FutureOr<bool> build();
  @$mustCallSuper
  @override
  void runBuild() {
    final created = build();
    final ref = this.ref as $Ref<AsyncValue<bool>, bool>;
    final element = ref.element as $ClassProviderElement<
        AnyNotifier<AsyncValue<bool>, bool>,
        AsyncValue<bool>,
        Object?,
        Object?>;
    element.handleValue(ref, created);
  }
}

@ProviderFor(accountsPageData)
const accountsPageDataProvider = AccountsPageDataProvider._();

final class AccountsPageDataProvider extends $FunctionalProvider<
        AsyncValue<AccountsPageData>,
        AccountsPageData,
        FutureOr<AccountsPageData>>
    with $FutureModifier<AccountsPageData>, $FutureProvider<AccountsPageData> {
  const AccountsPageDataProvider._()
      : super(
          from: null,
          argument: null,
          retry: null,
          name: r'accountsPageDataProvider',
          isAutoDispose: true,
          dependencies: null,
          $allTransitiveDependencies: null,
        );

  @override
  String debugGetCreateSourceHash() => _$accountsPageDataHash();

  @$internal
  @override
  $FutureProviderElement<AccountsPageData> $createElement(
          $ProviderPointer pointer) =>
      $FutureProviderElement(pointer);

  @override
  FutureOr<AccountsPageData> create(Ref ref) {
    return accountsPageData(ref);
  }
}

String _$accountsPageDataHash() => r'e37b6e048a3a3938c9c2b03ae41328036271956d';

@ProviderFor(basicAccountData)
const basicAccountDataProvider = BasicAccountDataProvider._();

final class BasicAccountDataProvider extends $FunctionalProvider<
        AsyncValue<BasicAccountData>,
        BasicAccountData,
        FutureOr<BasicAccountData>>
    with $FutureModifier<BasicAccountData>, $FutureProvider<BasicAccountData> {
  const BasicAccountDataProvider._()
      : super(
          from: null,
          argument: null,
          retry: null,
          name: r'basicAccountDataProvider',
          isAutoDispose: true,
          dependencies: null,
          $allTransitiveDependencies: null,
        );

  @override
  String debugGetCreateSourceHash() => _$basicAccountDataHash();

  @$internal
  @override
  $FutureProviderElement<BasicAccountData> $createElement(
          $ProviderPointer pointer) =>
      $FutureProviderElement(pointer);

  @override
  FutureOr<BasicAccountData> create(Ref ref) {
    return basicAccountData(ref);
  }
}

String _$basicAccountDataHash() => r'5f755167b7edd069b07888af935e53d49e425a16';

@ProviderFor(accountPageData)
const accountPageDataProvider = AccountPageDataProvider._();

final class AccountPageDataProvider extends $FunctionalProvider<
        AsyncValue<AccountPageData>, AccountPageData, FutureOr<AccountPageData>>
    with $FutureModifier<AccountPageData>, $FutureProvider<AccountPageData> {
  const AccountPageDataProvider._()
      : super(
          from: null,
          argument: null,
          retry: null,
          name: r'accountPageDataProvider',
          isAutoDispose: true,
          dependencies: null,
          $allTransitiveDependencies: null,
        );

  @override
  String debugGetCreateSourceHash() => _$accountPageDataHash();

  @$internal
  @override
  $FutureProviderElement<AccountPageData> $createElement(
          $ProviderPointer pointer) =>
      $FutureProviderElement(pointer);

  @override
  FutureOr<AccountPageData> create(Ref ref) {
    return accountPageData(ref);
  }
}

String _$accountPageDataHash() => r'1b27ca25c3ccb2705f247b0ca97780de0df30679';

@ProviderFor(fullAccountPageData)
const fullAccountPageDataProvider = FullAccountPageDataProvider._();

final class FullAccountPageDataProvider extends $FunctionalProvider<
        AsyncValue<FullAccountPageData>,
        FullAccountPageData,
        FutureOr<FullAccountPageData>>
    with
        $FutureModifier<FullAccountPageData>,
        $FutureProvider<FullAccountPageData> {
  const FullAccountPageDataProvider._()
      : super(
          from: null,
          argument: null,
          retry: null,
          name: r'fullAccountPageDataProvider',
          isAutoDispose: true,
          dependencies: null,
          $allTransitiveDependencies: null,
        );

  @override
  String debugGetCreateSourceHash() => _$fullAccountPageDataHash();

  @$internal
  @override
  $FutureProviderElement<FullAccountPageData> $createElement(
          $ProviderPointer pointer) =>
      $FutureProviderElement(pointer);

  @override
  FutureOr<FullAccountPageData> create(Ref ref) {
    return fullAccountPageData(ref);
  }
}

String _$fullAccountPageDataHash() =>
    r'742c766717c6b4f146d1f6fa7c6a5aa2512fa0b6';

@ProviderFor(VaultNotifier)
const vaultProvider = VaultNotifierProvider._();

final class VaultNotifierProvider
    extends $AsyncNotifierProvider<VaultNotifier, Vault> {
  const VaultNotifierProvider._()
      : super(
          from: null,
          argument: null,
          retry: null,
          name: r'vaultProvider',
          isAutoDispose: false,
          dependencies: null,
          $allTransitiveDependencies: null,
        );

  @override
  String debugGetCreateSourceHash() => _$vaultNotifierHash();

  @$internal
  @override
  VaultNotifier create() => VaultNotifier();
}

String _$vaultNotifierHash() => r'ad17084e8f4fbc34a37bf08a77b4b49ea50f2c0d';

abstract class _$VaultNotifier extends $AsyncNotifier<Vault> {
  FutureOr<Vault> build();
  @$mustCallSuper
  @override
  void runBuild() {
    final created = build();
    final ref = this.ref as $Ref<AsyncValue<Vault>, Vault>;
    final element = ref.element as $ClassProviderElement<
        AnyNotifier<AsyncValue<Vault>, Vault>,
        AsyncValue<Vault>,
        Object?,
        Object?>;
    element.handleValue(ref, created);
  }
}
