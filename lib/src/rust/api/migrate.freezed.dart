// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'migrate.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;

/// @nodoc
mixin _$MigrationEvent {
  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType && other is MigrationEvent);
  }

  @override
  int get hashCode => runtimeType.hashCode;

  @override
  String toString() {
    return 'MigrationEvent()';
  }
}

/// @nodoc
class $MigrationEventCopyWith<$Res> {
  $MigrationEventCopyWith(MigrationEvent _, $Res Function(MigrationEvent) __);
}

/// Adds pattern-matching-related methods to [MigrationEvent].
extension MigrationEventPatterns on MigrationEvent {
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
    TResult Function(MigrationEvent_SplitComplete value)? splitComplete,
    TResult Function(MigrationEvent_MigrateComplete value)? migrateComplete,
    TResult Function(MigrationEvent_Complete value)? complete,
    TResult Function(MigrationEvent_NothingToDo value)? nothingToDo,
    TResult Function(MigrationEvent_Error value)? error,
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case MigrationEvent_SplitComplete() when splitComplete != null:
        return splitComplete(_that);
      case MigrationEvent_MigrateComplete() when migrateComplete != null:
        return migrateComplete(_that);
      case MigrationEvent_Complete() when complete != null:
        return complete(_that);
      case MigrationEvent_NothingToDo() when nothingToDo != null:
        return nothingToDo(_that);
      case MigrationEvent_Error() when error != null:
        return error(_that);
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
    required TResult Function(MigrationEvent_SplitComplete value) splitComplete,
    required TResult Function(MigrationEvent_MigrateComplete value)
        migrateComplete,
    required TResult Function(MigrationEvent_Complete value) complete,
    required TResult Function(MigrationEvent_NothingToDo value) nothingToDo,
    required TResult Function(MigrationEvent_Error value) error,
  }) {
    final _that = this;
    switch (_that) {
      case MigrationEvent_SplitComplete():
        return splitComplete(_that);
      case MigrationEvent_MigrateComplete():
        return migrateComplete(_that);
      case MigrationEvent_Complete():
        return complete(_that);
      case MigrationEvent_NothingToDo():
        return nothingToDo(_that);
      case MigrationEvent_Error():
        return error(_that);
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
    TResult? Function(MigrationEvent_SplitComplete value)? splitComplete,
    TResult? Function(MigrationEvent_MigrateComplete value)? migrateComplete,
    TResult? Function(MigrationEvent_Complete value)? complete,
    TResult? Function(MigrationEvent_NothingToDo value)? nothingToDo,
    TResult? Function(MigrationEvent_Error value)? error,
  }) {
    final _that = this;
    switch (_that) {
      case MigrationEvent_SplitComplete() when splitComplete != null:
        return splitComplete(_that);
      case MigrationEvent_MigrateComplete() when migrateComplete != null:
        return migrateComplete(_that);
      case MigrationEvent_Complete() when complete != null:
        return complete(_that);
      case MigrationEvent_NothingToDo() when nothingToDo != null:
        return nothingToDo(_that);
      case MigrationEvent_Error() when error != null:
        return error(_that);
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
    TResult Function(BigInt fee)? splitComplete,
    TResult Function(BigInt fee)? migrateComplete,
    TResult Function()? complete,
    TResult Function()? nothingToDo,
    TResult Function(String message)? error,
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case MigrationEvent_SplitComplete() when splitComplete != null:
        return splitComplete(_that.fee);
      case MigrationEvent_MigrateComplete() when migrateComplete != null:
        return migrateComplete(_that.fee);
      case MigrationEvent_Complete() when complete != null:
        return complete();
      case MigrationEvent_NothingToDo() when nothingToDo != null:
        return nothingToDo();
      case MigrationEvent_Error() when error != null:
        return error(_that.message);
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
    required TResult Function(BigInt fee) splitComplete,
    required TResult Function(BigInt fee) migrateComplete,
    required TResult Function() complete,
    required TResult Function() nothingToDo,
    required TResult Function(String message) error,
  }) {
    final _that = this;
    switch (_that) {
      case MigrationEvent_SplitComplete():
        return splitComplete(_that.fee);
      case MigrationEvent_MigrateComplete():
        return migrateComplete(_that.fee);
      case MigrationEvent_Complete():
        return complete();
      case MigrationEvent_NothingToDo():
        return nothingToDo();
      case MigrationEvent_Error():
        return error(_that.message);
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
    TResult? Function(BigInt fee)? splitComplete,
    TResult? Function(BigInt fee)? migrateComplete,
    TResult? Function()? complete,
    TResult? Function()? nothingToDo,
    TResult? Function(String message)? error,
  }) {
    final _that = this;
    switch (_that) {
      case MigrationEvent_SplitComplete() when splitComplete != null:
        return splitComplete(_that.fee);
      case MigrationEvent_MigrateComplete() when migrateComplete != null:
        return migrateComplete(_that.fee);
      case MigrationEvent_Complete() when complete != null:
        return complete();
      case MigrationEvent_NothingToDo() when nothingToDo != null:
        return nothingToDo();
      case MigrationEvent_Error() when error != null:
        return error(_that.message);
      case _:
        return null;
    }
  }
}

/// @nodoc

class MigrationEvent_SplitComplete extends MigrationEvent {
  const MigrationEvent_SplitComplete({required this.fee}) : super._();

  final BigInt fee;

