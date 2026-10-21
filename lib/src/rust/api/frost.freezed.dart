// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'frost.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;

/// @nodoc
mixin _$DKGStatus {
  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType && other is DKGStatus);
  }

  @override
  int get hashCode => runtimeType.hashCode;

  @override
  String toString() {
    return 'DKGStatus()';
  }
}

/// @nodoc
class $DKGStatusCopyWith<$Res> {
  $DKGStatusCopyWith(DKGStatus _, $Res Function(DKGStatus) __);
}

/// Adds pattern-matching-related methods to [DKGStatus].
extension DKGStatusPatterns on DKGStatus {
  /// A variant of `map` that fallback to returning `orElse`.
  ///
  /// It is equivalent to doing:
  /// ```dart
  /// switch (sealedClass) {
  ///   case final Subclass value:
  ///     return ...;
  ///   case _:
  ///     return orElse();
  /// }
  /// ```

  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(DKGStatus_WaitParams value)? waitParams,
    TResult Function(DKGStatus_WaitAddresses value)? waitAddresses,
    TResult Function(DKGStatus_PublishRound0Pkg value)? publishRound0Pkg,
    TResult Function(DKGStatus_WaitRound0Pkg value)? waitRound0Pkg,
    TResult Function(DKGStatus_PublishRound1Pkg value)? publishRound1Pkg,
    TResult Function(DKGStatus_WaitRound1Pkg value)? waitRound1Pkg,
    TResult Function(DKGStatus_PublishRound2Pkg value)? publishRound2Pkg,
    TResult Function(DKGStatus_WaitRound2Pkg value)? waitRound2Pkg,
    TResult Function(DKGStatus_WaitingForFunds value)? waitingForFunds,
    TResult Function(DKGStatus_WaitOwnPublish value)? waitOwnPublish,
    TResult Function(DKGStatus_Finalize value)? finalize,
    TResult Function(DKGStatus_SharedAddress value)? sharedAddress,
    TResult Function(DKGStatus_VersionMismatch value)? versionMismatch,
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case DKGStatus_WaitParams() when waitParams != null:
        return waitParams(_that);
      case DKGStatus_WaitAddresses() when waitAddresses != null:
        return waitAddresses(_that);
      case DKGStatus_PublishRound0Pkg() when publishRound0Pkg != null:
        return publishRound0Pkg(_that);
      case DKGStatus_WaitRound0Pkg() when waitRound0Pkg != null:
        return waitRound0Pkg(_that);
      case DKGStatus_PublishRound1Pkg() when publishRound1Pkg != null:
        return publishRound1Pkg(_that);
      case DKGStatus_WaitRound1Pkg() when waitRound1Pkg != null:
        return waitRound1Pkg(_that);
      case DKGStatus_PublishRound2Pkg() when publishRound2Pkg != null:
        return publishRound2Pkg(_that);
      case DKGStatus_WaitRound2Pkg() when waitRound2Pkg != null:
        return waitRound2Pkg(_that);
      case DKGStatus_WaitingForFunds() when waitingForFunds != null:
        return waitingForFunds(_that);
      case DKGStatus_WaitOwnPublish() when waitOwnPublish != null:
        return waitOwnPublish(_that);
      case DKGStatus_Finalize() when finalize != null:
        return finalize(_that);
      case DKGStatus_SharedAddress() when sharedAddress != null:
        return sharedAddress(_that);
      case DKGStatus_VersionMismatch() when versionMismatch != null:
        return versionMismatch(_that);
      case _:
        return orElse();
    }
  }

  /// A `switch`-like method, using callbacks.
  ///
  /// Callbacks receives the raw object, upcasted.
  /// It is equivalent to doing:
  /// ```dart
  /// switch (sealedClass) {
  ///   case final Subclass value:
  ///     return ...;
  ///   case final Subclass2 value:
  ///     return ...;
  /// }
  /// ```

  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(DKGStatus_WaitParams value) waitParams,
    required TResult Function(DKGStatus_WaitAddresses value) waitAddresses,
    required TResult Function(DKGStatus_PublishRound0Pkg value)
        publishRound0Pkg,
    required TResult Function(DKGStatus_WaitRound0Pkg value) waitRound0Pkg,
    required TResult Function(DKGStatus_PublishRound1Pkg value)
        publishRound1Pkg,
    required TResult Function(DKGStatus_WaitRound1Pkg value) waitRound1Pkg,
    required TResult Function(DKGStatus_PublishRound2Pkg value)
        publishRound2Pkg,
    required TResult Function(DKGStatus_WaitRound2Pkg value) waitRound2Pkg,
    required TResult Function(DKGStatus_WaitingForFunds value) waitingForFunds,
    required TResult Function(DKGStatus_WaitOwnPublish value) waitOwnPublish,
    required TResult Function(DKGStatus_Finalize value) finalize,
    required TResult Function(DKGStatus_SharedAddress value) sharedAddress,
    required TResult Function(DKGStatus_VersionMismatch value) versionMismatch,
  }) {
    final _that = this;
    switch (_that) {
      case DKGStatus_WaitParams():
        return waitParams(_that);
      case DKGStatus_WaitAddresses():
        return waitAddresses(_that);
      case DKGStatus_PublishRound0Pkg():
        return publishRound0Pkg(_that);
      case DKGStatus_WaitRound0Pkg():
        return waitRound0Pkg(_that);
      case DKGStatus_PublishRound1Pkg():
        return publishRound1Pkg(_that);
      case DKGStatus_WaitRound1Pkg():
        return waitRound1Pkg(_that);
      case DKGStatus_PublishRound2Pkg():
        return publishRound2Pkg(_that);
      case DKGStatus_WaitRound2Pkg():
        return waitRound2Pkg(_that);
      case DKGStatus_WaitingForFunds():
        return waitingForFunds(_that);
      case DKGStatus_WaitOwnPublish():
        return waitOwnPublish(_that);
      case DKGStatus_Finalize():
        return finalize(_that);
      case DKGStatus_SharedAddress():
        return sharedAddress(_that);
      case DKGStatus_VersionMismatch():
        return versionMismatch(_that);
    }
  }

  /// A variant of `map` that fallback to returning `null`.
  ///
  /// It is equivalent to doing:
  /// ```dart
  /// switch (sealedClass) {
  ///   case final Subclass value:
  ///     return ...;
  ///   case _:
  ///     return null;
  /// }
  /// ```

  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(DKGStatus_WaitParams value)? waitParams,
    TResult? Function(DKGStatus_WaitAddresses value)? waitAddresses,
    TResult? Function(DKGStatus_PublishRound0Pkg value)? publishRound0Pkg,
    TResult? Function(DKGStatus_WaitRound0Pkg value)? waitRound0Pkg,
    TResult? Function(DKGStatus_PublishRound1Pkg value)? publishRound1Pkg,
    TResult? Function(DKGStatus_WaitRound1Pkg value)? waitRound1Pkg,
    TResult? Function(DKGStatus_PublishRound2Pkg value)? publishRound2Pkg,
    TResult? Function(DKGStatus_WaitRound2Pkg value)? waitRound2Pkg,
    TResult? Function(DKGStatus_WaitingForFunds value)? waitingForFunds,
    TResult? Function(DKGStatus_WaitOwnPublish value)? waitOwnPublish,
    TResult? Function(DKGStatus_Finalize value)? finalize,
    TResult? Function(DKGStatus_SharedAddress value)? sharedAddress,
    TResult? Function(DKGStatus_VersionMismatch value)? versionMismatch,
  }) {
    final _that = this;
    switch (_that) {
      case DKGStatus_WaitParams() when waitParams != null:
        return waitParams(_that);
      case DKGStatus_WaitAddresses() when waitAddresses != null:
        return waitAddresses(_that);
      case DKGStatus_PublishRound0Pkg() when publishRound0Pkg != null:
        return publishRound0Pkg(_that);
      case DKGStatus_WaitRound0Pkg() when waitRound0Pkg != null:
        return waitRound0Pkg(_that);
      case DKGStatus_PublishRound1Pkg() when publishRound1Pkg != null:
        return publishRound1Pkg(_that);
      case DKGStatus_WaitRound1Pkg() when waitRound1Pkg != null:
        return waitRound1Pkg(_that);
      case DKGStatus_PublishRound2Pkg() when publishRound2Pkg != null:
        return publishRound2Pkg(_that);
      case DKGStatus_WaitRound2Pkg() when waitRound2Pkg != null:
        return waitRound2Pkg(_that);
      case DKGStatus_WaitingForFunds() when waitingForFunds != null:
        return waitingForFunds(_that);
      case DKGStatus_WaitOwnPublish() when waitOwnPublish != null:
        return waitOwnPublish(_that);
      case DKGStatus_Finalize() when finalize != null:
        return finalize(_that);
      case DKGStatus_SharedAddress() when sharedAddress != null:
        return sharedAddress(_that);
      case DKGStatus_VersionMismatch() when versionMismatch != null:
        return versionMismatch(_that);
      case _:
        return null;
    }
  }

  /// A variant of `when` that fallback to an `orElse` callback.
  ///
  /// It is equivalent to doing:
  /// ```dart
  /// switch (sealedClass) {
  ///   case Subclass(:final field):
  ///     return ...;
  ///   case _:
  ///     return orElse();
  /// }
  /// ```

  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function()? waitParams,
    TResult Function(List<String> field0)? waitAddresses,
    TResult Function()? publishRound0Pkg,
    TResult Function()? waitRound0Pkg,
    TResult Function()? publishRound1Pkg,
    TResult Function()? waitRound1Pkg,
    TResult Function()? publishRound2Pkg,
    TResult Function()? waitRound2Pkg,
    TResult Function()? waitingForFunds,
    TResult Function()? waitOwnPublish,
    TResult Function()? finalize,
    TResult Function(String field0)? sharedAddress,
    TResult Function(int peer, int ours, int theirs)? versionMismatch,
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case DKGStatus_WaitParams() when waitParams != null:
        return waitParams();
      case DKGStatus_WaitAddresses() when waitAddresses != null:
        return waitAddresses(_that.field0);
      case DKGStatus_PublishRound0Pkg() when publishRound0Pkg != null:
        return publishRound0Pkg();
      case DKGStatus_WaitRound0Pkg() when waitRound0Pkg != null:
        return waitRound0Pkg();
      case DKGStatus_PublishRound1Pkg() when publishRound1Pkg != null:
        return publishRound1Pkg();
      case DKGStatus_WaitRound1Pkg() when waitRound1Pkg != null:
        return waitRound1Pkg();
      case DKGStatus_PublishRound2Pkg() when publishRound2Pkg != null:
        return publishRound2Pkg();
      case DKGStatus_WaitRound2Pkg() when waitRound2Pkg != null:
        return waitRound2Pkg();
      case DKGStatus_WaitingForFunds() when waitingForFunds != null:
        return waitingForFunds();
      case DKGStatus_WaitOwnPublish() when waitOwnPublish != null:
        return waitOwnPublish();
      case DKGStatus_Finalize() when finalize != null:
        return finalize();
      case DKGStatus_SharedAddress() when sharedAddress != null:
        return sharedAddress(_that.field0);
      case DKGStatus_VersionMismatch() when versionMismatch != null:
        return versionMismatch(_that.peer, _that.ours, _that.theirs);
      case _:
        return orElse();
    }
  }

  /// A `switch`-like method, using callbacks.
  ///
  /// As opposed to `map`, this offers destructuring.
  /// It is equivalent to doing:
  /// ```dart
  /// switch (sealedClass) {
  ///   case Subclass(:final field):
  ///     return ...;
  ///   case Subclass2(:final field2):
  ///     return ...;
  /// }
  /// ```

  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function() waitParams,
    required TResult Function(List<String> field0) waitAddresses,
    required TResult Function() publishRound0Pkg,
    required TResult Function() waitRound0Pkg,
    required TResult Function() publishRound1Pkg,
    required TResult Function() waitRound1Pkg,
    required TResult Function() publishRound2Pkg,
    required TResult Function() waitRound2Pkg,
    required TResult Function() waitingForFunds,
    required TResult Function() waitOwnPublish,
    required TResult Function() finalize,
    required TResult Function(String field0) sharedAddress,
    required TResult Function(int peer, int ours, int theirs) versionMismatch,
  }) {
    final _that = this;
    switch (_that) {
      case DKGStatus_WaitParams():
        return waitParams();
      case DKGStatus_WaitAddresses():
        return waitAddresses(_that.field0);
      case DKGStatus_PublishRound0Pkg():
        return publishRound0Pkg();
      case DKGStatus_WaitRound0Pkg():
        return waitRound0Pkg();
      case DKGStatus_PublishRound1Pkg():
        return publishRound1Pkg();
      case DKGStatus_WaitRound1Pkg():
        return waitRound1Pkg();
      case DKGStatus_PublishRound2Pkg():
        return publishRound2Pkg();
      case DKGStatus_WaitRound2Pkg():
        return waitRound2Pkg();
      case DKGStatus_WaitingForFunds():
        return waitingForFunds();
      case DKGStatus_WaitOwnPublish():
        return waitOwnPublish();
      case DKGStatus_Finalize():
        return finalize();
      case DKGStatus_SharedAddress():
        return sharedAddress(_that.field0);
      case DKGStatus_VersionMismatch():
        return versionMismatch(_that.peer, _that.ours, _that.theirs);
    }
  }

  /// A variant of `when` that fallback to returning `null`
  ///
  /// It is equivalent to doing:
  /// ```dart
  /// switch (sealedClass) {
  ///   case Subclass(:final field):
  ///     return ...;
  ///   case _:
  ///     return null;
  /// }
  /// ```

  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function()? waitParams,
    TResult? Function(List<String> field0)? waitAddresses,
    TResult? Function()? publishRound0Pkg,
    TResult? Function()? waitRound0Pkg,
    TResult? Function()? publishRound1Pkg,
    TResult? Function()? waitRound1Pkg,
    TResult? Function()? publishRound2Pkg,
    TResult? Function()? waitRound2Pkg,
    TResult? Function()? waitingForFunds,
    TResult? Function()? waitOwnPublish,
    TResult? Function()? finalize,
    TResult? Function(String field0)? sharedAddress,
    TResult? Function(int peer, int ours, int theirs)? versionMismatch,
  }) {
    final _that = this;
    switch (_that) {
      case DKGStatus_WaitParams() when waitParams != null:
        return waitParams();
      case DKGStatus_WaitAddresses() when waitAddresses != null:
        return waitAddresses(_that.field0);
      case DKGStatus_PublishRound0Pkg() when publishRound0Pkg != null:
        return publishRound0Pkg();
      case DKGStatus_WaitRound0Pkg() when waitRound0Pkg != null:
        return waitRound0Pkg();
      case DKGStatus_PublishRound1Pkg() when publishRound1Pkg != null:
        return publishRound1Pkg();
      case DKGStatus_WaitRound1Pkg() when waitRound1Pkg != null:
        return waitRound1Pkg();
      case DKGStatus_PublishRound2Pkg() when publishRound2Pkg != null:
        return publishRound2Pkg();
      case DKGStatus_WaitRound2Pkg() when waitRound2Pkg != null:
        return waitRound2Pkg();
      case DKGStatus_WaitingForFunds() when waitingForFunds != null:
        return waitingForFunds();
      case DKGStatus_WaitOwnPublish() when waitOwnPublish != null:
        return waitOwnPublish();
      case DKGStatus_Finalize() when finalize != null:
        return finalize();
      case DKGStatus_SharedAddress() when sharedAddress != null:
        return sharedAddress(_that.field0);
      case DKGStatus_VersionMismatch() when versionMismatch != null:
        return versionMismatch(_that.peer, _that.ours, _that.theirs);
      case _:
        return null;
    }
  }
}

