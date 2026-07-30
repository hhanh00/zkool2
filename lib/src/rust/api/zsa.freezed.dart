// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'zsa.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;

/// @nodoc
mixin _$ZsaHolding {
  PlatformInt64 get idAsset;
  Uint8List get assetDescHash;
  String get assetName;
  Uint8List get ik;
  Uint8List get assetBase;
  bool get finalized;
  int get firstSeenHeight;
  BigInt get balance;

  /// Create a copy of ZsaHolding
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $ZsaHoldingCopyWith<ZsaHolding> get copyWith => _$ZsaHoldingCopyWithImpl<ZsaHolding>(this as ZsaHolding, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is ZsaHolding &&
            (identical(other.idAsset, idAsset) || other.idAsset == idAsset) &&
            const DeepCollectionEquality().equals(other.assetDescHash, assetDescHash) &&
            (identical(other.assetName, assetName) || other.assetName == assetName) &&
            const DeepCollectionEquality().equals(other.ik, ik) &&
            const DeepCollectionEquality().equals(other.assetBase, assetBase) &&
            (identical(other.finalized, finalized) || other.finalized == finalized) &&
            (identical(other.firstSeenHeight, firstSeenHeight) || other.firstSeenHeight == firstSeenHeight) &&
            (identical(other.balance, balance) || other.balance == balance));
  }

  @override
  int get hashCode => Object.hash(runtimeType, idAsset, const DeepCollectionEquality().hash(assetDescHash), assetName, const DeepCollectionEquality().hash(ik),
      const DeepCollectionEquality().hash(assetBase), finalized, firstSeenHeight, balance);

  @override
  String toString() {
    return 'ZsaHolding(idAsset: $idAsset, assetDescHash: $assetDescHash, assetName: $assetName, ik: $ik, assetBase: $assetBase, finalized: $finalized, firstSeenHeight: $firstSeenHeight, balance: $balance)';
  }
}

/// @nodoc
abstract mixin class $ZsaHoldingCopyWith<$Res> {
  factory $ZsaHoldingCopyWith(ZsaHolding value, $Res Function(ZsaHolding) _then) = _$ZsaHoldingCopyWithImpl;
  @useResult
  $Res call(
      {PlatformInt64 idAsset,
      Uint8List assetDescHash,
      String assetName,
      Uint8List ik,
      Uint8List assetBase,
      bool finalized,
      int firstSeenHeight,
      BigInt balance});
}

/// @nodoc
class _$ZsaHoldingCopyWithImpl<$Res> implements $ZsaHoldingCopyWith<$Res> {
  _$ZsaHoldingCopyWithImpl(this._self, this._then);

  final ZsaHolding _self;
  final $Res Function(ZsaHolding) _then;

  /// Create a copy of ZsaHolding
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? idAsset = null,
    Object? assetDescHash = null,
    Object? assetName = null,
    Object? ik = null,
    Object? assetBase = null,
    Object? finalized = null,
    Object? firstSeenHeight = null,
    Object? balance = null,
  }) {
    return _then(_self.copyWith(
      idAsset: null == idAsset
          ? _self.idAsset
          : idAsset // ignore: cast_nullable_to_non_nullable
              as PlatformInt64,
      assetDescHash: null == assetDescHash
          ? _self.assetDescHash
          : assetDescHash // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      assetName: null == assetName
          ? _self.assetName
          : assetName // ignore: cast_nullable_to_non_nullable
              as String,
      ik: null == ik
          ? _self.ik
          : ik // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      assetBase: null == assetBase
          ? _self.assetBase
          : assetBase // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      finalized: null == finalized
          ? _self.finalized
          : finalized // ignore: cast_nullable_to_non_nullable
              as bool,
      firstSeenHeight: null == firstSeenHeight
          ? _self.firstSeenHeight
          : firstSeenHeight // ignore: cast_nullable_to_non_nullable
              as int,
      balance: null == balance
          ? _self.balance
          : balance // ignore: cast_nullable_to_non_nullable
              as BigInt,
    ));
  }
}

/// Adds pattern-matching-related methods to [ZsaHolding].
extension ZsaHoldingPatterns on ZsaHolding {
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
    TResult Function(_ZsaHolding value)? $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _ZsaHolding() when $default != null:
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
    TResult Function(_ZsaHolding value) $default,
  ) {
    final _that = this;
    switch (_that) {
      case _ZsaHolding():
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
    TResult? Function(_ZsaHolding value)? $default,
  ) {
    final _that = this;
    switch (_that) {
      case _ZsaHolding() when $default != null:
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
    TResult Function(PlatformInt64 idAsset, Uint8List assetDescHash, String assetName, Uint8List ik, Uint8List assetBase, bool finalized, int firstSeenHeight,
            BigInt balance)?
        $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _ZsaHolding() when $default != null:
        return $default(_that.idAsset, _that.assetDescHash, _that.assetName, _that.ik, _that.assetBase, _that.finalized, _that.firstSeenHeight, _that.balance);
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
    TResult Function(PlatformInt64 idAsset, Uint8List assetDescHash, String assetName, Uint8List ik, Uint8List assetBase, bool finalized, int firstSeenHeight,
            BigInt balance)
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _ZsaHolding():
        return $default(_that.idAsset, _that.assetDescHash, _that.assetName, _that.ik, _that.assetBase, _that.finalized, _that.firstSeenHeight, _that.balance);
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
    TResult? Function(PlatformInt64 idAsset, Uint8List assetDescHash, String assetName, Uint8List ik, Uint8List assetBase, bool finalized, int firstSeenHeight,
            BigInt balance)?
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _ZsaHolding() when $default != null:
        return $default(_that.idAsset, _that.assetDescHash, _that.assetName, _that.ik, _that.assetBase, _that.finalized, _that.firstSeenHeight, _that.balance);
      case _:
        return null;
    }
  }
}

