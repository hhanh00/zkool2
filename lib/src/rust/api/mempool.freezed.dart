// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'mempool.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;

/// @nodoc
mixin _$MempoolMsg {
  Object get field0;

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is MempoolMsg &&
            const DeepCollectionEquality().equals(other.field0, field0));
  }

  @override
  int get hashCode =>
      Object.hash(runtimeType, const DeepCollectionEquality().hash(field0));

  @override
  String toString() {
    return 'MempoolMsg(field0: $field0)';
  }
}

/// @nodoc
class $MempoolMsgCopyWith<$Res> {
  $MempoolMsgCopyWith(MempoolMsg _, $Res Function(MempoolMsg) __);
}

/// Adds pattern-matching-related methods to [MempoolMsg].
extension MempoolMsgPatterns on MempoolMsg {
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
    TResult Function(MempoolMsg_BlockHeight value)? blockHeight,
    TResult Function(MempoolMsg_TxId value)? txId,
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case MempoolMsg_BlockHeight() when blockHeight != null:
        return blockHeight(_that);
      case MempoolMsg_TxId() when txId != null:
        return txId(_that);
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
    required TResult Function(MempoolMsg_BlockHeight value) blockHeight,
    required TResult Function(MempoolMsg_TxId value) txId,
  }) {
    final _that = this;
    switch (_that) {
      case MempoolMsg_BlockHeight():
        return blockHeight(_that);
      case MempoolMsg_TxId():
        return txId(_that);
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
    TResult? Function(MempoolMsg_BlockHeight value)? blockHeight,
    TResult? Function(MempoolMsg_TxId value)? txId,
  }) {
    final _that = this;
    switch (_that) {
      case MempoolMsg_BlockHeight() when blockHeight != null:
        return blockHeight(_that);
      case MempoolMsg_TxId() when txId != null:
        return txId(_that);
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
    TResult Function(int field0)? blockHeight,
    TResult Function(MempoolTx field0)? txId,
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case MempoolMsg_BlockHeight() when blockHeight != null:
        return blockHeight(_that.field0);
      case MempoolMsg_TxId() when txId != null:
        return txId(_that.field0);
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
    required TResult Function(int field0) blockHeight,
    required TResult Function(MempoolTx field0) txId,
  }) {
    final _that = this;
    switch (_that) {
      case MempoolMsg_BlockHeight():
        return blockHeight(_that.field0);
      case MempoolMsg_TxId():
        return txId(_that.field0);
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
    TResult? Function(int field0)? blockHeight,
    TResult? Function(MempoolTx field0)? txId,
  }) {
    final _that = this;
    switch (_that) {
      case MempoolMsg_BlockHeight() when blockHeight != null:
        return blockHeight(_that.field0);
      case MempoolMsg_TxId() when txId != null:
        return txId(_that.field0);
      case _:
        return null;
    }
  }
}

/// @nodoc

class MempoolMsg_BlockHeight extends MempoolMsg {
  const MempoolMsg_BlockHeight(this.field0) : super._();

  @override
  final int field0;

  /// Create a copy of MempoolMsg
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $MempoolMsg_BlockHeightCopyWith<MempoolMsg_BlockHeight> get copyWith =>
      _$MempoolMsg_BlockHeightCopyWithImpl<MempoolMsg_BlockHeight>(
          this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is MempoolMsg_BlockHeight &&
            (identical(other.field0, field0) || other.field0 == field0));
  }

  @override
  int get hashCode => Object.hash(runtimeType, field0);

  @override
  String toString() {
    return 'MempoolMsg.blockHeight(field0: $field0)';
  }
}

/// @nodoc
abstract mixin class $MempoolMsg_BlockHeightCopyWith<$Res>
    implements $MempoolMsgCopyWith<$Res> {
  factory $MempoolMsg_BlockHeightCopyWith(MempoolMsg_BlockHeight value,
          $Res Function(MempoolMsg_BlockHeight) _then) =
      _$MempoolMsg_BlockHeightCopyWithImpl;
  @useResult
  $Res call({int field0});
}

/// @nodoc
class _$MempoolMsg_BlockHeightCopyWithImpl<$Res>
    implements $MempoolMsg_BlockHeightCopyWith<$Res> {
  _$MempoolMsg_BlockHeightCopyWithImpl(this._self, this._then);

  final MempoolMsg_BlockHeight _self;
  final $Res Function(MempoolMsg_BlockHeight) _then;

  /// Create a copy of MempoolMsg
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  $Res call({
    Object? field0 = null,
  }) {
    return _then(MempoolMsg_BlockHeight(
      null == field0
          ? _self.field0
          : field0 // ignore: cast_nullable_to_non_nullable
              as int,
    ));
  }
}

/// @nodoc

class MempoolMsg_TxId extends MempoolMsg {
  const MempoolMsg_TxId(this.field0) : super._();

  @override
  final MempoolTx field0;

  /// Create a copy of MempoolMsg
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $MempoolMsg_TxIdCopyWith<MempoolMsg_TxId> get copyWith =>
      _$MempoolMsg_TxIdCopyWithImpl<MempoolMsg_TxId>(this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is MempoolMsg_TxId &&
            (identical(other.field0, field0) || other.field0 == field0));
  }

  @override
  int get hashCode => Object.hash(runtimeType, field0);

  @override
  String toString() {
    return 'MempoolMsg.txId(field0: $field0)';
  }
}

/// @nodoc
abstract mixin class $MempoolMsg_TxIdCopyWith<$Res>
    implements $MempoolMsgCopyWith<$Res> {
  factory $MempoolMsg_TxIdCopyWith(
          MempoolMsg_TxId value, $Res Function(MempoolMsg_TxId) _then) =
      _$MempoolMsg_TxIdCopyWithImpl;
  @useResult
  $Res call({MempoolTx field0});
}

/// @nodoc
class _$MempoolMsg_TxIdCopyWithImpl<$Res>
    implements $MempoolMsg_TxIdCopyWith<$Res> {
  _$MempoolMsg_TxIdCopyWithImpl(this._self, this._then);

  final MempoolMsg_TxId _self;
  final $Res Function(MempoolMsg_TxId) _then;

  /// Create a copy of MempoolMsg
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  $Res call({
    Object? field0 = null,
  }) {
    return _then(MempoolMsg_TxId(
      null == field0
          ? _self.field0
          : field0 // ignore: cast_nullable_to_non_nullable
              as MempoolTx,
    ));
  }
}

// dart format on
