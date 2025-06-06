// coverage:ignore-file
// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'mempool.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

T _$identity<T>(T value) => value;

final _privateConstructorUsedError = UnsupportedError(
    'It seems like you constructed your class using `MyClass._()`. This constructor is only meant to be used by freezed and you are not supposed to need it nor use it.\nPlease check the documentation here for more information: https://github.com/rrousselGit/freezed#adding-getters-and-methods-to-our-models');

/// @nodoc
mixin _$MempoolMsg {
  String get field0 => throw _privateConstructorUsedError;
  List<(int, String, int)> get field1 => throw _privateConstructorUsedError;
  int get field2 => throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(
            String field0, List<(int, String, int)> field1, int field2)
        txId,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(
            String field0, List<(int, String, int)> field1, int field2)?
        txId,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(
            String field0, List<(int, String, int)> field1, int field2)?
        txId,
    required TResult orElse(),
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(MempoolMsg_TxId value) txId,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(MempoolMsg_TxId value)? txId,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(MempoolMsg_TxId value)? txId,
    required TResult orElse(),
  }) =>
      throw _privateConstructorUsedError;

  /// Create a copy of MempoolMsg
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $MempoolMsgCopyWith<MempoolMsg> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $MempoolMsgCopyWith<$Res> {
  factory $MempoolMsgCopyWith(
          MempoolMsg value, $Res Function(MempoolMsg) then) =
      _$MempoolMsgCopyWithImpl<$Res, MempoolMsg>;
  @useResult
  $Res call({String field0, List<(int, String, int)> field1, int field2});
}

/// @nodoc
class _$MempoolMsgCopyWithImpl<$Res, $Val extends MempoolMsg>
    implements $MempoolMsgCopyWith<$Res> {
  _$MempoolMsgCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of MempoolMsg
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? field0 = null,
    Object? field1 = null,
    Object? field2 = null,
  }) {
    return _then(_value.copyWith(
      field0: null == field0
          ? _value.field0
          : field0 // ignore: cast_nullable_to_non_nullable
              as String,
      field1: null == field1
          ? _value.field1
          : field1 // ignore: cast_nullable_to_non_nullable
              as List<(int, String, int)>,
      field2: null == field2
          ? _value.field2
          : field2 // ignore: cast_nullable_to_non_nullable
              as int,
    ) as $Val);
  }
}

/// @nodoc
abstract class _$$MempoolMsg_TxIdImplCopyWith<$Res>
    implements $MempoolMsgCopyWith<$Res> {
  factory _$$MempoolMsg_TxIdImplCopyWith(_$MempoolMsg_TxIdImpl value,
          $Res Function(_$MempoolMsg_TxIdImpl) then) =
      __$$MempoolMsg_TxIdImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call({String field0, List<(int, String, int)> field1, int field2});
}

/// @nodoc
class __$$MempoolMsg_TxIdImplCopyWithImpl<$Res>
    extends _$MempoolMsgCopyWithImpl<$Res, _$MempoolMsg_TxIdImpl>
    implements _$$MempoolMsg_TxIdImplCopyWith<$Res> {
  __$$MempoolMsg_TxIdImplCopyWithImpl(
      _$MempoolMsg_TxIdImpl _value, $Res Function(_$MempoolMsg_TxIdImpl) _then)
      : super(_value, _then);

  /// Create a copy of MempoolMsg
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? field0 = null,
    Object? field1 = null,
    Object? field2 = null,
  }) {
    return _then(_$MempoolMsg_TxIdImpl(
      null == field0
          ? _value.field0
          : field0 // ignore: cast_nullable_to_non_nullable
              as String,
      null == field1
          ? _value._field1
          : field1 // ignore: cast_nullable_to_non_nullable
              as List<(int, String, int)>,
      null == field2
          ? _value.field2
          : field2 // ignore: cast_nullable_to_non_nullable
              as int,
    ));
  }
}

/// @nodoc

class _$MempoolMsg_TxIdImpl extends MempoolMsg_TxId {
  const _$MempoolMsg_TxIdImpl(
      this.field0, final List<(int, String, int)> field1, this.field2)
      : _field1 = field1,
        super._();

  @override
  final String field0;
  final List<(int, String, int)> _field1;
  @override
  List<(int, String, int)> get field1 {
    if (_field1 is EqualUnmodifiableListView) return _field1;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_field1);
  }

  @override
  final int field2;

  @override
  String toString() {
    return 'MempoolMsg.txId(field0: $field0, field1: $field1, field2: $field2)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$MempoolMsg_TxIdImpl &&
            (identical(other.field0, field0) || other.field0 == field0) &&
            const DeepCollectionEquality().equals(other._field1, _field1) &&
            (identical(other.field2, field2) || other.field2 == field2));
  }

  @override
  int get hashCode => Object.hash(runtimeType, field0,
      const DeepCollectionEquality().hash(_field1), field2);

  /// Create a copy of MempoolMsg
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$MempoolMsg_TxIdImplCopyWith<_$MempoolMsg_TxIdImpl> get copyWith =>
      __$$MempoolMsg_TxIdImplCopyWithImpl<_$MempoolMsg_TxIdImpl>(
          this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(
            String field0, List<(int, String, int)> field1, int field2)
        txId,
  }) {
    return txId(field0, field1, field2);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(
            String field0, List<(int, String, int)> field1, int field2)?
        txId,
  }) {
    return txId?.call(field0, field1, field2);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(
            String field0, List<(int, String, int)> field1, int field2)?
        txId,
    required TResult orElse(),
  }) {
    if (txId != null) {
      return txId(field0, field1, field2);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(MempoolMsg_TxId value) txId,
  }) {
    return txId(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(MempoolMsg_TxId value)? txId,
  }) {
    return txId?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(MempoolMsg_TxId value)? txId,
    required TResult orElse(),
  }) {
    if (txId != null) {
      return txId(this);
    }
    return orElse();
  }
}

abstract class MempoolMsg_TxId extends MempoolMsg {
  const factory MempoolMsg_TxId(
      final String field0,
      final List<(int, String, int)> field1,
      final int field2) = _$MempoolMsg_TxIdImpl;
  const MempoolMsg_TxId._() : super._();

  @override
  String get field0;
  @override
  List<(int, String, int)> get field1;
  @override
  int get field2;

  /// Create a copy of MempoolMsg
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$MempoolMsg_TxIdImplCopyWith<_$MempoolMsg_TxIdImpl> get copyWith =>
      throw _privateConstructorUsedError;
}