/// @nodoc

class DKGStatus_WaitParams extends DKGStatus {
  const DKGStatus_WaitParams() : super._();

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType && other is DKGStatus_WaitParams);
  }

  @override
  int get hashCode => runtimeType.hashCode;

  @override
  String toString() {
    return 'DKGStatus.waitParams()';
  }
}

/// @nodoc

class DKGStatus_WaitAddresses extends DKGStatus {
  const DKGStatus_WaitAddresses(final List<String> field0)
      : _field0 = field0,
        super._();

  final List<String> _field0;
  List<String> get field0 {
    if (_field0 is EqualUnmodifiableListView) return _field0;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_field0);
  }

  /// Create a copy of DKGStatus
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $DKGStatus_WaitAddressesCopyWith<DKGStatus_WaitAddresses> get copyWith =>
      _$DKGStatus_WaitAddressesCopyWithImpl<DKGStatus_WaitAddresses>(
          this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is DKGStatus_WaitAddresses &&
            const DeepCollectionEquality().equals(other._field0, _field0));
  }

  @override
  int get hashCode =>
      Object.hash(runtimeType, const DeepCollectionEquality().hash(_field0));

  @override
  String toString() {
    return 'DKGStatus.waitAddresses(field0: $field0)';
  }
}

/// @nodoc
abstract mixin class $DKGStatus_WaitAddressesCopyWith<$Res>
    implements $DKGStatusCopyWith<$Res> {
  factory $DKGStatus_WaitAddressesCopyWith(DKGStatus_WaitAddresses value,
          $Res Function(DKGStatus_WaitAddresses) _then) =
      _$DKGStatus_WaitAddressesCopyWithImpl;
  @useResult
  $Res call({List<String> field0});
}

/// @nodoc
class _$DKGStatus_WaitAddressesCopyWithImpl<$Res>
    implements $DKGStatus_WaitAddressesCopyWith<$Res> {
  _$DKGStatus_WaitAddressesCopyWithImpl(this._self, this._then);

  final DKGStatus_WaitAddresses _self;
  final $Res Function(DKGStatus_WaitAddresses) _then;

  /// Create a copy of DKGStatus
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  $Res call({
    Object? field0 = null,
  }) {
    return _then(DKGStatus_WaitAddresses(
      null == field0
          ? _self._field0
          : field0 // ignore: cast_nullable_to_non_nullable
              as List<String>,
    ));
  }
}

/// @nodoc

class DKGStatus_PublishRound0Pkg extends DKGStatus {
  const DKGStatus_PublishRound0Pkg() : super._();

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is DKGStatus_PublishRound0Pkg);
  }

  @override
  int get hashCode => runtimeType.hashCode;

  @override
  String toString() {
    return 'DKGStatus.publishRound0Pkg()';
  }
}

/// @nodoc

class DKGStatus_WaitRound0Pkg extends DKGStatus {
  const DKGStatus_WaitRound0Pkg() : super._();

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType && other is DKGStatus_WaitRound0Pkg);
  }

  @override
  int get hashCode => runtimeType.hashCode;

  @override
  String toString() {
    return 'DKGStatus.waitRound0Pkg()';
  }
}

/// @nodoc

class DKGStatus_PublishRound1Pkg extends DKGStatus {
  const DKGStatus_PublishRound1Pkg() : super._();

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is DKGStatus_PublishRound1Pkg);
  }

  @override
  int get hashCode => runtimeType.hashCode;

  @override
  String toString() {
    return 'DKGStatus.publishRound1Pkg()';
  }
}

/// @nodoc

class DKGStatus_WaitRound1Pkg extends DKGStatus {
  const DKGStatus_WaitRound1Pkg() : super._();

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType && other is DKGStatus_WaitRound1Pkg);
  }

  @override
  int get hashCode => runtimeType.hashCode;

  @override
  String toString() {
    return 'DKGStatus.waitRound1Pkg()';
  }
}

/// @nodoc

class DKGStatus_PublishRound2Pkg extends DKGStatus {
  const DKGStatus_PublishRound2Pkg() : super._();

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is DKGStatus_PublishRound2Pkg);
  }

  @override
  int get hashCode => runtimeType.hashCode;

  @override
  String toString() {
    return 'DKGStatus.publishRound2Pkg()';
  }
}

/// @nodoc

class DKGStatus_WaitRound2Pkg extends DKGStatus {
  const DKGStatus_WaitRound2Pkg() : super._();

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType && other is DKGStatus_WaitRound2Pkg);
  }

  @override
  int get hashCode => runtimeType.hashCode;

  @override
  String toString() {
    return 'DKGStatus.waitRound2Pkg()';
  }
}

/// @nodoc

class DKGStatus_WaitingForFunds extends DKGStatus {
  const DKGStatus_WaitingForFunds() : super._();

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is DKGStatus_WaitingForFunds);
  }

  @override
  int get hashCode => runtimeType.hashCode;

  @override
  String toString() {
    return 'DKGStatus.waitingForFunds()';
  }
}

