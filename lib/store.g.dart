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

/// The account the account page last opened, persisted so the next launch
/// returns to it. Pages receive their account through the route (see
/// `accountRoute`); this is only the launch default.

@ProviderFor(SelectedAccountId)
const selectedAccountIdProvider = SelectedAccountIdProvider._();

/// The account the account page last opened, persisted so the next launch
/// returns to it. Pages receive their account through the route (see
/// `accountRoute`); this is only the launch default.
final class SelectedAccountIdProvider
    extends $NotifierProvider<SelectedAccountId, int> {
  /// The account the account page last opened, persisted so the next launch
  /// returns to it. Pages receive their account through the route (see
  /// `accountRoute`); this is only the launch default.
  const SelectedAccountIdProvider._()
      : super(
          from: null,
          argument: null,
          retry: null,
          name: r'selectedAccountIdProvider',
          isAutoDispose: false,
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

String _$selectedAccountIdHash() => r'ef55f6cf79a2b42d1942ad84ced7902950ab20d8';

/// The account the account page last opened, persisted so the next launch
/// returns to it. Pages receive their account through the route (see
/// `accountRoute`); this is only the launch default.

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

@ProviderFor(getContacts)
const getContactsProvider = GetContactsProvider._();

final class GetContactsProvider extends $FunctionalProvider<
        AsyncValue<List<Contact>>, List<Contact>, FutureOr<List<Contact>>>
    with $FutureModifier<List<Contact>>, $FutureProvider<List<Contact>> {
  const GetContactsProvider._()
      : super(
          from: null,
          argument: null,
          retry: null,
          name: r'getContactsProvider',
          isAutoDispose: true,
          dependencies: null,
          $allTransitiveDependencies: null,
        );

  @override
  String debugGetCreateSourceHash() => _$getContactsHash();

  @$internal
  @override
  $FutureProviderElement<List<Contact>> $createElement(
          $ProviderPointer pointer) =>
      $FutureProviderElement(pointer);

  @override
  FutureOr<List<Contact>> create(Ref ref) {
    return getContacts(ref);
  }
}

String _$getContactsHash() => r'e751c15648be7db79565969c43b1be3a0ac566de';

@ProviderFor(contactsForAddress)
const contactsForAddressProvider = ContactsForAddressFamily._();

final class ContactsForAddressProvider extends $FunctionalProvider<
        AsyncValue<List<ContactMatch>>,
        List<ContactMatch>,
        FutureOr<List<ContactMatch>>>
    with
        $FutureModifier<List<ContactMatch>>,
        $FutureProvider<List<ContactMatch>> {
  const ContactsForAddressProvider._(
      {required ContactsForAddressFamily super.from,
      required String super.argument})
      : super(
          retry: null,
          name: r'contactsForAddressProvider',
          isAutoDispose: true,
          dependencies: null,
          $allTransitiveDependencies: null,
        );

  @override
  String debugGetCreateSourceHash() => _$contactsForAddressHash();

  @override
  String toString() {
    return r'contactsForAddressProvider'
        ''
        '($argument)';
  }

  @$internal
  @override
  $FutureProviderElement<List<ContactMatch>> $createElement(
          $ProviderPointer pointer) =>
      $FutureProviderElement(pointer);

  @override
  FutureOr<List<ContactMatch>> create(Ref ref) {
    final argument = this.argument as String;
    return contactsForAddress(
      ref,
      argument,
    );
  }

  @override
  bool operator ==(Object other) {
    return other is ContactsForAddressProvider && other.argument == argument;
  }

  @override
  int get hashCode {
    return argument.hashCode;
  }
}

String _$contactsForAddressHash() =>
    r'3154c6f4ddf9d4e141da6f70d5a086d79afdc348';

final class ContactsForAddressFamily extends $Family
    with $FunctionalFamilyOverride<FutureOr<List<ContactMatch>>, String> {
  const ContactsForAddressFamily._()
      : super(
          retry: null,
          name: r'contactsForAddressProvider',
          dependencies: null,
          $allTransitiveDependencies: null,
          isAutoDispose: true,
        );

  ContactsForAddressProvider call(
    String address,
  ) =>
      ContactsForAddressProvider._(argument: address, from: this);

  @override
  String toString() => r'contactsForAddressProvider';
}

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

String _$accountHash() => r'fdbd9cc7fd127d371560a116cb72cb4317a3e0d5';

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
    r'f91232871b263e892c96119a48a7f1ef01cd5b37';

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

String _$priceNotifierHash() => r'bac957ccbb2c04fc8dc8ef2028c1c8aa284d91c4';

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

@ProviderFor(SupportedCurrenciesNotifier)
const supportedCurrenciesProvider = SupportedCurrenciesNotifierProvider._();

final class SupportedCurrenciesNotifierProvider
    extends $AsyncNotifierProvider<SupportedCurrenciesNotifier, List<String>> {
  const SupportedCurrenciesNotifierProvider._()
      : super(
          from: null,
          argument: null,
          retry: null,
          name: r'supportedCurrenciesProvider',
          isAutoDispose: false,
          dependencies: null,
          $allTransitiveDependencies: null,
        );

  @override
  String debugGetCreateSourceHash() => _$supportedCurrenciesNotifierHash();

  @$internal
  @override
  SupportedCurrenciesNotifier create() => SupportedCurrenciesNotifier();
}

String _$supportedCurrenciesNotifierHash() =>
    r'45e340c75ec9222b285919bf0c006302020e3501';

abstract class _$SupportedCurrenciesNotifier
    extends $AsyncNotifier<List<String>> {
  FutureOr<List<String>> build();
  @$mustCallSuper
  @override
  void runBuild() {
    final created = build();
    final ref = this.ref as $Ref<AsyncValue<List<String>>, List<String>>;
    final element = ref.element as $ClassProviderElement<
        AnyNotifier<AsyncValue<List<String>>, List<String>>,
        AsyncValue<List<String>>,
        Object?,
        Object?>;
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

@ProviderFor(CurrentHeight)
const currentHeightProvider = CurrentHeightProvider._();

final class CurrentHeightProvider
    extends $AsyncNotifierProvider<CurrentHeight, int?> {
  const CurrentHeightProvider._()
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
  String debugGetCreateSourceHash() => _$currentHeightHash();

  @$internal
  @override
  CurrentHeight create() => CurrentHeight();
}

String _$currentHeightHash() => r'0710021baa0ed9a9ad29fc8d49cefd60f3712134';

abstract class _$CurrentHeight extends $AsyncNotifier<int?> {
  FutureOr<int?> build();
  @$mustCallSuper
  @override
  void runBuild() {
    final created = build();
    final ref = this.ref as $Ref<AsyncValue<int?>, int?>;
    final element = ref.element as $ClassProviderElement<
        AnyNotifier<AsyncValue<int?>, int?>,
        AsyncValue<int?>,
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
    r'a7b0c7dda4b292793e86b3eece48644c43ea3aa1';

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

String _$transparentScanHash() => r'44d8fc7c776f599e3d67dbc7ff27ce4b9c30c810';

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
      {required GetTxDetailsFamily super.from,
      required (
        int,
        int,
      )
          super.argument})
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
        '$argument';
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

String _$getTxDetailsHash() => r'6edd60a3c0893b12c3e584dd7fa8b539c7a68e00';

final class GetTxDetailsFamily extends $Family
    with
        $ClassFamilyOverride<
            GetTxDetails,
            AsyncValue<TxAccount>,
            TxAccount,
            FutureOr<TxAccount>,
            (
              int,
              int,
            )> {
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
    int account,
  ) =>
      GetTxDetailsProvider._(argument: (
        id,
        account,
      ), from: this);

  @override
  String toString() => r'getTxDetailsProvider';
}

abstract class _$GetTxDetails extends $AsyncNotifier<TxAccount> {
  late final _$args = ref.$arg as (
    int,
    int,
  );
  int get id => _$args.$1;
  int get account => _$args.$2;

  FutureOr<TxAccount> build(
    int id,
    int account,
  );
  @$mustCallSuper
  @override
  void runBuild() {
    final created = build(
      _$args.$1,
      _$args.$2,
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
const basicAccountDataProvider = BasicAccountDataFamily._();

final class BasicAccountDataProvider extends $FunctionalProvider<
        AsyncValue<BasicAccountData>,
        BasicAccountData,
        FutureOr<BasicAccountData>>
    with $FutureModifier<BasicAccountData>, $FutureProvider<BasicAccountData> {
  const BasicAccountDataProvider._(
      {required BasicAccountDataFamily super.from, required int super.argument})
      : super(
          retry: null,
          name: r'basicAccountDataProvider',
          isAutoDispose: true,
          dependencies: null,
          $allTransitiveDependencies: null,
        );

  @override
  String debugGetCreateSourceHash() => _$basicAccountDataHash();

  @override
  String toString() {
    return r'basicAccountDataProvider'
        ''
        '($argument)';
  }

  @$internal
  @override
  $FutureProviderElement<BasicAccountData> $createElement(
          $ProviderPointer pointer) =>
      $FutureProviderElement(pointer);

  @override
  FutureOr<BasicAccountData> create(Ref ref) {
    final argument = this.argument as int;
    return basicAccountData(
      ref,
      argument,
    );
  }

  @override
  bool operator ==(Object other) {
    return other is BasicAccountDataProvider && other.argument == argument;
  }

  @override
  int get hashCode {
    return argument.hashCode;
  }
}

String _$basicAccountDataHash() => r'3cc4b47523156d8ff7ea917ceb8eda9424e31f3f';

final class BasicAccountDataFamily extends $Family
    with $FunctionalFamilyOverride<FutureOr<BasicAccountData>, int> {
  const BasicAccountDataFamily._()
      : super(
          retry: null,
          name: r'basicAccountDataProvider',
          dependencies: null,
          $allTransitiveDependencies: null,
          isAutoDispose: true,
        );

  BasicAccountDataProvider call(
    int account,
  ) =>
      BasicAccountDataProvider._(argument: account, from: this);

  @override
  String toString() => r'basicAccountDataProvider';
}

@ProviderFor(accountPageData)
const accountPageDataProvider = AccountPageDataFamily._();

final class AccountPageDataProvider extends $FunctionalProvider<
        AsyncValue<AccountPageData>, AccountPageData, FutureOr<AccountPageData>>
    with $FutureModifier<AccountPageData>, $FutureProvider<AccountPageData> {
  const AccountPageDataProvider._(
      {required AccountPageDataFamily super.from, required int super.argument})
      : super(
          retry: null,
          name: r'accountPageDataProvider',
          isAutoDispose: true,
          dependencies: null,
          $allTransitiveDependencies: null,
        );

  @override
  String debugGetCreateSourceHash() => _$accountPageDataHash();

  @override
  String toString() {
    return r'accountPageDataProvider'
        ''
        '($argument)';
  }

  @$internal
  @override
  $FutureProviderElement<AccountPageData> $createElement(
          $ProviderPointer pointer) =>
      $FutureProviderElement(pointer);

  @override
  FutureOr<AccountPageData> create(Ref ref) {
    final argument = this.argument as int;
    return accountPageData(
      ref,
      argument,
    );
  }

  @override
  bool operator ==(Object other) {
    return other is AccountPageDataProvider && other.argument == argument;
  }

  @override
  int get hashCode {
    return argument.hashCode;
  }
}

String _$accountPageDataHash() => r'ac586da8719bf779a4c6531029a41097c8ae99af';

final class AccountPageDataFamily extends $Family
    with $FunctionalFamilyOverride<FutureOr<AccountPageData>, int> {
  const AccountPageDataFamily._()
      : super(
          retry: null,
          name: r'accountPageDataProvider',
          dependencies: null,
          $allTransitiveDependencies: null,
          isAutoDispose: true,
        );

  AccountPageDataProvider call(
    int account,
  ) =>
      AccountPageDataProvider._(argument: account, from: this);

  @override
  String toString() => r'accountPageDataProvider';
}

@ProviderFor(fullAccountPageData)
const fullAccountPageDataProvider = FullAccountPageDataFamily._();

final class FullAccountPageDataProvider extends $FunctionalProvider<
        AsyncValue<FullAccountPageData>,
        FullAccountPageData,
        FutureOr<FullAccountPageData>>
    with
        $FutureModifier<FullAccountPageData>,
        $FutureProvider<FullAccountPageData> {
  const FullAccountPageDataProvider._(
      {required FullAccountPageDataFamily super.from,
      required int super.argument})
      : super(
          retry: null,
          name: r'fullAccountPageDataProvider',
          isAutoDispose: true,
          dependencies: null,
          $allTransitiveDependencies: null,
        );

  @override
  String debugGetCreateSourceHash() => _$fullAccountPageDataHash();

  @override
  String toString() {
    return r'fullAccountPageDataProvider'
        ''
        '($argument)';
  }

  @$internal
  @override
  $FutureProviderElement<FullAccountPageData> $createElement(
          $ProviderPointer pointer) =>
      $FutureProviderElement(pointer);

  @override
  FutureOr<FullAccountPageData> create(Ref ref) {
    final argument = this.argument as int;
    return fullAccountPageData(
      ref,
      argument,
    );
  }

  @override
  bool operator ==(Object other) {
    return other is FullAccountPageDataProvider && other.argument == argument;
  }

  @override
  int get hashCode {
    return argument.hashCode;
  }
}

String _$fullAccountPageDataHash() =>
    r'2bd1df3ea9210ab5767ea2cec12ec9ee9d1025be';

final class FullAccountPageDataFamily extends $Family
    with $FunctionalFamilyOverride<FutureOr<FullAccountPageData>, int> {
  const FullAccountPageDataFamily._()
      : super(
          retry: null,
          name: r'fullAccountPageDataProvider',
          dependencies: null,
          $allTransitiveDependencies: null,
          isAutoDispose: true,
        );

  FullAccountPageDataProvider call(
    int account,
  ) =>
      FullAccountPageDataProvider._(argument: account, from: this);

  @override
  String toString() => r'fullAccountPageDataProvider';
}

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

String _$vaultNotifierHash() => r'b2840c7cbc6b39a9eaea18ab703eaddc59d00259';

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

@ProviderFor(pluginList)
const pluginListProvider = PluginListProvider._();

final class PluginListProvider extends $FunctionalProvider<
        AsyncValue<List<plugin_api.PluginInfo>>,
        List<plugin_api.PluginInfo>,
        FutureOr<List<plugin_api.PluginInfo>>>
    with
        $FutureModifier<List<plugin_api.PluginInfo>>,
        $FutureProvider<List<plugin_api.PluginInfo>> {
  const PluginListProvider._()
      : super(
          from: null,
          argument: null,
          retry: null,
          name: r'pluginListProvider',
          isAutoDispose: true,
          dependencies: null,
          $allTransitiveDependencies: null,
        );

  @override
  String debugGetCreateSourceHash() => _$pluginListHash();

  @$internal
  @override
  $FutureProviderElement<List<plugin_api.PluginInfo>> $createElement(
          $ProviderPointer pointer) =>
      $FutureProviderElement(pointer);

  @override
  FutureOr<List<plugin_api.PluginInfo>> create(Ref ref) {
    return pluginList(ref);
  }
}

String _$pluginListHash() => r'f396f236f61820f586848210e153f83881ad09c1';

@ProviderFor(pluginMemoSections)
const pluginMemoSectionsProvider = PluginMemoSectionsFamily._();

final class PluginMemoSectionsProvider extends $FunctionalProvider<
        AsyncValue<List<plugin_api.MemoSection>>,
        List<plugin_api.MemoSection>,
        FutureOr<List<plugin_api.MemoSection>>>
    with
        $FutureModifier<List<plugin_api.MemoSection>>,
        $FutureProvider<List<plugin_api.MemoSection>> {
  const PluginMemoSectionsProvider._(
      {required PluginMemoSectionsFamily super.from,
      required (
        List<int>,
        Coin,
      )
          super.argument})
      : super(
          retry: null,
          name: r'pluginMemoSectionsProvider',
          isAutoDispose: true,
          dependencies: null,
          $allTransitiveDependencies: null,
        );

  @override
  String debugGetCreateSourceHash() => _$pluginMemoSectionsHash();

  @override
  String toString() {
    return r'pluginMemoSectionsProvider'
        ''
        '$argument';
  }

  @$internal
  @override
  $FutureProviderElement<List<plugin_api.MemoSection>> $createElement(
          $ProviderPointer pointer) =>
      $FutureProviderElement(pointer);

  @override
  FutureOr<List<plugin_api.MemoSection>> create(Ref ref) {
    final argument = this.argument as (
      List<int>,
      Coin,
    );
    return pluginMemoSections(
      ref,
      argument.$1,
      argument.$2,
    );
  }

  @override
  bool operator ==(Object other) {
    return other is PluginMemoSectionsProvider && other.argument == argument;
  }

  @override
  int get hashCode {
    return argument.hashCode;
  }
}

String _$pluginMemoSectionsHash() =>
    r'ecd47c3bc96fdf29a00b04a6d8b8f97742208824';

final class PluginMemoSectionsFamily extends $Family
    with
        $FunctionalFamilyOverride<
            FutureOr<List<plugin_api.MemoSection>>,
            (
              List<int>,
              Coin,
            )> {
  const PluginMemoSectionsFamily._()
      : super(
          retry: null,
          name: r'pluginMemoSectionsProvider',
          dependencies: null,
          $allTransitiveDependencies: null,
          isAutoDispose: true,
        );

  PluginMemoSectionsProvider call(
    List<int> memoBytes,
    Coin c,
  ) =>
      PluginMemoSectionsProvider._(argument: (
        memoBytes,
        c,
      ), from: this);

  @override
  String toString() => r'pluginMemoSectionsProvider';
}

/// Blocks destructive wallet actions (account deletion, vault sign-out,
/// wallet removal) while a voting submission run is in flight.

@ProviderFor(VotingSubmissionGuard)
const votingSubmissionGuardProvider = VotingSubmissionGuardProvider._();

/// Blocks destructive wallet actions (account deletion, vault sign-out,
/// wallet removal) while a voting submission run is in flight.
final class VotingSubmissionGuardProvider
    extends $NotifierProvider<VotingSubmissionGuard, bool> {
  /// Blocks destructive wallet actions (account deletion, vault sign-out,
  /// wallet removal) while a voting submission run is in flight.
  const VotingSubmissionGuardProvider._()
      : super(
          from: null,
          argument: null,
          retry: null,
          name: r'votingSubmissionGuardProvider',
          isAutoDispose: false,
          dependencies: null,
          $allTransitiveDependencies: null,
        );

  @override
  String debugGetCreateSourceHash() => _$votingSubmissionGuardHash();

  @$internal
  @override
  VotingSubmissionGuard create() => VotingSubmissionGuard();

  /// {@macro riverpod.override_with_value}
  Override overrideWithValue(bool value) {
    return $ProviderOverride(
      origin: this,
      providerOverride: $SyncValueProvider<bool>(value),
    );
  }
}

String _$votingSubmissionGuardHash() =>
    r'7fbcb88ac8361aa94083e9d98321f96c521a2777';

/// Blocks destructive wallet actions (account deletion, vault sign-out,
/// wallet removal) while a voting submission run is in flight.

abstract class _$VotingSubmissionGuard extends $Notifier<bool> {
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

/// Submission job for one round. `prepare` persists the bundle layout and
/// returns the eligibility preview the UI shows before the voter commits;
/// `start` hands the round to the Rust driver, which runs to quiescence on
/// its own task. Status polling is owned here, so it survives page
/// navigation; a run already live in Rust is adopted on first read.

@ProviderFor(VotingDriveJob)
const votingDriveJobProvider = VotingDriveJobFamily._();

/// Submission job for one round. `prepare` persists the bundle layout and
/// returns the eligibility preview the UI shows before the voter commits;
/// `start` hands the round to the Rust driver, which runs to quiescence on
/// its own task. Status polling is owned here, so it survives page
/// navigation; a run already live in Rust is adopted on first read.
final class VotingDriveJobProvider
    extends $NotifierProvider<VotingDriveJob, VotingDriveJobState> {
  /// Submission job for one round. `prepare` persists the bundle layout and
  /// returns the eligibility preview the UI shows before the voter commits;
  /// `start` hands the round to the Rust driver, which runs to quiescence on
  /// its own task. Status polling is owned here, so it survives page
  /// navigation; a run already live in Rust is adopted on first read.
  const VotingDriveJobProvider._(
      {required VotingDriveJobFamily super.from,
      required (
        String,
        int,
      )
          super.argument})
      : super(
          retry: null,
          name: r'votingDriveJobProvider',
          isAutoDispose: false,
          dependencies: null,
          $allTransitiveDependencies: null,
        );

  @override
  String debugGetCreateSourceHash() => _$votingDriveJobHash();

  @override
  String toString() {
    return r'votingDriveJobProvider'
        ''
        '$argument';
  }

  @$internal
  @override
  VotingDriveJob create() => VotingDriveJob();

  /// {@macro riverpod.override_with_value}
  Override overrideWithValue(VotingDriveJobState value) {
    return $ProviderOverride(
      origin: this,
      providerOverride: $SyncValueProvider<VotingDriveJobState>(value),
    );
  }

  @override
  bool operator ==(Object other) {
    return other is VotingDriveJobProvider && other.argument == argument;
  }

  @override
  int get hashCode {
    return argument.hashCode;
  }
}

String _$votingDriveJobHash() => r'dcd077904c83d81eb8351ee05c9a7e01c49e24ca';

/// Submission job for one round. `prepare` persists the bundle layout and
/// returns the eligibility preview the UI shows before the voter commits;
/// `start` hands the round to the Rust driver, which runs to quiescence on
/// its own task. Status polling is owned here, so it survives page
/// navigation; a run already live in Rust is adopted on first read.

final class VotingDriveJobFamily extends $Family
    with
        $ClassFamilyOverride<
            VotingDriveJob,
            VotingDriveJobState,
            VotingDriveJobState,
            VotingDriveJobState,
            (
              String,
              int,
            )> {
  const VotingDriveJobFamily._()
      : super(
          retry: null,
          name: r'votingDriveJobProvider',
          dependencies: null,
          $allTransitiveDependencies: null,
          isAutoDispose: false,
        );

  /// Submission job for one round. `prepare` persists the bundle layout and
  /// returns the eligibility preview the UI shows before the voter commits;
  /// `start` hands the round to the Rust driver, which runs to quiescence on
  /// its own task. Status polling is owned here, so it survives page
  /// navigation; a run already live in Rust is adopted on first read.

  VotingDriveJobProvider call(
    String roundId,
    int account,
  ) =>
      VotingDriveJobProvider._(argument: (
        roundId,
        account,
      ), from: this);

  @override
  String toString() => r'votingDriveJobProvider';
}

/// Submission job for one round. `prepare` persists the bundle layout and
/// returns the eligibility preview the UI shows before the voter commits;
/// `start` hands the round to the Rust driver, which runs to quiescence on
/// its own task. Status polling is owned here, so it survives page
/// navigation; a run already live in Rust is adopted on first read.

abstract class _$VotingDriveJob extends $Notifier<VotingDriveJobState> {
  late final _$args = ref.$arg as (
    String,
    int,
  );
  String get roundId => _$args.$1;
  int get account => _$args.$2;

  VotingDriveJobState build(
    String roundId,
    int account,
  );
  @$mustCallSuper
  @override
  void runBuild() {
    final created = build(
      _$args.$1,
      _$args.$2,
    );
    final ref = this.ref as $Ref<VotingDriveJobState, VotingDriveJobState>;
    final element = ref.element as $ClassProviderElement<
        AnyNotifier<VotingDriveJobState, VotingDriveJobState>,
        VotingDriveJobState,
        Object?,
        Object?>;
    element.handleValue(ref, created);
  }
}

/// Re-runnable trigger to stop every live driver run for the current wallet.
/// Backgrounding and wallet switches invoke it so no run outlives its wallet;
/// durable effects stay and a later start re-plans from the sidecar.

@ProviderFor(VotingDriveStopAll)
const votingDriveStopAllProvider = VotingDriveStopAllProvider._();

/// Re-runnable trigger to stop every live driver run for the current wallet.
/// Backgrounding and wallet switches invoke it so no run outlives its wallet;
/// durable effects stay and a later start re-plans from the sidecar.
final class VotingDriveStopAllProvider
    extends $NotifierProvider<VotingDriveStopAll, void> {
  /// Re-runnable trigger to stop every live driver run for the current wallet.
  /// Backgrounding and wallet switches invoke it so no run outlives its wallet;
  /// durable effects stay and a later start re-plans from the sidecar.
  const VotingDriveStopAllProvider._()
      : super(
          from: null,
          argument: null,
          retry: null,
          name: r'votingDriveStopAllProvider',
          isAutoDispose: false,
          dependencies: null,
          $allTransitiveDependencies: null,
        );

  @override
  String debugGetCreateSourceHash() => _$votingDriveStopAllHash();

  @$internal
  @override
  VotingDriveStopAll create() => VotingDriveStopAll();

  /// {@macro riverpod.override_with_value}
  Override overrideWithValue(void value) {
    return $ProviderOverride(
      origin: this,
      providerOverride: $SyncValueProvider<void>(value),
    );
  }
}

String _$votingDriveStopAllHash() =>
    r'1a6879dc0f9cd20551afa0ab28b54df794430f5d';

/// Re-runnable trigger to stop every live driver run for the current wallet.
/// Backgrounding and wallet switches invoke it so no run outlives its wallet;
/// durable effects stay and a later start re-plans from the sidecar.

abstract class _$VotingDriveStopAll extends $Notifier<void> {
  void build();
  @$mustCallSuper
  @override
  void runBuild() {
    build();
    final ref = this.ref as $Ref<void, void>;
    final element = ref.element as $ClassProviderElement<
        AnyNotifier<void, void>, void, Object?, Object?>;
    element.handleValue(ref, null);
  }
}
