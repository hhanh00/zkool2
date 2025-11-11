// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'init.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;

/// @nodoc
mixin _$LogMessage {
  int get level;
  String get message;
  String? get span;

  /// Create a copy of LogMessage
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $LogMessageCopyWith<LogMessage> get copyWith =>
      _$LogMessageCopyWithImpl<LogMessage>(this as LogMessage, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is LogMessage &&
            (identical(other.level, level) || other.level == level) &&
            (identical(other.message, message) || other.message == message) &&
            (identical(other.span, span) || other.span == span));
  }

  @override
  int get hashCode => Object.hash(runtimeType, level, message, span);

  @override
  String toString() {
    return 'LogMessage(level: $level, message: $message, span: $span)';
  }
}

/// @nodoc
abstract mixin class $LogMessageCopyWith<$Res> {
  factory $LogMessageCopyWith(
          LogMessage value, $Res Function(LogMessage) _then) =
      _$LogMessageCopyWithImpl;
  @useResult
  $Res call({int level, String message, String? span});
}

/// @nodoc
class _$LogMessageCopyWithImpl<$Res> implements $LogMessageCopyWith<$Res> {
  _$LogMessageCopyWithImpl(this._self, this._then);

  final LogMessage _self;
  final $Res Function(LogMessage) _then;

  /// Create a copy of LogMessage
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? level = null,
    Object? message = null,
    Object? span = freezed,
  }) {
    return _then(_self.copyWith(
      level: null == level
          ? _self.level
          : level // ignore: cast_nullable_to_non_nullable
              as int,
      message: null == message
          ? _self.message
          : message // ignore: cast_nullable_to_non_nullable
              as String,
      span: freezed == span
          ? _self.span
          : span // ignore: cast_nullable_to_non_nullable
              as String?,
    ));
  }
}

/// Adds pattern-matching-related methods to [LogMessage].
extension LogMessagePatterns on LogMessage {
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
    TResult Function(_LogMessage value)? $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _LogMessage() when $default != null:
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
    TResult Function(_LogMessage value) $default,
  ) {
    final _that = this;
    switch (_that) {
      case _LogMessage():
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
    TResult? Function(_LogMessage value)? $default,
  ) {
    final _that = this;
    switch (_that) {
      case _LogMessage() when $default != null:
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
    TResult Function(int level, String message, String? span)? $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _LogMessage() when $default != null:
        return $default(_that.level, _that.message, _that.span);
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
    TResult Function(int level, String message, String? span) $default,
  ) {
    final _that = this;
    switch (_that) {
      case _LogMessage():
        return $default(_that.level, _that.message, _that.span);
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
    TResult? Function(int level, String message, String? span)? $default,
  ) {
    final _that = this;
    switch (_that) {
      case _LogMessage() when $default != null:
        return $default(_that.level, _that.message, _that.span);
      case _:
        return null;
    }
  }
}

/// @nodoc

class _LogMessage implements LogMessage {
  const _LogMessage({required this.level, required this.message, this.span});

  @override
  final int level;
  @override
  final String message;
  @override
  final String? span;

  /// Create a copy of LogMessage
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  _$LogMessageCopyWith<_LogMessage> get copyWith =>
      __$LogMessageCopyWithImpl<_LogMessage>(this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _LogMessage &&
            (identical(other.level, level) || other.level == level) &&
            (identical(other.message, message) || other.message == message) &&
            (identical(other.span, span) || other.span == span));
  }

  @override
  int get hashCode => Object.hash(runtimeType, level, message, span);

  @override
  String toString() {
    return 'LogMessage(level: $level, message: $message, span: $span)';
  }
}

/// @nodoc
abstract mixin class _$LogMessageCopyWith<$Res>
    implements $LogMessageCopyWith<$Res> {
  factory _$LogMessageCopyWith(
          _LogMessage value, $Res Function(_LogMessage) _then) =
      __$LogMessageCopyWithImpl;
  @override
  @useResult
  $Res call({int level, String message, String? span});
}

/// @nodoc
class __$LogMessageCopyWithImpl<$Res> implements _$LogMessageCopyWith<$Res> {
  __$LogMessageCopyWithImpl(this._self, this._then);

  final _LogMessage _self;
  final $Res Function(_LogMessage) _then;

  /// Create a copy of LogMessage
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $Res call({
    Object? level = null,
    Object? message = null,
    Object? span = freezed,
  }) {
    return _then(_LogMessage(
      level: null == level
          ? _self.level
          : level // ignore: cast_nullable_to_non_nullable
              as int,
      message: null == message
          ? _self.message
          : message // ignore: cast_nullable_to_non_nullable
              as String,
      span: freezed == span
          ? _self.span
          : span // ignore: cast_nullable_to_non_nullable
              as String?,
    ));
  }
}

// dart format on
