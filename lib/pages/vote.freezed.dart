// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'vote.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;

/// @nodoc
mixin _$VoteContext {
  int get account;
  ElectionId get id;
  ElectionPropsPub? get election;
  Context get context;

  /// Create a copy of VoteContext
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $VoteContextCopyWith<VoteContext> get copyWith =>
      _$VoteContextCopyWithImpl<VoteContext>(this as VoteContext, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is VoteContext &&
            (identical(other.account, account) || other.account == account) &&
            (identical(other.id, id) || other.id == id) &&
            (identical(other.election, election) ||
                other.election == election) &&
            (identical(other.context, context) || other.context == context));
  }

  @override
  int get hashCode => Object.hash(runtimeType, account, id, election, context);

  @override
  String toString() {
    return 'VoteContext(account: $account, id: $id, election: $election, context: $context)';
  }
}

/// @nodoc
abstract mixin class $VoteContextCopyWith<$Res> {
  factory $VoteContextCopyWith(
          VoteContext value, $Res Function(VoteContext) _then) =
      _$VoteContextCopyWithImpl;
  @useResult
  $Res call(
      {int account,
      ElectionId id,
      ElectionPropsPub? election,
      Context context});

  $ElectionIdCopyWith<$Res> get id;
}

/// @nodoc
class _$VoteContextCopyWithImpl<$Res> implements $VoteContextCopyWith<$Res> {
  _$VoteContextCopyWithImpl(this._self, this._then);

  final VoteContext _self;
  final $Res Function(VoteContext) _then;

  /// Create a copy of VoteContext
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? account = null,
    Object? id = null,
    Object? election = freezed,
    Object? context = null,
  }) {
    return _then(_self.copyWith(
      account: null == account
          ? _self.account
          : account // ignore: cast_nullable_to_non_nullable
              as int,
      id: null == id
          ? _self.id
          : id // ignore: cast_nullable_to_non_nullable
              as ElectionId,
      election: freezed == election
          ? _self.election
          : election // ignore: cast_nullable_to_non_nullable
              as ElectionPropsPub?,
      context: null == context
          ? _self.context
          : context // ignore: cast_nullable_to_non_nullable
              as Context,
    ));
  }

  /// Create a copy of VoteContext
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $ElectionIdCopyWith<$Res> get id {
    return $ElectionIdCopyWith<$Res>(_self.id, (value) {
      return _then(_self.copyWith(id: value));
    });
  }
}

/// Adds pattern-matching-related methods to [VoteContext].
extension VoteContextPatterns on VoteContext {
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
    TResult Function(_VoteContext value)? $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _VoteContext() when $default != null:
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
    TResult Function(_VoteContext value) $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VoteContext():
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
    TResult? Function(_VoteContext value)? $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VoteContext() when $default != null:
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
    TResult Function(int account, ElectionId id, ElectionPropsPub? election,
            Context context)?
        $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _VoteContext() when $default != null:
        return $default(_that.account, _that.id, _that.election, _that.context);
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
    TResult Function(int account, ElectionId id, ElectionPropsPub? election,
            Context context)
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VoteContext():
        return $default(_that.account, _that.id, _that.election, _that.context);
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
    TResult? Function(int account, ElectionId id, ElectionPropsPub? election,
            Context context)?
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VoteContext() when $default != null:
        return $default(_that.account, _that.id, _that.election, _that.context);
      case _:
        return null;
    }
  }
}

/// @nodoc

class _VoteContext extends VoteContext {
  _VoteContext(
      {required this.account,
      required this.id,
      required this.election,
      required this.context})
      : super._();

  @override
  final int account;
  @override
  final ElectionId id;
  @override
  final ElectionPropsPub? election;
  @override
  final Context context;

  /// Create a copy of VoteContext
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  _$VoteContextCopyWith<_VoteContext> get copyWith =>
      __$VoteContextCopyWithImpl<_VoteContext>(this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _VoteContext &&
            (identical(other.account, account) || other.account == account) &&
            (identical(other.id, id) || other.id == id) &&
            (identical(other.election, election) ||
                other.election == election) &&
            (identical(other.context, context) || other.context == context));
  }

  @override
  int get hashCode => Object.hash(runtimeType, account, id, election, context);

  @override
  String toString() {
    return 'VoteContext(account: $account, id: $id, election: $election, context: $context)';
  }
}

/// @nodoc
abstract mixin class _$VoteContextCopyWith<$Res>
    implements $VoteContextCopyWith<$Res> {
  factory _$VoteContextCopyWith(
          _VoteContext value, $Res Function(_VoteContext) _then) =
      __$VoteContextCopyWithImpl;
  @override
  @useResult
  $Res call(
      {int account,
      ElectionId id,
      ElectionPropsPub? election,
      Context context});

  @override
  $ElectionIdCopyWith<$Res> get id;
}

/// @nodoc
class __$VoteContextCopyWithImpl<$Res> implements _$VoteContextCopyWith<$Res> {
  __$VoteContextCopyWithImpl(this._self, this._then);

  final _VoteContext _self;
  final $Res Function(_VoteContext) _then;

  /// Create a copy of VoteContext
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $Res call({
    Object? account = null,
    Object? id = null,
    Object? election = freezed,
    Object? context = null,
  }) {
    return _then(_VoteContext(
      account: null == account
          ? _self.account
          : account // ignore: cast_nullable_to_non_nullable
              as int,
      id: null == id
          ? _self.id
          : id // ignore: cast_nullable_to_non_nullable
              as ElectionId,
      election: freezed == election
          ? _self.election
          : election // ignore: cast_nullable_to_non_nullable
              as ElectionPropsPub?,
      context: null == context
          ? _self.context
          : context // ignore: cast_nullable_to_non_nullable
              as Context,
    ));
  }

  /// Create a copy of VoteContext
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $ElectionIdCopyWith<$Res> get id {
    return $ElectionIdCopyWith<$Res>(_self.id, (value) {
      return _then(_self.copyWith(id: value));
    });
  }
}

// dart format on