/// @nodoc

class DKGStatus_WaitOwnPublish extends DKGStatus {
  const DKGStatus_WaitOwnPublish() : super._();

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType && other is DKGStatus_WaitOwnPublish);
  }

  @override
  int get hashCode => runtimeType.hashCode;

  @override
  String toString() {
    return 'DKGStatus.waitOwnPublish()';
  }
}

/// @nodoc

class DKGStatus_Finalize extends DKGStatus {
  const DKGStatus_Finalize() : super._();

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType && other is DKGStatus_Finalize);
  }

  @override
  int get hashCode => runtimeType.hashCode;

  @override
  String toString() {
    return 'DKGStatus.finalize()';
  }
}

/// @nodoc

class DKGStatus_SharedAddress extends DKGStatus {
  const DKGStatus_SharedAddress(this.field0) : super._();

  final String field0;

  /// Create a copy of DKGStatus
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $DKGStatus_SharedAddressCopyWith<DKGStatus_SharedAddress> get copyWith =>
      _$DKGStatus_SharedAddressCopyWithImpl<DKGStatus_SharedAddress>(
          this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is DKGStatus_SharedAddress &&
            (identical(other.field0, field0) || other.field0 == field0));
  }

  @override
  int get hashCode => Object.hash(runtimeType, field0);

  @override
  String toString() {
    return 'DKGStatus.sharedAddress(field0: $field0)';
  }
}

/// @nodoc
abstract mixin class $DKGStatus_SharedAddressCopyWith<$Res>
    implements $DKGStatusCopyWith<$Res> {
  factory $DKGStatus_SharedAddressCopyWith(DKGStatus_SharedAddress value,
          $Res Function(DKGStatus_SharedAddress) _then) =
      _$DKGStatus_SharedAddressCopyWithImpl;
  @useResult
  $Res call({String field0});
}

/// @nodoc
class _$DKGStatus_SharedAddressCopyWithImpl<$Res>
    implements $DKGStatus_SharedAddressCopyWith<$Res> {
  _$DKGStatus_SharedAddressCopyWithImpl(this._self, this._then);

  final DKGStatus_SharedAddress _self;
  final $Res Function(DKGStatus_SharedAddress) _then;

  /// Create a copy of DKGStatus
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  $Res call({
    Object? field0 = null,
  }) {
    return _then(DKGStatus_SharedAddress(
      null == field0
          ? _self.field0
          : field0 // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// @nodoc

class DKGStatus_VersionMismatch extends DKGStatus {
  const DKGStatus_VersionMismatch(
      {required this.peer, required this.ours, required this.theirs})
      : super._();

  final int peer;
  final int ours;
  final int theirs;

  /// Create a copy of DKGStatus
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $DKGStatus_VersionMismatchCopyWith<DKGStatus_VersionMismatch> get copyWith =>
      _$DKGStatus_VersionMismatchCopyWithImpl<DKGStatus_VersionMismatch>(
          this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is DKGStatus_VersionMismatch &&
            (identical(other.peer, peer) || other.peer == peer) &&
            (identical(other.ours, ours) || other.ours == ours) &&
            (identical(other.theirs, theirs) || other.theirs == theirs));
  }

  @override
  int get hashCode => Object.hash(runtimeType, peer, ours, theirs);

  @override
  String toString() {
    return 'DKGStatus.versionMismatch(peer: $peer, ours: $ours, theirs: $theirs)';
  }
}

/// @nodoc
abstract mixin class $DKGStatus_VersionMismatchCopyWith<$Res>
    implements $DKGStatusCopyWith<$Res> {
  factory $DKGStatus_VersionMismatchCopyWith(DKGStatus_VersionMismatch value,
          $Res Function(DKGStatus_VersionMismatch) _then) =
      _$DKGStatus_VersionMismatchCopyWithImpl;
  @useResult
  $Res call({int peer, int ours, int theirs});
}

/// @nodoc
class _$DKGStatus_VersionMismatchCopyWithImpl<$Res>
    implements $DKGStatus_VersionMismatchCopyWith<$Res> {
  _$DKGStatus_VersionMismatchCopyWithImpl(this._self, this._then);

  final DKGStatus_VersionMismatch _self;
  final $Res Function(DKGStatus_VersionMismatch) _then;

  /// Create a copy of DKGStatus
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  $Res call({
    Object? peer = null,
    Object? ours = null,
    Object? theirs = null,
  }) {
    return _then(DKGStatus_VersionMismatch(
      peer: null == peer
          ? _self.peer
          : peer // ignore: cast_nullable_to_non_nullable
              as int,
      ours: null == ours
          ? _self.ours
          : ours // ignore: cast_nullable_to_non_nullable
              as int,
      theirs: null == theirs
          ? _self.theirs
          : theirs // ignore: cast_nullable_to_non_nullable
              as int,
    ));
  }
}

/// @nodoc
mixin _$FrostRestore {
  int get account;
  int get dkgVersion;

  /// Create a copy of FrostRestore
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $FrostRestoreCopyWith<FrostRestore> get copyWith =>
      _$FrostRestoreCopyWithImpl<FrostRestore>(
          this as FrostRestore, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is FrostRestore &&
            (identical(other.account, account) || other.account == account) &&
            (identical(other.dkgVersion, dkgVersion) ||
                other.dkgVersion == dkgVersion));
  }

  @override
  int get hashCode => Object.hash(runtimeType, account, dkgVersion);

  @override
  String toString() {
    return 'FrostRestore(account: $account, dkgVersion: $dkgVersion)';
  }
}

/// @nodoc
abstract mixin class $FrostRestoreCopyWith<$Res> {
  factory $FrostRestoreCopyWith(
          FrostRestore value, $Res Function(FrostRestore) _then) =
      _$FrostRestoreCopyWithImpl;
  @useResult
  $Res call({int account, int dkgVersion});
}

/// @nodoc
class _$FrostRestoreCopyWithImpl<$Res> implements $FrostRestoreCopyWith<$Res> {
  _$FrostRestoreCopyWithImpl(this._self, this._then);

  final FrostRestore _self;
  final $Res Function(FrostRestore) _then;

  /// Create a copy of FrostRestore
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? account = null,
    Object? dkgVersion = null,
  }) {
    return _then(_self.copyWith(
      account: null == account
          ? _self.account
          : account // ignore: cast_nullable_to_non_nullable
              as int,
      dkgVersion: null == dkgVersion
          ? _self.dkgVersion
          : dkgVersion // ignore: cast_nullable_to_non_nullable
              as int,
    ));
  }
}

/// Adds pattern-matching-related methods to [FrostRestore].
extension FrostRestorePatterns on FrostRestore {
  /// A variant of `map` that fallback to returning `orElse`.
  ///
  /// It is equivalent to doing:
  /// ```dart
  /// switch (sealedClass) {
  ///   case final Subclass value:
  ///     return ...;
  ///   case _:
  ///     return orElse();
  /// }
  /// ```

  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>(
    TResult Function(_FrostRestore value)? $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _FrostRestore() when $default != null:
        return $default(_that);
      case _:
        return orElse();
    }
  }

  /// A `switch`-like method, using callbacks.
  ///
  /// Callbacks receives the raw object, upcasted.
  /// It is equivalent to doing:
  /// ```dart
  /// switch (sealedClass) {
  ///   case final Subclass value:
  ///     return ...;
  ///   case final Subclass2 value:
  ///     return ...;
  /// }
  /// ```

  @optionalTypeArgs
  TResult map<TResult extends Object?>(
    TResult Function(_FrostRestore value) $default,
  ) {
    final _that = this;
    switch (_that) {
      case _FrostRestore():
        return $default(_that);
    }
  }

  /// A variant of `map` that fallback to returning `null`.
  ///
  /// It is equivalent to doing:
  /// ```dart
  /// switch (sealedClass) {
  ///   case final Subclass value:
  ///     return ...;
  ///   case _:
  ///     return null;
  /// }
  /// ```

  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>(
    TResult? Function(_FrostRestore value)? $default,
  ) {
    final _that = this;
    switch (_that) {
      case _FrostRestore() when $default != null:
        return $default(_that);
      case _:
        return null;
    }
  }

  /// A variant of `when` that fallback to an `orElse` callback.
  ///
  /// It is equivalent to doing:
  /// ```dart
  /// switch (sealedClass) {
  ///   case Subclass(:final field):
  ///     return ...;
  ///   case _:
  ///     return orElse();
  /// }
  /// ```

  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>(
    TResult Function(int account, int dkgVersion)? $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _FrostRestore() when $default != null:
        return $default(_that.account, _that.dkgVersion);
      case _:
        return orElse();
    }
  }

  /// A `switch`-like method, using callbacks.
  ///
  /// As opposed to `map`, this offers destructuring.
  /// It is equivalent to doing:
  /// ```dart
  /// switch (sealedClass) {
  ///   case Subclass(:final field):
  ///     return ...;
  ///   case Subclass2(:final field2):
  ///     return ...;
  /// }
  /// ```

  @optionalTypeArgs
  TResult when<TResult extends Object?>(
    TResult Function(int account, int dkgVersion) $default,
  ) {
    final _that = this;
    switch (_that) {
      case _FrostRestore():
        return $default(_that.account, _that.dkgVersion);
    }
  }

  /// A variant of `when` that fallback to returning `null`
  ///
  /// It is equivalent to doing:
  /// ```dart
  /// switch (sealedClass) {
  ///   case Subclass(:final field):
  ///     return ...;
  ///   case _:
  ///     return null;
  /// }
  /// ```

  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>(
    TResult? Function(int account, int dkgVersion)? $default,
  ) {
    final _that = this;
    switch (_that) {
      case _FrostRestore() when $default != null:
        return $default(_that.account, _that.dkgVersion);
      case _:
        return null;
    }
  }
}

/// @nodoc

class _FrostRestore implements FrostRestore {
  const _FrostRestore({required this.account, required this.dkgVersion});

  @override
  final int account;
  @override
  final int dkgVersion;