/// @nodoc

class _ZsaHolding implements ZsaHolding {
  const _ZsaHolding(
      {required this.idAsset,
      required this.assetDescHash,
      required this.assetName,
      required this.ik,
      required this.assetBase,
      required this.finalized,
      required this.firstSeenHeight,
      required this.balance});

  @override
  final PlatformInt64 idAsset;
  @override
  final Uint8List assetDescHash;
  @override
  final String assetName;
  @override
  final Uint8List ik;
  @override
  final Uint8List assetBase;
  @override
  final bool finalized;
  @override
  final int firstSeenHeight;
  @override
  final BigInt balance;

  /// Create a copy of ZsaHolding
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  _$ZsaHoldingCopyWith<_ZsaHolding> get copyWith => __$ZsaHoldingCopyWithImpl<_ZsaHolding>(this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _ZsaHolding &&
            (identical(other.idAsset, idAsset) || other.idAsset == idAsset) &&
            const DeepCollectionEquality().equals(other.assetDescHash, assetDescHash) &&
            (identical(other.assetName, assetName) || other.assetName == assetName) &&
            const DeepCollectionEquality().equals(other.ik, ik) &&
            const DeepCollectionEquality().equals(other.assetBase, assetBase) &&
            (identical(other.finalized, finalized) || other.finalized == finalized) &&
            (identical(other.firstSeenHeight, firstSeenHeight) || other.firstSeenHeight == firstSeenHeight) &&
            (identical(other.balance, balance) || other.balance == balance));
  }

  @override
  int get hashCode => Object.hash(runtimeType, idAsset, const DeepCollectionEquality().hash(assetDescHash), assetName, const DeepCollectionEquality().hash(ik),
      const DeepCollectionEquality().hash(assetBase), finalized, firstSeenHeight, balance);

  @override
  String toString() {
    return 'ZsaHolding(idAsset: $idAsset, assetDescHash: $assetDescHash, assetName: $assetName, ik: $ik, assetBase: $assetBase, finalized: $finalized, firstSeenHeight: $firstSeenHeight, balance: $balance)';
  }
}

/// @nodoc
abstract mixin class _$ZsaHoldingCopyWith<$Res> implements $ZsaHoldingCopyWith<$Res> {
  factory _$ZsaHoldingCopyWith(_ZsaHolding value, $Res Function(_ZsaHolding) _then) = __$ZsaHoldingCopyWithImpl;
  @override
  @useResult
  $Res call(
      {PlatformInt64 idAsset,
      Uint8List assetDescHash,
      String assetName,
      Uint8List ik,
      Uint8List assetBase,
      bool finalized,
      int firstSeenHeight,
      BigInt balance});
}

/// @nodoc
class __$ZsaHoldingCopyWithImpl<$Res> implements _$ZsaHoldingCopyWith<$Res> {
  __$ZsaHoldingCopyWithImpl(this._self, this._then);

  final _ZsaHolding _self;
  final $Res Function(_ZsaHolding) _then;

  /// Create a copy of ZsaHolding
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $Res call({
    Object? idAsset = null,
    Object? assetDescHash = null,
    Object? assetName = null,
    Object? ik = null,
    Object? assetBase = null,
    Object? finalized = null,
    Object? firstSeenHeight = null,
    Object? balance = null,
  }) {
    return _then(_ZsaHolding(
      idAsset: null == idAsset
          ? _self.idAsset
          : idAsset // ignore: cast_nullable_to_non_nullable
              as PlatformInt64,
      assetDescHash: null == assetDescHash
          ? _self.assetDescHash
          : assetDescHash // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      assetName: null == assetName
          ? _self.assetName
          : assetName // ignore: cast_nullable_to_non_nullable
              as String,
      ik: null == ik
          ? _self.ik
          : ik // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      assetBase: null == assetBase
          ? _self.assetBase
          : assetBase // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      finalized: null == finalized
          ? _self.finalized
          : finalized // ignore: cast_nullable_to_non_nullable
              as bool,
      firstSeenHeight: null == firstSeenHeight
          ? _self.firstSeenHeight
          : firstSeenHeight // ignore: cast_nullable_to_non_nullable
              as int,
      balance: null == balance
          ? _self.balance
          : balance // ignore: cast_nullable_to_non_nullable
              as BigInt,
    ));
  }
}

// dart format on