  /// Create a copy of MigrationEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $MigrationEvent_SplitCompleteCopyWith<MigrationEvent_SplitComplete>
      get copyWith => _$MigrationEvent_SplitCompleteCopyWithImpl<
          MigrationEvent_SplitComplete>(this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is MigrationEvent_SplitComplete &&
            (identical(other.fee, fee) || other.fee == fee));
  }

  @override
  int get hashCode => Object.hash(runtimeType, fee);

  @override
  String toString() {
    return 'MigrationEvent.splitComplete(fee: $fee)';
  }
}

/// @nodoc
abstract mixin class $MigrationEvent_SplitCompleteCopyWith<$Res>
    implements $MigrationEventCopyWith<$Res> {
  factory $MigrationEvent_SplitCompleteCopyWith(
          MigrationEvent_SplitComplete value,
          $Res Function(MigrationEvent_SplitComplete) _then) =
      _$MigrationEvent_SplitCompleteCopyWithImpl;
  @useResult
  $Res call({BigInt fee});
}

/// @nodoc
class _$MigrationEvent_SplitCompleteCopyWithImpl<$Res>
    implements $MigrationEvent_SplitCompleteCopyWith<$Res> {
  _$MigrationEvent_SplitCompleteCopyWithImpl(this._self, this._then);

  final MigrationEvent_SplitComplete _self;
  final $Res Function(MigrationEvent_SplitComplete) _then;

  /// Create a copy of MigrationEvent
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  $Res call({
    Object? fee = null,
  }) {
    return _then(MigrationEvent_SplitComplete(
      fee: null == fee
          ? _self.fee
          : fee // ignore: cast_nullable_to_non_nullable
              as BigInt,
    ));
  }
}

/// @nodoc

class MigrationEvent_MigrateComplete extends MigrationEvent {
  const MigrationEvent_MigrateComplete({required this.fee}) : super._();

  final BigInt fee;

  /// Create a copy of MigrationEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $MigrationEvent_MigrateCompleteCopyWith<MigrationEvent_MigrateComplete>
      get copyWith => _$MigrationEvent_MigrateCompleteCopyWithImpl<
          MigrationEvent_MigrateComplete>(this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is MigrationEvent_MigrateComplete &&
            (identical(other.fee, fee) || other.fee == fee));
  }

  @override
  int get hashCode => Object.hash(runtimeType, fee);

  @override
  String toString() {
    return 'MigrationEvent.migrateComplete(fee: $fee)';
  }
}

/// @nodoc
abstract mixin class $MigrationEvent_MigrateCompleteCopyWith<$Res>
    implements $MigrationEventCopyWith<$Res> {
  factory $MigrationEvent_MigrateCompleteCopyWith(
          MigrationEvent_MigrateComplete value,
          $Res Function(MigrationEvent_MigrateComplete) _then) =
      _$MigrationEvent_MigrateCompleteCopyWithImpl;
  @useResult
  $Res call({BigInt fee});
}

/// @nodoc
class _$MigrationEvent_MigrateCompleteCopyWithImpl<$Res>
    implements $MigrationEvent_MigrateCompleteCopyWith<$Res> {
  _$MigrationEvent_MigrateCompleteCopyWithImpl(this._self, this._then);

  final MigrationEvent_MigrateComplete _self;
  final $Res Function(MigrationEvent_MigrateComplete) _then;

  /// Create a copy of MigrationEvent
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  $Res call({
    Object? fee = null,
  }) {
    return _then(MigrationEvent_MigrateComplete(
      fee: null == fee
          ? _self.fee
          : fee // ignore: cast_nullable_to_non_nullable
              as BigInt,
    ));
  }
}

/// @nodoc

class MigrationEvent_Complete extends MigrationEvent {
  const MigrationEvent_Complete() : super._();

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType && other is MigrationEvent_Complete);
  }

  @override
  int get hashCode => runtimeType.hashCode;

  @override
  String toString() {
    return 'MigrationEvent.complete()';
  }
}

/// @nodoc

class MigrationEvent_NothingToDo extends MigrationEvent {
  const MigrationEvent_NothingToDo() : super._();

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is MigrationEvent_NothingToDo);
  }

  @override
  int get hashCode => runtimeType.hashCode;

  @override
  String toString() {
    return 'MigrationEvent.nothingToDo()';
  }
}

/// @nodoc

class MigrationEvent_Error extends MigrationEvent {
  const MigrationEvent_Error({required this.message}) : super._();

  final String message;

  /// Create a copy of MigrationEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $MigrationEvent_ErrorCopyWith<MigrationEvent_Error> get copyWith =>
      _$MigrationEvent_ErrorCopyWithImpl<MigrationEvent_Error>(
          this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is MigrationEvent_Error &&
            (identical(other.message, message) || other.message == message));
  }

  @override
  int get hashCode => Object.hash(runtimeType, message);

  @override
  String toString() {
    return 'MigrationEvent.error(message: $message)';
  }
}

/// @nodoc
abstract mixin class $MigrationEvent_ErrorCopyWith<$Res>
    implements $MigrationEventCopyWith<$Res> {
  factory $MigrationEvent_ErrorCopyWith(MigrationEvent_Error value,
          $Res Function(MigrationEvent_Error) _then) =
      _$MigrationEvent_ErrorCopyWithImpl;
  @useResult
  $Res call({String message});
}

/// @nodoc
class _$MigrationEvent_ErrorCopyWithImpl<$Res>
    implements $MigrationEvent_ErrorCopyWith<$Res> {
  _$MigrationEvent_ErrorCopyWithImpl(this._self, this._then);

  final MigrationEvent_Error _self;
  final $Res Function(MigrationEvent_Error) _then;

  /// Create a copy of MigrationEvent
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  $Res call({
    Object? message = null,
  }) {
    return _then(MigrationEvent_Error(
      message: null == message
          ? _self.message
          : message // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

// dart format on
