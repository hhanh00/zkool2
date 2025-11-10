// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'router.dart';

// **************************************************************************
// RiverpodGenerator
// **************************************************************************

// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint, type=warning

@ProviderFor(PinLocked)
const pinLockedProvider = PinLockedProvider._();

final class PinLockedProvider extends $NotifierProvider<PinLocked, bool> {
  const PinLockedProvider._()
      : super(
          from: null,
          argument: null,
          retry: null,
          name: r'pinLockedProvider',
          isAutoDispose: true,
          dependencies: null,
          $allTransitiveDependencies: null,
        );

  @override
  String debugGetCreateSourceHash() => _$pinLockedHash();

  @$internal
  @override
  PinLocked create() => PinLocked();

  /// {@macro riverpod.override_with_value}
  Override overrideWithValue(bool value) {
    return $ProviderOverride(
      origin: this,
      providerOverride: $SyncValueProvider<bool>(value),
    );
  }
}

String _$pinLockedHash() => r'b1f500b7b64de7dbad44b4d2dbe047afe6894e81';

abstract class _$PinLocked extends $Notifier<bool> {
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
