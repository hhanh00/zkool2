// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'voting_share_tracking.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;

/// @nodoc
mixin _$VotingPendingShareRound {
  String get roundId;
  String? get sessionJson;

  /// Create a copy of VotingPendingShareRound
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $VotingPendingShareRoundCopyWith<VotingPendingShareRound> get copyWith =>
      _$VotingPendingShareRoundCopyWithImpl<VotingPendingShareRound>(
          this as VotingPendingShareRound, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is VotingPendingShareRound &&
            (identical(other.roundId, roundId) || other.roundId == roundId) &&
            (identical(other.sessionJson, sessionJson) ||
                other.sessionJson == sessionJson));
  }

  @override
  int get hashCode => Object.hash(runtimeType, roundId, sessionJson);

  @override
  String toString() {
    return 'VotingPendingShareRound(roundId: $roundId, sessionJson: $sessionJson)';
  }
}

/// @nodoc
abstract mixin class $VotingPendingShareRoundCopyWith<$Res> {
  factory $VotingPendingShareRoundCopyWith(VotingPendingShareRound value,
          $Res Function(VotingPendingShareRound) _then) =
      _$VotingPendingShareRoundCopyWithImpl;
  @useResult
  $Res call({String roundId, String? sessionJson});
}

/// @nodoc
class _$VotingPendingShareRoundCopyWithImpl<$Res>
    implements $VotingPendingShareRoundCopyWith<$Res> {
  _$VotingPendingShareRoundCopyWithImpl(this._self, this._then);

  final VotingPendingShareRound _self;
  final $Res Function(VotingPendingShareRound) _then;

  /// Create a copy of VotingPendingShareRound
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? roundId = null,
    Object? sessionJson = freezed,
  }) {
    return _then(_self.copyWith(
      roundId: null == roundId
          ? _self.roundId
          : roundId // ignore: cast_nullable_to_non_nullable
              as String,
      sessionJson: freezed == sessionJson
          ? _self.sessionJson
          : sessionJson // ignore: cast_nullable_to_non_nullable
              as String?,
    ));
  }
}

/// Adds pattern-matching-related methods to [VotingPendingShareRound].
extension VotingPendingShareRoundPatterns on VotingPendingShareRound {
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
    TResult Function(_VotingPendingShareRound value)? $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _VotingPendingShareRound() when $default != null:
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
    TResult Function(_VotingPendingShareRound value) $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingPendingShareRound():
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
    TResult? Function(_VotingPendingShareRound value)? $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingPendingShareRound() when $default != null:
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
    TResult Function(String roundId, String? sessionJson)? $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _VotingPendingShareRound() when $default != null:
        return $default(_that.roundId, _that.sessionJson);
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
    TResult Function(String roundId, String? sessionJson) $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingPendingShareRound():
        return $default(_that.roundId, _that.sessionJson);
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
    TResult? Function(String roundId, String? sessionJson)? $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingPendingShareRound() when $default != null:
        return $default(_that.roundId, _that.sessionJson);
      case _:
        return null;
    }
  }
}

/// @nodoc

class _VotingPendingShareRound implements VotingPendingShareRound {
  const _VotingPendingShareRound({required this.roundId, this.sessionJson});

  @override
  final String roundId;
  @override
  final String? sessionJson;

  /// Create a copy of VotingPendingShareRound
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  _$VotingPendingShareRoundCopyWith<_VotingPendingShareRound> get copyWith =>
      __$VotingPendingShareRoundCopyWithImpl<_VotingPendingShareRound>(
          this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _VotingPendingShareRound &&
            (identical(other.roundId, roundId) || other.roundId == roundId) &&
            (identical(other.sessionJson, sessionJson) ||
                other.sessionJson == sessionJson));
  }

  @override
  int get hashCode => Object.hash(runtimeType, roundId, sessionJson);

  @override
  String toString() {
    return 'VotingPendingShareRound(roundId: $roundId, sessionJson: $sessionJson)';
  }
}

/// @nodoc
abstract mixin class _$VotingPendingShareRoundCopyWith<$Res>
    implements $VotingPendingShareRoundCopyWith<$Res> {
  factory _$VotingPendingShareRoundCopyWith(_VotingPendingShareRound value,
          $Res Function(_VotingPendingShareRound) _then) =
      __$VotingPendingShareRoundCopyWithImpl;
  @override
  @useResult
  $Res call({String roundId, String? sessionJson});
}

/// @nodoc
class __$VotingPendingShareRoundCopyWithImpl<$Res>
    implements _$VotingPendingShareRoundCopyWith<$Res> {
  __$VotingPendingShareRoundCopyWithImpl(this._self, this._then);

  final _VotingPendingShareRound _self;
  final $Res Function(_VotingPendingShareRound) _then;

  /// Create a copy of VotingPendingShareRound
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $Res call({
    Object? roundId = null,
    Object? sessionJson = freezed,
  }) {
    return _then(_VotingPendingShareRound(
      roundId: null == roundId
          ? _self.roundId
          : roundId // ignore: cast_nullable_to_non_nullable
              as String,
      sessionJson: freezed == sessionJson
          ? _self.sessionJson
          : sessionJson // ignore: cast_nullable_to_non_nullable
              as String?,
    ));
  }
}

// dart format on
