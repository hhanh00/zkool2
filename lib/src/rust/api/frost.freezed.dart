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
    TResult Function(DKGStatus_PublishRound1Pkg value)? publishRound1Pkg,
    TResult Function(DKGStatus_WaitRound1Pkg value)? waitRound1Pkg,
    TResult Function(DKGStatus_PublishRound2Pkg value)? publishRound2Pkg,
    TResult Function(DKGStatus_WaitRound2Pkg value)? waitRound2Pkg,
    TResult Function(DKGStatus_Finalize value)? finalize,
    TResult Function(DKGStatus_SharedAddress value)? sharedAddress,
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case DKGStatus_WaitParams() when waitParams != null:
        return waitParams(_that);
      case DKGStatus_WaitAddresses() when waitAddresses != null:
        return waitAddresses(_that);
      case DKGStatus_PublishRound1Pkg() when publishRound1Pkg != null:
        return publishRound1Pkg(_that);
      case DKGStatus_WaitRound1Pkg() when waitRound1Pkg != null:
        return waitRound1Pkg(_that);
      case DKGStatus_PublishRound2Pkg() when publishRound2Pkg != null:
        return publishRound2Pkg(_that);
      case DKGStatus_WaitRound2Pkg() when waitRound2Pkg != null:
        return waitRound2Pkg(_that);
      case DKGStatus_Finalize() when finalize != null:
        return finalize(_that);
      case DKGStatus_SharedAddress() when sharedAddress != null:
        return sharedAddress(_that);
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
    required TResult Function(DKGStatus_PublishRound1Pkg value)
        publishRound1Pkg,
    required TResult Function(DKGStatus_WaitRound1Pkg value) waitRound1Pkg,
    required TResult Function(DKGStatus_PublishRound2Pkg value)
        publishRound2Pkg,
    required TResult Function(DKGStatus_WaitRound2Pkg value) waitRound2Pkg,
    required TResult Function(DKGStatus_Finalize value) finalize,
    required TResult Function(DKGStatus_SharedAddress value) sharedAddress,
  }) {
    final _that = this;
    switch (_that) {
      case DKGStatus_WaitParams():
        return waitParams(_that);
      case DKGStatus_WaitAddresses():
        return waitAddresses(_that);
      case DKGStatus_PublishRound1Pkg():
        return publishRound1Pkg(_that);
      case DKGStatus_WaitRound1Pkg():
        return waitRound1Pkg(_that);
      case DKGStatus_PublishRound2Pkg():
        return publishRound2Pkg(_that);
      case DKGStatus_WaitRound2Pkg():
        return waitRound2Pkg(_that);
      case DKGStatus_Finalize():
        return finalize(_that);
      case DKGStatus_SharedAddress():
        return sharedAddress(_that);
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
    TResult? Function(DKGStatus_PublishRound1Pkg value)? publishRound1Pkg,
    TResult? Function(DKGStatus_WaitRound1Pkg value)? waitRound1Pkg,
    TResult? Function(DKGStatus_PublishRound2Pkg value)? publishRound2Pkg,
    TResult? Function(DKGStatus_WaitRound2Pkg value)? waitRound2Pkg,
    TResult? Function(DKGStatus_Finalize value)? finalize,
    TResult? Function(DKGStatus_SharedAddress value)? sharedAddress,
  }) {
    final _that = this;
    switch (_that) {
      case DKGStatus_WaitParams() when waitParams != null:
        return waitParams(_that);
      case DKGStatus_WaitAddresses() when waitAddresses != null:
        return waitAddresses(_that);
      case DKGStatus_PublishRound1Pkg() when publishRound1Pkg != null:
        return publishRound1Pkg(_that);
      case DKGStatus_WaitRound1Pkg() when waitRound1Pkg != null:
        return waitRound1Pkg(_that);
      case DKGStatus_PublishRound2Pkg() when publishRound2Pkg != null:
        return publishRound2Pkg(_that);
      case DKGStatus_WaitRound2Pkg() when waitRound2Pkg != null:
        return waitRound2Pkg(_that);
      case DKGStatus_Finalize() when finalize != null:
        return finalize(_that);
      case DKGStatus_SharedAddress() when sharedAddress != null:
        return sharedAddress(_that);
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
    TResult Function()? publishRound1Pkg,
    TResult Function()? waitRound1Pkg,
    TResult Function()? publishRound2Pkg,
    TResult Function()? waitRound2Pkg,
    TResult Function()? finalize,
    TResult Function(String field0)? sharedAddress,
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case DKGStatus_WaitParams() when waitParams != null:
        return waitParams();
      case DKGStatus_WaitAddresses() when waitAddresses != null:
        return waitAddresses(_that.field0);
      case DKGStatus_PublishRound1Pkg() when publishRound1Pkg != null:
        return publishRound1Pkg();
      case DKGStatus_WaitRound1Pkg() when waitRound1Pkg != null:
        return waitRound1Pkg();
      case DKGStatus_PublishRound2Pkg() when publishRound2Pkg != null:
        return publishRound2Pkg();
      case DKGStatus_WaitRound2Pkg() when waitRound2Pkg != null:
        return waitRound2Pkg();
      case DKGStatus_Finalize() when finalize != null:
        return finalize();
      case DKGStatus_SharedAddress() when sharedAddress != null:
        return sharedAddress(_that.field0);
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
    required TResult Function() publishRound1Pkg,
    required TResult Function() waitRound1Pkg,
    required TResult Function() publishRound2Pkg,
    required TResult Function() waitRound2Pkg,
    required TResult Function() finalize,
    required TResult Function(String field0) sharedAddress,
  }) {
    final _that = this;
    switch (_that) {
      case DKGStatus_WaitParams():
        return waitParams();
      case DKGStatus_WaitAddresses():
        return waitAddresses(_that.field0);
      case DKGStatus_PublishRound1Pkg():
        return publishRound1Pkg();
      case DKGStatus_WaitRound1Pkg():
        return waitRound1Pkg();
      case DKGStatus_PublishRound2Pkg():
        return publishRound2Pkg();
      case DKGStatus_WaitRound2Pkg():
        return waitRound2Pkg();
      case DKGStatus_Finalize():
        return finalize();
      case DKGStatus_SharedAddress():
        return sharedAddress(_that.field0);
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
    TResult? Function()? publishRound1Pkg,
    TResult? Function()? waitRound1Pkg,
    TResult? Function()? publishRound2Pkg,
    TResult? Function()? waitRound2Pkg,
    TResult? Function()? finalize,
    TResult? Function(String field0)? sharedAddress,
  }) {
    final _that = this;
    switch (_that) {
      case DKGStatus_WaitParams() when waitParams != null:
        return waitParams();
      case DKGStatus_WaitAddresses() when waitAddresses != null:
        return waitAddresses(_that.field0);
      case DKGStatus_PublishRound1Pkg() when publishRound1Pkg != null:
        return publishRound1Pkg();
      case DKGStatus_WaitRound1Pkg() when waitRound1Pkg != null:
        return waitRound1Pkg();
      case DKGStatus_PublishRound2Pkg() when publishRound2Pkg != null:
        return publishRound2Pkg();
      case DKGStatus_WaitRound2Pkg() when waitRound2Pkg != null:
        return waitRound2Pkg();
      case DKGStatus_Finalize() when finalize != null:
        return finalize();
      case DKGStatus_SharedAddress() when sharedAddress != null:
        return sharedAddress(_that.field0);
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
    TResult Function(SigningStatus_PreparingTransaction value)?
        preparingTransaction,
    TResult Function(SigningStatus_SendingTransaction value)?
        sendingTransaction,
    TResult Function(SigningStatus_TransactionSent value)? transactionSent,
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
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
    required TResult Function(SigningStatus_PreparingTransaction value)
        preparingTransaction,
    required TResult Function(SigningStatus_SendingTransaction value)
        sendingTransaction,
    required TResult Function(SigningStatus_TransactionSent value)
        transactionSent,
  }) {
    final _that = this;
    switch (_that) {
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
    TResult? Function(SigningStatus_PreparingTransaction value)?
        preparingTransaction,
    TResult? Function(SigningStatus_SendingTransaction value)?
        sendingTransaction,
    TResult? Function(SigningStatus_TransactionSent value)? transactionSent,
  }) {
    final _that = this;
    switch (_that) {
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
    TResult Function()? sendingCommitment,
    TResult Function()? waitingForCommitments,
    TResult Function()? sendingSigningPackage,
    TResult Function()? waitingForSigningPackage,
    TResult Function()? sendingSignatureShare,
    TResult Function()? signingCompleted,
    TResult Function()? waitingForSignatureShares,
    TResult Function()? preparingTransaction,
    TResult Function()? sendingTransaction,
    TResult Function(String field0)? transactionSent,
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
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
    required TResult Function() sendingCommitment,
    required TResult Function() waitingForCommitments,
    required TResult Function() sendingSigningPackage,
    required TResult Function() waitingForSigningPackage,
    required TResult Function() sendingSignatureShare,
    required TResult Function() signingCompleted,
    required TResult Function() waitingForSignatureShares,
    required TResult Function() preparingTransaction,
    required TResult Function() sendingTransaction,
    required TResult Function(String field0) transactionSent,
  }) {
    final _that = this;
    switch (_that) {
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
    TResult? Function()? sendingCommitment,
    TResult? Function()? waitingForCommitments,
    TResult? Function()? sendingSigningPackage,
    TResult? Function()? waitingForSigningPackage,
    TResult? Function()? sendingSignatureShare,
    TResult? Function()? signingCompleted,
    TResult? Function()? waitingForSignatureShares,
    TResult? Function()? preparingTransaction,
    TResult? Function()? sendingTransaction,
    TResult? Function(String field0)? transactionSent,
  }) {
    final _that = this;
    switch (_that) {
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