  /// Create a copy of FrostRestore
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  _$FrostRestoreCopyWith<_FrostRestore> get copyWith =>
      __$FrostRestoreCopyWithImpl<_FrostRestore>(this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _FrostRestore &&
            (identical(other.account, account) || other.account == account) &&
            (identical(other.dkgVersion, dkgVersion) ||
                other.dkgVersion == dkgVersion));
  }

  @override
  int get hashCode => Object.hash(runtimeType, account, dkgVersion);

  @override
  String toString() {
    return 'FrostRestore(account: $account, dkgVersion: $dkgVersion)';
  }
}

/// @nodoc
abstract mixin class _$FrostRestoreCopyWith<$Res>
    implements $FrostRestoreCopyWith<$Res> {
  factory _$FrostRestoreCopyWith(
          _FrostRestore value, $Res Function(_FrostRestore) _then) =
      __$FrostRestoreCopyWithImpl;
  @override
  @useResult
  $Res call({int account, int dkgVersion});
}

/// @nodoc
class __$FrostRestoreCopyWithImpl<$Res>
    implements _$FrostRestoreCopyWith<$Res> {
  __$FrostRestoreCopyWithImpl(this._self, this._then);

  final _FrostRestore _self;
  final $Res Function(_FrostRestore) _then;

  /// Create a copy of FrostRestore
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $Res call({
    Object? account = null,
    Object? dkgVersion = null,
  }) {
    return _then(_FrostRestore(
      account: null == account
          ? _self.account
          : account // ignore: cast_nullable_to_non_nullable
              as int,
      dkgVersion: null == dkgVersion
          ? _self.dkgVersion
          : dkgVersion // ignore: cast_nullable_to_non_nullable
              as int,
    ));
  }
}

/// @nodoc
mixin _$FrostSignParams {
  int get account;
  int get coordinator;
  int get fundingAccount;

  /// Create a copy of FrostSignParams
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $FrostSignParamsCopyWith<FrostSignParams> get copyWith =>
      _$FrostSignParamsCopyWithImpl<FrostSignParams>(
          this as FrostSignParams, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is FrostSignParams &&
            (identical(other.account, account) || other.account == account) &&
            (identical(other.coordinator, coordinator) ||
                other.coordinator == coordinator) &&
            (identical(other.fundingAccount, fundingAccount) ||
                other.fundingAccount == fundingAccount));
  }

  @override
  int get hashCode =>
      Object.hash(runtimeType, account, coordinator, fundingAccount);

  @override
  String toString() {
    return 'FrostSignParams(account: $account, coordinator: $coordinator, fundingAccount: $fundingAccount)';
  }
}

/// @nodoc
abstract mixin class $FrostSignParamsCopyWith<$Res> {
  factory $FrostSignParamsCopyWith(
          FrostSignParams value, $Res Function(FrostSignParams) _then) =
      _$FrostSignParamsCopyWithImpl;
  @useResult
  $Res call({int account, int coordinator, int fundingAccount});
}

/// @nodoc
class _$FrostSignParamsCopyWithImpl<$Res>
    implements $FrostSignParamsCopyWith<$Res> {
  _$FrostSignParamsCopyWithImpl(this._self, this._then);

  final FrostSignParams _self;
  final $Res Function(FrostSignParams) _then;

  /// Create a copy of FrostSignParams
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? account = null,
    Object? coordinator = null,
    Object? fundingAccount = null,
  }) {
    return _then(_self.copyWith(
      account: null == account
          ? _self.account
          : account // ignore: cast_nullable_to_non_nullable
              as int,
      coordinator: null == coordinator
          ? _self.coordinator
          : coordinator // ignore: cast_nullable_to_non_nullable
              as int,
      fundingAccount: null == fundingAccount
          ? _self.fundingAccount
          : fundingAccount // ignore: cast_nullable_to_non_nullable
              as int,
    ));
  }
}

/// Adds pattern-matching-related methods to [FrostSignParams].
extension FrostSignParamsPatterns on FrostSignParams {
  /// A variant of `map` that fallback to returning `orElse`.
  ///
  /// It is equivalent to doing:
  /// ```dart
  /// switch (sealedClass) {
  ///   case final Subclass value:
  ///     return ...;
  ///   case _:
  ///     return orElse();
  /// }
  /// ```

  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>(
    TResult Function(_FrostSignParams value)? $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _FrostSignParams() when $default != null:
        return $default(_that);
      case _:
        return orElse();
    }
  }

  /// A `switch`-like method, using callbacks.
  ///
  /// Callbacks receives the raw object, upcasted.
  /// It is equivalent to doing:
  /// ```dart
  /// switch (sealedClass) {
  ///   case final Subclass value:
  ///     return ...;
  ///   case final Subclass2 value:
  ///     return ...;
  /// }
  /// ```

  @optionalTypeArgs
  TResult map<TResult extends Object?>(
    TResult Function(_FrostSignParams value) $default,
  ) {
    final _that = this;
    switch (_that) {
      case _FrostSignParams():
        return $default(_that);
    }
  }

  /// A variant of `map` that fallback to returning `null`.
  ///
  /// It is equivalent to doing:
  /// ```dart
  /// switch (sealedClass) {
  ///   case final Subclass value:
  ///     return ...;
  ///   case _:
  ///     return null;
  /// }
  /// ```

  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>(
    TResult? Function(_FrostSignParams value)? $default,
  ) {
    final _that = this;
    switch (_that) {
      case _FrostSignParams() when $default != null:
        return $default(_that);
      case _:
        return null;
    }
  }

  /// A variant of `when` that fallback to an `orElse` callback.
  ///
  /// It is equivalent to doing:
  /// ```dart
  /// switch (sealedClass) {
  ///   case Subclass(:final field):
  ///     return ...;
  ///   case _:
  ///     return orElse();
  /// }
  /// ```

  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>(
    TResult Function(int account, int coordinator, int fundingAccount)?
        $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _FrostSignParams() when $default != null:
        return $default(_that.account, _that.coordinator, _that.fundingAccount);
      case _:
        return orElse();
    }
  }

  /// A `switch`-like method, using callbacks.
  ///
  /// As opposed to `map`, this offers destructuring.
  /// It is equivalent to doing:
  /// ```dart
  /// switch (sealedClass) {
  ///   case Subclass(:final field):
  ///     return ...;
  ///   case Subclass2(:final field2):
  ///     return ...;
  /// }
  /// ```

  @optionalTypeArgs
  TResult when<TResult extends Object?>(
    TResult Function(int account, int coordinator, int fundingAccount) $default,
  ) {
    final _that = this;
    switch (_that) {
      case _FrostSignParams():
        return $default(_that.account, _that.coordinator, _that.fundingAccount);
    }
  }

  /// A variant of `when` that fallback to returning `null`
  ///
  /// It is equivalent to doing:
  /// ```dart
  /// switch (sealedClass) {
  ///   case Subclass(:final field):
  ///     return ...;
  ///   case _:
  ///     return null;
  /// }
  /// ```

  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>(
    TResult? Function(int account, int coordinator, int fundingAccount)?
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _FrostSignParams() when $default != null:
        return $default(_that.account, _that.coordinator, _that.fundingAccount);
      case _:
        return null;
    }
  }
}

/// @nodoc

class _FrostSignParams extends FrostSignParams {
  const _FrostSignParams(
      {required this.account,
      required this.coordinator,
      required this.fundingAccount})
      : super._();

  @override
  final int account;
  @override
  final int coordinator;
  @override
  final int fundingAccount;

  /// Create a copy of FrostSignParams
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  _$FrostSignParamsCopyWith<_FrostSignParams> get copyWith =>
      __$FrostSignParamsCopyWithImpl<_FrostSignParams>(this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _FrostSignParams &&
            (identical(other.account, account) || other.account == account) &&
            (identical(other.coordinator, coordinator) ||
                other.coordinator == coordinator) &&
            (identical(other.fundingAccount, fundingAccount) ||
                other.fundingAccount == fundingAccount));
  }

  @override
  int get hashCode =>
      Object.hash(runtimeType, account, coordinator, fundingAccount);

  @override
  String toString() {
    return 'FrostSignParams(account: $account, coordinator: $coordinator, fundingAccount: $fundingAccount)';
  }
}

/// @nodoc
abstract mixin class _$FrostSignParamsCopyWith<$Res>
    implements $FrostSignParamsCopyWith<$Res> {
  factory _$FrostSignParamsCopyWith(
          _FrostSignParams value, $Res Function(_FrostSignParams) _then) =
      __$FrostSignParamsCopyWithImpl;
  @override
  @useResult
  $Res call({int account, int coordinator, int fundingAccount});
}

/// @nodoc
class __$FrostSignParamsCopyWithImpl<$Res>
    implements _$FrostSignParamsCopyWith<$Res> {
  __$FrostSignParamsCopyWithImpl(this._self, this._then);

  final _FrostSignParams _self;
  final $Res Function(_FrostSignParams) _then;

  /// Create a copy of FrostSignParams
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $Res call({
    Object? account = null,
    Object? coordinator = null,
    Object? fundingAccount = null,
  }) {
    return _then(_FrostSignParams(
      account: null == account
          ? _self.account
          : account // ignore: cast_nullable_to_non_nullable
              as int,
      coordinator: null == coordinator
          ? _self.coordinator
          : coordinator // ignore: cast_nullable_to_non_nullable
              as int,
      fundingAccount: null == fundingAccount
          ? _self.fundingAccount
          : fundingAccount // ignore: cast_nullable_to_non_nullable
              as int,
    ));
  }
}

/// @nodoc
mixin _$FundingShortfall {
  int get account;
  BigInt get balance;
  BigInt get messageCost;

  /// Create a copy of FundingShortfall
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $FundingShortfallCopyWith<FundingShortfall> get copyWith =>
      _$FundingShortfallCopyWithImpl<FundingShortfall>(
          this as FundingShortfall, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is FundingShortfall &&
            (identical(other.account, account) || other.account == account) &&
            (identical(other.balance, balance) || other.balance == balance) &&
            (identical(other.messageCost, messageCost) ||
                other.messageCost == messageCost));
  }

  @override
  int get hashCode => Object.hash(runtimeType, account, balance, messageCost);

  @override
  String toString() {
    return 'FundingShortfall(account: $account, balance: $balance, messageCost: $messageCost)';
  }
}

