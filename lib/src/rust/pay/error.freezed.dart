// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'error.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;

/// @nodoc
mixin _$Error {
  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType && other is Error);
  }

  @override
  int get hashCode => runtimeType.hashCode;

  @override
  String toString() {
    return 'Error()';
  }
}

/// @nodoc
class $ErrorCopyWith<$Res> {
  $ErrorCopyWith(Error _, $Res Function(Error) __);
}

/// Adds pattern-matching-related methods to [Error].
extension ErrorPatterns on Error {
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
    TResult Function(Error_InvalidPoolMask value)? invalidPoolMask,
    TResult Function(Error_NotEnoughFunds value)? notEnoughFunds,
    TResult Function(Error_NoSigningKey value)? noSigningKey,
    TResult Function(Error_Sqlx value)? sqlx,
    TResult Function(Error_Other value)? other,
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case Error_InvalidPoolMask() when invalidPoolMask != null:
        return invalidPoolMask(_that);
      case Error_NotEnoughFunds() when notEnoughFunds != null:
        return notEnoughFunds(_that);
      case Error_NoSigningKey() when noSigningKey != null:
        return noSigningKey(_that);
      case Error_Sqlx() when sqlx != null:
        return sqlx(_that);
      case Error_Other() when other != null:
        return other(_that);
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
    required TResult Function(Error_InvalidPoolMask value) invalidPoolMask,
    required TResult Function(Error_NotEnoughFunds value) notEnoughFunds,
    required TResult Function(Error_NoSigningKey value) noSigningKey,
    required TResult Function(Error_Sqlx value) sqlx,
    required TResult Function(Error_Other value) other,
  }) {
    final _that = this;
    switch (_that) {
      case Error_InvalidPoolMask():
        return invalidPoolMask(_that);
      case Error_NotEnoughFunds():
        return notEnoughFunds(_that);
      case Error_NoSigningKey():
        return noSigningKey(_that);
      case Error_Sqlx():
        return sqlx(_that);
      case Error_Other():
        return other(_that);
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
    TResult? Function(Error_InvalidPoolMask value)? invalidPoolMask,
    TResult? Function(Error_NotEnoughFunds value)? notEnoughFunds,
    TResult? Function(Error_NoSigningKey value)? noSigningKey,
    TResult? Function(Error_Sqlx value)? sqlx,
    TResult? Function(Error_Other value)? other,
  }) {
    final _that = this;
    switch (_that) {
      case Error_InvalidPoolMask() when invalidPoolMask != null:
        return invalidPoolMask(_that);
      case Error_NotEnoughFunds() when notEnoughFunds != null:
        return notEnoughFunds(_that);
      case Error_NoSigningKey() when noSigningKey != null:
        return noSigningKey(_that);
      case Error_Sqlx() when sqlx != null:
        return sqlx(_that);
      case Error_Other() when other != null:
        return other(_that);
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
    TResult Function()? invalidPoolMask,
    TResult Function(String field0)? notEnoughFunds,
    TResult Function()? noSigningKey,
    TResult Function(Error field0)? sqlx,
    TResult Function(Error field0)? other,
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case Error_InvalidPoolMask() when invalidPoolMask != null:
        return invalidPoolMask();
      case Error_NotEnoughFunds() when notEnoughFunds != null:
        return notEnoughFunds(_that.field0);
      case Error_NoSigningKey() when noSigningKey != null:
        return noSigningKey();
      case Error_Sqlx() when sqlx != null:
        return sqlx(_that.field0);
      case Error_Other() when other != null:
        return other(_that.field0);
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
    required TResult Function() invalidPoolMask,
    required TResult Function(String field0) notEnoughFunds,
    required TResult Function() noSigningKey,
    required TResult Function(Error field0) sqlx,
    required TResult Function(Error field0) other,
  }) {
    final _that = this;
    switch (_that) {
      case Error_InvalidPoolMask():
        return invalidPoolMask();
      case Error_NotEnoughFunds():
        return notEnoughFunds(_that.field0);
      case Error_NoSigningKey():
        return noSigningKey();
      case Error_Sqlx():
        return sqlx(_that.field0);
      case Error_Other():
        return other(_that.field0);
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
    TResult? Function()? invalidPoolMask,
    TResult? Function(String field0)? notEnoughFunds,
    TResult? Function()? noSigningKey,
    TResult? Function(Error field0)? sqlx,
    TResult? Function(Error field0)? other,
  }) {
    final _that = this;
    switch (_that) {
      case Error_InvalidPoolMask() when invalidPoolMask != null:
        return invalidPoolMask();
      case Error_NotEnoughFunds() when notEnoughFunds != null:
        return notEnoughFunds(_that.field0);
      case Error_NoSigningKey() when noSigningKey != null:
        return noSigningKey();
      case Error_Sqlx() when sqlx != null:
        return sqlx(_that.field0);
      case Error_Other() when other != null:
        return other(_that.field0);
      case _:
        return null;
    }
  }
}

/// @nodoc

class Error_InvalidPoolMask extends Error {
  const Error_InvalidPoolMask() : super._();

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType && other is Error_InvalidPoolMask);
  }

  @override
  int get hashCode => runtimeType.hashCode;

  @override
  String toString() {
    return 'Error.invalidPoolMask()';
  }
}

/// @nodoc

