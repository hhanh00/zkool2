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
    TResult Function(String field0, List<(int, String, PlatformInt64)> field1,
            int field2)?
        txId,
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case MempoolMsg_BlockHeight() when blockHeight != null:
        return blockHeight(_that.field0);
      case MempoolMsg_TxId() when txId != null:
        return txId(_that.field0, _that.field1, _that.field2);
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
    required TResult Function(String field0,
            List<(int, String, PlatformInt64)> field1, int field2)
        txId,
  }) {
    final _that = this;
    switch (_that) {
      case MempoolMsg_BlockHeight():
        return blockHeight(_that.field0);
      case MempoolMsg_TxId():
        return txId(_that.field0, _that.field1, _that.field2);
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
    TResult? Function(String field0, List<(int, String, PlatformInt64)> field1,
            int field2)?
        txId,
  }) {
    final _that = this;
    switch (_that) {
      case MempoolMsg_BlockHeight() when blockHeight != null:
        return blockHeight(_that.field0);
      case MempoolMsg_TxId() when txId != null:
        return txId(_that.field0, _that.field1, _that.field2);
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
  const MempoolMsg_TxId(
      this.field0, final List<(int, String, PlatformInt64)> field1, this.field2)
      : _field1 = field1,
        super._();

  @override
  final String field0;
  final List<(int, String, PlatformInt64)> _field1;
  List<(int, String, PlatformInt64)> get field1 {
    if (_field1 is EqualUnmodifiableListView) return _field1;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_field1);
  }

  final int field2;

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
            (identical(other.field0, field0) || other.field0 == field0) &&
            const DeepCollectionEquality().equals(other._field1, _field1) &&
            (identical(other.field2, field2) || other.field2 == field2));
  }

  @override
  int get hashCode => Object.hash(runtimeType, field0,
      const DeepCollectionEquality().hash(_field1), field2);

  @override
  String toString() {
    return 'MempoolMsg.txId(field0: $field0, field1: $field1, field2: $field2)';
  }
}

/// @nodoc
abstract mixin class $MempoolMsg_TxIdCopyWith<$Res>
    implements $MempoolMsgCopyWith<$Res> {
  factory $MempoolMsg_TxIdCopyWith(
          MempoolMsg_TxId value, $Res Function(MempoolMsg_TxId) _then) =
      _$MempoolMsg_TxIdCopyWithImpl;
  @useResult
  $Res call(
      {String field0, List<(int, String, PlatformInt64)> field1, int field2});
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
    Object? field1 = null,
    Object? field2 = null,
  }) {
    return _then(MempoolMsg_TxId(
      null == field0
          ? _self.field0
          : field0 // ignore: cast_nullable_to_non_nullable
              as String,
      null == field1
          ? _self._field1
          : field1 // ignore: cast_nullable_to_non_nullable
              as List<(int, String, PlatformInt64)>,
      null == field2
          ? _self.field2
          : field2 // ignore: cast_nullable_to_non_nullable
              as int,
    ));
  }
}

// dart format on