/// @nodoc
abstract mixin class $FundingShortfallCopyWith<$Res> {
  factory $FundingShortfallCopyWith(
          FundingShortfall value, $Res Function(FundingShortfall) _then) =
      _$FundingShortfallCopyWithImpl;
  @useResult
  $Res call({int account, BigInt balance, BigInt messageCost});
}

/// @nodoc
class _$FundingShortfallCopyWithImpl<$Res>
    implements $FundingShortfallCopyWith<$Res> {
  _$FundingShortfallCopyWithImpl(this._self, this._then);

  final FundingShortfall _self;
  final $Res Function(FundingShortfall) _then;

  /// Create a copy of FundingShortfall
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? account = null,
    Object? balance = null,
    Object? messageCost = null,
  }) {
    return _then(_self.copyWith(
      account: null == account
          ? _self.account
          : account // ignore: cast_nullable_to_non_nullable
              as int,
      balance: null == balance
          ? _self.balance
          : balance // ignore: cast_nullable_to_non_nullable
              as BigInt,
      messageCost: null == messageCost
          ? _self.messageCost
          : messageCost // ignore: cast_nullable_to_non_nullable
              as BigInt,
    ));
  }
}

/// Adds pattern-matching-related methods to [FundingShortfall].
extension FundingShortfallPatterns on FundingShortfall {
  /// A variant of `map` that fallback to returning `orElse`.
  ///
  /// It is equivalent to doing:
  /// ```dart
  /// switch (sealedClass) {
  ///   case final Subclass value:
  ///     return ...;
  ///   case _:
  ///     return orElse();
  /// }
  /// ```

  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>(
    TResult Function(_FundingShortfall value)? $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _FundingShortfall() when $default != null:
        return $default(_that);
      case _:
        return orElse();
    }
  }

  /// A `switch`-like method, using callbacks.
  ///
  /// Callbacks receives the raw object, upcasted.
  /// It is equivalent to doing:
  /// ```dart
  /// switch (sealedClass) {
  ///   case final Subclass value:
  ///     return ...;
  ///   case final Subclass2 value:
  ///     return ...;
  /// }
  /// ```

  @optionalTypeArgs
  TResult map<TResult extends Object?>(
    TResult Function(_FundingShortfall value) $default,
  ) {
    final _that = this;
    switch (_that) {
      case _FundingShortfall():
        return $default(_that);
    }
  }

  /// A variant of `map` that fallback to returning `null`.
  ///
  /// It is equivalent to doing:
  /// ```dart
  /// switch (sealedClass) {
  ///   case final Subclass value:
  ///     return ...;
  ///   case _:
  ///     return null;
  /// }
  /// ```

  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>(
    TResult? Function(_FundingShortfall value)? $default,
  ) {
    final _that = this;
    switch (_that) {
      case _FundingShortfall() when $default != null:
        return $default(_that);
      case _:
        return null;
    }
  }

  /// A variant of `when` that fallback to an `orElse` callback.
  ///
  /// It is equivalent to doing:
  /// ```dart
  /// switch (sealedClass) {
  ///   case Subclass(:final field):
  ///     return ...;
  ///   case _:
  ///     return orElse();
  /// }
  /// ```

  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>(
    TResult Function(int account, BigInt balance, BigInt messageCost)?
        $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _FundingShortfall() when $default != null:
        return $default(_that.account, _that.balance, _that.messageCost);
      case _:
        return orElse();
    }
  }

  /// A `switch`-like method, using callbacks.
  ///
  /// As opposed to `map`, this offers destructuring.
  /// It is equivalent to doing:
  /// ```dart
  /// switch (sealedClass) {
  ///   case Subclass(:final field):
  ///     return ...;
  ///   case Subclass2(:final field2):
  ///     return ...;
  /// }
  /// ```

  @optionalTypeArgs
  TResult when<TResult extends Object?>(
    TResult Function(int account, BigInt balance, BigInt messageCost) $default,
  ) {
    final _that = this;
    switch (_that) {
      case _FundingShortfall():
        return $default(_that.account, _that.balance, _that.messageCost);
    }
  }

  /// A variant of `when` that fallback to returning `null`
  ///
  /// It is equivalent to doing:
  /// ```dart
  /// switch (sealedClass) {
  ///   case Subclass(:final field):
  ///     return ...;
  ///   case _:
  ///     return null;
  /// }
  /// ```

  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>(
    TResult? Function(int account, BigInt balance, BigInt messageCost)?
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _FundingShortfall() when $default != null:
        return $default(_that.account, _that.balance, _that.messageCost);
      case _:
        return null;
    }
  }
}

/// @nodoc

class _FundingShortfall implements FundingShortfall {
  const _FundingShortfall(
      {required this.account,
      required this.balance,
      required this.messageCost});

  @override
  final int account;
  @override
  final BigInt balance;
  @override
  final BigInt messageCost;

  /// Create a copy of FundingShortfall
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  _$FundingShortfallCopyWith<_FundingShortfall> get copyWith =>
      __$FundingShortfallCopyWithImpl<_FundingShortfall>(this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _FundingShortfall &&
            (identical(other.account, account) || other.account == account) &&
            (identical(other.balance, balance) || other.balance == balance) &&
            (identical(other.messageCost, messageCost) ||
                other.messageCost == messageCost));
  }

  @override
  int get hashCode => Object.hash(runtimeType, account, balance, messageCost);

  @override
  String toString() {
    return 'FundingShortfall(account: $account, balance: $balance, messageCost: $messageCost)';
  }
}

/// @nodoc
abstract mixin class _$FundingShortfallCopyWith<$Res>
    implements $FundingShortfallCopyWith<$Res> {
  factory _$FundingShortfallCopyWith(
          _FundingShortfall value, $Res Function(_FundingShortfall) _then) =
      __$FundingShortfallCopyWithImpl;
  @override
  @useResult
  $Res call({int account, BigInt balance, BigInt messageCost});
}

/// @nodoc
class __$FundingShortfallCopyWithImpl<$Res>
    implements _$FundingShortfallCopyWith<$Res> {
  __$FundingShortfallCopyWithImpl(this._self, this._then);

  final _FundingShortfall _self;
  final $Res Function(_FundingShortfall) _then;

  /// Create a copy of FundingShortfall
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $Res call({
    Object? account = null,
    Object? balance = null,
    Object? messageCost = null,
  }) {
    return _then(_FundingShortfall(
      account: null == account
          ? _self.account
          : account // ignore: cast_nullable_to_non_nullable
              as int,
      balance: null == balance
          ? _self.balance
          : balance // ignore: cast_nullable_to_non_nullable
              as BigInt,
      messageCost: null == messageCost
          ? _self.messageCost
          : messageCost // ignore: cast_nullable_to_non_nullable
              as BigInt,
    ));
  }
}

/// @nodoc
mixin _$ProtocolStatus {
  Object get field0;

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is ProtocolStatus &&
            const DeepCollectionEquality().equals(other.field0, field0));
  }

  @override
  int get hashCode =>
      Object.hash(runtimeType, const DeepCollectionEquality().hash(field0));

  @override
  String toString() {
    return 'ProtocolStatus(field0: $field0)';
  }
}

/// @nodoc
class $ProtocolStatusCopyWith<$Res> {
  $ProtocolStatusCopyWith(ProtocolStatus _, $Res Function(ProtocolStatus) __);
}

/// Adds pattern-matching-related methods to [ProtocolStatus].
extension ProtocolStatusPatterns on ProtocolStatus {
  /// A variant of `map` that fallback to returning `orElse`.
  ///
  /// It is equivalent to doing:
  /// ```dart
  /// switch (sealedClass) {
  ///   case final Subclass value:
  ///     return ...;
  ///   case _:
  ///     return orElse();
  /// }
  /// ```

  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(ProtocolStatus_Dkg value)? dkg,
    TResult Function(ProtocolStatus_Sign value)? sign,
    TResult Function(ProtocolStatus_Warning value)? warning,
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case ProtocolStatus_Dkg() when dkg != null:
        return dkg(_that);
      case ProtocolStatus_Sign() when sign != null:
        return sign(_that);
      case ProtocolStatus_Warning() when warning != null:
        return warning(_that);
      case _:
        return orElse();
    }
  }

  /// A `switch`-like method, using callbacks.
  ///
  /// Callbacks receives the raw object, upcasted.
  /// It is equivalent to doing:
  /// ```dart
  /// switch (sealedClass) {
  ///   case final Subclass value:
  ///     return ...;
  ///   case final Subclass2 value:
  ///     return ...;
  /// }
  /// ```

  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(ProtocolStatus_Dkg value) dkg,
    required TResult Function(ProtocolStatus_Sign value) sign,
    required TResult Function(ProtocolStatus_Warning value) warning,
  }) {
    final _that = this;
    switch (_that) {
      case ProtocolStatus_Dkg():
        return dkg(_that);
      case ProtocolStatus_Sign():
        return sign(_that);
      case ProtocolStatus_Warning():
        return warning(_that);
    }
  }

  /// A variant of `map` that fallback to returning `null`.
  ///
  /// It is equivalent to doing:
  /// ```dart
  /// switch (sealedClass) {
  ///   case final Subclass value:
  ///     return ...;
  ///   case _:
  ///     return null;
  /// }
  /// ```

  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(ProtocolStatus_Dkg value)? dkg,
    TResult? Function(ProtocolStatus_Sign value)? sign,
    TResult? Function(ProtocolStatus_Warning value)? warning,
  }) {
    final _that = this;
    switch (_that) {
      case ProtocolStatus_Dkg() when dkg != null:
        return dkg(_that);
      case ProtocolStatus_Sign() when sign != null:
        return sign(_that);
      case ProtocolStatus_Warning() when warning != null:
        return warning(_that);
      case _:
        return null;
    }
  }

  /// A variant of `when` that fallback to an `orElse` callback.
  ///
  /// It is equivalent to doing:
  /// ```dart
  /// switch (sealedClass) {
  ///   case Subclass(:final field):
  ///     return ...;
  ///   case _:
  ///     return orElse();
  /// }
  /// ```

  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(DKGStatus field0)? dkg,
    TResult Function(SigningStatus field0)? sign,
    TResult Function(String field0)? warning,
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case ProtocolStatus_Dkg() when dkg != null:
        return dkg(_that.field0);
      case ProtocolStatus_Sign() when sign != null:
        return sign(_that.field0);
      case ProtocolStatus_Warning() when warning != null:
        return warning(_that.field0);
      case _:
        return orElse();
    }
  }

  /// A `switch`-like method, using callbacks.
  ///
  /// As opposed to `map`, this offers destructuring.
  /// It is equivalent to doing:
  /// ```dart
  /// switch (sealedClass) {
  ///   case Subclass(:final field):
  ///     return ...;
  ///   case Subclass2(:final field2):
  ///     return ...;
  /// }
  /// ```

  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(DKGStatus field0) dkg,
    required TResult Function(SigningStatus field0) sign,
    required TResult Function(String field0) warning,
  }) {
    final _that = this;
    switch (_that) {
      case ProtocolStatus_Dkg():
        return dkg(_that.field0);
      case ProtocolStatus_Sign():
        return sign(_that.field0);
      case ProtocolStatus_Warning():
        return warning(_that.field0);
    }
  }

  /// A variant of `when` that fallback to returning `null`
  ///
  /// It is equivalent to doing:
  /// ```dart
  /// switch (sealedClass) {
  ///   case Subclass(:final field):
  ///     return ...;
  ///   case _:
  ///     return null;
  /// }
  /// ```

  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(DKGStatus field0)? dkg,
    TResult? Function(SigningStatus field0)? sign,
    TResult? Function(String field0)? warning,
  }) {
    final _that = this;
    switch (_that) {
      case ProtocolStatus_Dkg() when dkg != null:
        return dkg(_that.field0);
      case ProtocolStatus_Sign() when sign != null:
        return sign(_that.field0);
      case ProtocolStatus_Warning() when warning != null:
        return warning(_that.field0);
      case _:
        return null;
    }
  }
}

