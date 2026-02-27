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

@ProviderFor(CoinContext)
const coinContextProvider = CoinContextProvider._();

final class CoinContextProvider extends $NotifierProvider<CoinContext, Coin> {
  const CoinContextProvider._()
      : super(
          from: null,
          argument: null,
          retry: null,
          name: r'coinContextProvider',
          isAutoDispose: true,
          dependencies: null,
          $allTransitiveDependencies: null,
        );

  @override
  String debugGetCreateSourceHash() => _$coinContextHash();

  @$internal
  @override
  CoinContext create() => CoinContext();

  /// {@macro riverpod.override_with_value}
  Override overrideWithValue(Coin value) {
    return $ProviderOverride(
      origin: this,
      providerOverride: $SyncValueProvider<Coin>(value),
    );
  }
}

String _$coinContextHash() => r'91273110574fa3fe4e8412e92c99d1cb4fff0997';

abstract class _$CoinContext extends $Notifier<Coin> {
  Coin build();
  @$mustCallSuper
  @override
  void runBuild() {
    final created = build();
    final ref = this.ref as $Ref<Coin, Coin>;
    final element = ref.element as $ClassProviderElement<
        AnyNotifier<Coin, Coin>, Coin, Object?, Object?>;
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

String _$selectedAccountHash() => r'730f32748a616d776d3781bb7c627e095f779c4c';

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

String _$getAccountsHash() => r'0b15066a7f8a3dfccb92626b55f4de7c5fac850b';

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

String _$getFoldersHash() => r'5a55b95e2e52415b9ee7208ddca359f635e36acf';

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

String _$getCategoriesHash() => r'ff720b53a6f29d3cc907abf278f14e2fc7ffc7e5';

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

String _$accountHash() => r'9594c3daf455e8f1905a6fbf6d26727e69ed579c';

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
          isAutoDispose: true,
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

String _$getCurrentAccountHash() => r'd96b535adcd641dbfc5444f924a82c621d592e56';

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
    r'ef07020c62113b0017338ac37daf414162ce2325';

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

String _$mempoolNotifierHash() => r'1ee6eb5ad07e3b638f9b1e731aaabfa622dfc2d4';

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
    r'56bfd5d266a17e0da9fbc0be7afa3559f634e7e1';

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

String _$transparentScanHash() => r'550050f25e966a0eec3b5c1bea6955df5b76a962';

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

String _$getTxDetailsHash() => r'4f15b77e50e66ecfb895bc53a2e5c5cf017689e0';

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