class Error_NotEnoughFunds extends Error {
  const Error_NotEnoughFunds(this.field0) : super._();

  final String field0;

  /// Create a copy of Error
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $Error_NotEnoughFundsCopyWith<Error_NotEnoughFunds> get copyWith =>
      _$Error_NotEnoughFundsCopyWithImpl<Error_NotEnoughFunds>(
          this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is Error_NotEnoughFunds &&
            (identical(other.field0, field0) || other.field0 == field0));
  }

  @override
  int get hashCode => Object.hash(runtimeType, field0);

  @override
  String toString() {
    return 'Error.notEnoughFunds(field0: $field0)';
  }
}

/// @nodoc
abstract mixin class $Error_NotEnoughFundsCopyWith<$Res>
    implements $ErrorCopyWith<$Res> {
  factory $Error_NotEnoughFundsCopyWith(Error_NotEnoughFunds value,
          $Res Function(Error_NotEnoughFunds) _then) =
      _$Error_NotEnoughFundsCopyWithImpl;
  @useResult
  $Res call({String field0});
}

/// @nodoc
class _$Error_NotEnoughFundsCopyWithImpl<$Res>
    implements $Error_NotEnoughFundsCopyWith<$Res> {
  _$Error_NotEnoughFundsCopyWithImpl(this._self, this._then);

  final Error_NotEnoughFunds _self;
  final $Res Function(Error_NotEnoughFunds) _then;

  /// Create a copy of Error
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  $Res call({
    Object? field0 = null,
  }) {
    return _then(Error_NotEnoughFunds(
      null == field0
          ? _self.field0
          : field0 // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// @nodoc

class Error_NoSigningKey extends Error {
  const Error_NoSigningKey() : super._();

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType && other is Error_NoSigningKey);
  }

  @override
  int get hashCode => runtimeType.hashCode;

  @override
  String toString() {
    return 'Error.noSigningKey()';
  }
}

/// @nodoc

class Error_Sqlx extends Error {
  const Error_Sqlx(this.field0) : super._();

  final Error field0;

  /// Create a copy of Error
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $Error_SqlxCopyWith<Error_Sqlx> get copyWith =>
      _$Error_SqlxCopyWithImpl<Error_Sqlx>(this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is Error_Sqlx &&
            (identical(other.field0, field0) || other.field0 == field0));
  }

  @override
  int get hashCode => Object.hash(runtimeType, field0);

  @override
  String toString() {
    return 'Error.sqlx(field0: $field0)';
  }
}

/// @nodoc
abstract mixin class $Error_SqlxCopyWith<$Res> implements $ErrorCopyWith<$Res> {
  factory $Error_SqlxCopyWith(
          Error_Sqlx value, $Res Function(Error_Sqlx) _then) =
      _$Error_SqlxCopyWithImpl;
  @useResult
  $Res call({Error field0});

  $ErrorCopyWith<$Res> get field0;
}

/// @nodoc
class _$Error_SqlxCopyWithImpl<$Res> implements $Error_SqlxCopyWith<$Res> {
  _$Error_SqlxCopyWithImpl(this._self, this._then);

  final Error_Sqlx _self;
  final $Res Function(Error_Sqlx) _then;

  /// Create a copy of Error
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  $Res call({
    Object? field0 = null,
  }) {
    return _then(Error_Sqlx(
      null == field0
          ? _self.field0
          : field0 // ignore: cast_nullable_to_non_nullable
              as Error,
    ));
  }

  /// Create a copy of Error
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $ErrorCopyWith<$Res> get field0 {
    return $ErrorCopyWith<$Res>(_self.field0, (value) {
      return _then(_self.copyWith(field0: value));
    });
  }
}

/// @nodoc

class Error_Other extends Error {
  const Error_Other(this.field0) : super._();

  final Error field0;

  /// Create a copy of Error
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $Error_OtherCopyWith<Error_Other> get copyWith =>
      _$Error_OtherCopyWithImpl<Error_Other>(this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is Error_Other &&
            (identical(other.field0, field0) || other.field0 == field0));
  }

  @override
  int get hashCode => Object.hash(runtimeType, field0);

  @override
  String toString() {
    return 'Error.other(field0: $field0)';
  }
}

/// @nodoc
abstract mixin class $Error_OtherCopyWith<$Res>
    implements $ErrorCopyWith<$Res> {
  factory $Error_OtherCopyWith(
          Error_Other value, $Res Function(Error_Other) _then) =
      _$Error_OtherCopyWithImpl;
  @useResult
  $Res call({Error field0});

  $ErrorCopyWith<$Res> get field0;
}

/// @nodoc
class _$Error_OtherCopyWithImpl<$Res> implements $Error_OtherCopyWith<$Res> {
  _$Error_OtherCopyWithImpl(this._self, this._then);

  final Error_Other _self;
  final $Res Function(Error_Other) _then;

  /// Create a copy of Error
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  $Res call({
    Object? field0 = null,
  }) {
    return _then(Error_Other(
      null == field0
          ? _self.field0
          : field0 // ignore: cast_nullable_to_non_nullable
              as Error,
    ));
  }

  /// Create a copy of Error
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $ErrorCopyWith<$Res> get field0 {
    return $ErrorCopyWith<$Res>(_self.field0, (value) {
      return _then(_self.copyWith(field0: value));
    });
  }
}

// dart format on