/// @nodoc

class ProtocolStatus_Dkg extends ProtocolStatus {
  const ProtocolStatus_Dkg(this.field0) : super._();

  @override
  final DKGStatus field0;

  /// Create a copy of ProtocolStatus
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $ProtocolStatus_DkgCopyWith<ProtocolStatus_Dkg> get copyWith =>
      _$ProtocolStatus_DkgCopyWithImpl<ProtocolStatus_Dkg>(this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is ProtocolStatus_Dkg &&
            (identical(other.field0, field0) || other.field0 == field0));
  }

  @override
  int get hashCode => Object.hash(runtimeType, field0);

  @override
  String toString() {
    return 'ProtocolStatus.dkg(field0: $field0)';
  }
}

/// @nodoc
abstract mixin class $ProtocolStatus_DkgCopyWith<$Res>
    implements $ProtocolStatusCopyWith<$Res> {
  factory $ProtocolStatus_DkgCopyWith(
          ProtocolStatus_Dkg value, $Res Function(ProtocolStatus_Dkg) _then) =
      _$ProtocolStatus_DkgCopyWithImpl;
  @useResult
  $Res call({DKGStatus field0});

  $DKGStatusCopyWith<$Res> get field0;
}

/// @nodoc
class _$ProtocolStatus_DkgCopyWithImpl<$Res>
    implements $ProtocolStatus_DkgCopyWith<$Res> {
  _$ProtocolStatus_DkgCopyWithImpl(this._self, this._then);

  final ProtocolStatus_Dkg _self;
  final $Res Function(ProtocolStatus_Dkg) _then;

  /// Create a copy of ProtocolStatus
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  $Res call({
    Object? field0 = null,
  }) {
    return _then(ProtocolStatus_Dkg(
      null == field0
          ? _self.field0
          : field0 // ignore: cast_nullable_to_non_nullable
              as DKGStatus,
    ));
  }

  /// Create a copy of ProtocolStatus
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $DKGStatusCopyWith<$Res> get field0 {
    return $DKGStatusCopyWith<$Res>(_self.field0, (value) {
      return _then(_self.copyWith(field0: value));
    });
  }
}

/// @nodoc

class ProtocolStatus_Sign extends ProtocolStatus {
  const ProtocolStatus_Sign(this.field0) : super._();

  @override
  final SigningStatus field0;

  /// Create a copy of ProtocolStatus
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $ProtocolStatus_SignCopyWith<ProtocolStatus_Sign> get copyWith =>
      _$ProtocolStatus_SignCopyWithImpl<ProtocolStatus_Sign>(this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is ProtocolStatus_Sign &&
            (identical(other.field0, field0) || other.field0 == field0));
  }

  @override
  int get hashCode => Object.hash(runtimeType, field0);

  @override
  String toString() {
    return 'ProtocolStatus.sign(field0: $field0)';
  }
}

/// @nodoc
abstract mixin class $ProtocolStatus_SignCopyWith<$Res>
    implements $ProtocolStatusCopyWith<$Res> {
  factory $ProtocolStatus_SignCopyWith(
          ProtocolStatus_Sign value, $Res Function(ProtocolStatus_Sign) _then) =
      _$ProtocolStatus_SignCopyWithImpl;
  @useResult
  $Res call({SigningStatus field0});

  $SigningStatusCopyWith<$Res> get field0;
}

/// @nodoc
class _$ProtocolStatus_SignCopyWithImpl<$Res>
    implements $ProtocolStatus_SignCopyWith<$Res> {
  _$ProtocolStatus_SignCopyWithImpl(this._self, this._then);

  final ProtocolStatus_Sign _self;
  final $Res Function(ProtocolStatus_Sign) _then;

  /// Create a copy of ProtocolStatus
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  $Res call({
    Object? field0 = null,
  }) {
    return _then(ProtocolStatus_Sign(
      null == field0
          ? _self.field0
          : field0 // ignore: cast_nullable_to_non_nullable
              as SigningStatus,
    ));
  }

  /// Create a copy of ProtocolStatus
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $SigningStatusCopyWith<$Res> get field0 {
    return $SigningStatusCopyWith<$Res>(_self.field0, (value) {
      return _then(_self.copyWith(field0: value));
    });
  }
}

/// @nodoc

class ProtocolStatus_Warning extends ProtocolStatus {
  const ProtocolStatus_Warning(this.field0) : super._();

  @override
  final String field0;

  /// Create a copy of ProtocolStatus
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $ProtocolStatus_WarningCopyWith<ProtocolStatus_Warning> get copyWith =>
      _$ProtocolStatus_WarningCopyWithImpl<ProtocolStatus_Warning>(
          this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is ProtocolStatus_Warning &&
            (identical(other.field0, field0) || other.field0 == field0));
  }

  @override
  int get hashCode => Object.hash(runtimeType, field0);

  @override
  String toString() {
    return 'ProtocolStatus.warning(field0: $field0)';
  }
}

/// @nodoc
abstract mixin class $ProtocolStatus_WarningCopyWith<$Res>
    implements $ProtocolStatusCopyWith<$Res> {
  factory $ProtocolStatus_WarningCopyWith(ProtocolStatus_Warning value,
          $Res Function(ProtocolStatus_Warning) _then) =
      _$ProtocolStatus_WarningCopyWithImpl;
  @useResult
  $Res call({String field0});
}

