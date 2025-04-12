// coverage:ignore-file
// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'init.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

T _$identity<T>(T value) => value;

final _privateConstructorUsedError = UnsupportedError(
    'It seems like you constructed your class using `MyClass._()`. This constructor is only meant to be used by freezed and you are not supposed to need it nor use it.\nPlease check the documentation here for more information: https://github.com/rrousselGit/freezed#adding-getters-and-methods-to-our-models');

/// @nodoc
mixin _$LogMessage {
  int get level => throw _privateConstructorUsedError;
  String get message => throw _privateConstructorUsedError;

  /// Create a copy of LogMessage
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $LogMessageCopyWith<LogMessage> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $LogMessageCopyWith<$Res> {
  factory $LogMessageCopyWith(
          LogMessage value, $Res Function(LogMessage) then) =
      _$LogMessageCopyWithImpl<$Res, LogMessage>;
  @useResult
  $Res call({int level, String message});
}

/// @nodoc
class _$LogMessageCopyWithImpl<$Res, $Val extends LogMessage>
    implements $LogMessageCopyWith<$Res> {
  _$LogMessageCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of LogMessage
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? level = null,
    Object? message = null,
  }) {
    return _then(_value.copyWith(
      level: null == level
          ? _value.level
          : level // ignore: cast_nullable_to_non_nullable
              as int,
      message: null == message
          ? _value.message
          : message // ignore: cast_nullable_to_non_nullable
              as String,
    ) as $Val);
  }
}

/// @nodoc
abstract class _$$LogMessageImplCopyWith<$Res>
    implements $LogMessageCopyWith<$Res> {
  factory _$$LogMessageImplCopyWith(
          _$LogMessageImpl value, $Res Function(_$LogMessageImpl) then) =
      __$$LogMessageImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call({int level, String message});
}

/// @nodoc
class __$$LogMessageImplCopyWithImpl<$Res>
    extends _$LogMessageCopyWithImpl<$Res, _$LogMessageImpl>
    implements _$$LogMessageImplCopyWith<$Res> {
  __$$LogMessageImplCopyWithImpl(
      _$LogMessageImpl _value, $Res Function(_$LogMessageImpl) _then)
      : super(_value, _then);

  /// Create a copy of LogMessage
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? level = null,
    Object? message = null,
  }) {
    return _then(_$LogMessageImpl(
      level: null == level
          ? _value.level
          : level // ignore: cast_nullable_to_non_nullable
              as int,
      message: null == message
          ? _value.message
          : message // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// @nodoc

class _$LogMessageImpl implements _LogMessage {
  const _$LogMessageImpl({required this.level, required this.message});

  @override
  final int level;
  @override
  final String message;

  @override
  String toString() {
    return 'LogMessage(level: $level, message: $message)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$LogMessageImpl &&
            (identical(other.level, level) || other.level == level) &&
            (identical(other.message, message) || other.message == message));
  }

  @override
  int get hashCode => Object.hash(runtimeType, level, message);

  /// Create a copy of LogMessage
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$LogMessageImplCopyWith<_$LogMessageImpl> get copyWith =>
      __$$LogMessageImplCopyWithImpl<_$LogMessageImpl>(this, _$identity);
}

abstract class _LogMessage implements LogMessage {
  const factory _LogMessage(
      {required final int level,
      required final String message}) = _$LogMessageImpl;

  @override
  int get level;
  @override
  String get message;

  /// Create a copy of LogMessage
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$LogMessageImplCopyWith<_$LogMessageImpl> get copyWith =>
      throw _privateConstructorUsedError;
}