/// @nodoc
class _$ProtocolStatus_WarningCopyWithImpl<$Res>
    implements $ProtocolStatus_WarningCopyWith<$Res> {
  _$ProtocolStatus_WarningCopyWithImpl(this._self, this._then);

  final ProtocolStatus_Warning _self;
  final $Res Function(ProtocolStatus_Warning) _then;

  /// Create a copy of ProtocolStatus
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  $Res call({
    Object? field0 = null,
  }) {
    return _then(ProtocolStatus_Warning(
      null == field0
          ? _self.field0
          : field0 // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// @nodoc
mixin _$SigningStatus {
  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType && other is SigningStatus);
  }

  @override
  int get hashCode => runtimeType.hashCode;

  @override
  String toString() {
    return 'SigningStatus()';
  }
}

/// @nodoc
class $SigningStatusCopyWith<$Res> {
  $SigningStatusCopyWith(SigningStatus _, $Res Function(SigningStatus) __);
}

/// Adds pattern-matching-related methods to [SigningStatus].
extension SigningStatusPatterns on SigningStatus {
  /// A variant of `map` that fallback to returning `orElse`.
  ///
  /// It is equivalent to doing:
  /// ```dart
  /// switch (sealedClass) {
  ///   case final Subclass value:
  ///     return ...;
  ///   case _:
  ///     return orElse();
  /// }
  /// ```

  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(SigningStatus_AnnouncingCoordinator value)?
        announcingCoordinator,
    TResult Function(SigningStatus_WaitingForCoordinator value)?
        waitingForCoordinator,
    TResult Function(SigningStatus_SendingCommitment value)? sendingCommitment,
    TResult Function(SigningStatus_WaitingForCommitments value)?
        waitingForCommitments,
    TResult Function(SigningStatus_SendingSigningPackage value)?
        sendingSigningPackage,
    TResult Function(SigningStatus_WaitingForSigningPackage value)?
        waitingForSigningPackage,
    TResult Function(SigningStatus_SendingSignatureShare value)?
        sendingSignatureShare,
    TResult Function(SigningStatus_SigningCompleted value)? signingCompleted,
    TResult Function(SigningStatus_WaitingForSignatureShares value)?
        waitingForSignatureShares,
    TResult Function(SigningStatus_WaitingForFunds value)? waitingForFunds,
    TResult Function(SigningStatus_WaitOwnPublish value)? waitOwnPublish,
    TResult Function(SigningStatus_PreparingTransaction value)?
        preparingTransaction,
    TResult Function(SigningStatus_SendingTransaction value)?
        sendingTransaction,
    TResult Function(SigningStatus_TransactionSent value)? transactionSent,
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case SigningStatus_AnnouncingCoordinator()
          when announcingCoordinator != null:
        return announcingCoordinator(_that);
      case SigningStatus_WaitingForCoordinator()
          when waitingForCoordinator != null:
        return waitingForCoordinator(_that);
      case SigningStatus_SendingCommitment() when sendingCommitment != null:
        return sendingCommitment(_that);
      case SigningStatus_WaitingForCommitments()
          when waitingForCommitments != null:
        return waitingForCommitments(_that);
      case SigningStatus_SendingSigningPackage()
          when sendingSigningPackage != null:
        return sendingSigningPackage(_that);
      case SigningStatus_WaitingForSigningPackage()
          when waitingForSigningPackage != null:
        return waitingForSigningPackage(_that);
      case SigningStatus_SendingSignatureShare()
          when sendingSignatureShare != null:
        return sendingSignatureShare(_that);
      case SigningStatus_SigningCompleted() when signingCompleted != null:
        return signingCompleted(_that);
      case SigningStatus_WaitingForSignatureShares()
          when waitingForSignatureShares != null:
        return waitingForSignatureShares(_that);
      case SigningStatus_WaitingForFunds() when waitingForFunds != null:
        return waitingForFunds(_that);
      case SigningStatus_WaitOwnPublish() when waitOwnPublish != null:
        return waitOwnPublish(_that);
      case SigningStatus_PreparingTransaction()
          when preparingTransaction != null:
        return preparingTransaction(_that);
      case SigningStatus_SendingTransaction() when sendingTransaction != null:
        return sendingTransaction(_that);
      case SigningStatus_TransactionSent() when transactionSent != null:
        return transactionSent(_that);
      case _:
        return orElse();
    }
  }

  /// A `switch`-like method, using callbacks.
  ///
  /// Callbacks receives the raw object, upcasted.
  /// It is equivalent to doing:
  /// ```dart
  /// switch (sealedClass) {
  ///   case final Subclass value:
  ///     return ...;
  ///   case final Subclass2 value:
  ///     return ...;
  /// }
  /// ```

  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(SigningStatus_AnnouncingCoordinator value)
        announcingCoordinator,
    required TResult Function(SigningStatus_WaitingForCoordinator value)
        waitingForCoordinator,
    required TResult Function(SigningStatus_SendingCommitment value)
        sendingCommitment,
    required TResult Function(SigningStatus_WaitingForCommitments value)
        waitingForCommitments,
    required TResult Function(SigningStatus_SendingSigningPackage value)
        sendingSigningPackage,
    required TResult Function(SigningStatus_WaitingForSigningPackage value)
        waitingForSigningPackage,
    required TResult Function(SigningStatus_SendingSignatureShare value)
        sendingSignatureShare,
    required TResult Function(SigningStatus_SigningCompleted value)
        signingCompleted,
    required TResult Function(SigningStatus_WaitingForSignatureShares value)
        waitingForSignatureShares,
    required TResult Function(SigningStatus_WaitingForFunds value)
        waitingForFunds,
    required TResult Function(SigningStatus_WaitOwnPublish value)
        waitOwnPublish,
    required TResult Function(SigningStatus_PreparingTransaction value)
        preparingTransaction,
    required TResult Function(SigningStatus_SendingTransaction value)
        sendingTransaction,
    required TResult Function(SigningStatus_TransactionSent value)
        transactionSent,
  }) {
    final _that = this;
    switch (_that) {
      case SigningStatus_AnnouncingCoordinator():
        return announcingCoordinator(_that);
      case SigningStatus_WaitingForCoordinator():
        return waitingForCoordinator(_that);
      case SigningStatus_SendingCommitment():
        return sendingCommitment(_that);
      case SigningStatus_WaitingForCommitments():
        return waitingForCommitments(_that);
      case SigningStatus_SendingSigningPackage():
        return sendingSigningPackage(_that);
      case SigningStatus_WaitingForSigningPackage():
        return waitingForSigningPackage(_that);
      case SigningStatus_SendingSignatureShare():
        return sendingSignatureShare(_that);
      case SigningStatus_SigningCompleted():
        return signingCompleted(_that);
      case SigningStatus_WaitingForSignatureShares():
        return waitingForSignatureShares(_that);
      case SigningStatus_WaitingForFunds():
        return waitingForFunds(_that);
      case SigningStatus_WaitOwnPublish():
        return waitOwnPublish(_that);
      case SigningStatus_PreparingTransaction():
        return preparingTransaction(_that);
      case SigningStatus_SendingTransaction():
        return sendingTransaction(_that);
      case SigningStatus_TransactionSent():
        return transactionSent(_that);
    }
  }

  /// A variant of `map` that fallback to returning `null`.
  ///
  /// It is equivalent to doing:
  /// ```dart
  /// switch (sealedClass) {
  ///   case final Subclass value:
  ///     return ...;
  ///   case _:
  ///     return null;
  /// }
  /// ```

  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(SigningStatus_AnnouncingCoordinator value)?
        announcingCoordinator,
    TResult? Function(SigningStatus_WaitingForCoordinator value)?
        waitingForCoordinator,
    TResult? Function(SigningStatus_SendingCommitment value)? sendingCommitment,
    TResult? Function(SigningStatus_WaitingForCommitments value)?
        waitingForCommitments,
    TResult? Function(SigningStatus_SendingSigningPackage value)?
        sendingSigningPackage,
    TResult? Function(SigningStatus_WaitingForSigningPackage value)?
        waitingForSigningPackage,
    TResult? Function(SigningStatus_SendingSignatureShare value)?
        sendingSignatureShare,
    TResult? Function(SigningStatus_SigningCompleted value)? signingCompleted,
    TResult? Function(SigningStatus_WaitingForSignatureShares value)?
        waitingForSignatureShares,
    TResult? Function(SigningStatus_WaitingForFunds value)? waitingForFunds,
    TResult? Function(SigningStatus_WaitOwnPublish value)? waitOwnPublish,
    TResult? Function(SigningStatus_PreparingTransaction value)?
        preparingTransaction,
    TResult? Function(SigningStatus_SendingTransaction value)?
        sendingTransaction,
    TResult? Function(SigningStatus_TransactionSent value)? transactionSent,
  }) {
    final _that = this;
    switch (_that) {
      case SigningStatus_AnnouncingCoordinator()
          when announcingCoordinator != null:
        return announcingCoordinator(_that);
      case SigningStatus_WaitingForCoordinator()
          when waitingForCoordinator != null:
        return waitingForCoordinator(_that);
      case SigningStatus_SendingCommitment() when sendingCommitment != null:
        return sendingCommitment(_that);
      case SigningStatus_WaitingForCommitments()
          when waitingForCommitments != null:
        return waitingForCommitments(_that);
      case SigningStatus_SendingSigningPackage()
          when sendingSigningPackage != null:
        return sendingSigningPackage(_that);
      case SigningStatus_WaitingForSigningPackage()
          when waitingForSigningPackage != null:
        return waitingForSigningPackage(_that);
      case SigningStatus_SendingSignatureShare()
          when sendingSignatureShare != null:
        return sendingSignatureShare(_that);
      case SigningStatus_SigningCompleted() when signingCompleted != null:
        return signingCompleted(_that);
      case SigningStatus_WaitingForSignatureShares()
          when waitingForSignatureShares != null:
        return waitingForSignatureShares(_that);
      case SigningStatus_WaitingForFunds() when waitingForFunds != null:
        return waitingForFunds(_that);
      case SigningStatus_WaitOwnPublish() when waitOwnPublish != null:
        return waitOwnPublish(_that);
      case SigningStatus_PreparingTransaction()
          when preparingTransaction != null:
        return preparingTransaction(_that);
      case SigningStatus_SendingTransaction() when sendingTransaction != null:
        return sendingTransaction(_that);
      case SigningStatus_TransactionSent() when transactionSent != null:
        return transactionSent(_that);
      case _:
        return null;
    }
  }

  /// A variant of `when` that fallback to an `orElse` callback.
  ///
  /// It is equivalent to doing:
  /// ```dart
  /// switch (sealedClass) {
  ///   case Subclass(:final field):
  ///     return ...;
  ///   case _:
  ///     return orElse();
  /// }
  /// ```

  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function()? announcingCoordinator,
    TResult Function()? waitingForCoordinator,
    TResult Function()? sendingCommitment,
    TResult Function()? waitingForCommitments,
    TResult Function()? sendingSigningPackage,
    TResult Function()? waitingForSigningPackage,
    TResult Function()? sendingSignatureShare,
    TResult Function()? signingCompleted,
    TResult Function()? waitingForSignatureShares,
    TResult Function()? waitingForFunds,
    TResult Function()? waitOwnPublish,
    TResult Function()? preparingTransaction,
    TResult Function()? sendingTransaction,
    TResult Function(String field0)? transactionSent,
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case SigningStatus_AnnouncingCoordinator()
          when announcingCoordinator != null:
        return announcingCoordinator();
      case SigningStatus_WaitingForCoordinator()
          when waitingForCoordinator != null:
        return waitingForCoordinator();
      case SigningStatus_SendingCommitment() when sendingCommitment != null:
        return sendingCommitment();
      case SigningStatus_WaitingForCommitments()
          when waitingForCommitments != null:
        return waitingForCommitments();
      case SigningStatus_SendingSigningPackage()
          when sendingSigningPackage != null:
        return sendingSigningPackage();
      case SigningStatus_WaitingForSigningPackage()
          when waitingForSigningPackage != null:
        return waitingForSigningPackage();
      case SigningStatus_SendingSignatureShare()
          when sendingSignatureShare != null:
        return sendingSignatureShare();
      case SigningStatus_SigningCompleted() when signingCompleted != null:
        return signingCompleted();
      case SigningStatus_WaitingForSignatureShares()
          when waitingForSignatureShares != null:
        return waitingForSignatureShares();
      case SigningStatus_WaitingForFunds() when waitingForFunds != null:
        return waitingForFunds();
      case SigningStatus_WaitOwnPublish() when waitOwnPublish != null:
        return waitOwnPublish();
      case SigningStatus_PreparingTransaction()
          when preparingTransaction != null:
        return preparingTransaction();
      case SigningStatus_SendingTransaction() when sendingTransaction != null:
        return sendingTransaction();
      case SigningStatus_TransactionSent() when transactionSent != null:
        return transactionSent(_that.field0);
      case _:
        return orElse();
    }
  }

  /// A `switch`-like method, using callbacks.
  ///
  /// As opposed to `map`, this offers destructuring.
  /// It is equivalent to doing:
  /// ```dart
  /// switch (sealedClass) {
  ///   case Subclass(:final field):
  ///     return ...;
  ///   case Subclass2(:final field2):
  ///     return ...;
  /// }
  /// ```

  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function() announcingCoordinator,
    required TResult Function() waitingForCoordinator,
    required TResult Function() sendingCommitment,
    required TResult Function() waitingForCommitments,
    required TResult Function() sendingSigningPackage,
    required TResult Function() waitingForSigningPackage,
    required TResult Function() sendingSignatureShare,
    required TResult Function() signingCompleted,
    required TResult Function() waitingForSignatureShares,
    required TResult Function() waitingForFunds,
    required TResult Function() waitOwnPublish,
    required TResult Function() preparingTransaction,
    required TResult Function() sendingTransaction,
    required TResult Function(String field0) transactionSent,
  }) {
    final _that = this;
    switch (_that) {
      case SigningStatus_AnnouncingCoordinator():
        return announcingCoordinator();
      case SigningStatus_WaitingForCoordinator():
        return waitingForCoordinator();
      case SigningStatus_SendingCommitment():
        return sendingCommitment();
      case SigningStatus_WaitingForCommitments():
        return waitingForCommitments();
      case SigningStatus_SendingSigningPackage():
        return sendingSigningPackage();
      case SigningStatus_WaitingForSigningPackage():
        return waitingForSigningPackage();
      case SigningStatus_SendingSignatureShare():
        return sendingSignatureShare();
      case SigningStatus_SigningCompleted():
        return signingCompleted();
      case SigningStatus_WaitingForSignatureShares():
        return waitingForSignatureShares();
      case SigningStatus_WaitingForFunds():
        return waitingForFunds();
      case SigningStatus_WaitOwnPublish():
        return waitOwnPublish();
      case SigningStatus_PreparingTransaction():
        return preparingTransaction();
      case SigningStatus_SendingTransaction():
        return sendingTransaction();
      case SigningStatus_TransactionSent():
        return transactionSent(_that.field0);
    }
  }

  /// A variant of `when` that fallback to returning `null`
  ///
  /// It is equivalent to doing:
  /// ```dart
  /// switch (sealedClass) {
  ///   case Subclass(:final field):
  ///     return ...;
  ///   case _:
  ///     return null;
  /// }
  /// ```

  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function()? announcingCoordinator,
    TResult? Function()? waitingForCoordinator,
    TResult? Function()? sendingCommitment,
    TResult? Function()? waitingForCommitments,
    TResult? Function()? sendingSigningPackage,
    TResult? Function()? waitingForSigningPackage,
    TResult? Function()? sendingSignatureShare,
    TResult? Function()? signingCompleted,
    TResult? Function()? waitingForSignatureShares,
    TResult? Function()? waitingForFunds,
    TResult? Function()? waitOwnPublish,
    TResult? Function()? preparingTransaction,
    TResult? Function()? sendingTransaction,
    TResult? Function(String field0)? transactionSent,
  }) {
    final _that = this;
    switch (_that) {
      case SigningStatus_AnnouncingCoordinator()
          when announcingCoordinator != null:
        return announcingCoordinator();
      case SigningStatus_WaitingForCoordinator()
          when waitingForCoordinator != null:
        return waitingForCoordinator();
      case SigningStatus_SendingCommitment() when sendingCommitment != null:
        return sendingCommitment();
      case SigningStatus_WaitingForCommitments()
          when waitingForCommitments != null:
        return waitingForCommitments();
      case SigningStatus_SendingSigningPackage()
          when sendingSigningPackage != null:
        return sendingSigningPackage();
      case SigningStatus_WaitingForSigningPackage()
          when waitingForSigningPackage != null:
        return waitingForSigningPackage();
      case SigningStatus_SendingSignatureShare()
          when sendingSignatureShare != null:
        return sendingSignatureShare();
      case SigningStatus_SigningCompleted() when signingCompleted != null:
        return signingCompleted();
      case SigningStatus_WaitingForSignatureShares()
          when waitingForSignatureShares != null:
        return waitingForSignatureShares();
      case SigningStatus_WaitingForFunds() when waitingForFunds != null:
        return waitingForFunds();
      case SigningStatus_WaitOwnPublish() when waitOwnPublish != null:
        return waitOwnPublish();
      case SigningStatus_PreparingTransaction()
          when preparingTransaction != null:
        return preparingTransaction();
      case SigningStatus_SendingTransaction() when sendingTransaction != null:
        return sendingTransaction();
      case SigningStatus_TransactionSent() when transactionSent != null:
        return transactionSent(_that.field0);
      case _:
        return null;
    }
  }
}

/// @nodoc

class SigningStatus_AnnouncingCoordinator extends SigningStatus {
  const SigningStatus_AnnouncingCoordinator() : super._();

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is SigningStatus_AnnouncingCoordinator);
  }

  @override
  int get hashCode => runtimeType.hashCode;

  @override
  String toString() {
    return 'SigningStatus.announcingCoordinator()';
  }
}

/// @nodoc

class SigningStatus_WaitingForCoordinator extends SigningStatus {
  const SigningStatus_WaitingForCoordinator() : super._();

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is SigningStatus_WaitingForCoordinator);
  }

  @override
  int get hashCode => runtimeType.hashCode;

  @override
  String toString() {
    return 'SigningStatus.waitingForCoordinator()';
  }
}

/// @nodoc

class SigningStatus_SendingCommitment extends SigningStatus {
  const SigningStatus_SendingCommitment() : super._();

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is SigningStatus_SendingCommitment);
  }

  @override
  int get hashCode => runtimeType.hashCode;

  @override
  String toString() {
    return 'SigningStatus.sendingCommitment()';
  }
}

/// @nodoc

class SigningStatus_WaitingForCommitments extends SigningStatus {
  const SigningStatus_WaitingForCommitments() : super._();

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is SigningStatus_WaitingForCommitments);
  }

  @override
  int get hashCode => runtimeType.hashCode;

  @override
  String toString() {
    return 'SigningStatus.waitingForCommitments()';
  }
}

/// @nodoc

class SigningStatus_SendingSigningPackage extends SigningStatus {
  const SigningStatus_SendingSigningPackage() : super._();

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is SigningStatus_SendingSigningPackage);
  }

  @override
  int get hashCode => runtimeType.hashCode;

  @override
  String toString() {
    return 'SigningStatus.sendingSigningPackage()';
  }
}

/// @nodoc

class SigningStatus_WaitingForSigningPackage extends SigningStatus {
  const SigningStatus_WaitingForSigningPackage() : super._();

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is SigningStatus_WaitingForSigningPackage);
  }

  @override
  int get hashCode => runtimeType.hashCode;

  @override
  String toString() {
    return 'SigningStatus.waitingForSigningPackage()';
  }
}

/// @nodoc

class SigningStatus_SendingSignatureShare extends SigningStatus {
  const SigningStatus_SendingSignatureShare() : super._();

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is SigningStatus_SendingSignatureShare);
  }

  @override
  int get hashCode => runtimeType.hashCode;

  @override
  String toString() {
    return 'SigningStatus.sendingSignatureShare()';
  }
}

/// @nodoc

class SigningStatus_SigningCompleted extends SigningStatus {
  const SigningStatus_SigningCompleted() : super._();

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is SigningStatus_SigningCompleted);
  }

  @override
  int get hashCode => runtimeType.hashCode;

  @override
  String toString() {
    return 'SigningStatus.signingCompleted()';
  }
}

/// @nodoc

class SigningStatus_WaitingForSignatureShares extends SigningStatus {
  const SigningStatus_WaitingForSignatureShares() : super._();

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is SigningStatus_WaitingForSignatureShares);
  }

  @override
  int get hashCode => runtimeType.hashCode;

  @override
  String toString() {
    return 'SigningStatus.waitingForSignatureShares()';
  }
}

/// @nodoc

class SigningStatus_WaitingForFunds extends SigningStatus {
  const SigningStatus_WaitingForFunds() : super._();

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is SigningStatus_WaitingForFunds);
  }

  @override
  int get hashCode => runtimeType.hashCode;

  @override
  String toString() {
    return 'SigningStatus.waitingForFunds()';
  }
}

/// @nodoc

class SigningStatus_WaitOwnPublish extends SigningStatus {
  const SigningStatus_WaitOwnPublish() : super._();

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is SigningStatus_WaitOwnPublish);
  }

  @override
  int get hashCode => runtimeType.hashCode;

  @override
  String toString() {
    return 'SigningStatus.waitOwnPublish()';
  }
}

/// @nodoc

class SigningStatus_PreparingTransaction extends SigningStatus {
  const SigningStatus_PreparingTransaction() : super._();

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is SigningStatus_PreparingTransaction);
  }

  @override
  int get hashCode => runtimeType.hashCode;

  @override
  String toString() {
    return 'SigningStatus.preparingTransaction()';
  }
}

/// @nodoc

class SigningStatus_SendingTransaction extends SigningStatus {
  const SigningStatus_SendingTransaction() : super._();

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is SigningStatus_SendingTransaction);
  }

  @override
  int get hashCode => runtimeType.hashCode;

  @override
  String toString() {
    return 'SigningStatus.sendingTransaction()';
  }
}

/// @nodoc

class SigningStatus_TransactionSent extends SigningStatus {
  const SigningStatus_TransactionSent(this.field0) : super._();

  final String field0;

  /// Create a copy of SigningStatus
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $SigningStatus_TransactionSentCopyWith<SigningStatus_TransactionSent>
      get copyWith => _$SigningStatus_TransactionSentCopyWithImpl<
          SigningStatus_TransactionSent>(this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is SigningStatus_TransactionSent &&
            (identical(other.field0, field0) || other.field0 == field0));
  }

  @override
  int get hashCode => Object.hash(runtimeType, field0);

  @override
  String toString() {
    return 'SigningStatus.transactionSent(field0: $field0)';
  }
}

/// @nodoc
abstract mixin class $SigningStatus_TransactionSentCopyWith<$Res>
    implements $SigningStatusCopyWith<$Res> {
  factory $SigningStatus_TransactionSentCopyWith(
          SigningStatus_TransactionSent value,
          $Res Function(SigningStatus_TransactionSent) _then) =
      _$SigningStatus_TransactionSentCopyWithImpl;
  @useResult
  $Res call({String field0});
}

/// @nodoc
class _$SigningStatus_TransactionSentCopyWithImpl<$Res>
    implements $SigningStatus_TransactionSentCopyWith<$Res> {
  _$SigningStatus_TransactionSentCopyWithImpl(this._self, this._then);

  final SigningStatus_TransactionSent _self;
  final $Res Function(SigningStatus_TransactionSent) _then;

  /// Create a copy of SigningStatus
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  $Res call({
    Object? field0 = null,
  }) {
    return _then(SigningStatus_TransactionSent(
      null == field0
          ? _self.field0
          : field0 // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

// dart format on
