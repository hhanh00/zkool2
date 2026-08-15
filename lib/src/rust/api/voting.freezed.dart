// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'voting.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;

/// @nodoc
mixin _$VotingBallotIntent {
  int get proposalId;
  bool get skipped;
  int? get choice;

  /// Create a copy of VotingBallotIntent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $VotingBallotIntentCopyWith<VotingBallotIntent> get copyWith =>
      _$VotingBallotIntentCopyWithImpl<VotingBallotIntent>(
          this as VotingBallotIntent, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is VotingBallotIntent &&
            (identical(other.proposalId, proposalId) ||
                other.proposalId == proposalId) &&
            (identical(other.skipped, skipped) || other.skipped == skipped) &&
            (identical(other.choice, choice) || other.choice == choice));
  }

  @override
  int get hashCode => Object.hash(runtimeType, proposalId, skipped, choice);

  @override
  String toString() {
    return 'VotingBallotIntent(proposalId: $proposalId, skipped: $skipped, choice: $choice)';
  }
}

/// @nodoc
abstract mixin class $VotingBallotIntentCopyWith<$Res> {
  factory $VotingBallotIntentCopyWith(
          VotingBallotIntent value, $Res Function(VotingBallotIntent) _then) =
      _$VotingBallotIntentCopyWithImpl;
  @useResult
  $Res call({int proposalId, bool skipped, int? choice});
}

/// @nodoc
class _$VotingBallotIntentCopyWithImpl<$Res>
    implements $VotingBallotIntentCopyWith<$Res> {
  _$VotingBallotIntentCopyWithImpl(this._self, this._then);

  final VotingBallotIntent _self;
  final $Res Function(VotingBallotIntent) _then;

  /// Create a copy of VotingBallotIntent
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? proposalId = null,
    Object? skipped = null,
    Object? choice = freezed,
  }) {
    return _then(_self.copyWith(
      proposalId: null == proposalId
          ? _self.proposalId
          : proposalId // ignore: cast_nullable_to_non_nullable
              as int,
      skipped: null == skipped
          ? _self.skipped
          : skipped // ignore: cast_nullable_to_non_nullable
              as bool,
      choice: freezed == choice
          ? _self.choice
          : choice // ignore: cast_nullable_to_non_nullable
              as int?,
    ));
  }
}

/// Adds pattern-matching-related methods to [VotingBallotIntent].
extension VotingBallotIntentPatterns on VotingBallotIntent {
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
    TResult Function(_VotingBallotIntent value)? $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _VotingBallotIntent() when $default != null:
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
    TResult Function(_VotingBallotIntent value) $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingBallotIntent():
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
    TResult? Function(_VotingBallotIntent value)? $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingBallotIntent() when $default != null:
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
    TResult Function(int proposalId, bool skipped, int? choice)? $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _VotingBallotIntent() when $default != null:
        return $default(_that.proposalId, _that.skipped, _that.choice);
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
    TResult Function(int proposalId, bool skipped, int? choice) $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingBallotIntent():
        return $default(_that.proposalId, _that.skipped, _that.choice);
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
    TResult? Function(int proposalId, bool skipped, int? choice)? $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingBallotIntent() when $default != null:
        return $default(_that.proposalId, _that.skipped, _that.choice);
      case _:
        return null;
    }
  }
}

/// @nodoc

class _VotingBallotIntent implements VotingBallotIntent {
  const _VotingBallotIntent(
      {required this.proposalId, required this.skipped, this.choice});

  @override
  final int proposalId;
  @override
  final bool skipped;
  @override
  final int? choice;

  /// Create a copy of VotingBallotIntent
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  _$VotingBallotIntentCopyWith<_VotingBallotIntent> get copyWith =>
      __$VotingBallotIntentCopyWithImpl<_VotingBallotIntent>(this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _VotingBallotIntent &&
            (identical(other.proposalId, proposalId) ||
                other.proposalId == proposalId) &&
            (identical(other.skipped, skipped) || other.skipped == skipped) &&
            (identical(other.choice, choice) || other.choice == choice));
  }

  @override
  int get hashCode => Object.hash(runtimeType, proposalId, skipped, choice);

  @override
  String toString() {
    return 'VotingBallotIntent(proposalId: $proposalId, skipped: $skipped, choice: $choice)';
  }
}

/// @nodoc
abstract mixin class _$VotingBallotIntentCopyWith<$Res>
    implements $VotingBallotIntentCopyWith<$Res> {
  factory _$VotingBallotIntentCopyWith(
          _VotingBallotIntent value, $Res Function(_VotingBallotIntent) _then) =
      __$VotingBallotIntentCopyWithImpl;
  @override
  @useResult
  $Res call({int proposalId, bool skipped, int? choice});
}

/// @nodoc
class __$VotingBallotIntentCopyWithImpl<$Res>
    implements _$VotingBallotIntentCopyWith<$Res> {
  __$VotingBallotIntentCopyWithImpl(this._self, this._then);

  final _VotingBallotIntent _self;
  final $Res Function(_VotingBallotIntent) _then;

  /// Create a copy of VotingBallotIntent
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $Res call({
    Object? proposalId = null,
    Object? skipped = null,
    Object? choice = freezed,
  }) {
    return _then(_VotingBallotIntent(
      proposalId: null == proposalId
          ? _self.proposalId
          : proposalId // ignore: cast_nullable_to_non_nullable
              as int,
      skipped: null == skipped
          ? _self.skipped
          : skipped // ignore: cast_nullable_to_non_nullable
              as bool,
      choice: freezed == choice
          ? _self.choice
          : choice // ignore: cast_nullable_to_non_nullable
              as int?,
    ));
  }
}

/// @nodoc
mixin _$VotingChainResponse {
  int get statusCode;
  String get body;

  /// Create a copy of VotingChainResponse
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $VotingChainResponseCopyWith<VotingChainResponse> get copyWith =>
      _$VotingChainResponseCopyWithImpl<VotingChainResponse>(
          this as VotingChainResponse, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is VotingChainResponse &&
            (identical(other.statusCode, statusCode) ||
                other.statusCode == statusCode) &&
            (identical(other.body, body) || other.body == body));
  }

  @override
  int get hashCode => Object.hash(runtimeType, statusCode, body);

  @override
  String toString() {
    return 'VotingChainResponse(statusCode: $statusCode, body: $body)';
  }
}

/// @nodoc
abstract mixin class $VotingChainResponseCopyWith<$Res> {
  factory $VotingChainResponseCopyWith(
          VotingChainResponse value, $Res Function(VotingChainResponse) _then) =
      _$VotingChainResponseCopyWithImpl;
  @useResult
  $Res call({int statusCode, String body});
}

/// @nodoc
class _$VotingChainResponseCopyWithImpl<$Res>
    implements $VotingChainResponseCopyWith<$Res> {
  _$VotingChainResponseCopyWithImpl(this._self, this._then);

  final VotingChainResponse _self;
  final $Res Function(VotingChainResponse) _then;

  /// Create a copy of VotingChainResponse
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? statusCode = null,
    Object? body = null,
  }) {
    return _then(_self.copyWith(
      statusCode: null == statusCode
          ? _self.statusCode
          : statusCode // ignore: cast_nullable_to_non_nullable
              as int,
      body: null == body
          ? _self.body
          : body // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// Adds pattern-matching-related methods to [VotingChainResponse].
extension VotingChainResponsePatterns on VotingChainResponse {
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
    TResult Function(_VotingChainResponse value)? $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _VotingChainResponse() when $default != null:
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
    TResult Function(_VotingChainResponse value) $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingChainResponse():
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
    TResult? Function(_VotingChainResponse value)? $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingChainResponse() when $default != null:
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
    TResult Function(int statusCode, String body)? $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _VotingChainResponse() when $default != null:
        return $default(_that.statusCode, _that.body);
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
    TResult Function(int statusCode, String body) $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingChainResponse():
        return $default(_that.statusCode, _that.body);
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
    TResult? Function(int statusCode, String body)? $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingChainResponse() when $default != null:
        return $default(_that.statusCode, _that.body);
      case _:
        return null;
    }
  }
}

/// @nodoc

class _VotingChainResponse implements VotingChainResponse {
  const _VotingChainResponse({required this.statusCode, required this.body});

  @override
  final int statusCode;
  @override
  final String body;

  /// Create a copy of VotingChainResponse
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  _$VotingChainResponseCopyWith<_VotingChainResponse> get copyWith =>
      __$VotingChainResponseCopyWithImpl<_VotingChainResponse>(
          this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _VotingChainResponse &&
            (identical(other.statusCode, statusCode) ||
                other.statusCode == statusCode) &&
            (identical(other.body, body) || other.body == body));
  }

  @override
  int get hashCode => Object.hash(runtimeType, statusCode, body);

  @override
  String toString() {
    return 'VotingChainResponse(statusCode: $statusCode, body: $body)';
  }
}

/// @nodoc
abstract mixin class _$VotingChainResponseCopyWith<$Res>
    implements $VotingChainResponseCopyWith<$Res> {
  factory _$VotingChainResponseCopyWith(_VotingChainResponse value,
          $Res Function(_VotingChainResponse) _then) =
      __$VotingChainResponseCopyWithImpl;
  @override
  @useResult
  $Res call({int statusCode, String body});
}

/// @nodoc
class __$VotingChainResponseCopyWithImpl<$Res>
    implements _$VotingChainResponseCopyWith<$Res> {
  __$VotingChainResponseCopyWithImpl(this._self, this._then);

  final _VotingChainResponse _self;
  final $Res Function(_VotingChainResponse) _then;

  /// Create a copy of VotingChainResponse
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $Res call({
    Object? statusCode = null,
    Object? body = null,
  }) {
    return _then(_VotingChainResponse(
      statusCode: null == statusCode
          ? _self.statusCode
          : statusCode // ignore: cast_nullable_to_non_nullable
              as int,
      body: null == body
          ? _self.body
          : body // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// @nodoc
mixin _$VotingCompletedVoteChoice {
  int get proposalId;
  int? get choice;

  /// Create a copy of VotingCompletedVoteChoice
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $VotingCompletedVoteChoiceCopyWith<VotingCompletedVoteChoice> get copyWith =>
      _$VotingCompletedVoteChoiceCopyWithImpl<VotingCompletedVoteChoice>(
          this as VotingCompletedVoteChoice, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is VotingCompletedVoteChoice &&
            (identical(other.proposalId, proposalId) ||
                other.proposalId == proposalId) &&
            (identical(other.choice, choice) || other.choice == choice));
  }

  @override
  int get hashCode => Object.hash(runtimeType, proposalId, choice);

  @override
  String toString() {
    return 'VotingCompletedVoteChoice(proposalId: $proposalId, choice: $choice)';
  }
}

/// @nodoc
abstract mixin class $VotingCompletedVoteChoiceCopyWith<$Res> {
  factory $VotingCompletedVoteChoiceCopyWith(VotingCompletedVoteChoice value,
          $Res Function(VotingCompletedVoteChoice) _then) =
      _$VotingCompletedVoteChoiceCopyWithImpl;
  @useResult
  $Res call({int proposalId, int? choice});
}

/// @nodoc
class _$VotingCompletedVoteChoiceCopyWithImpl<$Res>
    implements $VotingCompletedVoteChoiceCopyWith<$Res> {
  _$VotingCompletedVoteChoiceCopyWithImpl(this._self, this._then);

  final VotingCompletedVoteChoice _self;
  final $Res Function(VotingCompletedVoteChoice) _then;

  /// Create a copy of VotingCompletedVoteChoice
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? proposalId = null,
    Object? choice = freezed,
  }) {
    return _then(_self.copyWith(
      proposalId: null == proposalId
          ? _self.proposalId
          : proposalId // ignore: cast_nullable_to_non_nullable
              as int,
      choice: freezed == choice
          ? _self.choice
          : choice // ignore: cast_nullable_to_non_nullable
              as int?,
    ));
  }
}

/// Adds pattern-matching-related methods to [VotingCompletedVoteChoice].
extension VotingCompletedVoteChoicePatterns on VotingCompletedVoteChoice {
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
    TResult Function(_VotingCompletedVoteChoice value)? $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _VotingCompletedVoteChoice() when $default != null:
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
    TResult Function(_VotingCompletedVoteChoice value) $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingCompletedVoteChoice():
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
    TResult? Function(_VotingCompletedVoteChoice value)? $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingCompletedVoteChoice() when $default != null:
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
    TResult Function(int proposalId, int? choice)? $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _VotingCompletedVoteChoice() when $default != null:
        return $default(_that.proposalId, _that.choice);
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
    TResult Function(int proposalId, int? choice) $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingCompletedVoteChoice():
        return $default(_that.proposalId, _that.choice);
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
    TResult? Function(int proposalId, int? choice)? $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingCompletedVoteChoice() when $default != null:
        return $default(_that.proposalId, _that.choice);
      case _:
        return null;
    }
  }
}

/// @nodoc

class _VotingCompletedVoteChoice implements VotingCompletedVoteChoice {
  const _VotingCompletedVoteChoice({required this.proposalId, this.choice});

  @override
  final int proposalId;
  @override
  final int? choice;

  /// Create a copy of VotingCompletedVoteChoice
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  _$VotingCompletedVoteChoiceCopyWith<_VotingCompletedVoteChoice>
      get copyWith =>
          __$VotingCompletedVoteChoiceCopyWithImpl<_VotingCompletedVoteChoice>(
              this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _VotingCompletedVoteChoice &&
            (identical(other.proposalId, proposalId) ||
                other.proposalId == proposalId) &&
            (identical(other.choice, choice) || other.choice == choice));
  }

  @override
  int get hashCode => Object.hash(runtimeType, proposalId, choice);

  @override
  String toString() {
    return 'VotingCompletedVoteChoice(proposalId: $proposalId, choice: $choice)';
  }
}

/// @nodoc
abstract mixin class _$VotingCompletedVoteChoiceCopyWith<$Res>
    implements $VotingCompletedVoteChoiceCopyWith<$Res> {
  factory _$VotingCompletedVoteChoiceCopyWith(_VotingCompletedVoteChoice value,
          $Res Function(_VotingCompletedVoteChoice) _then) =
      __$VotingCompletedVoteChoiceCopyWithImpl;
  @override
  @useResult
  $Res call({int proposalId, int? choice});
}

/// @nodoc
class __$VotingCompletedVoteChoiceCopyWithImpl<$Res>
    implements _$VotingCompletedVoteChoiceCopyWith<$Res> {
  __$VotingCompletedVoteChoiceCopyWithImpl(this._self, this._then);

  final _VotingCompletedVoteChoice _self;
  final $Res Function(_VotingCompletedVoteChoice) _then;

  /// Create a copy of VotingCompletedVoteChoice
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $Res call({
    Object? proposalId = null,
    Object? choice = freezed,
  }) {
    return _then(_VotingCompletedVoteChoice(
      proposalId: null == proposalId
          ? _self.proposalId
          : proposalId // ignore: cast_nullable_to_non_nullable
              as int,
      choice: freezed == choice
          ? _self.choice
          : choice // ignore: cast_nullable_to_non_nullable
              as int?,
    ));
  }
}

/// @nodoc
mixin _$VotingCompletedVoteDisplay {
  List<VotingCompletedVoteChoice> get choices;
  BigInt? get votedAt;

  /// Create a copy of VotingCompletedVoteDisplay
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $VotingCompletedVoteDisplayCopyWith<VotingCompletedVoteDisplay>
      get copyWith =>
          _$VotingCompletedVoteDisplayCopyWithImpl<VotingCompletedVoteDisplay>(
              this as VotingCompletedVoteDisplay, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is VotingCompletedVoteDisplay &&
            const DeepCollectionEquality().equals(other.choices, choices) &&
            (identical(other.votedAt, votedAt) || other.votedAt == votedAt));
  }

  @override
  int get hashCode => Object.hash(
      runtimeType, const DeepCollectionEquality().hash(choices), votedAt);

  @override
  String toString() {
    return 'VotingCompletedVoteDisplay(choices: $choices, votedAt: $votedAt)';
  }
}

/// @nodoc
abstract mixin class $VotingCompletedVoteDisplayCopyWith<$Res> {
  factory $VotingCompletedVoteDisplayCopyWith(VotingCompletedVoteDisplay value,
          $Res Function(VotingCompletedVoteDisplay) _then) =
      _$VotingCompletedVoteDisplayCopyWithImpl;
  @useResult
  $Res call({List<VotingCompletedVoteChoice> choices, BigInt? votedAt});
}

/// @nodoc
class _$VotingCompletedVoteDisplayCopyWithImpl<$Res>
    implements $VotingCompletedVoteDisplayCopyWith<$Res> {
  _$VotingCompletedVoteDisplayCopyWithImpl(this._self, this._then);

  final VotingCompletedVoteDisplay _self;
  final $Res Function(VotingCompletedVoteDisplay) _then;

  /// Create a copy of VotingCompletedVoteDisplay
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? choices = null,
    Object? votedAt = freezed,
  }) {
    return _then(_self.copyWith(
      choices: null == choices
          ? _self.choices
          : choices // ignore: cast_nullable_to_non_nullable
              as List<VotingCompletedVoteChoice>,
      votedAt: freezed == votedAt
          ? _self.votedAt
          : votedAt // ignore: cast_nullable_to_non_nullable
              as BigInt?,
    ));
  }
}

/// Adds pattern-matching-related methods to [VotingCompletedVoteDisplay].
extension VotingCompletedVoteDisplayPatterns on VotingCompletedVoteDisplay {
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
    TResult Function(_VotingCompletedVoteDisplay value)? $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _VotingCompletedVoteDisplay() when $default != null:
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
    TResult Function(_VotingCompletedVoteDisplay value) $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingCompletedVoteDisplay():
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
    TResult? Function(_VotingCompletedVoteDisplay value)? $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingCompletedVoteDisplay() when $default != null:
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
    TResult Function(List<VotingCompletedVoteChoice> choices, BigInt? votedAt)?
        $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _VotingCompletedVoteDisplay() when $default != null:
        return $default(_that.choices, _that.votedAt);
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
    TResult Function(List<VotingCompletedVoteChoice> choices, BigInt? votedAt)
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingCompletedVoteDisplay():
        return $default(_that.choices, _that.votedAt);
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
    TResult? Function(List<VotingCompletedVoteChoice> choices, BigInt? votedAt)?
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingCompletedVoteDisplay() when $default != null:
        return $default(_that.choices, _that.votedAt);
      case _:
        return null;
    }
  }
}

/// @nodoc

class _VotingCompletedVoteDisplay implements VotingCompletedVoteDisplay {
  const _VotingCompletedVoteDisplay(
      {required final List<VotingCompletedVoteChoice> choices, this.votedAt})
      : _choices = choices;

  final List<VotingCompletedVoteChoice> _choices;
  @override
  List<VotingCompletedVoteChoice> get choices {
    if (_choices is EqualUnmodifiableListView) return _choices;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_choices);
  }

  @override
  final BigInt? votedAt;

  /// Create a copy of VotingCompletedVoteDisplay
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  _$VotingCompletedVoteDisplayCopyWith<_VotingCompletedVoteDisplay>
      get copyWith => __$VotingCompletedVoteDisplayCopyWithImpl<
          _VotingCompletedVoteDisplay>(this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _VotingCompletedVoteDisplay &&
            const DeepCollectionEquality().equals(other._choices, _choices) &&
            (identical(other.votedAt, votedAt) || other.votedAt == votedAt));
  }

  @override
  int get hashCode => Object.hash(
      runtimeType, const DeepCollectionEquality().hash(_choices), votedAt);

  @override
  String toString() {
    return 'VotingCompletedVoteDisplay(choices: $choices, votedAt: $votedAt)';
  }
}

/// @nodoc
abstract mixin class _$VotingCompletedVoteDisplayCopyWith<$Res>
    implements $VotingCompletedVoteDisplayCopyWith<$Res> {
  factory _$VotingCompletedVoteDisplayCopyWith(
          _VotingCompletedVoteDisplay value,
          $Res Function(_VotingCompletedVoteDisplay) _then) =
      __$VotingCompletedVoteDisplayCopyWithImpl;
  @override
  @useResult
  $Res call({List<VotingCompletedVoteChoice> choices, BigInt? votedAt});
}

/// @nodoc
class __$VotingCompletedVoteDisplayCopyWithImpl<$Res>
    implements _$VotingCompletedVoteDisplayCopyWith<$Res> {
  __$VotingCompletedVoteDisplayCopyWithImpl(this._self, this._then);

  final _VotingCompletedVoteDisplay _self;
  final $Res Function(_VotingCompletedVoteDisplay) _then;

  /// Create a copy of VotingCompletedVoteDisplay
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $Res call({
    Object? choices = null,
    Object? votedAt = freezed,
  }) {
    return _then(_VotingCompletedVoteDisplay(
      choices: null == choices
          ? _self._choices
          : choices // ignore: cast_nullable_to_non_nullable
              as List<VotingCompletedVoteChoice>,
      votedAt: freezed == votedAt
          ? _self.votedAt
          : votedAt // ignore: cast_nullable_to_non_nullable
              as BigInt?,
    ));
  }
}

/// @nodoc
mixin _$VotingConfig {
  String get source;
  String get sourceFingerprint;
  String get trustedKeyFingerprint;
  String get switchKind;
  List<VotingServiceEndpoint> get voteServers;
  List<VotingServiceEndpoint> get pirServers;
  VotingPirLayout? get pirLayout;
  List<VotingConfigRound> get rounds;

  /// Create a copy of VotingConfig
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $VotingConfigCopyWith<VotingConfig> get copyWith =>
      _$VotingConfigCopyWithImpl<VotingConfig>(
          this as VotingConfig, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is VotingConfig &&
            (identical(other.source, source) || other.source == source) &&
            (identical(other.sourceFingerprint, sourceFingerprint) ||
                other.sourceFingerprint == sourceFingerprint) &&
            (identical(other.trustedKeyFingerprint, trustedKeyFingerprint) ||
                other.trustedKeyFingerprint == trustedKeyFingerprint) &&
            (identical(other.switchKind, switchKind) ||
                other.switchKind == switchKind) &&
            const DeepCollectionEquality()
                .equals(other.voteServers, voteServers) &&
            const DeepCollectionEquality()
                .equals(other.pirServers, pirServers) &&
            (identical(other.pirLayout, pirLayout) ||
                other.pirLayout == pirLayout) &&
            const DeepCollectionEquality().equals(other.rounds, rounds));
  }

  @override
  int get hashCode => Object.hash(
      runtimeType,
      source,
      sourceFingerprint,
      trustedKeyFingerprint,
      switchKind,
      const DeepCollectionEquality().hash(voteServers),
      const DeepCollectionEquality().hash(pirServers),
      pirLayout,
      const DeepCollectionEquality().hash(rounds));

  @override
  String toString() {
    return 'VotingConfig(source: $source, sourceFingerprint: $sourceFingerprint, trustedKeyFingerprint: $trustedKeyFingerprint, switchKind: $switchKind, voteServers: $voteServers, pirServers: $pirServers, pirLayout: $pirLayout, rounds: $rounds)';
  }
}

/// @nodoc
abstract mixin class $VotingConfigCopyWith<$Res> {
  factory $VotingConfigCopyWith(
          VotingConfig value, $Res Function(VotingConfig) _then) =
      _$VotingConfigCopyWithImpl;
  @useResult
  $Res call(
      {String source,
      String sourceFingerprint,
      String trustedKeyFingerprint,
      String switchKind,
      List<VotingServiceEndpoint> voteServers,
      List<VotingServiceEndpoint> pirServers,
      VotingPirLayout? pirLayout,
      List<VotingConfigRound> rounds});

  $VotingPirLayoutCopyWith<$Res>? get pirLayout;
}

/// @nodoc
class _$VotingConfigCopyWithImpl<$Res> implements $VotingConfigCopyWith<$Res> {
  _$VotingConfigCopyWithImpl(this._self, this._then);

  final VotingConfig _self;
  final $Res Function(VotingConfig) _then;

  /// Create a copy of VotingConfig
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? source = null,
    Object? sourceFingerprint = null,
    Object? trustedKeyFingerprint = null,
    Object? switchKind = null,
    Object? voteServers = null,
    Object? pirServers = null,
    Object? pirLayout = freezed,
    Object? rounds = null,
  }) {
    return _then(_self.copyWith(
      source: null == source
          ? _self.source
          : source // ignore: cast_nullable_to_non_nullable
              as String,
      sourceFingerprint: null == sourceFingerprint
          ? _self.sourceFingerprint
          : sourceFingerprint // ignore: cast_nullable_to_non_nullable
              as String,
      trustedKeyFingerprint: null == trustedKeyFingerprint
          ? _self.trustedKeyFingerprint
          : trustedKeyFingerprint // ignore: cast_nullable_to_non_nullable
              as String,
      switchKind: null == switchKind
          ? _self.switchKind
          : switchKind // ignore: cast_nullable_to_non_nullable
              as String,
      voteServers: null == voteServers
          ? _self.voteServers
          : voteServers // ignore: cast_nullable_to_non_nullable
              as List<VotingServiceEndpoint>,
      pirServers: null == pirServers
          ? _self.pirServers
          : pirServers // ignore: cast_nullable_to_non_nullable
              as List<VotingServiceEndpoint>,
      pirLayout: freezed == pirLayout
          ? _self.pirLayout
          : pirLayout // ignore: cast_nullable_to_non_nullable
              as VotingPirLayout?,
      rounds: null == rounds
          ? _self.rounds
          : rounds // ignore: cast_nullable_to_non_nullable
              as List<VotingConfigRound>,
    ));
  }

  /// Create a copy of VotingConfig
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $VotingPirLayoutCopyWith<$Res>? get pirLayout {
    if (_self.pirLayout == null) {
      return null;
    }

    return $VotingPirLayoutCopyWith<$Res>(_self.pirLayout!, (value) {
      return _then(_self.copyWith(pirLayout: value));
    });
  }
}

/// Adds pattern-matching-related methods to [VotingConfig].
extension VotingConfigPatterns on VotingConfig {
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
    TResult Function(_VotingConfig value)? $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _VotingConfig() when $default != null:
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
    TResult Function(_VotingConfig value) $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingConfig():
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
    TResult? Function(_VotingConfig value)? $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingConfig() when $default != null:
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
    TResult Function(
            String source,
            String sourceFingerprint,
            String trustedKeyFingerprint,
            String switchKind,
            List<VotingServiceEndpoint> voteServers,
            List<VotingServiceEndpoint> pirServers,
            VotingPirLayout? pirLayout,
            List<VotingConfigRound> rounds)?
        $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _VotingConfig() when $default != null:
        return $default(
            _that.source,
            _that.sourceFingerprint,
            _that.trustedKeyFingerprint,
            _that.switchKind,
            _that.voteServers,
            _that.pirServers,
            _that.pirLayout,
            _that.rounds);
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
    TResult Function(
            String source,
            String sourceFingerprint,
            String trustedKeyFingerprint,
            String switchKind,
            List<VotingServiceEndpoint> voteServers,
            List<VotingServiceEndpoint> pirServers,
            VotingPirLayout? pirLayout,
            List<VotingConfigRound> rounds)
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingConfig():
        return $default(
            _that.source,
            _that.sourceFingerprint,
            _that.trustedKeyFingerprint,
            _that.switchKind,
            _that.voteServers,
            _that.pirServers,
            _that.pirLayout,
            _that.rounds);
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
    TResult? Function(
            String source,
            String sourceFingerprint,
            String trustedKeyFingerprint,
            String switchKind,
            List<VotingServiceEndpoint> voteServers,
            List<VotingServiceEndpoint> pirServers,
            VotingPirLayout? pirLayout,
            List<VotingConfigRound> rounds)?
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingConfig() when $default != null:
        return $default(
            _that.source,
            _that.sourceFingerprint,
            _that.trustedKeyFingerprint,
            _that.switchKind,
            _that.voteServers,
            _that.pirServers,
            _that.pirLayout,
            _that.rounds);
      case _:
        return null;
    }
  }
}

/// @nodoc

class _VotingConfig implements VotingConfig {
  const _VotingConfig(
      {required this.source,
      required this.sourceFingerprint,
      required this.trustedKeyFingerprint,
      required this.switchKind,
      required final List<VotingServiceEndpoint> voteServers,
      required final List<VotingServiceEndpoint> pirServers,
      this.pirLayout,
      required final List<VotingConfigRound> rounds})
      : _voteServers = voteServers,
        _pirServers = pirServers,
        _rounds = rounds;

  @override
  final String source;
  @override
  final String sourceFingerprint;
  @override
  final String trustedKeyFingerprint;
  @override
  final String switchKind;
  final List<VotingServiceEndpoint> _voteServers;
  @override
  List<VotingServiceEndpoint> get voteServers {
    if (_voteServers is EqualUnmodifiableListView) return _voteServers;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_voteServers);
  }

  final List<VotingServiceEndpoint> _pirServers;
  @override
  List<VotingServiceEndpoint> get pirServers {
    if (_pirServers is EqualUnmodifiableListView) return _pirServers;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_pirServers);
  }

  @override
  final VotingPirLayout? pirLayout;
  final List<VotingConfigRound> _rounds;
  @override
  List<VotingConfigRound> get rounds {
    if (_rounds is EqualUnmodifiableListView) return _rounds;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_rounds);
  }

  /// Create a copy of VotingConfig
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  _$VotingConfigCopyWith<_VotingConfig> get copyWith =>
      __$VotingConfigCopyWithImpl<_VotingConfig>(this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _VotingConfig &&
            (identical(other.source, source) || other.source == source) &&
            (identical(other.sourceFingerprint, sourceFingerprint) ||
                other.sourceFingerprint == sourceFingerprint) &&
            (identical(other.trustedKeyFingerprint, trustedKeyFingerprint) ||
                other.trustedKeyFingerprint == trustedKeyFingerprint) &&
            (identical(other.switchKind, switchKind) ||
                other.switchKind == switchKind) &&
            const DeepCollectionEquality()
                .equals(other._voteServers, _voteServers) &&
            const DeepCollectionEquality()
                .equals(other._pirServers, _pirServers) &&
            (identical(other.pirLayout, pirLayout) ||
                other.pirLayout == pirLayout) &&
            const DeepCollectionEquality().equals(other._rounds, _rounds));
  }

  @override
  int get hashCode => Object.hash(
      runtimeType,
      source,
      sourceFingerprint,
      trustedKeyFingerprint,
      switchKind,
      const DeepCollectionEquality().hash(_voteServers),
      const DeepCollectionEquality().hash(_pirServers),
      pirLayout,
      const DeepCollectionEquality().hash(_rounds));

  @override
  String toString() {
    return 'VotingConfig(source: $source, sourceFingerprint: $sourceFingerprint, trustedKeyFingerprint: $trustedKeyFingerprint, switchKind: $switchKind, voteServers: $voteServers, pirServers: $pirServers, pirLayout: $pirLayout, rounds: $rounds)';
  }
}

/// @nodoc
abstract mixin class _$VotingConfigCopyWith<$Res>
    implements $VotingConfigCopyWith<$Res> {
  factory _$VotingConfigCopyWith(
          _VotingConfig value, $Res Function(_VotingConfig) _then) =
      __$VotingConfigCopyWithImpl;
  @override
  @useResult
  $Res call(
      {String source,
      String sourceFingerprint,
      String trustedKeyFingerprint,
      String switchKind,
      List<VotingServiceEndpoint> voteServers,
      List<VotingServiceEndpoint> pirServers,
      VotingPirLayout? pirLayout,
      List<VotingConfigRound> rounds});

  @override
  $VotingPirLayoutCopyWith<$Res>? get pirLayout;
}

/// @nodoc
class __$VotingConfigCopyWithImpl<$Res>
    implements _$VotingConfigCopyWith<$Res> {
  __$VotingConfigCopyWithImpl(this._self, this._then);

  final _VotingConfig _self;
  final $Res Function(_VotingConfig) _then;

  /// Create a copy of VotingConfig
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $Res call({
    Object? source = null,
    Object? sourceFingerprint = null,
    Object? trustedKeyFingerprint = null,
    Object? switchKind = null,
    Object? voteServers = null,
    Object? pirServers = null,
    Object? pirLayout = freezed,
    Object? rounds = null,
  }) {
    return _then(_VotingConfig(
      source: null == source
          ? _self.source
          : source // ignore: cast_nullable_to_non_nullable
              as String,
      sourceFingerprint: null == sourceFingerprint
          ? _self.sourceFingerprint
          : sourceFingerprint // ignore: cast_nullable_to_non_nullable
              as String,
      trustedKeyFingerprint: null == trustedKeyFingerprint
          ? _self.trustedKeyFingerprint
          : trustedKeyFingerprint // ignore: cast_nullable_to_non_nullable
              as String,
      switchKind: null == switchKind
          ? _self.switchKind
          : switchKind // ignore: cast_nullable_to_non_nullable
              as String,
      voteServers: null == voteServers
          ? _self._voteServers
          : voteServers // ignore: cast_nullable_to_non_nullable
              as List<VotingServiceEndpoint>,
      pirServers: null == pirServers
          ? _self._pirServers
          : pirServers // ignore: cast_nullable_to_non_nullable
              as List<VotingServiceEndpoint>,
      pirLayout: freezed == pirLayout
          ? _self.pirLayout
          : pirLayout // ignore: cast_nullable_to_non_nullable
              as VotingPirLayout?,
      rounds: null == rounds
          ? _self._rounds
          : rounds // ignore: cast_nullable_to_non_nullable
              as List<VotingConfigRound>,
    ));
  }

  /// Create a copy of VotingConfig
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $VotingPirLayoutCopyWith<$Res>? get pirLayout {
    if (_self.pirLayout == null) {
      return null;
    }

    return $VotingPirLayoutCopyWith<$Res>(_self.pirLayout!, (value) {
      return _then(_self.copyWith(pirLayout: value));
    });
  }
}

/// @nodoc
mixin _$VotingConfigRound {
  String get roundId;
  Uint8List get eaPk;

  /// Create a copy of VotingConfigRound
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $VotingConfigRoundCopyWith<VotingConfigRound> get copyWith =>
      _$VotingConfigRoundCopyWithImpl<VotingConfigRound>(
          this as VotingConfigRound, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is VotingConfigRound &&
            (identical(other.roundId, roundId) || other.roundId == roundId) &&
            const DeepCollectionEquality().equals(other.eaPk, eaPk));
  }

  @override
  int get hashCode => Object.hash(
      runtimeType, roundId, const DeepCollectionEquality().hash(eaPk));

  @override
  String toString() {
    return 'VotingConfigRound(roundId: $roundId, eaPk: $eaPk)';
  }
}

/// @nodoc
abstract mixin class $VotingConfigRoundCopyWith<$Res> {
  factory $VotingConfigRoundCopyWith(
          VotingConfigRound value, $Res Function(VotingConfigRound) _then) =
      _$VotingConfigRoundCopyWithImpl;
  @useResult
  $Res call({String roundId, Uint8List eaPk});
}

/// @nodoc
class _$VotingConfigRoundCopyWithImpl<$Res>
    implements $VotingConfigRoundCopyWith<$Res> {
  _$VotingConfigRoundCopyWithImpl(this._self, this._then);

  final VotingConfigRound _self;
  final $Res Function(VotingConfigRound) _then;

  /// Create a copy of VotingConfigRound
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? roundId = null,
    Object? eaPk = null,
  }) {
    return _then(_self.copyWith(
      roundId: null == roundId
          ? _self.roundId
          : roundId // ignore: cast_nullable_to_non_nullable
              as String,
      eaPk: null == eaPk
          ? _self.eaPk
          : eaPk // ignore: cast_nullable_to_non_nullable
              as Uint8List,
    ));
  }
}

/// Adds pattern-matching-related methods to [VotingConfigRound].
extension VotingConfigRoundPatterns on VotingConfigRound {
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
    TResult Function(_VotingConfigRound value)? $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _VotingConfigRound() when $default != null:
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
    TResult Function(_VotingConfigRound value) $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingConfigRound():
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
    TResult? Function(_VotingConfigRound value)? $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingConfigRound() when $default != null:
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
    TResult Function(String roundId, Uint8List eaPk)? $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _VotingConfigRound() when $default != null:
        return $default(_that.roundId, _that.eaPk);
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
    TResult Function(String roundId, Uint8List eaPk) $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingConfigRound():
        return $default(_that.roundId, _that.eaPk);
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
    TResult? Function(String roundId, Uint8List eaPk)? $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingConfigRound() when $default != null:
        return $default(_that.roundId, _that.eaPk);
      case _:
        return null;
    }
  }
}

/// @nodoc

class _VotingConfigRound implements VotingConfigRound {
  const _VotingConfigRound({required this.roundId, required this.eaPk});

  @override
  final String roundId;
  @override
  final Uint8List eaPk;

  /// Create a copy of VotingConfigRound
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  _$VotingConfigRoundCopyWith<_VotingConfigRound> get copyWith =>
      __$VotingConfigRoundCopyWithImpl<_VotingConfigRound>(this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _VotingConfigRound &&
            (identical(other.roundId, roundId) || other.roundId == roundId) &&
            const DeepCollectionEquality().equals(other.eaPk, eaPk));
  }

  @override
  int get hashCode => Object.hash(
      runtimeType, roundId, const DeepCollectionEquality().hash(eaPk));

  @override
  String toString() {
    return 'VotingConfigRound(roundId: $roundId, eaPk: $eaPk)';
  }
}

/// @nodoc
abstract mixin class _$VotingConfigRoundCopyWith<$Res>
    implements $VotingConfigRoundCopyWith<$Res> {
  factory _$VotingConfigRoundCopyWith(
          _VotingConfigRound value, $Res Function(_VotingConfigRound) _then) =
      __$VotingConfigRoundCopyWithImpl;
  @override
  @useResult
  $Res call({String roundId, Uint8List eaPk});
}

/// @nodoc
class __$VotingConfigRoundCopyWithImpl<$Res>
    implements _$VotingConfigRoundCopyWith<$Res> {
  __$VotingConfigRoundCopyWithImpl(this._self, this._then);

  final _VotingConfigRound _self;
  final $Res Function(_VotingConfigRound) _then;

  /// Create a copy of VotingConfigRound
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $Res call({
    Object? roundId = null,
    Object? eaPk = null,
  }) {
    return _then(_VotingConfigRound(
      roundId: null == roundId
          ? _self.roundId
          : roundId // ignore: cast_nullable_to_non_nullable
              as String,
      eaPk: null == eaPk
          ? _self.eaPk
          : eaPk // ignore: cast_nullable_to_non_nullable
              as Uint8List,
    ));
  }
}

/// @nodoc
mixin _$VotingDelegationBuild {
  VotingDelegationSubmission get submission;
  String get wireJson;

  /// Create a copy of VotingDelegationBuild
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $VotingDelegationBuildCopyWith<VotingDelegationBuild> get copyWith =>
      _$VotingDelegationBuildCopyWithImpl<VotingDelegationBuild>(
          this as VotingDelegationBuild, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is VotingDelegationBuild &&
            (identical(other.submission, submission) ||
                other.submission == submission) &&
            (identical(other.wireJson, wireJson) ||
                other.wireJson == wireJson));
  }

  @override
  int get hashCode => Object.hash(runtimeType, submission, wireJson);

  @override
  String toString() {
    return 'VotingDelegationBuild(submission: $submission, wireJson: $wireJson)';
  }
}

/// @nodoc
abstract mixin class $VotingDelegationBuildCopyWith<$Res> {
  factory $VotingDelegationBuildCopyWith(VotingDelegationBuild value,
          $Res Function(VotingDelegationBuild) _then) =
      _$VotingDelegationBuildCopyWithImpl;
  @useResult
  $Res call({VotingDelegationSubmission submission, String wireJson});

  $VotingDelegationSubmissionCopyWith<$Res> get submission;
}

/// @nodoc
class _$VotingDelegationBuildCopyWithImpl<$Res>
    implements $VotingDelegationBuildCopyWith<$Res> {
  _$VotingDelegationBuildCopyWithImpl(this._self, this._then);

  final VotingDelegationBuild _self;
  final $Res Function(VotingDelegationBuild) _then;

  /// Create a copy of VotingDelegationBuild
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? submission = null,
    Object? wireJson = null,
  }) {
    return _then(_self.copyWith(
      submission: null == submission
          ? _self.submission
          : submission // ignore: cast_nullable_to_non_nullable
              as VotingDelegationSubmission,
      wireJson: null == wireJson
          ? _self.wireJson
          : wireJson // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }

  /// Create a copy of VotingDelegationBuild
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $VotingDelegationSubmissionCopyWith<$Res> get submission {
    return $VotingDelegationSubmissionCopyWith<$Res>(_self.submission, (value) {
      return _then(_self.copyWith(submission: value));
    });
  }
}

/// Adds pattern-matching-related methods to [VotingDelegationBuild].
extension VotingDelegationBuildPatterns on VotingDelegationBuild {
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
    TResult Function(_VotingDelegationBuild value)? $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _VotingDelegationBuild() when $default != null:
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
    TResult Function(_VotingDelegationBuild value) $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingDelegationBuild():
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
    TResult? Function(_VotingDelegationBuild value)? $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingDelegationBuild() when $default != null:
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
    TResult Function(VotingDelegationSubmission submission, String wireJson)?
        $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _VotingDelegationBuild() when $default != null:
        return $default(_that.submission, _that.wireJson);
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
    TResult Function(VotingDelegationSubmission submission, String wireJson)
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingDelegationBuild():
        return $default(_that.submission, _that.wireJson);
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
    TResult? Function(VotingDelegationSubmission submission, String wireJson)?
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingDelegationBuild() when $default != null:
        return $default(_that.submission, _that.wireJson);
      case _:
        return null;
    }
  }
}

/// @nodoc

class _VotingDelegationBuild implements VotingDelegationBuild {
  const _VotingDelegationBuild(
      {required this.submission, required this.wireJson});

  @override
  final VotingDelegationSubmission submission;
  @override
  final String wireJson;

  /// Create a copy of VotingDelegationBuild
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  _$VotingDelegationBuildCopyWith<_VotingDelegationBuild> get copyWith =>
      __$VotingDelegationBuildCopyWithImpl<_VotingDelegationBuild>(
          this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _VotingDelegationBuild &&
            (identical(other.submission, submission) ||
                other.submission == submission) &&
            (identical(other.wireJson, wireJson) ||
                other.wireJson == wireJson));
  }

  @override
  int get hashCode => Object.hash(runtimeType, submission, wireJson);

  @override
  String toString() {
    return 'VotingDelegationBuild(submission: $submission, wireJson: $wireJson)';
  }
}

/// @nodoc
abstract mixin class _$VotingDelegationBuildCopyWith<$Res>
    implements $VotingDelegationBuildCopyWith<$Res> {
  factory _$VotingDelegationBuildCopyWith(_VotingDelegationBuild value,
          $Res Function(_VotingDelegationBuild) _then) =
      __$VotingDelegationBuildCopyWithImpl;
  @override
  @useResult
  $Res call({VotingDelegationSubmission submission, String wireJson});

  @override
  $VotingDelegationSubmissionCopyWith<$Res> get submission;
}

/// @nodoc
class __$VotingDelegationBuildCopyWithImpl<$Res>
    implements _$VotingDelegationBuildCopyWith<$Res> {
  __$VotingDelegationBuildCopyWithImpl(this._self, this._then);

  final _VotingDelegationBuild _self;
  final $Res Function(_VotingDelegationBuild) _then;

  /// Create a copy of VotingDelegationBuild
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $Res call({
    Object? submission = null,
    Object? wireJson = null,
  }) {
    return _then(_VotingDelegationBuild(
      submission: null == submission
          ? _self.submission
          : submission // ignore: cast_nullable_to_non_nullable
              as VotingDelegationSubmission,
      wireJson: null == wireJson
          ? _self.wireJson
          : wireJson // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }

  /// Create a copy of VotingDelegationBuild
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $VotingDelegationSubmissionCopyWith<$Res> get submission {
    return $VotingDelegationSubmissionCopyWith<$Res>(_self.submission, (value) {
      return _then(_self.copyWith(submission: value));
    });
  }
}

/// @nodoc
mixin _$VotingDelegationConfirmation {
  String get txHash;
  int get vanLeafPosition;

  /// Create a copy of VotingDelegationConfirmation
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $VotingDelegationConfirmationCopyWith<VotingDelegationConfirmation>
      get copyWith => _$VotingDelegationConfirmationCopyWithImpl<
              VotingDelegationConfirmation>(
          this as VotingDelegationConfirmation, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is VotingDelegationConfirmation &&
            (identical(other.txHash, txHash) || other.txHash == txHash) &&
            (identical(other.vanLeafPosition, vanLeafPosition) ||
                other.vanLeafPosition == vanLeafPosition));
  }

  @override
  int get hashCode => Object.hash(runtimeType, txHash, vanLeafPosition);

  @override
  String toString() {
    return 'VotingDelegationConfirmation(txHash: $txHash, vanLeafPosition: $vanLeafPosition)';
  }
}

/// @nodoc
abstract mixin class $VotingDelegationConfirmationCopyWith<$Res> {
  factory $VotingDelegationConfirmationCopyWith(
          VotingDelegationConfirmation value,
          $Res Function(VotingDelegationConfirmation) _then) =
      _$VotingDelegationConfirmationCopyWithImpl;
  @useResult
  $Res call({String txHash, int vanLeafPosition});
}

/// @nodoc
class _$VotingDelegationConfirmationCopyWithImpl<$Res>
    implements $VotingDelegationConfirmationCopyWith<$Res> {
  _$VotingDelegationConfirmationCopyWithImpl(this._self, this._then);

  final VotingDelegationConfirmation _self;
  final $Res Function(VotingDelegationConfirmation) _then;

  /// Create a copy of VotingDelegationConfirmation
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? txHash = null,
    Object? vanLeafPosition = null,
  }) {
    return _then(_self.copyWith(
      txHash: null == txHash
          ? _self.txHash
          : txHash // ignore: cast_nullable_to_non_nullable
              as String,
      vanLeafPosition: null == vanLeafPosition
          ? _self.vanLeafPosition
          : vanLeafPosition // ignore: cast_nullable_to_non_nullable
              as int,
    ));
  }
}

/// Adds pattern-matching-related methods to [VotingDelegationConfirmation].
extension VotingDelegationConfirmationPatterns on VotingDelegationConfirmation {
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
    TResult Function(_VotingDelegationConfirmation value)? $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _VotingDelegationConfirmation() when $default != null:
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
    TResult Function(_VotingDelegationConfirmation value) $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingDelegationConfirmation():
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
    TResult? Function(_VotingDelegationConfirmation value)? $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingDelegationConfirmation() when $default != null:
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
    TResult Function(String txHash, int vanLeafPosition)? $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _VotingDelegationConfirmation() when $default != null:
        return $default(_that.txHash, _that.vanLeafPosition);
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
    TResult Function(String txHash, int vanLeafPosition) $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingDelegationConfirmation():
        return $default(_that.txHash, _that.vanLeafPosition);
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
    TResult? Function(String txHash, int vanLeafPosition)? $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingDelegationConfirmation() when $default != null:
        return $default(_that.txHash, _that.vanLeafPosition);
      case _:
        return null;
    }
  }
}

/// @nodoc

class _VotingDelegationConfirmation implements VotingDelegationConfirmation {
  const _VotingDelegationConfirmation(
      {required this.txHash, required this.vanLeafPosition});

  @override
  final String txHash;
  @override
  final int vanLeafPosition;

  /// Create a copy of VotingDelegationConfirmation
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  _$VotingDelegationConfirmationCopyWith<_VotingDelegationConfirmation>
      get copyWith => __$VotingDelegationConfirmationCopyWithImpl<
          _VotingDelegationConfirmation>(this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _VotingDelegationConfirmation &&
            (identical(other.txHash, txHash) || other.txHash == txHash) &&
            (identical(other.vanLeafPosition, vanLeafPosition) ||
                other.vanLeafPosition == vanLeafPosition));
  }

  @override
  int get hashCode => Object.hash(runtimeType, txHash, vanLeafPosition);

  @override
  String toString() {
    return 'VotingDelegationConfirmation(txHash: $txHash, vanLeafPosition: $vanLeafPosition)';
  }
}

/// @nodoc
abstract mixin class _$VotingDelegationConfirmationCopyWith<$Res>
    implements $VotingDelegationConfirmationCopyWith<$Res> {
  factory _$VotingDelegationConfirmationCopyWith(
          _VotingDelegationConfirmation value,
          $Res Function(_VotingDelegationConfirmation) _then) =
      __$VotingDelegationConfirmationCopyWithImpl;
  @override
  @useResult
  $Res call({String txHash, int vanLeafPosition});
}

/// @nodoc
class __$VotingDelegationConfirmationCopyWithImpl<$Res>
    implements _$VotingDelegationConfirmationCopyWith<$Res> {
  __$VotingDelegationConfirmationCopyWithImpl(this._self, this._then);

  final _VotingDelegationConfirmation _self;
  final $Res Function(_VotingDelegationConfirmation) _then;

  /// Create a copy of VotingDelegationConfirmation
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $Res call({
    Object? txHash = null,
    Object? vanLeafPosition = null,
  }) {
    return _then(_VotingDelegationConfirmation(
      txHash: null == txHash
          ? _self.txHash
          : txHash // ignore: cast_nullable_to_non_nullable
              as String,
      vanLeafPosition: null == vanLeafPosition
          ? _self.vanLeafPosition
          : vanLeafPosition // ignore: cast_nullable_to_non_nullable
              as int,
    ));
  }
}

/// @nodoc
mixin _$VotingDelegationProgress {
  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType && other is VotingDelegationProgress);
  }

  @override
  int get hashCode => runtimeType.hashCode;

  @override
  String toString() {
    return 'VotingDelegationProgress()';
  }
}

/// @nodoc
class $VotingDelegationProgressCopyWith<$Res> {
  $VotingDelegationProgressCopyWith(
      VotingDelegationProgress _, $Res Function(VotingDelegationProgress) __);
}

/// Adds pattern-matching-related methods to [VotingDelegationProgress].
extension VotingDelegationProgressPatterns on VotingDelegationProgress {
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
    TResult Function(VotingDelegationProgress_SelectingNotes value)?
        selectingNotes,
    TResult Function(VotingDelegationProgress_PcztBuilding value)? pcztBuilding,
    TResult Function(VotingDelegationProgress_PcztBuilt value)? pcztBuilt,
    TResult Function(VotingDelegationProgress_ProofStarting value)?
        proofStarting,
    TResult Function(VotingDelegationProgress_ProofProgress value)?
        proofProgress,
    TResult Function(VotingDelegationProgress_ProofComplete value)?
        proofComplete,
    TResult Function(VotingDelegationProgress_SigningPayload value)?
        signingPayload,
    TResult Function(VotingDelegationProgress_PayloadReady value)? payloadReady,
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case VotingDelegationProgress_SelectingNotes()
          when selectingNotes != null:
        return selectingNotes(_that);
      case VotingDelegationProgress_PcztBuilding() when pcztBuilding != null:
        return pcztBuilding(_that);
      case VotingDelegationProgress_PcztBuilt() when pcztBuilt != null:
        return pcztBuilt(_that);
      case VotingDelegationProgress_ProofStarting() when proofStarting != null:
        return proofStarting(_that);
      case VotingDelegationProgress_ProofProgress() when proofProgress != null:
        return proofProgress(_that);
      case VotingDelegationProgress_ProofComplete() when proofComplete != null:
        return proofComplete(_that);
      case VotingDelegationProgress_SigningPayload()
          when signingPayload != null:
        return signingPayload(_that);
      case VotingDelegationProgress_PayloadReady() when payloadReady != null:
        return payloadReady(_that);
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
    required TResult Function(VotingDelegationProgress_SelectingNotes value)
        selectingNotes,
    required TResult Function(VotingDelegationProgress_PcztBuilding value)
        pcztBuilding,
    required TResult Function(VotingDelegationProgress_PcztBuilt value)
        pcztBuilt,
    required TResult Function(VotingDelegationProgress_ProofStarting value)
        proofStarting,
    required TResult Function(VotingDelegationProgress_ProofProgress value)
        proofProgress,
    required TResult Function(VotingDelegationProgress_ProofComplete value)
        proofComplete,
    required TResult Function(VotingDelegationProgress_SigningPayload value)
        signingPayload,
    required TResult Function(VotingDelegationProgress_PayloadReady value)
        payloadReady,
  }) {
    final _that = this;
    switch (_that) {
      case VotingDelegationProgress_SelectingNotes():
        return selectingNotes(_that);
      case VotingDelegationProgress_PcztBuilding():
        return pcztBuilding(_that);
      case VotingDelegationProgress_PcztBuilt():
        return pcztBuilt(_that);
      case VotingDelegationProgress_ProofStarting():
        return proofStarting(_that);
      case VotingDelegationProgress_ProofProgress():
        return proofProgress(_that);
      case VotingDelegationProgress_ProofComplete():
        return proofComplete(_that);
      case VotingDelegationProgress_SigningPayload():
        return signingPayload(_that);
      case VotingDelegationProgress_PayloadReady():
        return payloadReady(_that);
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
    TResult? Function(VotingDelegationProgress_SelectingNotes value)?
        selectingNotes,
    TResult? Function(VotingDelegationProgress_PcztBuilding value)?
        pcztBuilding,
    TResult? Function(VotingDelegationProgress_PcztBuilt value)? pcztBuilt,
    TResult? Function(VotingDelegationProgress_ProofStarting value)?
        proofStarting,
    TResult? Function(VotingDelegationProgress_ProofProgress value)?
        proofProgress,
    TResult? Function(VotingDelegationProgress_ProofComplete value)?
        proofComplete,
    TResult? Function(VotingDelegationProgress_SigningPayload value)?
        signingPayload,
    TResult? Function(VotingDelegationProgress_PayloadReady value)?
        payloadReady,
  }) {
    final _that = this;
    switch (_that) {
      case VotingDelegationProgress_SelectingNotes()
          when selectingNotes != null:
        return selectingNotes(_that);
      case VotingDelegationProgress_PcztBuilding() when pcztBuilding != null:
        return pcztBuilding(_that);
      case VotingDelegationProgress_PcztBuilt() when pcztBuilt != null:
        return pcztBuilt(_that);
      case VotingDelegationProgress_ProofStarting() when proofStarting != null:
        return proofStarting(_that);
      case VotingDelegationProgress_ProofProgress() when proofProgress != null:
        return proofProgress(_that);
      case VotingDelegationProgress_ProofComplete() when proofComplete != null:
        return proofComplete(_that);
      case VotingDelegationProgress_SigningPayload()
          when signingPayload != null:
        return signingPayload(_that);
      case VotingDelegationProgress_PayloadReady() when payloadReady != null:
        return payloadReady(_that);
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
    TResult Function()? selectingNotes,
    TResult Function()? pcztBuilding,
    TResult Function()? pcztBuilt,
    TResult Function()? proofStarting,
    TResult Function(double progress)? proofProgress,
    TResult Function()? proofComplete,
    TResult Function()? signingPayload,
    TResult Function()? payloadReady,
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case VotingDelegationProgress_SelectingNotes()
          when selectingNotes != null:
        return selectingNotes();
      case VotingDelegationProgress_PcztBuilding() when pcztBuilding != null:
        return pcztBuilding();
      case VotingDelegationProgress_PcztBuilt() when pcztBuilt != null:
        return pcztBuilt();
      case VotingDelegationProgress_ProofStarting() when proofStarting != null:
        return proofStarting();
      case VotingDelegationProgress_ProofProgress() when proofProgress != null:
        return proofProgress(_that.progress);
      case VotingDelegationProgress_ProofComplete() when proofComplete != null:
        return proofComplete();
      case VotingDelegationProgress_SigningPayload()
          when signingPayload != null:
        return signingPayload();
      case VotingDelegationProgress_PayloadReady() when payloadReady != null:
        return payloadReady();
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
    required TResult Function() selectingNotes,
    required TResult Function() pcztBuilding,
    required TResult Function() pcztBuilt,
    required TResult Function() proofStarting,
    required TResult Function(double progress) proofProgress,
    required TResult Function() proofComplete,
    required TResult Function() signingPayload,
    required TResult Function() payloadReady,
  }) {
    final _that = this;
    switch (_that) {
      case VotingDelegationProgress_SelectingNotes():
        return selectingNotes();
      case VotingDelegationProgress_PcztBuilding():
        return pcztBuilding();
      case VotingDelegationProgress_PcztBuilt():
        return pcztBuilt();
      case VotingDelegationProgress_ProofStarting():
        return proofStarting();
      case VotingDelegationProgress_ProofProgress():
        return proofProgress(_that.progress);
      case VotingDelegationProgress_ProofComplete():
        return proofComplete();
      case VotingDelegationProgress_SigningPayload():
        return signingPayload();
      case VotingDelegationProgress_PayloadReady():
        return payloadReady();
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
    TResult? Function()? selectingNotes,
    TResult? Function()? pcztBuilding,
    TResult? Function()? pcztBuilt,
    TResult? Function()? proofStarting,
    TResult? Function(double progress)? proofProgress,
    TResult? Function()? proofComplete,
    TResult? Function()? signingPayload,
    TResult? Function()? payloadReady,
  }) {
    final _that = this;
    switch (_that) {
      case VotingDelegationProgress_SelectingNotes()
          when selectingNotes != null:
        return selectingNotes();
      case VotingDelegationProgress_PcztBuilding() when pcztBuilding != null:
        return pcztBuilding();
      case VotingDelegationProgress_PcztBuilt() when pcztBuilt != null:
        return pcztBuilt();
      case VotingDelegationProgress_ProofStarting() when proofStarting != null:
        return proofStarting();
      case VotingDelegationProgress_ProofProgress() when proofProgress != null:
        return proofProgress(_that.progress);
      case VotingDelegationProgress_ProofComplete() when proofComplete != null:
        return proofComplete();
      case VotingDelegationProgress_SigningPayload()
          when signingPayload != null:
        return signingPayload();
      case VotingDelegationProgress_PayloadReady() when payloadReady != null:
        return payloadReady();
      case _:
        return null;
    }
  }
}

/// @nodoc

class VotingDelegationProgress_SelectingNotes extends VotingDelegationProgress {
  const VotingDelegationProgress_SelectingNotes() : super._();

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is VotingDelegationProgress_SelectingNotes);
  }

  @override
  int get hashCode => runtimeType.hashCode;

  @override
  String toString() {
    return 'VotingDelegationProgress.selectingNotes()';
  }
}

/// @nodoc

class VotingDelegationProgress_PcztBuilding extends VotingDelegationProgress {
  const VotingDelegationProgress_PcztBuilding() : super._();

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is VotingDelegationProgress_PcztBuilding);
  }

  @override
  int get hashCode => runtimeType.hashCode;

  @override
  String toString() {
    return 'VotingDelegationProgress.pcztBuilding()';
  }
}

/// @nodoc

class VotingDelegationProgress_PcztBuilt extends VotingDelegationProgress {
  const VotingDelegationProgress_PcztBuilt() : super._();

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is VotingDelegationProgress_PcztBuilt);
  }

  @override
  int get hashCode => runtimeType.hashCode;

  @override
  String toString() {
    return 'VotingDelegationProgress.pcztBuilt()';
  }
}

/// @nodoc

class VotingDelegationProgress_ProofStarting extends VotingDelegationProgress {
  const VotingDelegationProgress_ProofStarting() : super._();

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is VotingDelegationProgress_ProofStarting);
  }

  @override
  int get hashCode => runtimeType.hashCode;

  @override
  String toString() {
    return 'VotingDelegationProgress.proofStarting()';
  }
}

/// @nodoc

class VotingDelegationProgress_ProofProgress extends VotingDelegationProgress {
  const VotingDelegationProgress_ProofProgress({required this.progress})
      : super._();

  final double progress;

  /// Create a copy of VotingDelegationProgress
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $VotingDelegationProgress_ProofProgressCopyWith<
          VotingDelegationProgress_ProofProgress>
      get copyWith => _$VotingDelegationProgress_ProofProgressCopyWithImpl<
          VotingDelegationProgress_ProofProgress>(this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is VotingDelegationProgress_ProofProgress &&
            (identical(other.progress, progress) ||
                other.progress == progress));
  }

  @override
  int get hashCode => Object.hash(runtimeType, progress);

  @override
  String toString() {
    return 'VotingDelegationProgress.proofProgress(progress: $progress)';
  }
}

/// @nodoc
abstract mixin class $VotingDelegationProgress_ProofProgressCopyWith<$Res>
    implements $VotingDelegationProgressCopyWith<$Res> {
  factory $VotingDelegationProgress_ProofProgressCopyWith(
          VotingDelegationProgress_ProofProgress value,
          $Res Function(VotingDelegationProgress_ProofProgress) _then) =
      _$VotingDelegationProgress_ProofProgressCopyWithImpl;
  @useResult
  $Res call({double progress});
}

/// @nodoc
class _$VotingDelegationProgress_ProofProgressCopyWithImpl<$Res>
    implements $VotingDelegationProgress_ProofProgressCopyWith<$Res> {
  _$VotingDelegationProgress_ProofProgressCopyWithImpl(this._self, this._then);

  final VotingDelegationProgress_ProofProgress _self;
  final $Res Function(VotingDelegationProgress_ProofProgress) _then;

  /// Create a copy of VotingDelegationProgress
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  $Res call({
    Object? progress = null,
  }) {
    return _then(VotingDelegationProgress_ProofProgress(
      progress: null == progress
          ? _self.progress
          : progress // ignore: cast_nullable_to_non_nullable
              as double,
    ));
  }
}

/// @nodoc

class VotingDelegationProgress_ProofComplete extends VotingDelegationProgress {
  const VotingDelegationProgress_ProofComplete() : super._();

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is VotingDelegationProgress_ProofComplete);
  }

  @override
  int get hashCode => runtimeType.hashCode;

  @override
  String toString() {
    return 'VotingDelegationProgress.proofComplete()';
  }
}

/// @nodoc

class VotingDelegationProgress_SigningPayload extends VotingDelegationProgress {
  const VotingDelegationProgress_SigningPayload() : super._();

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is VotingDelegationProgress_SigningPayload);
  }

  @override
  int get hashCode => runtimeType.hashCode;

  @override
  String toString() {
    return 'VotingDelegationProgress.signingPayload()';
  }
}

/// @nodoc

class VotingDelegationProgress_PayloadReady extends VotingDelegationProgress {
  const VotingDelegationProgress_PayloadReady() : super._();

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is VotingDelegationProgress_PayloadReady);
  }

  @override
  int get hashCode => runtimeType.hashCode;

  @override
  String toString() {
    return 'VotingDelegationProgress.payloadReady()';
  }
}

/// @nodoc
mixin _$VotingDelegationRecovery {
  int get bundleIndex;
  String get phase;
  String get workflowPhase;
  String? get txHash;
  int? get vanLeafPosition;

  /// Create a copy of VotingDelegationRecovery
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $VotingDelegationRecoveryCopyWith<VotingDelegationRecovery> get copyWith =>
      _$VotingDelegationRecoveryCopyWithImpl<VotingDelegationRecovery>(
          this as VotingDelegationRecovery, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is VotingDelegationRecovery &&
            (identical(other.bundleIndex, bundleIndex) ||
                other.bundleIndex == bundleIndex) &&
            (identical(other.phase, phase) || other.phase == phase) &&
            (identical(other.workflowPhase, workflowPhase) ||
                other.workflowPhase == workflowPhase) &&
            (identical(other.txHash, txHash) || other.txHash == txHash) &&
            (identical(other.vanLeafPosition, vanLeafPosition) ||
                other.vanLeafPosition == vanLeafPosition));
  }

  @override
  int get hashCode => Object.hash(
      runtimeType, bundleIndex, phase, workflowPhase, txHash, vanLeafPosition);

  @override
  String toString() {
    return 'VotingDelegationRecovery(bundleIndex: $bundleIndex, phase: $phase, workflowPhase: $workflowPhase, txHash: $txHash, vanLeafPosition: $vanLeafPosition)';
  }
}

/// @nodoc
abstract mixin class $VotingDelegationRecoveryCopyWith<$Res> {
  factory $VotingDelegationRecoveryCopyWith(VotingDelegationRecovery value,
          $Res Function(VotingDelegationRecovery) _then) =
      _$VotingDelegationRecoveryCopyWithImpl;
  @useResult
  $Res call(
      {int bundleIndex,
      String phase,
      String workflowPhase,
      String? txHash,
      int? vanLeafPosition});
}

/// @nodoc
class _$VotingDelegationRecoveryCopyWithImpl<$Res>
    implements $VotingDelegationRecoveryCopyWith<$Res> {
  _$VotingDelegationRecoveryCopyWithImpl(this._self, this._then);

  final VotingDelegationRecovery _self;
  final $Res Function(VotingDelegationRecovery) _then;

  /// Create a copy of VotingDelegationRecovery
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? bundleIndex = null,
    Object? phase = null,
    Object? workflowPhase = null,
    Object? txHash = freezed,
    Object? vanLeafPosition = freezed,
  }) {
    return _then(_self.copyWith(
      bundleIndex: null == bundleIndex
          ? _self.bundleIndex
          : bundleIndex // ignore: cast_nullable_to_non_nullable
              as int,
      phase: null == phase
          ? _self.phase
          : phase // ignore: cast_nullable_to_non_nullable
              as String,
      workflowPhase: null == workflowPhase
          ? _self.workflowPhase
          : workflowPhase // ignore: cast_nullable_to_non_nullable
              as String,
      txHash: freezed == txHash
          ? _self.txHash
          : txHash // ignore: cast_nullable_to_non_nullable
              as String?,
      vanLeafPosition: freezed == vanLeafPosition
          ? _self.vanLeafPosition
          : vanLeafPosition // ignore: cast_nullable_to_non_nullable
              as int?,
    ));
  }
}

/// Adds pattern-matching-related methods to [VotingDelegationRecovery].
extension VotingDelegationRecoveryPatterns on VotingDelegationRecovery {
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
    TResult Function(_VotingDelegationRecovery value)? $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _VotingDelegationRecovery() when $default != null:
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
    TResult Function(_VotingDelegationRecovery value) $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingDelegationRecovery():
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
    TResult? Function(_VotingDelegationRecovery value)? $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingDelegationRecovery() when $default != null:
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
    TResult Function(int bundleIndex, String phase, String workflowPhase,
            String? txHash, int? vanLeafPosition)?
        $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _VotingDelegationRecovery() when $default != null:
        return $default(_that.bundleIndex, _that.phase, _that.workflowPhase,
            _that.txHash, _that.vanLeafPosition);
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
    TResult Function(int bundleIndex, String phase, String workflowPhase,
            String? txHash, int? vanLeafPosition)
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingDelegationRecovery():
        return $default(_that.bundleIndex, _that.phase, _that.workflowPhase,
            _that.txHash, _that.vanLeafPosition);
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
    TResult? Function(int bundleIndex, String phase, String workflowPhase,
            String? txHash, int? vanLeafPosition)?
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingDelegationRecovery() when $default != null:
        return $default(_that.bundleIndex, _that.phase, _that.workflowPhase,
            _that.txHash, _that.vanLeafPosition);
      case _:
        return null;
    }
  }
}

/// @nodoc

class _VotingDelegationRecovery implements VotingDelegationRecovery {
  const _VotingDelegationRecovery(
      {required this.bundleIndex,
      required this.phase,
      required this.workflowPhase,
      this.txHash,
      this.vanLeafPosition});

  @override
  final int bundleIndex;
  @override
  final String phase;
  @override
  final String workflowPhase;
  @override
  final String? txHash;
  @override
  final int? vanLeafPosition;

  /// Create a copy of VotingDelegationRecovery
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  _$VotingDelegationRecoveryCopyWith<_VotingDelegationRecovery> get copyWith =>
      __$VotingDelegationRecoveryCopyWithImpl<_VotingDelegationRecovery>(
          this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _VotingDelegationRecovery &&
            (identical(other.bundleIndex, bundleIndex) ||
                other.bundleIndex == bundleIndex) &&
            (identical(other.phase, phase) || other.phase == phase) &&
            (identical(other.workflowPhase, workflowPhase) ||
                other.workflowPhase == workflowPhase) &&
            (identical(other.txHash, txHash) || other.txHash == txHash) &&
            (identical(other.vanLeafPosition, vanLeafPosition) ||
                other.vanLeafPosition == vanLeafPosition));
  }

  @override
  int get hashCode => Object.hash(
      runtimeType, bundleIndex, phase, workflowPhase, txHash, vanLeafPosition);

  @override
  String toString() {
    return 'VotingDelegationRecovery(bundleIndex: $bundleIndex, phase: $phase, workflowPhase: $workflowPhase, txHash: $txHash, vanLeafPosition: $vanLeafPosition)';
  }
}

/// @nodoc
abstract mixin class _$VotingDelegationRecoveryCopyWith<$Res>
    implements $VotingDelegationRecoveryCopyWith<$Res> {
  factory _$VotingDelegationRecoveryCopyWith(_VotingDelegationRecovery value,
          $Res Function(_VotingDelegationRecovery) _then) =
      __$VotingDelegationRecoveryCopyWithImpl;
  @override
  @useResult
  $Res call(
      {int bundleIndex,
      String phase,
      String workflowPhase,
      String? txHash,
      int? vanLeafPosition});
}

/// @nodoc
class __$VotingDelegationRecoveryCopyWithImpl<$Res>
    implements _$VotingDelegationRecoveryCopyWith<$Res> {
  __$VotingDelegationRecoveryCopyWithImpl(this._self, this._then);

  final _VotingDelegationRecovery _self;
  final $Res Function(_VotingDelegationRecovery) _then;

  /// Create a copy of VotingDelegationRecovery
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $Res call({
    Object? bundleIndex = null,
    Object? phase = null,
    Object? workflowPhase = null,
    Object? txHash = freezed,
    Object? vanLeafPosition = freezed,
  }) {
    return _then(_VotingDelegationRecovery(
      bundleIndex: null == bundleIndex
          ? _self.bundleIndex
          : bundleIndex // ignore: cast_nullable_to_non_nullable
              as int,
      phase: null == phase
          ? _self.phase
          : phase // ignore: cast_nullable_to_non_nullable
              as String,
      workflowPhase: null == workflowPhase
          ? _self.workflowPhase
          : workflowPhase // ignore: cast_nullable_to_non_nullable
              as String,
      txHash: freezed == txHash
          ? _self.txHash
          : txHash // ignore: cast_nullable_to_non_nullable
              as String?,
      vanLeafPosition: freezed == vanLeafPosition
          ? _self.vanLeafPosition
          : vanLeafPosition // ignore: cast_nullable_to_non_nullable
              as int?,
    ));
  }
}

/// @nodoc
mixin _$VotingDelegationSetup {
  Uint8List get pcztBytes;
  Uint8List get pcztSighash;
  Uint8List get rk;
  int get actionIndex;
  Uint8List get actionBytes;
  Uint8List get tx1Effects;

  /// Create a copy of VotingDelegationSetup
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $VotingDelegationSetupCopyWith<VotingDelegationSetup> get copyWith =>
      _$VotingDelegationSetupCopyWithImpl<VotingDelegationSetup>(
          this as VotingDelegationSetup, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is VotingDelegationSetup &&
            const DeepCollectionEquality().equals(other.pcztBytes, pcztBytes) &&
            const DeepCollectionEquality()
                .equals(other.pcztSighash, pcztSighash) &&
            const DeepCollectionEquality().equals(other.rk, rk) &&
            (identical(other.actionIndex, actionIndex) ||
                other.actionIndex == actionIndex) &&
            const DeepCollectionEquality()
                .equals(other.actionBytes, actionBytes) &&
            const DeepCollectionEquality()
                .equals(other.tx1Effects, tx1Effects));
  }

  @override
  int get hashCode => Object.hash(
      runtimeType,
      const DeepCollectionEquality().hash(pcztBytes),
      const DeepCollectionEquality().hash(pcztSighash),
      const DeepCollectionEquality().hash(rk),
      actionIndex,
      const DeepCollectionEquality().hash(actionBytes),
      const DeepCollectionEquality().hash(tx1Effects));

  @override
  String toString() {
    return 'VotingDelegationSetup(pcztBytes: $pcztBytes, pcztSighash: $pcztSighash, rk: $rk, actionIndex: $actionIndex, actionBytes: $actionBytes, tx1Effects: $tx1Effects)';
  }
}

/// @nodoc
abstract mixin class $VotingDelegationSetupCopyWith<$Res> {
  factory $VotingDelegationSetupCopyWith(VotingDelegationSetup value,
          $Res Function(VotingDelegationSetup) _then) =
      _$VotingDelegationSetupCopyWithImpl;
  @useResult
  $Res call(
      {Uint8List pcztBytes,
      Uint8List pcztSighash,
      Uint8List rk,
      int actionIndex,
      Uint8List actionBytes,
      Uint8List tx1Effects});
}

/// @nodoc
class _$VotingDelegationSetupCopyWithImpl<$Res>
    implements $VotingDelegationSetupCopyWith<$Res> {
  _$VotingDelegationSetupCopyWithImpl(this._self, this._then);

  final VotingDelegationSetup _self;
  final $Res Function(VotingDelegationSetup) _then;

  /// Create a copy of VotingDelegationSetup
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? pcztBytes = null,
    Object? pcztSighash = null,
    Object? rk = null,
    Object? actionIndex = null,
    Object? actionBytes = null,
    Object? tx1Effects = null,
  }) {
    return _then(_self.copyWith(
      pcztBytes: null == pcztBytes
          ? _self.pcztBytes
          : pcztBytes // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      pcztSighash: null == pcztSighash
          ? _self.pcztSighash
          : pcztSighash // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      rk: null == rk
          ? _self.rk
          : rk // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      actionIndex: null == actionIndex
          ? _self.actionIndex
          : actionIndex // ignore: cast_nullable_to_non_nullable
              as int,
      actionBytes: null == actionBytes
          ? _self.actionBytes
          : actionBytes // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      tx1Effects: null == tx1Effects
          ? _self.tx1Effects
          : tx1Effects // ignore: cast_nullable_to_non_nullable
              as Uint8List,
    ));
  }
}

/// Adds pattern-matching-related methods to [VotingDelegationSetup].
extension VotingDelegationSetupPatterns on VotingDelegationSetup {
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
    TResult Function(_VotingDelegationSetup value)? $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _VotingDelegationSetup() when $default != null:
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
    TResult Function(_VotingDelegationSetup value) $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingDelegationSetup():
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
    TResult? Function(_VotingDelegationSetup value)? $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingDelegationSetup() when $default != null:
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
    TResult Function(Uint8List pcztBytes, Uint8List pcztSighash, Uint8List rk,
            int actionIndex, Uint8List actionBytes, Uint8List tx1Effects)?
        $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _VotingDelegationSetup() when $default != null:
        return $default(_that.pcztBytes, _that.pcztSighash, _that.rk,
            _that.actionIndex, _that.actionBytes, _that.tx1Effects);
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
    TResult Function(Uint8List pcztBytes, Uint8List pcztSighash, Uint8List rk,
            int actionIndex, Uint8List actionBytes, Uint8List tx1Effects)
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingDelegationSetup():
        return $default(_that.pcztBytes, _that.pcztSighash, _that.rk,
            _that.actionIndex, _that.actionBytes, _that.tx1Effects);
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
    TResult? Function(Uint8List pcztBytes, Uint8List pcztSighash, Uint8List rk,
            int actionIndex, Uint8List actionBytes, Uint8List tx1Effects)?
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingDelegationSetup() when $default != null:
        return $default(_that.pcztBytes, _that.pcztSighash, _that.rk,
            _that.actionIndex, _that.actionBytes, _that.tx1Effects);
      case _:
        return null;
    }
  }
}

/// @nodoc

class _VotingDelegationSetup implements VotingDelegationSetup {
  const _VotingDelegationSetup(
      {required this.pcztBytes,
      required this.pcztSighash,
      required this.rk,
      required this.actionIndex,
      required this.actionBytes,
      required this.tx1Effects});

  @override
  final Uint8List pcztBytes;
  @override
  final Uint8List pcztSighash;
  @override
  final Uint8List rk;
  @override
  final int actionIndex;
  @override
  final Uint8List actionBytes;
  @override
  final Uint8List tx1Effects;

  /// Create a copy of VotingDelegationSetup
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  _$VotingDelegationSetupCopyWith<_VotingDelegationSetup> get copyWith =>
      __$VotingDelegationSetupCopyWithImpl<_VotingDelegationSetup>(
          this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _VotingDelegationSetup &&
            const DeepCollectionEquality().equals(other.pcztBytes, pcztBytes) &&
            const DeepCollectionEquality()
                .equals(other.pcztSighash, pcztSighash) &&
            const DeepCollectionEquality().equals(other.rk, rk) &&
            (identical(other.actionIndex, actionIndex) ||
                other.actionIndex == actionIndex) &&
            const DeepCollectionEquality()
                .equals(other.actionBytes, actionBytes) &&
            const DeepCollectionEquality()
                .equals(other.tx1Effects, tx1Effects));
  }

  @override
  int get hashCode => Object.hash(
      runtimeType,
      const DeepCollectionEquality().hash(pcztBytes),
      const DeepCollectionEquality().hash(pcztSighash),
      const DeepCollectionEquality().hash(rk),
      actionIndex,
      const DeepCollectionEquality().hash(actionBytes),
      const DeepCollectionEquality().hash(tx1Effects));

  @override
  String toString() {
    return 'VotingDelegationSetup(pcztBytes: $pcztBytes, pcztSighash: $pcztSighash, rk: $rk, actionIndex: $actionIndex, actionBytes: $actionBytes, tx1Effects: $tx1Effects)';
  }
}

/// @nodoc
abstract mixin class _$VotingDelegationSetupCopyWith<$Res>
    implements $VotingDelegationSetupCopyWith<$Res> {
  factory _$VotingDelegationSetupCopyWith(_VotingDelegationSetup value,
          $Res Function(_VotingDelegationSetup) _then) =
      __$VotingDelegationSetupCopyWithImpl;
  @override
  @useResult
  $Res call(
      {Uint8List pcztBytes,
      Uint8List pcztSighash,
      Uint8List rk,
      int actionIndex,
      Uint8List actionBytes,
      Uint8List tx1Effects});
}

/// @nodoc
class __$VotingDelegationSetupCopyWithImpl<$Res>
    implements _$VotingDelegationSetupCopyWith<$Res> {
  __$VotingDelegationSetupCopyWithImpl(this._self, this._then);

  final _VotingDelegationSetup _self;
  final $Res Function(_VotingDelegationSetup) _then;

  /// Create a copy of VotingDelegationSetup
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $Res call({
    Object? pcztBytes = null,
    Object? pcztSighash = null,
    Object? rk = null,
    Object? actionIndex = null,
    Object? actionBytes = null,
    Object? tx1Effects = null,
  }) {
    return _then(_VotingDelegationSetup(
      pcztBytes: null == pcztBytes
          ? _self.pcztBytes
          : pcztBytes // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      pcztSighash: null == pcztSighash
          ? _self.pcztSighash
          : pcztSighash // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      rk: null == rk
          ? _self.rk
          : rk // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      actionIndex: null == actionIndex
          ? _self.actionIndex
          : actionIndex // ignore: cast_nullable_to_non_nullable
              as int,
      actionBytes: null == actionBytes
          ? _self.actionBytes
          : actionBytes // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      tx1Effects: null == tx1Effects
          ? _self.tx1Effects
          : tx1Effects // ignore: cast_nullable_to_non_nullable
              as Uint8List,
    ));
  }
}

/// @nodoc
mixin _$VotingDelegationStatus {
  int get bundleIndex;
  String get phase;
  String? get txHash;

  /// Create a copy of VotingDelegationStatus
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $VotingDelegationStatusCopyWith<VotingDelegationStatus> get copyWith =>
      _$VotingDelegationStatusCopyWithImpl<VotingDelegationStatus>(
          this as VotingDelegationStatus, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is VotingDelegationStatus &&
            (identical(other.bundleIndex, bundleIndex) ||
                other.bundleIndex == bundleIndex) &&
            (identical(other.phase, phase) || other.phase == phase) &&
            (identical(other.txHash, txHash) || other.txHash == txHash));
  }

  @override
  int get hashCode => Object.hash(runtimeType, bundleIndex, phase, txHash);

  @override
  String toString() {
    return 'VotingDelegationStatus(bundleIndex: $bundleIndex, phase: $phase, txHash: $txHash)';
  }
}

/// @nodoc
abstract mixin class $VotingDelegationStatusCopyWith<$Res> {
  factory $VotingDelegationStatusCopyWith(VotingDelegationStatus value,
          $Res Function(VotingDelegationStatus) _then) =
      _$VotingDelegationStatusCopyWithImpl;
  @useResult
  $Res call({int bundleIndex, String phase, String? txHash});
}

/// @nodoc
class _$VotingDelegationStatusCopyWithImpl<$Res>
    implements $VotingDelegationStatusCopyWith<$Res> {
  _$VotingDelegationStatusCopyWithImpl(this._self, this._then);

  final VotingDelegationStatus _self;
  final $Res Function(VotingDelegationStatus) _then;

  /// Create a copy of VotingDelegationStatus
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? bundleIndex = null,
    Object? phase = null,
    Object? txHash = freezed,
  }) {
    return _then(_self.copyWith(
      bundleIndex: null == bundleIndex
          ? _self.bundleIndex
          : bundleIndex // ignore: cast_nullable_to_non_nullable
              as int,
      phase: null == phase
          ? _self.phase
          : phase // ignore: cast_nullable_to_non_nullable
              as String,
      txHash: freezed == txHash
          ? _self.txHash
          : txHash // ignore: cast_nullable_to_non_nullable
              as String?,
    ));
  }
}

/// Adds pattern-matching-related methods to [VotingDelegationStatus].
extension VotingDelegationStatusPatterns on VotingDelegationStatus {
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
    TResult Function(_VotingDelegationStatus value)? $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _VotingDelegationStatus() when $default != null:
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
    TResult Function(_VotingDelegationStatus value) $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingDelegationStatus():
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
    TResult? Function(_VotingDelegationStatus value)? $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingDelegationStatus() when $default != null:
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
    TResult Function(int bundleIndex, String phase, String? txHash)? $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _VotingDelegationStatus() when $default != null:
        return $default(_that.bundleIndex, _that.phase, _that.txHash);
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
    TResult Function(int bundleIndex, String phase, String? txHash) $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingDelegationStatus():
        return $default(_that.bundleIndex, _that.phase, _that.txHash);
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
    TResult? Function(int bundleIndex, String phase, String? txHash)? $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingDelegationStatus() when $default != null:
        return $default(_that.bundleIndex, _that.phase, _that.txHash);
      case _:
        return null;
    }
  }
}

/// @nodoc

class _VotingDelegationStatus implements VotingDelegationStatus {
  const _VotingDelegationStatus(
      {required this.bundleIndex, required this.phase, this.txHash});

  @override
  final int bundleIndex;
  @override
  final String phase;
  @override
  final String? txHash;

  /// Create a copy of VotingDelegationStatus
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  _$VotingDelegationStatusCopyWith<_VotingDelegationStatus> get copyWith =>
      __$VotingDelegationStatusCopyWithImpl<_VotingDelegationStatus>(
          this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _VotingDelegationStatus &&
            (identical(other.bundleIndex, bundleIndex) ||
                other.bundleIndex == bundleIndex) &&
            (identical(other.phase, phase) || other.phase == phase) &&
            (identical(other.txHash, txHash) || other.txHash == txHash));
  }

  @override
  int get hashCode => Object.hash(runtimeType, bundleIndex, phase, txHash);

  @override
  String toString() {
    return 'VotingDelegationStatus(bundleIndex: $bundleIndex, phase: $phase, txHash: $txHash)';
  }
}

/// @nodoc
abstract mixin class _$VotingDelegationStatusCopyWith<$Res>
    implements $VotingDelegationStatusCopyWith<$Res> {
  factory _$VotingDelegationStatusCopyWith(_VotingDelegationStatus value,
          $Res Function(_VotingDelegationStatus) _then) =
      __$VotingDelegationStatusCopyWithImpl;
  @override
  @useResult
  $Res call({int bundleIndex, String phase, String? txHash});
}

/// @nodoc
class __$VotingDelegationStatusCopyWithImpl<$Res>
    implements _$VotingDelegationStatusCopyWith<$Res> {
  __$VotingDelegationStatusCopyWithImpl(this._self, this._then);

  final _VotingDelegationStatus _self;
  final $Res Function(_VotingDelegationStatus) _then;

  /// Create a copy of VotingDelegationStatus
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $Res call({
    Object? bundleIndex = null,
    Object? phase = null,
    Object? txHash = freezed,
  }) {
    return _then(_VotingDelegationStatus(
      bundleIndex: null == bundleIndex
          ? _self.bundleIndex
          : bundleIndex // ignore: cast_nullable_to_non_nullable
              as int,
      phase: null == phase
          ? _self.phase
          : phase // ignore: cast_nullable_to_non_nullable
              as String,
      txHash: freezed == txHash
          ? _self.txHash
          : txHash // ignore: cast_nullable_to_non_nullable
              as String?,
    ));
  }
}

/// @nodoc
mixin _$VotingDelegationSubmission {
  Uint8List get proof;
  Uint8List get rk;
  Uint8List get nfSigned;
  Uint8List get cmxNew;
  Uint8List get govComm;
  List<Uint8List> get govNullifiers;
  Uint8List get alpha;
  String get voteRoundId;
  Uint8List get spendAuthSig;
  Uint8List get sighash;
  Uint8List get tx1Effects;

  /// Create a copy of VotingDelegationSubmission
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $VotingDelegationSubmissionCopyWith<VotingDelegationSubmission>
      get copyWith =>
          _$VotingDelegationSubmissionCopyWithImpl<VotingDelegationSubmission>(
              this as VotingDelegationSubmission, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is VotingDelegationSubmission &&
            const DeepCollectionEquality().equals(other.proof, proof) &&
            const DeepCollectionEquality().equals(other.rk, rk) &&
            const DeepCollectionEquality().equals(other.nfSigned, nfSigned) &&
            const DeepCollectionEquality().equals(other.cmxNew, cmxNew) &&
            const DeepCollectionEquality().equals(other.govComm, govComm) &&
            const DeepCollectionEquality()
                .equals(other.govNullifiers, govNullifiers) &&
            const DeepCollectionEquality().equals(other.alpha, alpha) &&
            (identical(other.voteRoundId, voteRoundId) ||
                other.voteRoundId == voteRoundId) &&
            const DeepCollectionEquality()
                .equals(other.spendAuthSig, spendAuthSig) &&
            const DeepCollectionEquality().equals(other.sighash, sighash) &&
            const DeepCollectionEquality()
                .equals(other.tx1Effects, tx1Effects));
  }

  @override
  int get hashCode => Object.hash(
      runtimeType,
      const DeepCollectionEquality().hash(proof),
      const DeepCollectionEquality().hash(rk),
      const DeepCollectionEquality().hash(nfSigned),
      const DeepCollectionEquality().hash(cmxNew),
      const DeepCollectionEquality().hash(govComm),
      const DeepCollectionEquality().hash(govNullifiers),
      const DeepCollectionEquality().hash(alpha),
      voteRoundId,
      const DeepCollectionEquality().hash(spendAuthSig),
      const DeepCollectionEquality().hash(sighash),
      const DeepCollectionEquality().hash(tx1Effects));

  @override
  String toString() {
    return 'VotingDelegationSubmission(proof: $proof, rk: $rk, nfSigned: $nfSigned, cmxNew: $cmxNew, govComm: $govComm, govNullifiers: $govNullifiers, alpha: $alpha, voteRoundId: $voteRoundId, spendAuthSig: $spendAuthSig, sighash: $sighash, tx1Effects: $tx1Effects)';
  }
}

/// @nodoc
abstract mixin class $VotingDelegationSubmissionCopyWith<$Res> {
  factory $VotingDelegationSubmissionCopyWith(VotingDelegationSubmission value,
          $Res Function(VotingDelegationSubmission) _then) =
      _$VotingDelegationSubmissionCopyWithImpl;
  @useResult
  $Res call(
      {Uint8List proof,
      Uint8List rk,
      Uint8List nfSigned,
      Uint8List cmxNew,
      Uint8List govComm,
      List<Uint8List> govNullifiers,
      Uint8List alpha,
      String voteRoundId,
      Uint8List spendAuthSig,
      Uint8List sighash,
      Uint8List tx1Effects});
}

/// @nodoc
class _$VotingDelegationSubmissionCopyWithImpl<$Res>
    implements $VotingDelegationSubmissionCopyWith<$Res> {
  _$VotingDelegationSubmissionCopyWithImpl(this._self, this._then);

  final VotingDelegationSubmission _self;
  final $Res Function(VotingDelegationSubmission) _then;

  /// Create a copy of VotingDelegationSubmission
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? proof = null,
    Object? rk = null,
    Object? nfSigned = null,
    Object? cmxNew = null,
    Object? govComm = null,
    Object? govNullifiers = null,
    Object? alpha = null,
    Object? voteRoundId = null,
    Object? spendAuthSig = null,
    Object? sighash = null,
    Object? tx1Effects = null,
  }) {
    return _then(_self.copyWith(
      proof: null == proof
          ? _self.proof
          : proof // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      rk: null == rk
          ? _self.rk
          : rk // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      nfSigned: null == nfSigned
          ? _self.nfSigned
          : nfSigned // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      cmxNew: null == cmxNew
          ? _self.cmxNew
          : cmxNew // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      govComm: null == govComm
          ? _self.govComm
          : govComm // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      govNullifiers: null == govNullifiers
          ? _self.govNullifiers
          : govNullifiers // ignore: cast_nullable_to_non_nullable
              as List<Uint8List>,
      alpha: null == alpha
          ? _self.alpha
          : alpha // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      voteRoundId: null == voteRoundId
          ? _self.voteRoundId
          : voteRoundId // ignore: cast_nullable_to_non_nullable
              as String,
      spendAuthSig: null == spendAuthSig
          ? _self.spendAuthSig
          : spendAuthSig // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      sighash: null == sighash
          ? _self.sighash
          : sighash // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      tx1Effects: null == tx1Effects
          ? _self.tx1Effects
          : tx1Effects // ignore: cast_nullable_to_non_nullable
              as Uint8List,
    ));
  }
}

/// Adds pattern-matching-related methods to [VotingDelegationSubmission].
extension VotingDelegationSubmissionPatterns on VotingDelegationSubmission {
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
    TResult Function(_VotingDelegationSubmission value)? $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _VotingDelegationSubmission() when $default != null:
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
    TResult Function(_VotingDelegationSubmission value) $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingDelegationSubmission():
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
    TResult? Function(_VotingDelegationSubmission value)? $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingDelegationSubmission() when $default != null:
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
    TResult Function(
            Uint8List proof,
            Uint8List rk,
            Uint8List nfSigned,
            Uint8List cmxNew,
            Uint8List govComm,
            List<Uint8List> govNullifiers,
            Uint8List alpha,
            String voteRoundId,
            Uint8List spendAuthSig,
            Uint8List sighash,
            Uint8List tx1Effects)?
        $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _VotingDelegationSubmission() when $default != null:
        return $default(
            _that.proof,
            _that.rk,
            _that.nfSigned,
            _that.cmxNew,
            _that.govComm,
            _that.govNullifiers,
            _that.alpha,
            _that.voteRoundId,
            _that.spendAuthSig,
            _that.sighash,
            _that.tx1Effects);
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
    TResult Function(
            Uint8List proof,
            Uint8List rk,
            Uint8List nfSigned,
            Uint8List cmxNew,
            Uint8List govComm,
            List<Uint8List> govNullifiers,
            Uint8List alpha,
            String voteRoundId,
            Uint8List spendAuthSig,
            Uint8List sighash,
            Uint8List tx1Effects)
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingDelegationSubmission():
        return $default(
            _that.proof,
            _that.rk,
            _that.nfSigned,
            _that.cmxNew,
            _that.govComm,
            _that.govNullifiers,
            _that.alpha,
            _that.voteRoundId,
            _that.spendAuthSig,
            _that.sighash,
            _that.tx1Effects);
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
    TResult? Function(
            Uint8List proof,
            Uint8List rk,
            Uint8List nfSigned,
            Uint8List cmxNew,
            Uint8List govComm,
            List<Uint8List> govNullifiers,
            Uint8List alpha,
            String voteRoundId,
            Uint8List spendAuthSig,
            Uint8List sighash,
            Uint8List tx1Effects)?
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingDelegationSubmission() when $default != null:
        return $default(
            _that.proof,
            _that.rk,
            _that.nfSigned,
            _that.cmxNew,
            _that.govComm,
            _that.govNullifiers,
            _that.alpha,
            _that.voteRoundId,
            _that.spendAuthSig,
            _that.sighash,
            _that.tx1Effects);
      case _:
        return null;
    }
  }
}

/// @nodoc

class _VotingDelegationSubmission implements VotingDelegationSubmission {
  const _VotingDelegationSubmission(
      {required this.proof,
      required this.rk,
      required this.nfSigned,
      required this.cmxNew,
      required this.govComm,
      required final List<Uint8List> govNullifiers,
      required this.alpha,
      required this.voteRoundId,
      required this.spendAuthSig,
      required this.sighash,
      required this.tx1Effects})
      : _govNullifiers = govNullifiers;

  @override
  final Uint8List proof;
  @override
  final Uint8List rk;
  @override
  final Uint8List nfSigned;
  @override
  final Uint8List cmxNew;
  @override
  final Uint8List govComm;
  final List<Uint8List> _govNullifiers;
  @override
  List<Uint8List> get govNullifiers {
    if (_govNullifiers is EqualUnmodifiableListView) return _govNullifiers;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_govNullifiers);
  }

  @override
  final Uint8List alpha;
  @override
  final String voteRoundId;
  @override
  final Uint8List spendAuthSig;
  @override
  final Uint8List sighash;
  @override
  final Uint8List tx1Effects;

  /// Create a copy of VotingDelegationSubmission
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  _$VotingDelegationSubmissionCopyWith<_VotingDelegationSubmission>
      get copyWith => __$VotingDelegationSubmissionCopyWithImpl<
          _VotingDelegationSubmission>(this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _VotingDelegationSubmission &&
            const DeepCollectionEquality().equals(other.proof, proof) &&
            const DeepCollectionEquality().equals(other.rk, rk) &&
            const DeepCollectionEquality().equals(other.nfSigned, nfSigned) &&
            const DeepCollectionEquality().equals(other.cmxNew, cmxNew) &&
            const DeepCollectionEquality().equals(other.govComm, govComm) &&
            const DeepCollectionEquality()
                .equals(other._govNullifiers, _govNullifiers) &&
            const DeepCollectionEquality().equals(other.alpha, alpha) &&
            (identical(other.voteRoundId, voteRoundId) ||
                other.voteRoundId == voteRoundId) &&
            const DeepCollectionEquality()
                .equals(other.spendAuthSig, spendAuthSig) &&
            const DeepCollectionEquality().equals(other.sighash, sighash) &&
            const DeepCollectionEquality()
                .equals(other.tx1Effects, tx1Effects));
  }

  @override
  int get hashCode => Object.hash(
      runtimeType,
      const DeepCollectionEquality().hash(proof),
      const DeepCollectionEquality().hash(rk),
      const DeepCollectionEquality().hash(nfSigned),
      const DeepCollectionEquality().hash(cmxNew),
      const DeepCollectionEquality().hash(govComm),
      const DeepCollectionEquality().hash(_govNullifiers),
      const DeepCollectionEquality().hash(alpha),
      voteRoundId,
      const DeepCollectionEquality().hash(spendAuthSig),
      const DeepCollectionEquality().hash(sighash),
      const DeepCollectionEquality().hash(tx1Effects));

  @override
  String toString() {
    return 'VotingDelegationSubmission(proof: $proof, rk: $rk, nfSigned: $nfSigned, cmxNew: $cmxNew, govComm: $govComm, govNullifiers: $govNullifiers, alpha: $alpha, voteRoundId: $voteRoundId, spendAuthSig: $spendAuthSig, sighash: $sighash, tx1Effects: $tx1Effects)';
  }
}

/// @nodoc
abstract mixin class _$VotingDelegationSubmissionCopyWith<$Res>
    implements $VotingDelegationSubmissionCopyWith<$Res> {
  factory _$VotingDelegationSubmissionCopyWith(
          _VotingDelegationSubmission value,
          $Res Function(_VotingDelegationSubmission) _then) =
      __$VotingDelegationSubmissionCopyWithImpl;
  @override
  @useResult
  $Res call(
      {Uint8List proof,
      Uint8List rk,
      Uint8List nfSigned,
      Uint8List cmxNew,
      Uint8List govComm,
      List<Uint8List> govNullifiers,
      Uint8List alpha,
      String voteRoundId,
      Uint8List spendAuthSig,
      Uint8List sighash,
      Uint8List tx1Effects});
}

/// @nodoc
class __$VotingDelegationSubmissionCopyWithImpl<$Res>
    implements _$VotingDelegationSubmissionCopyWith<$Res> {
  __$VotingDelegationSubmissionCopyWithImpl(this._self, this._then);

  final _VotingDelegationSubmission _self;
  final $Res Function(_VotingDelegationSubmission) _then;

  /// Create a copy of VotingDelegationSubmission
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $Res call({
    Object? proof = null,
    Object? rk = null,
    Object? nfSigned = null,
    Object? cmxNew = null,
    Object? govComm = null,
    Object? govNullifiers = null,
    Object? alpha = null,
    Object? voteRoundId = null,
    Object? spendAuthSig = null,
    Object? sighash = null,
    Object? tx1Effects = null,
  }) {
    return _then(_VotingDelegationSubmission(
      proof: null == proof
          ? _self.proof
          : proof // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      rk: null == rk
          ? _self.rk
          : rk // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      nfSigned: null == nfSigned
          ? _self.nfSigned
          : nfSigned // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      cmxNew: null == cmxNew
          ? _self.cmxNew
          : cmxNew // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      govComm: null == govComm
          ? _self.govComm
          : govComm // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      govNullifiers: null == govNullifiers
          ? _self._govNullifiers
          : govNullifiers // ignore: cast_nullable_to_non_nullable
              as List<Uint8List>,
      alpha: null == alpha
          ? _self.alpha
          : alpha // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      voteRoundId: null == voteRoundId
          ? _self.voteRoundId
          : voteRoundId // ignore: cast_nullable_to_non_nullable
              as String,
      spendAuthSig: null == spendAuthSig
          ? _self.spendAuthSig
          : spendAuthSig // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      sighash: null == sighash
          ? _self.sighash
          : sighash // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      tx1Effects: null == tx1Effects
          ? _self.tx1Effects
          : tx1Effects // ignore: cast_nullable_to_non_nullable
              as Uint8List,
    ));
  }
}

/// @nodoc
mixin _$VotingEncryptedShare {
  Uint8List get c1;
  Uint8List get c2;
  int get shareIndex;

  /// Create a copy of VotingEncryptedShare
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $VotingEncryptedShareCopyWith<VotingEncryptedShare> get copyWith =>
      _$VotingEncryptedShareCopyWithImpl<VotingEncryptedShare>(
          this as VotingEncryptedShare, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is VotingEncryptedShare &&
            const DeepCollectionEquality().equals(other.c1, c1) &&
            const DeepCollectionEquality().equals(other.c2, c2) &&
            (identical(other.shareIndex, shareIndex) ||
                other.shareIndex == shareIndex));
  }

  @override
  int get hashCode => Object.hash(
      runtimeType,
      const DeepCollectionEquality().hash(c1),
      const DeepCollectionEquality().hash(c2),
      shareIndex);

  @override
  String toString() {
    return 'VotingEncryptedShare(c1: $c1, c2: $c2, shareIndex: $shareIndex)';
  }
}

/// @nodoc
abstract mixin class $VotingEncryptedShareCopyWith<$Res> {
  factory $VotingEncryptedShareCopyWith(VotingEncryptedShare value,
          $Res Function(VotingEncryptedShare) _then) =
      _$VotingEncryptedShareCopyWithImpl;
  @useResult
  $Res call({Uint8List c1, Uint8List c2, int shareIndex});
}

/// @nodoc
class _$VotingEncryptedShareCopyWithImpl<$Res>
    implements $VotingEncryptedShareCopyWith<$Res> {
  _$VotingEncryptedShareCopyWithImpl(this._self, this._then);

  final VotingEncryptedShare _self;
  final $Res Function(VotingEncryptedShare) _then;

  /// Create a copy of VotingEncryptedShare
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? c1 = null,
    Object? c2 = null,
    Object? shareIndex = null,
  }) {
    return _then(_self.copyWith(
      c1: null == c1
          ? _self.c1
          : c1 // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      c2: null == c2
          ? _self.c2
          : c2 // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      shareIndex: null == shareIndex
          ? _self.shareIndex
          : shareIndex // ignore: cast_nullable_to_non_nullable
              as int,
    ));
  }
}

/// Adds pattern-matching-related methods to [VotingEncryptedShare].
extension VotingEncryptedSharePatterns on VotingEncryptedShare {
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
    TResult Function(_VotingEncryptedShare value)? $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _VotingEncryptedShare() when $default != null:
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
    TResult Function(_VotingEncryptedShare value) $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingEncryptedShare():
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
    TResult? Function(_VotingEncryptedShare value)? $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingEncryptedShare() when $default != null:
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
    TResult Function(Uint8List c1, Uint8List c2, int shareIndex)? $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _VotingEncryptedShare() when $default != null:
        return $default(_that.c1, _that.c2, _that.shareIndex);
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
    TResult Function(Uint8List c1, Uint8List c2, int shareIndex) $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingEncryptedShare():
        return $default(_that.c1, _that.c2, _that.shareIndex);
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
    TResult? Function(Uint8List c1, Uint8List c2, int shareIndex)? $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingEncryptedShare() when $default != null:
        return $default(_that.c1, _that.c2, _that.shareIndex);
      case _:
        return null;
    }
  }
}

/// @nodoc

class _VotingEncryptedShare implements VotingEncryptedShare {
  const _VotingEncryptedShare(
      {required this.c1, required this.c2, required this.shareIndex});

  @override
  final Uint8List c1;
  @override
  final Uint8List c2;
  @override
  final int shareIndex;

  /// Create a copy of VotingEncryptedShare
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  _$VotingEncryptedShareCopyWith<_VotingEncryptedShare> get copyWith =>
      __$VotingEncryptedShareCopyWithImpl<_VotingEncryptedShare>(
          this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _VotingEncryptedShare &&
            const DeepCollectionEquality().equals(other.c1, c1) &&
            const DeepCollectionEquality().equals(other.c2, c2) &&
            (identical(other.shareIndex, shareIndex) ||
                other.shareIndex == shareIndex));
  }

  @override
  int get hashCode => Object.hash(
      runtimeType,
      const DeepCollectionEquality().hash(c1),
      const DeepCollectionEquality().hash(c2),
      shareIndex);

  @override
  String toString() {
    return 'VotingEncryptedShare(c1: $c1, c2: $c2, shareIndex: $shareIndex)';
  }
}

/// @nodoc
abstract mixin class _$VotingEncryptedShareCopyWith<$Res>
    implements $VotingEncryptedShareCopyWith<$Res> {
  factory _$VotingEncryptedShareCopyWith(_VotingEncryptedShare value,
          $Res Function(_VotingEncryptedShare) _then) =
      __$VotingEncryptedShareCopyWithImpl;
  @override
  @useResult
  $Res call({Uint8List c1, Uint8List c2, int shareIndex});
}

/// @nodoc
class __$VotingEncryptedShareCopyWithImpl<$Res>
    implements _$VotingEncryptedShareCopyWith<$Res> {
  __$VotingEncryptedShareCopyWithImpl(this._self, this._then);

  final _VotingEncryptedShare _self;
  final $Res Function(_VotingEncryptedShare) _then;

  /// Create a copy of VotingEncryptedShare
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $Res call({
    Object? c1 = null,
    Object? c2 = null,
    Object? shareIndex = null,
  }) {
    return _then(_VotingEncryptedShare(
      c1: null == c1
          ? _self.c1
          : c1 // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      c2: null == c2
          ? _self.c2
          : c2 // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      shareIndex: null == shareIndex
          ? _self.shareIndex
          : shareIndex // ignore: cast_nullable_to_non_nullable
              as int,
    ));
  }
}

/// @nodoc
mixin _$VotingNextStep {
  String get kind;
  int get bundleIndex;
  int get proposalId;
  int get choice;
  int get shareIndex;

  /// Create a copy of VotingNextStep
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $VotingNextStepCopyWith<VotingNextStep> get copyWith =>
      _$VotingNextStepCopyWithImpl<VotingNextStep>(
          this as VotingNextStep, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is VotingNextStep &&
            (identical(other.kind, kind) || other.kind == kind) &&
            (identical(other.bundleIndex, bundleIndex) ||
                other.bundleIndex == bundleIndex) &&
            (identical(other.proposalId, proposalId) ||
                other.proposalId == proposalId) &&
            (identical(other.choice, choice) || other.choice == choice) &&
            (identical(other.shareIndex, shareIndex) ||
                other.shareIndex == shareIndex));
  }

  @override
  int get hashCode => Object.hash(
      runtimeType, kind, bundleIndex, proposalId, choice, shareIndex);

  @override
  String toString() {
    return 'VotingNextStep(kind: $kind, bundleIndex: $bundleIndex, proposalId: $proposalId, choice: $choice, shareIndex: $shareIndex)';
  }
}

/// @nodoc
abstract mixin class $VotingNextStepCopyWith<$Res> {
  factory $VotingNextStepCopyWith(
          VotingNextStep value, $Res Function(VotingNextStep) _then) =
      _$VotingNextStepCopyWithImpl;
  @useResult
  $Res call(
      {String kind,
      int bundleIndex,
      int proposalId,
      int choice,
      int shareIndex});
}

/// @nodoc
class _$VotingNextStepCopyWithImpl<$Res>
    implements $VotingNextStepCopyWith<$Res> {
  _$VotingNextStepCopyWithImpl(this._self, this._then);

  final VotingNextStep _self;
  final $Res Function(VotingNextStep) _then;

  /// Create a copy of VotingNextStep
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? kind = null,
    Object? bundleIndex = null,
    Object? proposalId = null,
    Object? choice = null,
    Object? shareIndex = null,
  }) {
    return _then(_self.copyWith(
      kind: null == kind
          ? _self.kind
          : kind // ignore: cast_nullable_to_non_nullable
              as String,
      bundleIndex: null == bundleIndex
          ? _self.bundleIndex
          : bundleIndex // ignore: cast_nullable_to_non_nullable
              as int,
      proposalId: null == proposalId
          ? _self.proposalId
          : proposalId // ignore: cast_nullable_to_non_nullable
              as int,
      choice: null == choice
          ? _self.choice
          : choice // ignore: cast_nullable_to_non_nullable
              as int,
      shareIndex: null == shareIndex
          ? _self.shareIndex
          : shareIndex // ignore: cast_nullable_to_non_nullable
              as int,
    ));
  }
}

/// Adds pattern-matching-related methods to [VotingNextStep].
extension VotingNextStepPatterns on VotingNextStep {
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
    TResult Function(_VotingNextStep value)? $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _VotingNextStep() when $default != null:
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
    TResult Function(_VotingNextStep value) $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingNextStep():
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
    TResult? Function(_VotingNextStep value)? $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingNextStep() when $default != null:
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
    TResult Function(String kind, int bundleIndex, int proposalId, int choice,
            int shareIndex)?
        $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _VotingNextStep() when $default != null:
        return $default(_that.kind, _that.bundleIndex, _that.proposalId,
            _that.choice, _that.shareIndex);
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
    TResult Function(String kind, int bundleIndex, int proposalId, int choice,
            int shareIndex)
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingNextStep():
        return $default(_that.kind, _that.bundleIndex, _that.proposalId,
            _that.choice, _that.shareIndex);
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
    TResult? Function(String kind, int bundleIndex, int proposalId, int choice,
            int shareIndex)?
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingNextStep() when $default != null:
        return $default(_that.kind, _that.bundleIndex, _that.proposalId,
            _that.choice, _that.shareIndex);
      case _:
        return null;
    }
  }
}

/// @nodoc

class _VotingNextStep implements VotingNextStep {
  const _VotingNextStep(
      {required this.kind,
      required this.bundleIndex,
      required this.proposalId,
      required this.choice,
      required this.shareIndex});

  @override
  final String kind;
  @override
  final int bundleIndex;
  @override
  final int proposalId;
  @override
  final int choice;
  @override
  final int shareIndex;

  /// Create a copy of VotingNextStep
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  _$VotingNextStepCopyWith<_VotingNextStep> get copyWith =>
      __$VotingNextStepCopyWithImpl<_VotingNextStep>(this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _VotingNextStep &&
            (identical(other.kind, kind) || other.kind == kind) &&
            (identical(other.bundleIndex, bundleIndex) ||
                other.bundleIndex == bundleIndex) &&
            (identical(other.proposalId, proposalId) ||
                other.proposalId == proposalId) &&
            (identical(other.choice, choice) || other.choice == choice) &&
            (identical(other.shareIndex, shareIndex) ||
                other.shareIndex == shareIndex));
  }

  @override
  int get hashCode => Object.hash(
      runtimeType, kind, bundleIndex, proposalId, choice, shareIndex);

  @override
  String toString() {
    return 'VotingNextStep(kind: $kind, bundleIndex: $bundleIndex, proposalId: $proposalId, choice: $choice, shareIndex: $shareIndex)';
  }
}

/// @nodoc
abstract mixin class _$VotingNextStepCopyWith<$Res>
    implements $VotingNextStepCopyWith<$Res> {
  factory _$VotingNextStepCopyWith(
          _VotingNextStep value, $Res Function(_VotingNextStep) _then) =
      __$VotingNextStepCopyWithImpl;
  @override
  @useResult
  $Res call(
      {String kind,
      int bundleIndex,
      int proposalId,
      int choice,
      int shareIndex});
}

/// @nodoc
class __$VotingNextStepCopyWithImpl<$Res>
    implements _$VotingNextStepCopyWith<$Res> {
  __$VotingNextStepCopyWithImpl(this._self, this._then);

  final _VotingNextStep _self;
  final $Res Function(_VotingNextStep) _then;

  /// Create a copy of VotingNextStep
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $Res call({
    Object? kind = null,
    Object? bundleIndex = null,
    Object? proposalId = null,
    Object? choice = null,
    Object? shareIndex = null,
  }) {
    return _then(_VotingNextStep(
      kind: null == kind
          ? _self.kind
          : kind // ignore: cast_nullable_to_non_nullable
              as String,
      bundleIndex: null == bundleIndex
          ? _self.bundleIndex
          : bundleIndex // ignore: cast_nullable_to_non_nullable
              as int,
      proposalId: null == proposalId
          ? _self.proposalId
          : proposalId // ignore: cast_nullable_to_non_nullable
              as int,
      choice: null == choice
          ? _self.choice
          : choice // ignore: cast_nullable_to_non_nullable
              as int,
      shareIndex: null == shareIndex
          ? _self.shareIndex
          : shareIndex // ignore: cast_nullable_to_non_nullable
              as int,
    ));
  }
}

/// @nodoc
mixin _$VotingPirLayout {
  int get pirDepth;
  int get tier0Layers;
  int get tier1Layers;
  int get polyLen;

  /// Create a copy of VotingPirLayout
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $VotingPirLayoutCopyWith<VotingPirLayout> get copyWith =>
      _$VotingPirLayoutCopyWithImpl<VotingPirLayout>(
          this as VotingPirLayout, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is VotingPirLayout &&
            (identical(other.pirDepth, pirDepth) ||
                other.pirDepth == pirDepth) &&
            (identical(other.tier0Layers, tier0Layers) ||
                other.tier0Layers == tier0Layers) &&
            (identical(other.tier1Layers, tier1Layers) ||
                other.tier1Layers == tier1Layers) &&
            (identical(other.polyLen, polyLen) || other.polyLen == polyLen));
  }

  @override
  int get hashCode =>
      Object.hash(runtimeType, pirDepth, tier0Layers, tier1Layers, polyLen);

  @override
  String toString() {
    return 'VotingPirLayout(pirDepth: $pirDepth, tier0Layers: $tier0Layers, tier1Layers: $tier1Layers, polyLen: $polyLen)';
  }
}

/// @nodoc
abstract mixin class $VotingPirLayoutCopyWith<$Res> {
  factory $VotingPirLayoutCopyWith(
          VotingPirLayout value, $Res Function(VotingPirLayout) _then) =
      _$VotingPirLayoutCopyWithImpl;
  @useResult
  $Res call({int pirDepth, int tier0Layers, int tier1Layers, int polyLen});
}

/// @nodoc
class _$VotingPirLayoutCopyWithImpl<$Res>
    implements $VotingPirLayoutCopyWith<$Res> {
  _$VotingPirLayoutCopyWithImpl(this._self, this._then);

  final VotingPirLayout _self;
  final $Res Function(VotingPirLayout) _then;

  /// Create a copy of VotingPirLayout
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? pirDepth = null,
    Object? tier0Layers = null,
    Object? tier1Layers = null,
    Object? polyLen = null,
  }) {
    return _then(_self.copyWith(
      pirDepth: null == pirDepth
          ? _self.pirDepth
          : pirDepth // ignore: cast_nullable_to_non_nullable
              as int,
      tier0Layers: null == tier0Layers
          ? _self.tier0Layers
          : tier0Layers // ignore: cast_nullable_to_non_nullable
              as int,
      tier1Layers: null == tier1Layers
          ? _self.tier1Layers
          : tier1Layers // ignore: cast_nullable_to_non_nullable
              as int,
      polyLen: null == polyLen
          ? _self.polyLen
          : polyLen // ignore: cast_nullable_to_non_nullable
              as int,
    ));
  }
}

/// Adds pattern-matching-related methods to [VotingPirLayout].
extension VotingPirLayoutPatterns on VotingPirLayout {
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
    TResult Function(_VotingPirLayout value)? $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _VotingPirLayout() when $default != null:
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
    TResult Function(_VotingPirLayout value) $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingPirLayout():
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
    TResult? Function(_VotingPirLayout value)? $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingPirLayout() when $default != null:
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
    TResult Function(
            int pirDepth, int tier0Layers, int tier1Layers, int polyLen)?
        $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _VotingPirLayout() when $default != null:
        return $default(_that.pirDepth, _that.tier0Layers, _that.tier1Layers,
            _that.polyLen);
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
    TResult Function(
            int pirDepth, int tier0Layers, int tier1Layers, int polyLen)
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingPirLayout():
        return $default(_that.pirDepth, _that.tier0Layers, _that.tier1Layers,
            _that.polyLen);
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
    TResult? Function(
            int pirDepth, int tier0Layers, int tier1Layers, int polyLen)?
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingPirLayout() when $default != null:
        return $default(_that.pirDepth, _that.tier0Layers, _that.tier1Layers,
            _that.polyLen);
      case _:
        return null;
    }
  }
}

/// @nodoc

class _VotingPirLayout implements VotingPirLayout {
  const _VotingPirLayout(
      {required this.pirDepth,
      required this.tier0Layers,
      required this.tier1Layers,
      required this.polyLen});

  @override
  final int pirDepth;
  @override
  final int tier0Layers;
  @override
  final int tier1Layers;
  @override
  final int polyLen;

  /// Create a copy of VotingPirLayout
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  _$VotingPirLayoutCopyWith<_VotingPirLayout> get copyWith =>
      __$VotingPirLayoutCopyWithImpl<_VotingPirLayout>(this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _VotingPirLayout &&
            (identical(other.pirDepth, pirDepth) ||
                other.pirDepth == pirDepth) &&
            (identical(other.tier0Layers, tier0Layers) ||
                other.tier0Layers == tier0Layers) &&
            (identical(other.tier1Layers, tier1Layers) ||
                other.tier1Layers == tier1Layers) &&
            (identical(other.polyLen, polyLen) || other.polyLen == polyLen));
  }

  @override
  int get hashCode =>
      Object.hash(runtimeType, pirDepth, tier0Layers, tier1Layers, polyLen);

  @override
  String toString() {
    return 'VotingPirLayout(pirDepth: $pirDepth, tier0Layers: $tier0Layers, tier1Layers: $tier1Layers, polyLen: $polyLen)';
  }
}

/// @nodoc
abstract mixin class _$VotingPirLayoutCopyWith<$Res>
    implements $VotingPirLayoutCopyWith<$Res> {
  factory _$VotingPirLayoutCopyWith(
          _VotingPirLayout value, $Res Function(_VotingPirLayout) _then) =
      __$VotingPirLayoutCopyWithImpl;
  @override
  @useResult
  $Res call({int pirDepth, int tier0Layers, int tier1Layers, int polyLen});
}

/// @nodoc
class __$VotingPirLayoutCopyWithImpl<$Res>
    implements _$VotingPirLayoutCopyWith<$Res> {
  __$VotingPirLayoutCopyWithImpl(this._self, this._then);

  final _VotingPirLayout _self;
  final $Res Function(_VotingPirLayout) _then;

  /// Create a copy of VotingPirLayout
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $Res call({
    Object? pirDepth = null,
    Object? tier0Layers = null,
    Object? tier1Layers = null,
    Object? polyLen = null,
  }) {
    return _then(_VotingPirLayout(
      pirDepth: null == pirDepth
          ? _self.pirDepth
          : pirDepth // ignore: cast_nullable_to_non_nullable
              as int,
      tier0Layers: null == tier0Layers
          ? _self.tier0Layers
          : tier0Layers // ignore: cast_nullable_to_non_nullable
              as int,
      tier1Layers: null == tier1Layers
          ? _self.tier1Layers
          : tier1Layers // ignore: cast_nullable_to_non_nullable
              as int,
      polyLen: null == polyLen
          ? _self.polyLen
          : polyLen // ignore: cast_nullable_to_non_nullable
              as int,
    ));
  }
}

/// @nodoc
mixin _$VotingPreparedInfo {
  String get roundId;
  int get bundleIndex;
  BigInt get eligibleWeightZatoshi;
  BigInt get delegatedWeightZatoshi;
  String get roundName;

  /// Create a copy of VotingPreparedInfo
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $VotingPreparedInfoCopyWith<VotingPreparedInfo> get copyWith =>
      _$VotingPreparedInfoCopyWithImpl<VotingPreparedInfo>(
          this as VotingPreparedInfo, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is VotingPreparedInfo &&
            (identical(other.roundId, roundId) || other.roundId == roundId) &&
            (identical(other.bundleIndex, bundleIndex) ||
                other.bundleIndex == bundleIndex) &&
            (identical(other.eligibleWeightZatoshi, eligibleWeightZatoshi) ||
                other.eligibleWeightZatoshi == eligibleWeightZatoshi) &&
            (identical(other.delegatedWeightZatoshi, delegatedWeightZatoshi) ||
                other.delegatedWeightZatoshi == delegatedWeightZatoshi) &&
            (identical(other.roundName, roundName) ||
                other.roundName == roundName));
  }

  @override
  int get hashCode => Object.hash(runtimeType, roundId, bundleIndex,
      eligibleWeightZatoshi, delegatedWeightZatoshi, roundName);

  @override
  String toString() {
    return 'VotingPreparedInfo(roundId: $roundId, bundleIndex: $bundleIndex, eligibleWeightZatoshi: $eligibleWeightZatoshi, delegatedWeightZatoshi: $delegatedWeightZatoshi, roundName: $roundName)';
  }
}

/// @nodoc
abstract mixin class $VotingPreparedInfoCopyWith<$Res> {
  factory $VotingPreparedInfoCopyWith(
          VotingPreparedInfo value, $Res Function(VotingPreparedInfo) _then) =
      _$VotingPreparedInfoCopyWithImpl;
  @useResult
  $Res call(
      {String roundId,
      int bundleIndex,
      BigInt eligibleWeightZatoshi,
      BigInt delegatedWeightZatoshi,
      String roundName});
}

/// @nodoc
class _$VotingPreparedInfoCopyWithImpl<$Res>
    implements $VotingPreparedInfoCopyWith<$Res> {
  _$VotingPreparedInfoCopyWithImpl(this._self, this._then);

  final VotingPreparedInfo _self;
  final $Res Function(VotingPreparedInfo) _then;

  /// Create a copy of VotingPreparedInfo
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? roundId = null,
    Object? bundleIndex = null,
    Object? eligibleWeightZatoshi = null,
    Object? delegatedWeightZatoshi = null,
    Object? roundName = null,
  }) {
    return _then(_self.copyWith(
      roundId: null == roundId
          ? _self.roundId
          : roundId // ignore: cast_nullable_to_non_nullable
              as String,
      bundleIndex: null == bundleIndex
          ? _self.bundleIndex
          : bundleIndex // ignore: cast_nullable_to_non_nullable
              as int,
      eligibleWeightZatoshi: null == eligibleWeightZatoshi
          ? _self.eligibleWeightZatoshi
          : eligibleWeightZatoshi // ignore: cast_nullable_to_non_nullable
              as BigInt,
      delegatedWeightZatoshi: null == delegatedWeightZatoshi
          ? _self.delegatedWeightZatoshi
          : delegatedWeightZatoshi // ignore: cast_nullable_to_non_nullable
              as BigInt,
      roundName: null == roundName
          ? _self.roundName
          : roundName // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// Adds pattern-matching-related methods to [VotingPreparedInfo].
extension VotingPreparedInfoPatterns on VotingPreparedInfo {
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
    TResult Function(_VotingPreparedInfo value)? $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _VotingPreparedInfo() when $default != null:
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
    TResult Function(_VotingPreparedInfo value) $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingPreparedInfo():
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
    TResult? Function(_VotingPreparedInfo value)? $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingPreparedInfo() when $default != null:
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
    TResult Function(
            String roundId,
            int bundleIndex,
            BigInt eligibleWeightZatoshi,
            BigInt delegatedWeightZatoshi,
            String roundName)?
        $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _VotingPreparedInfo() when $default != null:
        return $default(
            _that.roundId,
            _that.bundleIndex,
            _that.eligibleWeightZatoshi,
            _that.delegatedWeightZatoshi,
            _that.roundName);
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
    TResult Function(
            String roundId,
            int bundleIndex,
            BigInt eligibleWeightZatoshi,
            BigInt delegatedWeightZatoshi,
            String roundName)
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingPreparedInfo():
        return $default(
            _that.roundId,
            _that.bundleIndex,
            _that.eligibleWeightZatoshi,
            _that.delegatedWeightZatoshi,
            _that.roundName);
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
    TResult? Function(
            String roundId,
            int bundleIndex,
            BigInt eligibleWeightZatoshi,
            BigInt delegatedWeightZatoshi,
            String roundName)?
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingPreparedInfo() when $default != null:
        return $default(
            _that.roundId,
            _that.bundleIndex,
            _that.eligibleWeightZatoshi,
            _that.delegatedWeightZatoshi,
            _that.roundName);
      case _:
        return null;
    }
  }
}

/// @nodoc

class _VotingPreparedInfo implements VotingPreparedInfo {
  const _VotingPreparedInfo(
      {required this.roundId,
      required this.bundleIndex,
      required this.eligibleWeightZatoshi,
      required this.delegatedWeightZatoshi,
      required this.roundName});

  @override
  final String roundId;
  @override
  final int bundleIndex;
  @override
  final BigInt eligibleWeightZatoshi;
  @override
  final BigInt delegatedWeightZatoshi;
  @override
  final String roundName;

  /// Create a copy of VotingPreparedInfo
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  _$VotingPreparedInfoCopyWith<_VotingPreparedInfo> get copyWith =>
      __$VotingPreparedInfoCopyWithImpl<_VotingPreparedInfo>(this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _VotingPreparedInfo &&
            (identical(other.roundId, roundId) || other.roundId == roundId) &&
            (identical(other.bundleIndex, bundleIndex) ||
                other.bundleIndex == bundleIndex) &&
            (identical(other.eligibleWeightZatoshi, eligibleWeightZatoshi) ||
                other.eligibleWeightZatoshi == eligibleWeightZatoshi) &&
            (identical(other.delegatedWeightZatoshi, delegatedWeightZatoshi) ||
                other.delegatedWeightZatoshi == delegatedWeightZatoshi) &&
            (identical(other.roundName, roundName) ||
                other.roundName == roundName));
  }

  @override
  int get hashCode => Object.hash(runtimeType, roundId, bundleIndex,
      eligibleWeightZatoshi, delegatedWeightZatoshi, roundName);

  @override
  String toString() {
    return 'VotingPreparedInfo(roundId: $roundId, bundleIndex: $bundleIndex, eligibleWeightZatoshi: $eligibleWeightZatoshi, delegatedWeightZatoshi: $delegatedWeightZatoshi, roundName: $roundName)';
  }
}

/// @nodoc
abstract mixin class _$VotingPreparedInfoCopyWith<$Res>
    implements $VotingPreparedInfoCopyWith<$Res> {
  factory _$VotingPreparedInfoCopyWith(
          _VotingPreparedInfo value, $Res Function(_VotingPreparedInfo) _then) =
      __$VotingPreparedInfoCopyWithImpl;
  @override
  @useResult
  $Res call(
      {String roundId,
      int bundleIndex,
      BigInt eligibleWeightZatoshi,
      BigInt delegatedWeightZatoshi,
      String roundName});
}

/// @nodoc
class __$VotingPreparedInfoCopyWithImpl<$Res>
    implements _$VotingPreparedInfoCopyWith<$Res> {
  __$VotingPreparedInfoCopyWithImpl(this._self, this._then);

  final _VotingPreparedInfo _self;
  final $Res Function(_VotingPreparedInfo) _then;

  /// Create a copy of VotingPreparedInfo
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $Res call({
    Object? roundId = null,
    Object? bundleIndex = null,
    Object? eligibleWeightZatoshi = null,
    Object? delegatedWeightZatoshi = null,
    Object? roundName = null,
  }) {
    return _then(_VotingPreparedInfo(
      roundId: null == roundId
          ? _self.roundId
          : roundId // ignore: cast_nullable_to_non_nullable
              as String,
      bundleIndex: null == bundleIndex
          ? _self.bundleIndex
          : bundleIndex // ignore: cast_nullable_to_non_nullable
              as int,
      eligibleWeightZatoshi: null == eligibleWeightZatoshi
          ? _self.eligibleWeightZatoshi
          : eligibleWeightZatoshi // ignore: cast_nullable_to_non_nullable
              as BigInt,
      delegatedWeightZatoshi: null == delegatedWeightZatoshi
          ? _self.delegatedWeightZatoshi
          : delegatedWeightZatoshi // ignore: cast_nullable_to_non_nullable
              as BigInt,
      roundName: null == roundName
          ? _self.roundName
          : roundName // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// @nodoc
mixin _$VotingRoundInfo {
  String get roundId;
  String get network;
  BigInt get snapshotHeight;
  String? get hotkeyAddress;
  BigInt? get eligibleWeightZatoshi;
  int get bundleCount;
  BigInt get createdAt;

  /// Create a copy of VotingRoundInfo
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $VotingRoundInfoCopyWith<VotingRoundInfo> get copyWith =>
      _$VotingRoundInfoCopyWithImpl<VotingRoundInfo>(
          this as VotingRoundInfo, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is VotingRoundInfo &&
            (identical(other.roundId, roundId) || other.roundId == roundId) &&
            (identical(other.network, network) || other.network == network) &&
            (identical(other.snapshotHeight, snapshotHeight) ||
                other.snapshotHeight == snapshotHeight) &&
            (identical(other.hotkeyAddress, hotkeyAddress) ||
                other.hotkeyAddress == hotkeyAddress) &&
            (identical(other.eligibleWeightZatoshi, eligibleWeightZatoshi) ||
                other.eligibleWeightZatoshi == eligibleWeightZatoshi) &&
            (identical(other.bundleCount, bundleCount) ||
                other.bundleCount == bundleCount) &&
            (identical(other.createdAt, createdAt) ||
                other.createdAt == createdAt));
  }

  @override
  int get hashCode => Object.hash(runtimeType, roundId, network, snapshotHeight,
      hotkeyAddress, eligibleWeightZatoshi, bundleCount, createdAt);

  @override
  String toString() {
    return 'VotingRoundInfo(roundId: $roundId, network: $network, snapshotHeight: $snapshotHeight, hotkeyAddress: $hotkeyAddress, eligibleWeightZatoshi: $eligibleWeightZatoshi, bundleCount: $bundleCount, createdAt: $createdAt)';
  }
}

/// @nodoc
abstract mixin class $VotingRoundInfoCopyWith<$Res> {
  factory $VotingRoundInfoCopyWith(
          VotingRoundInfo value, $Res Function(VotingRoundInfo) _then) =
      _$VotingRoundInfoCopyWithImpl;
  @useResult
  $Res call(
      {String roundId,
      String network,
      BigInt snapshotHeight,
      String? hotkeyAddress,
      BigInt? eligibleWeightZatoshi,
      int bundleCount,
      BigInt createdAt});
}

/// @nodoc
class _$VotingRoundInfoCopyWithImpl<$Res>
    implements $VotingRoundInfoCopyWith<$Res> {
  _$VotingRoundInfoCopyWithImpl(this._self, this._then);

  final VotingRoundInfo _self;
  final $Res Function(VotingRoundInfo) _then;

  /// Create a copy of VotingRoundInfo
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? roundId = null,
    Object? network = null,
    Object? snapshotHeight = null,
    Object? hotkeyAddress = freezed,
    Object? eligibleWeightZatoshi = freezed,
    Object? bundleCount = null,
    Object? createdAt = null,
  }) {
    return _then(_self.copyWith(
      roundId: null == roundId
          ? _self.roundId
          : roundId // ignore: cast_nullable_to_non_nullable
              as String,
      network: null == network
          ? _self.network
          : network // ignore: cast_nullable_to_non_nullable
              as String,
      snapshotHeight: null == snapshotHeight
          ? _self.snapshotHeight
          : snapshotHeight // ignore: cast_nullable_to_non_nullable
              as BigInt,
      hotkeyAddress: freezed == hotkeyAddress
          ? _self.hotkeyAddress
          : hotkeyAddress // ignore: cast_nullable_to_non_nullable
              as String?,
      eligibleWeightZatoshi: freezed == eligibleWeightZatoshi
          ? _self.eligibleWeightZatoshi
          : eligibleWeightZatoshi // ignore: cast_nullable_to_non_nullable
              as BigInt?,
      bundleCount: null == bundleCount
          ? _self.bundleCount
          : bundleCount // ignore: cast_nullable_to_non_nullable
              as int,
      createdAt: null == createdAt
          ? _self.createdAt
          : createdAt // ignore: cast_nullable_to_non_nullable
              as BigInt,
    ));
  }
}

/// Adds pattern-matching-related methods to [VotingRoundInfo].
extension VotingRoundInfoPatterns on VotingRoundInfo {
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
    TResult Function(_VotingRoundInfo value)? $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _VotingRoundInfo() when $default != null:
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
    TResult Function(_VotingRoundInfo value) $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingRoundInfo():
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
    TResult? Function(_VotingRoundInfo value)? $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingRoundInfo() when $default != null:
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
    TResult Function(
            String roundId,
            String network,
            BigInt snapshotHeight,
            String? hotkeyAddress,
            BigInt? eligibleWeightZatoshi,
            int bundleCount,
            BigInt createdAt)?
        $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _VotingRoundInfo() when $default != null:
        return $default(
            _that.roundId,
            _that.network,
            _that.snapshotHeight,
            _that.hotkeyAddress,
            _that.eligibleWeightZatoshi,
            _that.bundleCount,
            _that.createdAt);
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
    TResult Function(
            String roundId,
            String network,
            BigInt snapshotHeight,
            String? hotkeyAddress,
            BigInt? eligibleWeightZatoshi,
            int bundleCount,
            BigInt createdAt)
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingRoundInfo():
        return $default(
            _that.roundId,
            _that.network,
            _that.snapshotHeight,
            _that.hotkeyAddress,
            _that.eligibleWeightZatoshi,
            _that.bundleCount,
            _that.createdAt);
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
    TResult? Function(
            String roundId,
            String network,
            BigInt snapshotHeight,
            String? hotkeyAddress,
            BigInt? eligibleWeightZatoshi,
            int bundleCount,
            BigInt createdAt)?
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingRoundInfo() when $default != null:
        return $default(
            _that.roundId,
            _that.network,
            _that.snapshotHeight,
            _that.hotkeyAddress,
            _that.eligibleWeightZatoshi,
            _that.bundleCount,
            _that.createdAt);
      case _:
        return null;
    }
  }
}

/// @nodoc

class _VotingRoundInfo implements VotingRoundInfo {
  const _VotingRoundInfo(
      {required this.roundId,
      required this.network,
      required this.snapshotHeight,
      this.hotkeyAddress,
      this.eligibleWeightZatoshi,
      required this.bundleCount,
      required this.createdAt});

  @override
  final String roundId;
  @override
  final String network;
  @override
  final BigInt snapshotHeight;
  @override
  final String? hotkeyAddress;
  @override
  final BigInt? eligibleWeightZatoshi;
  @override
  final int bundleCount;
  @override
  final BigInt createdAt;

  /// Create a copy of VotingRoundInfo
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  _$VotingRoundInfoCopyWith<_VotingRoundInfo> get copyWith =>
      __$VotingRoundInfoCopyWithImpl<_VotingRoundInfo>(this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _VotingRoundInfo &&
            (identical(other.roundId, roundId) || other.roundId == roundId) &&
            (identical(other.network, network) || other.network == network) &&
            (identical(other.snapshotHeight, snapshotHeight) ||
                other.snapshotHeight == snapshotHeight) &&
            (identical(other.hotkeyAddress, hotkeyAddress) ||
                other.hotkeyAddress == hotkeyAddress) &&
            (identical(other.eligibleWeightZatoshi, eligibleWeightZatoshi) ||
                other.eligibleWeightZatoshi == eligibleWeightZatoshi) &&
            (identical(other.bundleCount, bundleCount) ||
                other.bundleCount == bundleCount) &&
            (identical(other.createdAt, createdAt) ||
                other.createdAt == createdAt));
  }

  @override
  int get hashCode => Object.hash(runtimeType, roundId, network, snapshotHeight,
      hotkeyAddress, eligibleWeightZatoshi, bundleCount, createdAt);

  @override
  String toString() {
    return 'VotingRoundInfo(roundId: $roundId, network: $network, snapshotHeight: $snapshotHeight, hotkeyAddress: $hotkeyAddress, eligibleWeightZatoshi: $eligibleWeightZatoshi, bundleCount: $bundleCount, createdAt: $createdAt)';
  }
}

/// @nodoc
abstract mixin class _$VotingRoundInfoCopyWith<$Res>
    implements $VotingRoundInfoCopyWith<$Res> {
  factory _$VotingRoundInfoCopyWith(
          _VotingRoundInfo value, $Res Function(_VotingRoundInfo) _then) =
      __$VotingRoundInfoCopyWithImpl;
  @override
  @useResult
  $Res call(
      {String roundId,
      String network,
      BigInt snapshotHeight,
      String? hotkeyAddress,
      BigInt? eligibleWeightZatoshi,
      int bundleCount,
      BigInt createdAt});
}

/// @nodoc
class __$VotingRoundInfoCopyWithImpl<$Res>
    implements _$VotingRoundInfoCopyWith<$Res> {
  __$VotingRoundInfoCopyWithImpl(this._self, this._then);

  final _VotingRoundInfo _self;
  final $Res Function(_VotingRoundInfo) _then;

  /// Create a copy of VotingRoundInfo
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $Res call({
    Object? roundId = null,
    Object? network = null,
    Object? snapshotHeight = null,
    Object? hotkeyAddress = freezed,
    Object? eligibleWeightZatoshi = freezed,
    Object? bundleCount = null,
    Object? createdAt = null,
  }) {
    return _then(_VotingRoundInfo(
      roundId: null == roundId
          ? _self.roundId
          : roundId // ignore: cast_nullable_to_non_nullable
              as String,
      network: null == network
          ? _self.network
          : network // ignore: cast_nullable_to_non_nullable
              as String,
      snapshotHeight: null == snapshotHeight
          ? _self.snapshotHeight
          : snapshotHeight // ignore: cast_nullable_to_non_nullable
              as BigInt,
      hotkeyAddress: freezed == hotkeyAddress
          ? _self.hotkeyAddress
          : hotkeyAddress // ignore: cast_nullable_to_non_nullable
              as String?,
      eligibleWeightZatoshi: freezed == eligibleWeightZatoshi
          ? _self.eligibleWeightZatoshi
          : eligibleWeightZatoshi // ignore: cast_nullable_to_non_nullable
              as BigInt?,
      bundleCount: null == bundleCount
          ? _self.bundleCount
          : bundleCount // ignore: cast_nullable_to_non_nullable
              as int,
      createdAt: null == createdAt
          ? _self.createdAt
          : createdAt // ignore: cast_nullable_to_non_nullable
              as BigInt,
    ));
  }
}

/// @nodoc
mixin _$VotingRoundPlan {
  String get roundId;
  bool get pendingRecovery;
  List<VotingNextStep> get nextSteps;
  Uint32List get openProposals;
  bool get allDecided;
  List<VotingDelegationStatus> get delegationStatuses;
  bool get blockingRecovery;
  bool get blockingShareWork;
  bool get hotkeyBound;
  bool get completedVoteArtifact;
  bool get completedForDisplay;
  VotingCompletedVoteDisplay? get completedVoteDisplay;
  bool get needsDraftSetup;
  String get primaryAction;

  /// Create a copy of VotingRoundPlan
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $VotingRoundPlanCopyWith<VotingRoundPlan> get copyWith =>
      _$VotingRoundPlanCopyWithImpl<VotingRoundPlan>(
          this as VotingRoundPlan, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is VotingRoundPlan &&
            (identical(other.roundId, roundId) || other.roundId == roundId) &&
            (identical(other.pendingRecovery, pendingRecovery) ||
                other.pendingRecovery == pendingRecovery) &&
            const DeepCollectionEquality().equals(other.nextSteps, nextSteps) &&
            const DeepCollectionEquality()
                .equals(other.openProposals, openProposals) &&
            (identical(other.allDecided, allDecided) ||
                other.allDecided == allDecided) &&
            const DeepCollectionEquality()
                .equals(other.delegationStatuses, delegationStatuses) &&
            (identical(other.blockingRecovery, blockingRecovery) ||
                other.blockingRecovery == blockingRecovery) &&
            (identical(other.blockingShareWork, blockingShareWork) ||
                other.blockingShareWork == blockingShareWork) &&
            (identical(other.hotkeyBound, hotkeyBound) ||
                other.hotkeyBound == hotkeyBound) &&
            (identical(other.completedVoteArtifact, completedVoteArtifact) ||
                other.completedVoteArtifact == completedVoteArtifact) &&
            (identical(other.completedForDisplay, completedForDisplay) ||
                other.completedForDisplay == completedForDisplay) &&
            (identical(other.completedVoteDisplay, completedVoteDisplay) ||
                other.completedVoteDisplay == completedVoteDisplay) &&
            (identical(other.needsDraftSetup, needsDraftSetup) ||
                other.needsDraftSetup == needsDraftSetup) &&
            (identical(other.primaryAction, primaryAction) ||
                other.primaryAction == primaryAction));
  }

  @override
  int get hashCode => Object.hash(
      runtimeType,
      roundId,
      pendingRecovery,
      const DeepCollectionEquality().hash(nextSteps),
      const DeepCollectionEquality().hash(openProposals),
      allDecided,
      const DeepCollectionEquality().hash(delegationStatuses),
      blockingRecovery,
      blockingShareWork,
      hotkeyBound,
      completedVoteArtifact,
      completedForDisplay,
      completedVoteDisplay,
      needsDraftSetup,
      primaryAction);

  @override
  String toString() {
    return 'VotingRoundPlan(roundId: $roundId, pendingRecovery: $pendingRecovery, nextSteps: $nextSteps, openProposals: $openProposals, allDecided: $allDecided, delegationStatuses: $delegationStatuses, blockingRecovery: $blockingRecovery, blockingShareWork: $blockingShareWork, hotkeyBound: $hotkeyBound, completedVoteArtifact: $completedVoteArtifact, completedForDisplay: $completedForDisplay, completedVoteDisplay: $completedVoteDisplay, needsDraftSetup: $needsDraftSetup, primaryAction: $primaryAction)';
  }
}

/// @nodoc
abstract mixin class $VotingRoundPlanCopyWith<$Res> {
  factory $VotingRoundPlanCopyWith(
          VotingRoundPlan value, $Res Function(VotingRoundPlan) _then) =
      _$VotingRoundPlanCopyWithImpl;
  @useResult
  $Res call(
      {String roundId,
      bool pendingRecovery,
      List<VotingNextStep> nextSteps,
      Uint32List openProposals,
      bool allDecided,
      List<VotingDelegationStatus> delegationStatuses,
      bool blockingRecovery,
      bool blockingShareWork,
      bool hotkeyBound,
      bool completedVoteArtifact,
      bool completedForDisplay,
      VotingCompletedVoteDisplay? completedVoteDisplay,
      bool needsDraftSetup,
      String primaryAction});

  $VotingCompletedVoteDisplayCopyWith<$Res>? get completedVoteDisplay;
}

/// @nodoc
class _$VotingRoundPlanCopyWithImpl<$Res>
    implements $VotingRoundPlanCopyWith<$Res> {
  _$VotingRoundPlanCopyWithImpl(this._self, this._then);

  final VotingRoundPlan _self;
  final $Res Function(VotingRoundPlan) _then;

  /// Create a copy of VotingRoundPlan
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? roundId = null,
    Object? pendingRecovery = null,
    Object? nextSteps = null,
    Object? openProposals = null,
    Object? allDecided = null,
    Object? delegationStatuses = null,
    Object? blockingRecovery = null,
    Object? blockingShareWork = null,
    Object? hotkeyBound = null,
    Object? completedVoteArtifact = null,
    Object? completedForDisplay = null,
    Object? completedVoteDisplay = freezed,
    Object? needsDraftSetup = null,
    Object? primaryAction = null,
  }) {
    return _then(_self.copyWith(
      roundId: null == roundId
          ? _self.roundId
          : roundId // ignore: cast_nullable_to_non_nullable
              as String,
      pendingRecovery: null == pendingRecovery
          ? _self.pendingRecovery
          : pendingRecovery // ignore: cast_nullable_to_non_nullable
              as bool,
      nextSteps: null == nextSteps
          ? _self.nextSteps
          : nextSteps // ignore: cast_nullable_to_non_nullable
              as List<VotingNextStep>,
      openProposals: null == openProposals
          ? _self.openProposals
          : openProposals // ignore: cast_nullable_to_non_nullable
              as Uint32List,
      allDecided: null == allDecided
          ? _self.allDecided
          : allDecided // ignore: cast_nullable_to_non_nullable
              as bool,
      delegationStatuses: null == delegationStatuses
          ? _self.delegationStatuses
          : delegationStatuses // ignore: cast_nullable_to_non_nullable
              as List<VotingDelegationStatus>,
      blockingRecovery: null == blockingRecovery
          ? _self.blockingRecovery
          : blockingRecovery // ignore: cast_nullable_to_non_nullable
              as bool,
      blockingShareWork: null == blockingShareWork
          ? _self.blockingShareWork
          : blockingShareWork // ignore: cast_nullable_to_non_nullable
              as bool,
      hotkeyBound: null == hotkeyBound
          ? _self.hotkeyBound
          : hotkeyBound // ignore: cast_nullable_to_non_nullable
              as bool,
      completedVoteArtifact: null == completedVoteArtifact
          ? _self.completedVoteArtifact
          : completedVoteArtifact // ignore: cast_nullable_to_non_nullable
              as bool,
      completedForDisplay: null == completedForDisplay
          ? _self.completedForDisplay
          : completedForDisplay // ignore: cast_nullable_to_non_nullable
              as bool,
      completedVoteDisplay: freezed == completedVoteDisplay
          ? _self.completedVoteDisplay
          : completedVoteDisplay // ignore: cast_nullable_to_non_nullable
              as VotingCompletedVoteDisplay?,
      needsDraftSetup: null == needsDraftSetup
          ? _self.needsDraftSetup
          : needsDraftSetup // ignore: cast_nullable_to_non_nullable
              as bool,
      primaryAction: null == primaryAction
          ? _self.primaryAction
          : primaryAction // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }

  /// Create a copy of VotingRoundPlan
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $VotingCompletedVoteDisplayCopyWith<$Res>? get completedVoteDisplay {
    if (_self.completedVoteDisplay == null) {
      return null;
    }

    return $VotingCompletedVoteDisplayCopyWith<$Res>(
        _self.completedVoteDisplay!, (value) {
      return _then(_self.copyWith(completedVoteDisplay: value));
    });
  }
}

/// Adds pattern-matching-related methods to [VotingRoundPlan].
extension VotingRoundPlanPatterns on VotingRoundPlan {
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
    TResult Function(_VotingRoundPlan value)? $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _VotingRoundPlan() when $default != null:
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
    TResult Function(_VotingRoundPlan value) $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingRoundPlan():
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
    TResult? Function(_VotingRoundPlan value)? $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingRoundPlan() when $default != null:
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
    TResult Function(
            String roundId,
            bool pendingRecovery,
            List<VotingNextStep> nextSteps,
            Uint32List openProposals,
            bool allDecided,
            List<VotingDelegationStatus> delegationStatuses,
            bool blockingRecovery,
            bool blockingShareWork,
            bool hotkeyBound,
            bool completedVoteArtifact,
            bool completedForDisplay,
            VotingCompletedVoteDisplay? completedVoteDisplay,
            bool needsDraftSetup,
            String primaryAction)?
        $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _VotingRoundPlan() when $default != null:
        return $default(
            _that.roundId,
            _that.pendingRecovery,
            _that.nextSteps,
            _that.openProposals,
            _that.allDecided,
            _that.delegationStatuses,
            _that.blockingRecovery,
            _that.blockingShareWork,
            _that.hotkeyBound,
            _that.completedVoteArtifact,
            _that.completedForDisplay,
            _that.completedVoteDisplay,
            _that.needsDraftSetup,
            _that.primaryAction);
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
    TResult Function(
            String roundId,
            bool pendingRecovery,
            List<VotingNextStep> nextSteps,
            Uint32List openProposals,
            bool allDecided,
            List<VotingDelegationStatus> delegationStatuses,
            bool blockingRecovery,
            bool blockingShareWork,
            bool hotkeyBound,
            bool completedVoteArtifact,
            bool completedForDisplay,
            VotingCompletedVoteDisplay? completedVoteDisplay,
            bool needsDraftSetup,
            String primaryAction)
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingRoundPlan():
        return $default(
            _that.roundId,
            _that.pendingRecovery,
            _that.nextSteps,
            _that.openProposals,
            _that.allDecided,
            _that.delegationStatuses,
            _that.blockingRecovery,
            _that.blockingShareWork,
            _that.hotkeyBound,
            _that.completedVoteArtifact,
            _that.completedForDisplay,
            _that.completedVoteDisplay,
            _that.needsDraftSetup,
            _that.primaryAction);
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
    TResult? Function(
            String roundId,
            bool pendingRecovery,
            List<VotingNextStep> nextSteps,
            Uint32List openProposals,
            bool allDecided,
            List<VotingDelegationStatus> delegationStatuses,
            bool blockingRecovery,
            bool blockingShareWork,
            bool hotkeyBound,
            bool completedVoteArtifact,
            bool completedForDisplay,
            VotingCompletedVoteDisplay? completedVoteDisplay,
            bool needsDraftSetup,
            String primaryAction)?
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingRoundPlan() when $default != null:
        return $default(
            _that.roundId,
            _that.pendingRecovery,
            _that.nextSteps,
            _that.openProposals,
            _that.allDecided,
            _that.delegationStatuses,
            _that.blockingRecovery,
            _that.blockingShareWork,
            _that.hotkeyBound,
            _that.completedVoteArtifact,
            _that.completedForDisplay,
            _that.completedVoteDisplay,
            _that.needsDraftSetup,
            _that.primaryAction);
      case _:
        return null;
    }
  }
}

/// @nodoc

class _VotingRoundPlan implements VotingRoundPlan {
  const _VotingRoundPlan(
      {required this.roundId,
      required this.pendingRecovery,
      required final List<VotingNextStep> nextSteps,
      required this.openProposals,
      required this.allDecided,
      required final List<VotingDelegationStatus> delegationStatuses,
      required this.blockingRecovery,
      required this.blockingShareWork,
      required this.hotkeyBound,
      required this.completedVoteArtifact,
      required this.completedForDisplay,
      this.completedVoteDisplay,
      required this.needsDraftSetup,
      required this.primaryAction})
      : _nextSteps = nextSteps,
        _delegationStatuses = delegationStatuses;

  @override
  final String roundId;
  @override
  final bool pendingRecovery;
  final List<VotingNextStep> _nextSteps;
  @override
  List<VotingNextStep> get nextSteps {
    if (_nextSteps is EqualUnmodifiableListView) return _nextSteps;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_nextSteps);
  }

  @override
  final Uint32List openProposals;
  @override
  final bool allDecided;
  final List<VotingDelegationStatus> _delegationStatuses;
  @override
  List<VotingDelegationStatus> get delegationStatuses {
    if (_delegationStatuses is EqualUnmodifiableListView)
      return _delegationStatuses;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_delegationStatuses);
  }

  @override
  final bool blockingRecovery;
  @override
  final bool blockingShareWork;
  @override
  final bool hotkeyBound;
  @override
  final bool completedVoteArtifact;
  @override
  final bool completedForDisplay;
  @override
  final VotingCompletedVoteDisplay? completedVoteDisplay;
  @override
  final bool needsDraftSetup;
  @override
  final String primaryAction;

  /// Create a copy of VotingRoundPlan
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  _$VotingRoundPlanCopyWith<_VotingRoundPlan> get copyWith =>
      __$VotingRoundPlanCopyWithImpl<_VotingRoundPlan>(this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _VotingRoundPlan &&
            (identical(other.roundId, roundId) || other.roundId == roundId) &&
            (identical(other.pendingRecovery, pendingRecovery) ||
                other.pendingRecovery == pendingRecovery) &&
            const DeepCollectionEquality()
                .equals(other._nextSteps, _nextSteps) &&
            const DeepCollectionEquality()
                .equals(other.openProposals, openProposals) &&
            (identical(other.allDecided, allDecided) ||
                other.allDecided == allDecided) &&
            const DeepCollectionEquality()
                .equals(other._delegationStatuses, _delegationStatuses) &&
            (identical(other.blockingRecovery, blockingRecovery) ||
                other.blockingRecovery == blockingRecovery) &&
            (identical(other.blockingShareWork, blockingShareWork) ||
                other.blockingShareWork == blockingShareWork) &&
            (identical(other.hotkeyBound, hotkeyBound) ||
                other.hotkeyBound == hotkeyBound) &&
            (identical(other.completedVoteArtifact, completedVoteArtifact) ||
                other.completedVoteArtifact == completedVoteArtifact) &&
            (identical(other.completedForDisplay, completedForDisplay) ||
                other.completedForDisplay == completedForDisplay) &&
            (identical(other.completedVoteDisplay, completedVoteDisplay) ||
                other.completedVoteDisplay == completedVoteDisplay) &&
            (identical(other.needsDraftSetup, needsDraftSetup) ||
                other.needsDraftSetup == needsDraftSetup) &&
            (identical(other.primaryAction, primaryAction) ||
                other.primaryAction == primaryAction));
  }

  @override
  int get hashCode => Object.hash(
      runtimeType,
      roundId,
      pendingRecovery,
      const DeepCollectionEquality().hash(_nextSteps),
      const DeepCollectionEquality().hash(openProposals),
      allDecided,
      const DeepCollectionEquality().hash(_delegationStatuses),
      blockingRecovery,
      blockingShareWork,
      hotkeyBound,
      completedVoteArtifact,
      completedForDisplay,
      completedVoteDisplay,
      needsDraftSetup,
      primaryAction);

  @override
  String toString() {
    return 'VotingRoundPlan(roundId: $roundId, pendingRecovery: $pendingRecovery, nextSteps: $nextSteps, openProposals: $openProposals, allDecided: $allDecided, delegationStatuses: $delegationStatuses, blockingRecovery: $blockingRecovery, blockingShareWork: $blockingShareWork, hotkeyBound: $hotkeyBound, completedVoteArtifact: $completedVoteArtifact, completedForDisplay: $completedForDisplay, completedVoteDisplay: $completedVoteDisplay, needsDraftSetup: $needsDraftSetup, primaryAction: $primaryAction)';
  }
}

/// @nodoc
abstract mixin class _$VotingRoundPlanCopyWith<$Res>
    implements $VotingRoundPlanCopyWith<$Res> {
  factory _$VotingRoundPlanCopyWith(
          _VotingRoundPlan value, $Res Function(_VotingRoundPlan) _then) =
      __$VotingRoundPlanCopyWithImpl;
  @override
  @useResult
  $Res call(
      {String roundId,
      bool pendingRecovery,
      List<VotingNextStep> nextSteps,
      Uint32List openProposals,
      bool allDecided,
      List<VotingDelegationStatus> delegationStatuses,
      bool blockingRecovery,
      bool blockingShareWork,
      bool hotkeyBound,
      bool completedVoteArtifact,
      bool completedForDisplay,
      VotingCompletedVoteDisplay? completedVoteDisplay,
      bool needsDraftSetup,
      String primaryAction});

  @override
  $VotingCompletedVoteDisplayCopyWith<$Res>? get completedVoteDisplay;
}

/// @nodoc
class __$VotingRoundPlanCopyWithImpl<$Res>
    implements _$VotingRoundPlanCopyWith<$Res> {
  __$VotingRoundPlanCopyWithImpl(this._self, this._then);

  final _VotingRoundPlan _self;
  final $Res Function(_VotingRoundPlan) _then;

  /// Create a copy of VotingRoundPlan
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $Res call({
    Object? roundId = null,
    Object? pendingRecovery = null,
    Object? nextSteps = null,
    Object? openProposals = null,
    Object? allDecided = null,
    Object? delegationStatuses = null,
    Object? blockingRecovery = null,
    Object? blockingShareWork = null,
    Object? hotkeyBound = null,
    Object? completedVoteArtifact = null,
    Object? completedForDisplay = null,
    Object? completedVoteDisplay = freezed,
    Object? needsDraftSetup = null,
    Object? primaryAction = null,
  }) {
    return _then(_VotingRoundPlan(
      roundId: null == roundId
          ? _self.roundId
          : roundId // ignore: cast_nullable_to_non_nullable
              as String,
      pendingRecovery: null == pendingRecovery
          ? _self.pendingRecovery
          : pendingRecovery // ignore: cast_nullable_to_non_nullable
              as bool,
      nextSteps: null == nextSteps
          ? _self._nextSteps
          : nextSteps // ignore: cast_nullable_to_non_nullable
              as List<VotingNextStep>,
      openProposals: null == openProposals
          ? _self.openProposals
          : openProposals // ignore: cast_nullable_to_non_nullable
              as Uint32List,
      allDecided: null == allDecided
          ? _self.allDecided
          : allDecided // ignore: cast_nullable_to_non_nullable
              as bool,
      delegationStatuses: null == delegationStatuses
          ? _self._delegationStatuses
          : delegationStatuses // ignore: cast_nullable_to_non_nullable
              as List<VotingDelegationStatus>,
      blockingRecovery: null == blockingRecovery
          ? _self.blockingRecovery
          : blockingRecovery // ignore: cast_nullable_to_non_nullable
              as bool,
      blockingShareWork: null == blockingShareWork
          ? _self.blockingShareWork
          : blockingShareWork // ignore: cast_nullable_to_non_nullable
              as bool,
      hotkeyBound: null == hotkeyBound
          ? _self.hotkeyBound
          : hotkeyBound // ignore: cast_nullable_to_non_nullable
              as bool,
      completedVoteArtifact: null == completedVoteArtifact
          ? _self.completedVoteArtifact
          : completedVoteArtifact // ignore: cast_nullable_to_non_nullable
              as bool,
      completedForDisplay: null == completedForDisplay
          ? _self.completedForDisplay
          : completedForDisplay // ignore: cast_nullable_to_non_nullable
              as bool,
      completedVoteDisplay: freezed == completedVoteDisplay
          ? _self.completedVoteDisplay
          : completedVoteDisplay // ignore: cast_nullable_to_non_nullable
              as VotingCompletedVoteDisplay?,
      needsDraftSetup: null == needsDraftSetup
          ? _self.needsDraftSetup
          : needsDraftSetup // ignore: cast_nullable_to_non_nullable
              as bool,
      primaryAction: null == primaryAction
          ? _self.primaryAction
          : primaryAction // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }

  /// Create a copy of VotingRoundPlan
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $VotingCompletedVoteDisplayCopyWith<$Res>? get completedVoteDisplay {
    if (_self.completedVoteDisplay == null) {
      return null;
    }

    return $VotingCompletedVoteDisplayCopyWith<$Res>(
        _self.completedVoteDisplay!, (value) {
      return _then(_self.copyWith(completedVoteDisplay: value));
    });
  }
}

/// @nodoc
mixin _$VotingRoundRecovery {
  String get roundId;
  int get bundleCount;
  List<VotingDelegationRecovery> get delegation;
  List<VotingVoteRecovery> get votes;
  List<VotingShareWorkflow> get shares;
  List<VotingShareDelegationRecord> get shareDelegations;
  List<VotingShareDelegationRecord> get unconfirmedShareDelegations;

  /// Create a copy of VotingRoundRecovery
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $VotingRoundRecoveryCopyWith<VotingRoundRecovery> get copyWith =>
      _$VotingRoundRecoveryCopyWithImpl<VotingRoundRecovery>(
          this as VotingRoundRecovery, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is VotingRoundRecovery &&
            (identical(other.roundId, roundId) || other.roundId == roundId) &&
            (identical(other.bundleCount, bundleCount) ||
                other.bundleCount == bundleCount) &&
            const DeepCollectionEquality()
                .equals(other.delegation, delegation) &&
            const DeepCollectionEquality().equals(other.votes, votes) &&
            const DeepCollectionEquality().equals(other.shares, shares) &&
            const DeepCollectionEquality()
                .equals(other.shareDelegations, shareDelegations) &&
            const DeepCollectionEquality().equals(
                other.unconfirmedShareDelegations,
                unconfirmedShareDelegations));
  }

  @override
  int get hashCode => Object.hash(
      runtimeType,
      roundId,
      bundleCount,
      const DeepCollectionEquality().hash(delegation),
      const DeepCollectionEquality().hash(votes),
      const DeepCollectionEquality().hash(shares),
      const DeepCollectionEquality().hash(shareDelegations),
      const DeepCollectionEquality().hash(unconfirmedShareDelegations));

  @override
  String toString() {
    return 'VotingRoundRecovery(roundId: $roundId, bundleCount: $bundleCount, delegation: $delegation, votes: $votes, shares: $shares, shareDelegations: $shareDelegations, unconfirmedShareDelegations: $unconfirmedShareDelegations)';
  }
}

/// @nodoc
abstract mixin class $VotingRoundRecoveryCopyWith<$Res> {
  factory $VotingRoundRecoveryCopyWith(
          VotingRoundRecovery value, $Res Function(VotingRoundRecovery) _then) =
      _$VotingRoundRecoveryCopyWithImpl;
  @useResult
  $Res call(
      {String roundId,
      int bundleCount,
      List<VotingDelegationRecovery> delegation,
      List<VotingVoteRecovery> votes,
      List<VotingShareWorkflow> shares,
      List<VotingShareDelegationRecord> shareDelegations,
      List<VotingShareDelegationRecord> unconfirmedShareDelegations});
}

/// @nodoc
class _$VotingRoundRecoveryCopyWithImpl<$Res>
    implements $VotingRoundRecoveryCopyWith<$Res> {
  _$VotingRoundRecoveryCopyWithImpl(this._self, this._then);

  final VotingRoundRecovery _self;
  final $Res Function(VotingRoundRecovery) _then;

  /// Create a copy of VotingRoundRecovery
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? roundId = null,
    Object? bundleCount = null,
    Object? delegation = null,
    Object? votes = null,
    Object? shares = null,
    Object? shareDelegations = null,
    Object? unconfirmedShareDelegations = null,
  }) {
    return _then(_self.copyWith(
      roundId: null == roundId
          ? _self.roundId
          : roundId // ignore: cast_nullable_to_non_nullable
              as String,
      bundleCount: null == bundleCount
          ? _self.bundleCount
          : bundleCount // ignore: cast_nullable_to_non_nullable
              as int,
      delegation: null == delegation
          ? _self.delegation
          : delegation // ignore: cast_nullable_to_non_nullable
              as List<VotingDelegationRecovery>,
      votes: null == votes
          ? _self.votes
          : votes // ignore: cast_nullable_to_non_nullable
              as List<VotingVoteRecovery>,
      shares: null == shares
          ? _self.shares
          : shares // ignore: cast_nullable_to_non_nullable
              as List<VotingShareWorkflow>,
      shareDelegations: null == shareDelegations
          ? _self.shareDelegations
          : shareDelegations // ignore: cast_nullable_to_non_nullable
              as List<VotingShareDelegationRecord>,
      unconfirmedShareDelegations: null == unconfirmedShareDelegations
          ? _self.unconfirmedShareDelegations
          : unconfirmedShareDelegations // ignore: cast_nullable_to_non_nullable
              as List<VotingShareDelegationRecord>,
    ));
  }
}

/// Adds pattern-matching-related methods to [VotingRoundRecovery].
extension VotingRoundRecoveryPatterns on VotingRoundRecovery {
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
    TResult Function(_VotingRoundRecovery value)? $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _VotingRoundRecovery() when $default != null:
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
    TResult Function(_VotingRoundRecovery value) $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingRoundRecovery():
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
    TResult? Function(_VotingRoundRecovery value)? $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingRoundRecovery() when $default != null:
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
    TResult Function(
            String roundId,
            int bundleCount,
            List<VotingDelegationRecovery> delegation,
            List<VotingVoteRecovery> votes,
            List<VotingShareWorkflow> shares,
            List<VotingShareDelegationRecord> shareDelegations,
            List<VotingShareDelegationRecord> unconfirmedShareDelegations)?
        $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _VotingRoundRecovery() when $default != null:
        return $default(
            _that.roundId,
            _that.bundleCount,
            _that.delegation,
            _that.votes,
            _that.shares,
            _that.shareDelegations,
            _that.unconfirmedShareDelegations);
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
    TResult Function(
            String roundId,
            int bundleCount,
            List<VotingDelegationRecovery> delegation,
            List<VotingVoteRecovery> votes,
            List<VotingShareWorkflow> shares,
            List<VotingShareDelegationRecord> shareDelegations,
            List<VotingShareDelegationRecord> unconfirmedShareDelegations)
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingRoundRecovery():
        return $default(
            _that.roundId,
            _that.bundleCount,
            _that.delegation,
            _that.votes,
            _that.shares,
            _that.shareDelegations,
            _that.unconfirmedShareDelegations);
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
    TResult? Function(
            String roundId,
            int bundleCount,
            List<VotingDelegationRecovery> delegation,
            List<VotingVoteRecovery> votes,
            List<VotingShareWorkflow> shares,
            List<VotingShareDelegationRecord> shareDelegations,
            List<VotingShareDelegationRecord> unconfirmedShareDelegations)?
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingRoundRecovery() when $default != null:
        return $default(
            _that.roundId,
            _that.bundleCount,
            _that.delegation,
            _that.votes,
            _that.shares,
            _that.shareDelegations,
            _that.unconfirmedShareDelegations);
      case _:
        return null;
    }
  }
}

/// @nodoc

class _VotingRoundRecovery implements VotingRoundRecovery {
  const _VotingRoundRecovery(
      {required this.roundId,
      required this.bundleCount,
      required final List<VotingDelegationRecovery> delegation,
      required final List<VotingVoteRecovery> votes,
      required final List<VotingShareWorkflow> shares,
      required final List<VotingShareDelegationRecord> shareDelegations,
      required final List<VotingShareDelegationRecord>
          unconfirmedShareDelegations})
      : _delegation = delegation,
        _votes = votes,
        _shares = shares,
        _shareDelegations = shareDelegations,
        _unconfirmedShareDelegations = unconfirmedShareDelegations;

  @override
  final String roundId;
  @override
  final int bundleCount;
  final List<VotingDelegationRecovery> _delegation;
  @override
  List<VotingDelegationRecovery> get delegation {
    if (_delegation is EqualUnmodifiableListView) return _delegation;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_delegation);
  }

  final List<VotingVoteRecovery> _votes;
  @override
  List<VotingVoteRecovery> get votes {
    if (_votes is EqualUnmodifiableListView) return _votes;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_votes);
  }

  final List<VotingShareWorkflow> _shares;
  @override
  List<VotingShareWorkflow> get shares {
    if (_shares is EqualUnmodifiableListView) return _shares;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_shares);
  }

  final List<VotingShareDelegationRecord> _shareDelegations;
  @override
  List<VotingShareDelegationRecord> get shareDelegations {
    if (_shareDelegations is EqualUnmodifiableListView)
      return _shareDelegations;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_shareDelegations);
  }

  final List<VotingShareDelegationRecord> _unconfirmedShareDelegations;
  @override
  List<VotingShareDelegationRecord> get unconfirmedShareDelegations {
    if (_unconfirmedShareDelegations is EqualUnmodifiableListView)
      return _unconfirmedShareDelegations;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_unconfirmedShareDelegations);
  }

  /// Create a copy of VotingRoundRecovery
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  _$VotingRoundRecoveryCopyWith<_VotingRoundRecovery> get copyWith =>
      __$VotingRoundRecoveryCopyWithImpl<_VotingRoundRecovery>(
          this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _VotingRoundRecovery &&
            (identical(other.roundId, roundId) || other.roundId == roundId) &&
            (identical(other.bundleCount, bundleCount) ||
                other.bundleCount == bundleCount) &&
            const DeepCollectionEquality()
                .equals(other._delegation, _delegation) &&
            const DeepCollectionEquality().equals(other._votes, _votes) &&
            const DeepCollectionEquality().equals(other._shares, _shares) &&
            const DeepCollectionEquality()
                .equals(other._shareDelegations, _shareDelegations) &&
            const DeepCollectionEquality().equals(
                other._unconfirmedShareDelegations,
                _unconfirmedShareDelegations));
  }

  @override
  int get hashCode => Object.hash(
      runtimeType,
      roundId,
      bundleCount,
      const DeepCollectionEquality().hash(_delegation),
      const DeepCollectionEquality().hash(_votes),
      const DeepCollectionEquality().hash(_shares),
      const DeepCollectionEquality().hash(_shareDelegations),
      const DeepCollectionEquality().hash(_unconfirmedShareDelegations));

  @override
  String toString() {
    return 'VotingRoundRecovery(roundId: $roundId, bundleCount: $bundleCount, delegation: $delegation, votes: $votes, shares: $shares, shareDelegations: $shareDelegations, unconfirmedShareDelegations: $unconfirmedShareDelegations)';
  }
}

/// @nodoc
abstract mixin class _$VotingRoundRecoveryCopyWith<$Res>
    implements $VotingRoundRecoveryCopyWith<$Res> {
  factory _$VotingRoundRecoveryCopyWith(_VotingRoundRecovery value,
          $Res Function(_VotingRoundRecovery) _then) =
      __$VotingRoundRecoveryCopyWithImpl;
  @override
  @useResult
  $Res call(
      {String roundId,
      int bundleCount,
      List<VotingDelegationRecovery> delegation,
      List<VotingVoteRecovery> votes,
      List<VotingShareWorkflow> shares,
      List<VotingShareDelegationRecord> shareDelegations,
      List<VotingShareDelegationRecord> unconfirmedShareDelegations});
}

/// @nodoc
class __$VotingRoundRecoveryCopyWithImpl<$Res>
    implements _$VotingRoundRecoveryCopyWith<$Res> {
  __$VotingRoundRecoveryCopyWithImpl(this._self, this._then);

  final _VotingRoundRecovery _self;
  final $Res Function(_VotingRoundRecovery) _then;

  /// Create a copy of VotingRoundRecovery
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $Res call({
    Object? roundId = null,
    Object? bundleCount = null,
    Object? delegation = null,
    Object? votes = null,
    Object? shares = null,
    Object? shareDelegations = null,
    Object? unconfirmedShareDelegations = null,
  }) {
    return _then(_VotingRoundRecovery(
      roundId: null == roundId
          ? _self.roundId
          : roundId // ignore: cast_nullable_to_non_nullable
              as String,
      bundleCount: null == bundleCount
          ? _self.bundleCount
          : bundleCount // ignore: cast_nullable_to_non_nullable
              as int,
      delegation: null == delegation
          ? _self._delegation
          : delegation // ignore: cast_nullable_to_non_nullable
              as List<VotingDelegationRecovery>,
      votes: null == votes
          ? _self._votes
          : votes // ignore: cast_nullable_to_non_nullable
              as List<VotingVoteRecovery>,
      shares: null == shares
          ? _self._shares
          : shares // ignore: cast_nullable_to_non_nullable
              as List<VotingShareWorkflow>,
      shareDelegations: null == shareDelegations
          ? _self._shareDelegations
          : shareDelegations // ignore: cast_nullable_to_non_nullable
              as List<VotingShareDelegationRecord>,
      unconfirmedShareDelegations: null == unconfirmedShareDelegations
          ? _self._unconfirmedShareDelegations
          : unconfirmedShareDelegations // ignore: cast_nullable_to_non_nullable
              as List<VotingShareDelegationRecord>,
    ));
  }
}

/// @nodoc
mixin _$VotingServiceEndpoint {
  String get url;
  String get label;

  /// Create a copy of VotingServiceEndpoint
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $VotingServiceEndpointCopyWith<VotingServiceEndpoint> get copyWith =>
      _$VotingServiceEndpointCopyWithImpl<VotingServiceEndpoint>(
          this as VotingServiceEndpoint, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is VotingServiceEndpoint &&
            (identical(other.url, url) || other.url == url) &&
            (identical(other.label, label) || other.label == label));
  }

  @override
  int get hashCode => Object.hash(runtimeType, url, label);

  @override
  String toString() {
    return 'VotingServiceEndpoint(url: $url, label: $label)';
  }
}

/// @nodoc
abstract mixin class $VotingServiceEndpointCopyWith<$Res> {
  factory $VotingServiceEndpointCopyWith(VotingServiceEndpoint value,
          $Res Function(VotingServiceEndpoint) _then) =
      _$VotingServiceEndpointCopyWithImpl;
  @useResult
  $Res call({String url, String label});
}

/// @nodoc
class _$VotingServiceEndpointCopyWithImpl<$Res>
    implements $VotingServiceEndpointCopyWith<$Res> {
  _$VotingServiceEndpointCopyWithImpl(this._self, this._then);

  final VotingServiceEndpoint _self;
  final $Res Function(VotingServiceEndpoint) _then;

  /// Create a copy of VotingServiceEndpoint
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? url = null,
    Object? label = null,
  }) {
    return _then(_self.copyWith(
      url: null == url
          ? _self.url
          : url // ignore: cast_nullable_to_non_nullable
              as String,
      label: null == label
          ? _self.label
          : label // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// Adds pattern-matching-related methods to [VotingServiceEndpoint].
extension VotingServiceEndpointPatterns on VotingServiceEndpoint {
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
    TResult Function(_VotingServiceEndpoint value)? $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _VotingServiceEndpoint() when $default != null:
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
    TResult Function(_VotingServiceEndpoint value) $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingServiceEndpoint():
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
    TResult? Function(_VotingServiceEndpoint value)? $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingServiceEndpoint() when $default != null:
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
    TResult Function(String url, String label)? $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _VotingServiceEndpoint() when $default != null:
        return $default(_that.url, _that.label);
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
    TResult Function(String url, String label) $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingServiceEndpoint():
        return $default(_that.url, _that.label);
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
    TResult? Function(String url, String label)? $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingServiceEndpoint() when $default != null:
        return $default(_that.url, _that.label);
      case _:
        return null;
    }
  }
}

/// @nodoc

class _VotingServiceEndpoint implements VotingServiceEndpoint {
  const _VotingServiceEndpoint({required this.url, required this.label});

  @override
  final String url;
  @override
  final String label;

  /// Create a copy of VotingServiceEndpoint
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  _$VotingServiceEndpointCopyWith<_VotingServiceEndpoint> get copyWith =>
      __$VotingServiceEndpointCopyWithImpl<_VotingServiceEndpoint>(
          this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _VotingServiceEndpoint &&
            (identical(other.url, url) || other.url == url) &&
            (identical(other.label, label) || other.label == label));
  }

  @override
  int get hashCode => Object.hash(runtimeType, url, label);

  @override
  String toString() {
    return 'VotingServiceEndpoint(url: $url, label: $label)';
  }
}

/// @nodoc
abstract mixin class _$VotingServiceEndpointCopyWith<$Res>
    implements $VotingServiceEndpointCopyWith<$Res> {
  factory _$VotingServiceEndpointCopyWith(_VotingServiceEndpoint value,
          $Res Function(_VotingServiceEndpoint) _then) =
      __$VotingServiceEndpointCopyWithImpl;
  @override
  @useResult
  $Res call({String url, String label});
}

/// @nodoc
class __$VotingServiceEndpointCopyWithImpl<$Res>
    implements _$VotingServiceEndpointCopyWith<$Res> {
  __$VotingServiceEndpointCopyWithImpl(this._self, this._then);

  final _VotingServiceEndpoint _self;
  final $Res Function(_VotingServiceEndpoint) _then;

  /// Create a copy of VotingServiceEndpoint
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $Res call({
    Object? url = null,
    Object? label = null,
  }) {
    return _then(_VotingServiceEndpoint(
      url: null == url
          ? _self.url
          : url // ignore: cast_nullable_to_non_nullable
              as String,
      label: null == label
          ? _self.label
          : label // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// @nodoc
mixin _$VotingShareDelegationRecord {
  String get roundId;
  int get bundleIndex;
  int get proposalId;
  int get shareIndex;
  List<String> get sentToUrls;
  Uint8List get nullifier;
  bool get confirmed;
  BigInt get submitAt;
  BigInt get createdAt;

  /// Create a copy of VotingShareDelegationRecord
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $VotingShareDelegationRecordCopyWith<VotingShareDelegationRecord>
      get copyWith => _$VotingShareDelegationRecordCopyWithImpl<
              VotingShareDelegationRecord>(
          this as VotingShareDelegationRecord, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is VotingShareDelegationRecord &&
            (identical(other.roundId, roundId) || other.roundId == roundId) &&
            (identical(other.bundleIndex, bundleIndex) ||
                other.bundleIndex == bundleIndex) &&
            (identical(other.proposalId, proposalId) ||
                other.proposalId == proposalId) &&
            (identical(other.shareIndex, shareIndex) ||
                other.shareIndex == shareIndex) &&
            const DeepCollectionEquality()
                .equals(other.sentToUrls, sentToUrls) &&
            const DeepCollectionEquality().equals(other.nullifier, nullifier) &&
            (identical(other.confirmed, confirmed) ||
                other.confirmed == confirmed) &&
            (identical(other.submitAt, submitAt) ||
                other.submitAt == submitAt) &&
            (identical(other.createdAt, createdAt) ||
                other.createdAt == createdAt));
  }

  @override
  int get hashCode => Object.hash(
      runtimeType,
      roundId,
      bundleIndex,
      proposalId,
      shareIndex,
      const DeepCollectionEquality().hash(sentToUrls),
      const DeepCollectionEquality().hash(nullifier),
      confirmed,
      submitAt,
      createdAt);

  @override
  String toString() {
    return 'VotingShareDelegationRecord(roundId: $roundId, bundleIndex: $bundleIndex, proposalId: $proposalId, shareIndex: $shareIndex, sentToUrls: $sentToUrls, nullifier: $nullifier, confirmed: $confirmed, submitAt: $submitAt, createdAt: $createdAt)';
  }
}

/// @nodoc
abstract mixin class $VotingShareDelegationRecordCopyWith<$Res> {
  factory $VotingShareDelegationRecordCopyWith(
          VotingShareDelegationRecord value,
          $Res Function(VotingShareDelegationRecord) _then) =
      _$VotingShareDelegationRecordCopyWithImpl;
  @useResult
  $Res call(
      {String roundId,
      int bundleIndex,
      int proposalId,
      int shareIndex,
      List<String> sentToUrls,
      Uint8List nullifier,
      bool confirmed,
      BigInt submitAt,
      BigInt createdAt});
}

/// @nodoc
class _$VotingShareDelegationRecordCopyWithImpl<$Res>
    implements $VotingShareDelegationRecordCopyWith<$Res> {
  _$VotingShareDelegationRecordCopyWithImpl(this._self, this._then);

  final VotingShareDelegationRecord _self;
  final $Res Function(VotingShareDelegationRecord) _then;

  /// Create a copy of VotingShareDelegationRecord
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? roundId = null,
    Object? bundleIndex = null,
    Object? proposalId = null,
    Object? shareIndex = null,
    Object? sentToUrls = null,
    Object? nullifier = null,
    Object? confirmed = null,
    Object? submitAt = null,
    Object? createdAt = null,
  }) {
    return _then(_self.copyWith(
      roundId: null == roundId
          ? _self.roundId
          : roundId // ignore: cast_nullable_to_non_nullable
              as String,
      bundleIndex: null == bundleIndex
          ? _self.bundleIndex
          : bundleIndex // ignore: cast_nullable_to_non_nullable
              as int,
      proposalId: null == proposalId
          ? _self.proposalId
          : proposalId // ignore: cast_nullable_to_non_nullable
              as int,
      shareIndex: null == shareIndex
          ? _self.shareIndex
          : shareIndex // ignore: cast_nullable_to_non_nullable
              as int,
      sentToUrls: null == sentToUrls
          ? _self.sentToUrls
          : sentToUrls // ignore: cast_nullable_to_non_nullable
              as List<String>,
      nullifier: null == nullifier
          ? _self.nullifier
          : nullifier // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      confirmed: null == confirmed
          ? _self.confirmed
          : confirmed // ignore: cast_nullable_to_non_nullable
              as bool,
      submitAt: null == submitAt
          ? _self.submitAt
          : submitAt // ignore: cast_nullable_to_non_nullable
              as BigInt,
      createdAt: null == createdAt
          ? _self.createdAt
          : createdAt // ignore: cast_nullable_to_non_nullable
              as BigInt,
    ));
  }
}

/// Adds pattern-matching-related methods to [VotingShareDelegationRecord].
extension VotingShareDelegationRecordPatterns on VotingShareDelegationRecord {
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
    TResult Function(_VotingShareDelegationRecord value)? $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _VotingShareDelegationRecord() when $default != null:
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
    TResult Function(_VotingShareDelegationRecord value) $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingShareDelegationRecord():
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
    TResult? Function(_VotingShareDelegationRecord value)? $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingShareDelegationRecord() when $default != null:
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
    TResult Function(
            String roundId,
            int bundleIndex,
            int proposalId,
            int shareIndex,
            List<String> sentToUrls,
            Uint8List nullifier,
            bool confirmed,
            BigInt submitAt,
            BigInt createdAt)?
        $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _VotingShareDelegationRecord() when $default != null:
        return $default(
            _that.roundId,
            _that.bundleIndex,
            _that.proposalId,
            _that.shareIndex,
            _that.sentToUrls,
            _that.nullifier,
            _that.confirmed,
            _that.submitAt,
            _that.createdAt);
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
    TResult Function(
            String roundId,
            int bundleIndex,
            int proposalId,
            int shareIndex,
            List<String> sentToUrls,
            Uint8List nullifier,
            bool confirmed,
            BigInt submitAt,
            BigInt createdAt)
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingShareDelegationRecord():
        return $default(
            _that.roundId,
            _that.bundleIndex,
            _that.proposalId,
            _that.shareIndex,
            _that.sentToUrls,
            _that.nullifier,
            _that.confirmed,
            _that.submitAt,
            _that.createdAt);
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
    TResult? Function(
            String roundId,
            int bundleIndex,
            int proposalId,
            int shareIndex,
            List<String> sentToUrls,
            Uint8List nullifier,
            bool confirmed,
            BigInt submitAt,
            BigInt createdAt)?
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingShareDelegationRecord() when $default != null:
        return $default(
            _that.roundId,
            _that.bundleIndex,
            _that.proposalId,
            _that.shareIndex,
            _that.sentToUrls,
            _that.nullifier,
            _that.confirmed,
            _that.submitAt,
            _that.createdAt);
      case _:
        return null;
    }
  }
}

/// @nodoc

class _VotingShareDelegationRecord implements VotingShareDelegationRecord {
  const _VotingShareDelegationRecord(
      {required this.roundId,
      required this.bundleIndex,
      required this.proposalId,
      required this.shareIndex,
      required final List<String> sentToUrls,
      required this.nullifier,
      required this.confirmed,
      required this.submitAt,
      required this.createdAt})
      : _sentToUrls = sentToUrls;

  @override
  final String roundId;
  @override
  final int bundleIndex;
  @override
  final int proposalId;
  @override
  final int shareIndex;
  final List<String> _sentToUrls;
  @override
  List<String> get sentToUrls {
    if (_sentToUrls is EqualUnmodifiableListView) return _sentToUrls;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_sentToUrls);
  }

  @override
  final Uint8List nullifier;
  @override
  final bool confirmed;
  @override
  final BigInt submitAt;
  @override
  final BigInt createdAt;

  /// Create a copy of VotingShareDelegationRecord
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  _$VotingShareDelegationRecordCopyWith<_VotingShareDelegationRecord>
      get copyWith => __$VotingShareDelegationRecordCopyWithImpl<
          _VotingShareDelegationRecord>(this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _VotingShareDelegationRecord &&
            (identical(other.roundId, roundId) || other.roundId == roundId) &&
            (identical(other.bundleIndex, bundleIndex) ||
                other.bundleIndex == bundleIndex) &&
            (identical(other.proposalId, proposalId) ||
                other.proposalId == proposalId) &&
            (identical(other.shareIndex, shareIndex) ||
                other.shareIndex == shareIndex) &&
            const DeepCollectionEquality()
                .equals(other._sentToUrls, _sentToUrls) &&
            const DeepCollectionEquality().equals(other.nullifier, nullifier) &&
            (identical(other.confirmed, confirmed) ||
                other.confirmed == confirmed) &&
            (identical(other.submitAt, submitAt) ||
                other.submitAt == submitAt) &&
            (identical(other.createdAt, createdAt) ||
                other.createdAt == createdAt));
  }

  @override
  int get hashCode => Object.hash(
      runtimeType,
      roundId,
      bundleIndex,
      proposalId,
      shareIndex,
      const DeepCollectionEquality().hash(_sentToUrls),
      const DeepCollectionEquality().hash(nullifier),
      confirmed,
      submitAt,
      createdAt);

  @override
  String toString() {
    return 'VotingShareDelegationRecord(roundId: $roundId, bundleIndex: $bundleIndex, proposalId: $proposalId, shareIndex: $shareIndex, sentToUrls: $sentToUrls, nullifier: $nullifier, confirmed: $confirmed, submitAt: $submitAt, createdAt: $createdAt)';
  }
}

/// @nodoc
abstract mixin class _$VotingShareDelegationRecordCopyWith<$Res>
    implements $VotingShareDelegationRecordCopyWith<$Res> {
  factory _$VotingShareDelegationRecordCopyWith(
          _VotingShareDelegationRecord value,
          $Res Function(_VotingShareDelegationRecord) _then) =
      __$VotingShareDelegationRecordCopyWithImpl;
  @override
  @useResult
  $Res call(
      {String roundId,
      int bundleIndex,
      int proposalId,
      int shareIndex,
      List<String> sentToUrls,
      Uint8List nullifier,
      bool confirmed,
      BigInt submitAt,
      BigInt createdAt});
}

/// @nodoc
class __$VotingShareDelegationRecordCopyWithImpl<$Res>
    implements _$VotingShareDelegationRecordCopyWith<$Res> {
  __$VotingShareDelegationRecordCopyWithImpl(this._self, this._then);

  final _VotingShareDelegationRecord _self;
  final $Res Function(_VotingShareDelegationRecord) _then;

  /// Create a copy of VotingShareDelegationRecord
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $Res call({
    Object? roundId = null,
    Object? bundleIndex = null,
    Object? proposalId = null,
    Object? shareIndex = null,
    Object? sentToUrls = null,
    Object? nullifier = null,
    Object? confirmed = null,
    Object? submitAt = null,
    Object? createdAt = null,
  }) {
    return _then(_VotingShareDelegationRecord(
      roundId: null == roundId
          ? _self.roundId
          : roundId // ignore: cast_nullable_to_non_nullable
              as String,
      bundleIndex: null == bundleIndex
          ? _self.bundleIndex
          : bundleIndex // ignore: cast_nullable_to_non_nullable
              as int,
      proposalId: null == proposalId
          ? _self.proposalId
          : proposalId // ignore: cast_nullable_to_non_nullable
              as int,
      shareIndex: null == shareIndex
          ? _self.shareIndex
          : shareIndex // ignore: cast_nullable_to_non_nullable
              as int,
      sentToUrls: null == sentToUrls
          ? _self._sentToUrls
          : sentToUrls // ignore: cast_nullable_to_non_nullable
              as List<String>,
      nullifier: null == nullifier
          ? _self.nullifier
          : nullifier // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      confirmed: null == confirmed
          ? _self.confirmed
          : confirmed // ignore: cast_nullable_to_non_nullable
              as bool,
      submitAt: null == submitAt
          ? _self.submitAt
          : submitAt // ignore: cast_nullable_to_non_nullable
              as BigInt,
      createdAt: null == createdAt
          ? _self.createdAt
          : createdAt // ignore: cast_nullable_to_non_nullable
              as BigInt,
    ));
  }
}

/// @nodoc
mixin _$VotingSharePayload {
  Uint8List get sharesHash;
  int get proposalId;
  int get voteDecision;
  VotingEncryptedShare get encShare;
  BigInt get treePosition;
  List<VotingEncryptedShare> get allEncShares;
  List<Uint8List> get shareComms;
  Uint8List get primaryBlind;

  /// Create a copy of VotingSharePayload
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $VotingSharePayloadCopyWith<VotingSharePayload> get copyWith =>
      _$VotingSharePayloadCopyWithImpl<VotingSharePayload>(
          this as VotingSharePayload, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is VotingSharePayload &&
            const DeepCollectionEquality()
                .equals(other.sharesHash, sharesHash) &&
            (identical(other.proposalId, proposalId) ||
                other.proposalId == proposalId) &&
            (identical(other.voteDecision, voteDecision) ||
                other.voteDecision == voteDecision) &&
            (identical(other.encShare, encShare) ||
                other.encShare == encShare) &&
            (identical(other.treePosition, treePosition) ||
                other.treePosition == treePosition) &&
            const DeepCollectionEquality()
                .equals(other.allEncShares, allEncShares) &&
            const DeepCollectionEquality()
                .equals(other.shareComms, shareComms) &&
            const DeepCollectionEquality()
                .equals(other.primaryBlind, primaryBlind));
  }

  @override
  int get hashCode => Object.hash(
      runtimeType,
      const DeepCollectionEquality().hash(sharesHash),
      proposalId,
      voteDecision,
      encShare,
      treePosition,
      const DeepCollectionEquality().hash(allEncShares),
      const DeepCollectionEquality().hash(shareComms),
      const DeepCollectionEquality().hash(primaryBlind));

  @override
  String toString() {
    return 'VotingSharePayload(sharesHash: $sharesHash, proposalId: $proposalId, voteDecision: $voteDecision, encShare: $encShare, treePosition: $treePosition, allEncShares: $allEncShares, shareComms: $shareComms, primaryBlind: $primaryBlind)';
  }
}

/// @nodoc
abstract mixin class $VotingSharePayloadCopyWith<$Res> {
  factory $VotingSharePayloadCopyWith(
          VotingSharePayload value, $Res Function(VotingSharePayload) _then) =
      _$VotingSharePayloadCopyWithImpl;
  @useResult
  $Res call(
      {Uint8List sharesHash,
      int proposalId,
      int voteDecision,
      VotingEncryptedShare encShare,
      BigInt treePosition,
      List<VotingEncryptedShare> allEncShares,
      List<Uint8List> shareComms,
      Uint8List primaryBlind});

  $VotingEncryptedShareCopyWith<$Res> get encShare;
}

/// @nodoc
class _$VotingSharePayloadCopyWithImpl<$Res>
    implements $VotingSharePayloadCopyWith<$Res> {
  _$VotingSharePayloadCopyWithImpl(this._self, this._then);

  final VotingSharePayload _self;
  final $Res Function(VotingSharePayload) _then;

  /// Create a copy of VotingSharePayload
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? sharesHash = null,
    Object? proposalId = null,
    Object? voteDecision = null,
    Object? encShare = null,
    Object? treePosition = null,
    Object? allEncShares = null,
    Object? shareComms = null,
    Object? primaryBlind = null,
  }) {
    return _then(_self.copyWith(
      sharesHash: null == sharesHash
          ? _self.sharesHash
          : sharesHash // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      proposalId: null == proposalId
          ? _self.proposalId
          : proposalId // ignore: cast_nullable_to_non_nullable
              as int,
      voteDecision: null == voteDecision
          ? _self.voteDecision
          : voteDecision // ignore: cast_nullable_to_non_nullable
              as int,
      encShare: null == encShare
          ? _self.encShare
          : encShare // ignore: cast_nullable_to_non_nullable
              as VotingEncryptedShare,
      treePosition: null == treePosition
          ? _self.treePosition
          : treePosition // ignore: cast_nullable_to_non_nullable
              as BigInt,
      allEncShares: null == allEncShares
          ? _self.allEncShares
          : allEncShares // ignore: cast_nullable_to_non_nullable
              as List<VotingEncryptedShare>,
      shareComms: null == shareComms
          ? _self.shareComms
          : shareComms // ignore: cast_nullable_to_non_nullable
              as List<Uint8List>,
      primaryBlind: null == primaryBlind
          ? _self.primaryBlind
          : primaryBlind // ignore: cast_nullable_to_non_nullable
              as Uint8List,
    ));
  }

  /// Create a copy of VotingSharePayload
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $VotingEncryptedShareCopyWith<$Res> get encShare {
    return $VotingEncryptedShareCopyWith<$Res>(_self.encShare, (value) {
      return _then(_self.copyWith(encShare: value));
    });
  }
}

/// Adds pattern-matching-related methods to [VotingSharePayload].
extension VotingSharePayloadPatterns on VotingSharePayload {
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
    TResult Function(_VotingSharePayload value)? $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _VotingSharePayload() when $default != null:
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
    TResult Function(_VotingSharePayload value) $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingSharePayload():
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
    TResult? Function(_VotingSharePayload value)? $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingSharePayload() when $default != null:
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
    TResult Function(
            Uint8List sharesHash,
            int proposalId,
            int voteDecision,
            VotingEncryptedShare encShare,
            BigInt treePosition,
            List<VotingEncryptedShare> allEncShares,
            List<Uint8List> shareComms,
            Uint8List primaryBlind)?
        $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _VotingSharePayload() when $default != null:
        return $default(
            _that.sharesHash,
            _that.proposalId,
            _that.voteDecision,
            _that.encShare,
            _that.treePosition,
            _that.allEncShares,
            _that.shareComms,
            _that.primaryBlind);
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
    TResult Function(
            Uint8List sharesHash,
            int proposalId,
            int voteDecision,
            VotingEncryptedShare encShare,
            BigInt treePosition,
            List<VotingEncryptedShare> allEncShares,
            List<Uint8List> shareComms,
            Uint8List primaryBlind)
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingSharePayload():
        return $default(
            _that.sharesHash,
            _that.proposalId,
            _that.voteDecision,
            _that.encShare,
            _that.treePosition,
            _that.allEncShares,
            _that.shareComms,
            _that.primaryBlind);
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
    TResult? Function(
            Uint8List sharesHash,
            int proposalId,
            int voteDecision,
            VotingEncryptedShare encShare,
            BigInt treePosition,
            List<VotingEncryptedShare> allEncShares,
            List<Uint8List> shareComms,
            Uint8List primaryBlind)?
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingSharePayload() when $default != null:
        return $default(
            _that.sharesHash,
            _that.proposalId,
            _that.voteDecision,
            _that.encShare,
            _that.treePosition,
            _that.allEncShares,
            _that.shareComms,
            _that.primaryBlind);
      case _:
        return null;
    }
  }
}

/// @nodoc

class _VotingSharePayload implements VotingSharePayload {
  const _VotingSharePayload(
      {required this.sharesHash,
      required this.proposalId,
      required this.voteDecision,
      required this.encShare,
      required this.treePosition,
      required final List<VotingEncryptedShare> allEncShares,
      required final List<Uint8List> shareComms,
      required this.primaryBlind})
      : _allEncShares = allEncShares,
        _shareComms = shareComms;

  @override
  final Uint8List sharesHash;
  @override
  final int proposalId;
  @override
  final int voteDecision;
  @override
  final VotingEncryptedShare encShare;
  @override
  final BigInt treePosition;
  final List<VotingEncryptedShare> _allEncShares;
  @override
  List<VotingEncryptedShare> get allEncShares {
    if (_allEncShares is EqualUnmodifiableListView) return _allEncShares;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_allEncShares);
  }

  final List<Uint8List> _shareComms;
  @override
  List<Uint8List> get shareComms {
    if (_shareComms is EqualUnmodifiableListView) return _shareComms;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_shareComms);
  }

  @override
  final Uint8List primaryBlind;

  /// Create a copy of VotingSharePayload
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  _$VotingSharePayloadCopyWith<_VotingSharePayload> get copyWith =>
      __$VotingSharePayloadCopyWithImpl<_VotingSharePayload>(this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _VotingSharePayload &&
            const DeepCollectionEquality()
                .equals(other.sharesHash, sharesHash) &&
            (identical(other.proposalId, proposalId) ||
                other.proposalId == proposalId) &&
            (identical(other.voteDecision, voteDecision) ||
                other.voteDecision == voteDecision) &&
            (identical(other.encShare, encShare) ||
                other.encShare == encShare) &&
            (identical(other.treePosition, treePosition) ||
                other.treePosition == treePosition) &&
            const DeepCollectionEquality()
                .equals(other._allEncShares, _allEncShares) &&
            const DeepCollectionEquality()
                .equals(other._shareComms, _shareComms) &&
            const DeepCollectionEquality()
                .equals(other.primaryBlind, primaryBlind));
  }

  @override
  int get hashCode => Object.hash(
      runtimeType,
      const DeepCollectionEquality().hash(sharesHash),
      proposalId,
      voteDecision,
      encShare,
      treePosition,
      const DeepCollectionEquality().hash(_allEncShares),
      const DeepCollectionEquality().hash(_shareComms),
      const DeepCollectionEquality().hash(primaryBlind));

  @override
  String toString() {
    return 'VotingSharePayload(sharesHash: $sharesHash, proposalId: $proposalId, voteDecision: $voteDecision, encShare: $encShare, treePosition: $treePosition, allEncShares: $allEncShares, shareComms: $shareComms, primaryBlind: $primaryBlind)';
  }
}

/// @nodoc
abstract mixin class _$VotingSharePayloadCopyWith<$Res>
    implements $VotingSharePayloadCopyWith<$Res> {
  factory _$VotingSharePayloadCopyWith(
          _VotingSharePayload value, $Res Function(_VotingSharePayload) _then) =
      __$VotingSharePayloadCopyWithImpl;
  @override
  @useResult
  $Res call(
      {Uint8List sharesHash,
      int proposalId,
      int voteDecision,
      VotingEncryptedShare encShare,
      BigInt treePosition,
      List<VotingEncryptedShare> allEncShares,
      List<Uint8List> shareComms,
      Uint8List primaryBlind});

  @override
  $VotingEncryptedShareCopyWith<$Res> get encShare;
}

/// @nodoc
class __$VotingSharePayloadCopyWithImpl<$Res>
    implements _$VotingSharePayloadCopyWith<$Res> {
  __$VotingSharePayloadCopyWithImpl(this._self, this._then);

  final _VotingSharePayload _self;
  final $Res Function(_VotingSharePayload) _then;

  /// Create a copy of VotingSharePayload
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $Res call({
    Object? sharesHash = null,
    Object? proposalId = null,
    Object? voteDecision = null,
    Object? encShare = null,
    Object? treePosition = null,
    Object? allEncShares = null,
    Object? shareComms = null,
    Object? primaryBlind = null,
  }) {
    return _then(_VotingSharePayload(
      sharesHash: null == sharesHash
          ? _self.sharesHash
          : sharesHash // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      proposalId: null == proposalId
          ? _self.proposalId
          : proposalId // ignore: cast_nullable_to_non_nullable
              as int,
      voteDecision: null == voteDecision
          ? _self.voteDecision
          : voteDecision // ignore: cast_nullable_to_non_nullable
              as int,
      encShare: null == encShare
          ? _self.encShare
          : encShare // ignore: cast_nullable_to_non_nullable
              as VotingEncryptedShare,
      treePosition: null == treePosition
          ? _self.treePosition
          : treePosition // ignore: cast_nullable_to_non_nullable
              as BigInt,
      allEncShares: null == allEncShares
          ? _self._allEncShares
          : allEncShares // ignore: cast_nullable_to_non_nullable
              as List<VotingEncryptedShare>,
      shareComms: null == shareComms
          ? _self._shareComms
          : shareComms // ignore: cast_nullable_to_non_nullable
              as List<Uint8List>,
      primaryBlind: null == primaryBlind
          ? _self.primaryBlind
          : primaryBlind // ignore: cast_nullable_to_non_nullable
              as Uint8List,
    ));
  }

  /// Create a copy of VotingSharePayload
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $VotingEncryptedShareCopyWith<$Res> get encShare {
    return $VotingEncryptedShareCopyWith<$Res>(_self.encShare, (value) {
      return _then(_self.copyWith(encShare: value));
    });
  }
}

/// @nodoc
mixin _$VotingSharePlan {
  VotingShareTrackingSummary get summary;
  BigInt? get nextTrackingDelaySecs;
  bool get lastMoment;
  List<VotingSharePlanItem> get submissions;

  /// Create a copy of VotingSharePlan
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $VotingSharePlanCopyWith<VotingSharePlan> get copyWith =>
      _$VotingSharePlanCopyWithImpl<VotingSharePlan>(
          this as VotingSharePlan, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is VotingSharePlan &&
            (identical(other.summary, summary) || other.summary == summary) &&
            (identical(other.nextTrackingDelaySecs, nextTrackingDelaySecs) ||
                other.nextTrackingDelaySecs == nextTrackingDelaySecs) &&
            (identical(other.lastMoment, lastMoment) ||
                other.lastMoment == lastMoment) &&
            const DeepCollectionEquality()
                .equals(other.submissions, submissions));
  }

  @override
  int get hashCode => Object.hash(runtimeType, summary, nextTrackingDelaySecs,
      lastMoment, const DeepCollectionEquality().hash(submissions));

  @override
  String toString() {
    return 'VotingSharePlan(summary: $summary, nextTrackingDelaySecs: $nextTrackingDelaySecs, lastMoment: $lastMoment, submissions: $submissions)';
  }
}

/// @nodoc
abstract mixin class $VotingSharePlanCopyWith<$Res> {
  factory $VotingSharePlanCopyWith(
          VotingSharePlan value, $Res Function(VotingSharePlan) _then) =
      _$VotingSharePlanCopyWithImpl;
  @useResult
  $Res call(
      {VotingShareTrackingSummary summary,
      BigInt? nextTrackingDelaySecs,
      bool lastMoment,
      List<VotingSharePlanItem> submissions});

  $VotingShareTrackingSummaryCopyWith<$Res> get summary;
}

/// @nodoc
class _$VotingSharePlanCopyWithImpl<$Res>
    implements $VotingSharePlanCopyWith<$Res> {
  _$VotingSharePlanCopyWithImpl(this._self, this._then);

  final VotingSharePlan _self;
  final $Res Function(VotingSharePlan) _then;

  /// Create a copy of VotingSharePlan
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? summary = null,
    Object? nextTrackingDelaySecs = freezed,
    Object? lastMoment = null,
    Object? submissions = null,
  }) {
    return _then(_self.copyWith(
      summary: null == summary
          ? _self.summary
          : summary // ignore: cast_nullable_to_non_nullable
              as VotingShareTrackingSummary,
      nextTrackingDelaySecs: freezed == nextTrackingDelaySecs
          ? _self.nextTrackingDelaySecs
          : nextTrackingDelaySecs // ignore: cast_nullable_to_non_nullable
              as BigInt?,
      lastMoment: null == lastMoment
          ? _self.lastMoment
          : lastMoment // ignore: cast_nullable_to_non_nullable
              as bool,
      submissions: null == submissions
          ? _self.submissions
          : submissions // ignore: cast_nullable_to_non_nullable
              as List<VotingSharePlanItem>,
    ));
  }

  /// Create a copy of VotingSharePlan
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $VotingShareTrackingSummaryCopyWith<$Res> get summary {
    return $VotingShareTrackingSummaryCopyWith<$Res>(_self.summary, (value) {
      return _then(_self.copyWith(summary: value));
    });
  }
}

/// Adds pattern-matching-related methods to [VotingSharePlan].
extension VotingSharePlanPatterns on VotingSharePlan {
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
    TResult Function(_VotingSharePlan value)? $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _VotingSharePlan() when $default != null:
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
    TResult Function(_VotingSharePlan value) $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingSharePlan():
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
    TResult? Function(_VotingSharePlan value)? $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingSharePlan() when $default != null:
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
    TResult Function(
            VotingShareTrackingSummary summary,
            BigInt? nextTrackingDelaySecs,
            bool lastMoment,
            List<VotingSharePlanItem> submissions)?
        $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _VotingSharePlan() when $default != null:
        return $default(_that.summary, _that.nextTrackingDelaySecs,
            _that.lastMoment, _that.submissions);
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
    TResult Function(
            VotingShareTrackingSummary summary,
            BigInt? nextTrackingDelaySecs,
            bool lastMoment,
            List<VotingSharePlanItem> submissions)
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingSharePlan():
        return $default(_that.summary, _that.nextTrackingDelaySecs,
            _that.lastMoment, _that.submissions);
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
    TResult? Function(
            VotingShareTrackingSummary summary,
            BigInt? nextTrackingDelaySecs,
            bool lastMoment,
            List<VotingSharePlanItem> submissions)?
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingSharePlan() when $default != null:
        return $default(_that.summary, _that.nextTrackingDelaySecs,
            _that.lastMoment, _that.submissions);
      case _:
        return null;
    }
  }
}

/// @nodoc

class _VotingSharePlan implements VotingSharePlan {
  const _VotingSharePlan(
      {required this.summary,
      this.nextTrackingDelaySecs,
      required this.lastMoment,
      required final List<VotingSharePlanItem> submissions})
      : _submissions = submissions;

  @override
  final VotingShareTrackingSummary summary;
  @override
  final BigInt? nextTrackingDelaySecs;
  @override
  final bool lastMoment;
  final List<VotingSharePlanItem> _submissions;
  @override
  List<VotingSharePlanItem> get submissions {
    if (_submissions is EqualUnmodifiableListView) return _submissions;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_submissions);
  }

  /// Create a copy of VotingSharePlan
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  _$VotingSharePlanCopyWith<_VotingSharePlan> get copyWith =>
      __$VotingSharePlanCopyWithImpl<_VotingSharePlan>(this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _VotingSharePlan &&
            (identical(other.summary, summary) || other.summary == summary) &&
            (identical(other.nextTrackingDelaySecs, nextTrackingDelaySecs) ||
                other.nextTrackingDelaySecs == nextTrackingDelaySecs) &&
            (identical(other.lastMoment, lastMoment) ||
                other.lastMoment == lastMoment) &&
            const DeepCollectionEquality()
                .equals(other._submissions, _submissions));
  }

  @override
  int get hashCode => Object.hash(runtimeType, summary, nextTrackingDelaySecs,
      lastMoment, const DeepCollectionEquality().hash(_submissions));

  @override
  String toString() {
    return 'VotingSharePlan(summary: $summary, nextTrackingDelaySecs: $nextTrackingDelaySecs, lastMoment: $lastMoment, submissions: $submissions)';
  }
}

/// @nodoc
abstract mixin class _$VotingSharePlanCopyWith<$Res>
    implements $VotingSharePlanCopyWith<$Res> {
  factory _$VotingSharePlanCopyWith(
          _VotingSharePlan value, $Res Function(_VotingSharePlan) _then) =
      __$VotingSharePlanCopyWithImpl;
  @override
  @useResult
  $Res call(
      {VotingShareTrackingSummary summary,
      BigInt? nextTrackingDelaySecs,
      bool lastMoment,
      List<VotingSharePlanItem> submissions});

  @override
  $VotingShareTrackingSummaryCopyWith<$Res> get summary;
}

/// @nodoc
class __$VotingSharePlanCopyWithImpl<$Res>
    implements _$VotingSharePlanCopyWith<$Res> {
  __$VotingSharePlanCopyWithImpl(this._self, this._then);

  final _VotingSharePlan _self;
  final $Res Function(_VotingSharePlan) _then;

  /// Create a copy of VotingSharePlan
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $Res call({
    Object? summary = null,
    Object? nextTrackingDelaySecs = freezed,
    Object? lastMoment = null,
    Object? submissions = null,
  }) {
    return _then(_VotingSharePlan(
      summary: null == summary
          ? _self.summary
          : summary // ignore: cast_nullable_to_non_nullable
              as VotingShareTrackingSummary,
      nextTrackingDelaySecs: freezed == nextTrackingDelaySecs
          ? _self.nextTrackingDelaySecs
          : nextTrackingDelaySecs // ignore: cast_nullable_to_non_nullable
              as BigInt?,
      lastMoment: null == lastMoment
          ? _self.lastMoment
          : lastMoment // ignore: cast_nullable_to_non_nullable
              as bool,
      submissions: null == submissions
          ? _self._submissions
          : submissions // ignore: cast_nullable_to_non_nullable
              as List<VotingSharePlanItem>,
    ));
  }

  /// Create a copy of VotingSharePlan
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $VotingShareTrackingSummaryCopyWith<$Res> get summary {
    return $VotingShareTrackingSummaryCopyWith<$Res>(_self.summary, (value) {
      return _then(_self.copyWith(summary: value));
    });
  }
}

/// @nodoc
mixin _$VotingSharePlanItem {
  BigInt get submitAt;
  int get targetCount;
  List<String> get targetServers;

  /// Create a copy of VotingSharePlanItem
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $VotingSharePlanItemCopyWith<VotingSharePlanItem> get copyWith =>
      _$VotingSharePlanItemCopyWithImpl<VotingSharePlanItem>(
          this as VotingSharePlanItem, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is VotingSharePlanItem &&
            (identical(other.submitAt, submitAt) ||
                other.submitAt == submitAt) &&
            (identical(other.targetCount, targetCount) ||
                other.targetCount == targetCount) &&
            const DeepCollectionEquality()
                .equals(other.targetServers, targetServers));
  }

  @override
  int get hashCode => Object.hash(runtimeType, submitAt, targetCount,
      const DeepCollectionEquality().hash(targetServers));

  @override
  String toString() {
    return 'VotingSharePlanItem(submitAt: $submitAt, targetCount: $targetCount, targetServers: $targetServers)';
  }
}

/// @nodoc
abstract mixin class $VotingSharePlanItemCopyWith<$Res> {
  factory $VotingSharePlanItemCopyWith(
          VotingSharePlanItem value, $Res Function(VotingSharePlanItem) _then) =
      _$VotingSharePlanItemCopyWithImpl;
  @useResult
  $Res call({BigInt submitAt, int targetCount, List<String> targetServers});
}

/// @nodoc
class _$VotingSharePlanItemCopyWithImpl<$Res>
    implements $VotingSharePlanItemCopyWith<$Res> {
  _$VotingSharePlanItemCopyWithImpl(this._self, this._then);

  final VotingSharePlanItem _self;
  final $Res Function(VotingSharePlanItem) _then;

  /// Create a copy of VotingSharePlanItem
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? submitAt = null,
    Object? targetCount = null,
    Object? targetServers = null,
  }) {
    return _then(_self.copyWith(
      submitAt: null == submitAt
          ? _self.submitAt
          : submitAt // ignore: cast_nullable_to_non_nullable
              as BigInt,
      targetCount: null == targetCount
          ? _self.targetCount
          : targetCount // ignore: cast_nullable_to_non_nullable
              as int,
      targetServers: null == targetServers
          ? _self.targetServers
          : targetServers // ignore: cast_nullable_to_non_nullable
              as List<String>,
    ));
  }
}

/// Adds pattern-matching-related methods to [VotingSharePlanItem].
extension VotingSharePlanItemPatterns on VotingSharePlanItem {
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
    TResult Function(_VotingSharePlanItem value)? $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _VotingSharePlanItem() when $default != null:
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
    TResult Function(_VotingSharePlanItem value) $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingSharePlanItem():
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
    TResult? Function(_VotingSharePlanItem value)? $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingSharePlanItem() when $default != null:
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
    TResult Function(
            BigInt submitAt, int targetCount, List<String> targetServers)?
        $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _VotingSharePlanItem() when $default != null:
        return $default(_that.submitAt, _that.targetCount, _that.targetServers);
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
    TResult Function(
            BigInt submitAt, int targetCount, List<String> targetServers)
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingSharePlanItem():
        return $default(_that.submitAt, _that.targetCount, _that.targetServers);
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
    TResult? Function(
            BigInt submitAt, int targetCount, List<String> targetServers)?
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingSharePlanItem() when $default != null:
        return $default(_that.submitAt, _that.targetCount, _that.targetServers);
      case _:
        return null;
    }
  }
}

/// @nodoc

class _VotingSharePlanItem implements VotingSharePlanItem {
  const _VotingSharePlanItem(
      {required this.submitAt,
      required this.targetCount,
      required final List<String> targetServers})
      : _targetServers = targetServers;

  @override
  final BigInt submitAt;
  @override
  final int targetCount;
  final List<String> _targetServers;
  @override
  List<String> get targetServers {
    if (_targetServers is EqualUnmodifiableListView) return _targetServers;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_targetServers);
  }

  /// Create a copy of VotingSharePlanItem
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  _$VotingSharePlanItemCopyWith<_VotingSharePlanItem> get copyWith =>
      __$VotingSharePlanItemCopyWithImpl<_VotingSharePlanItem>(
          this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _VotingSharePlanItem &&
            (identical(other.submitAt, submitAt) ||
                other.submitAt == submitAt) &&
            (identical(other.targetCount, targetCount) ||
                other.targetCount == targetCount) &&
            const DeepCollectionEquality()
                .equals(other._targetServers, _targetServers));
  }

  @override
  int get hashCode => Object.hash(runtimeType, submitAt, targetCount,
      const DeepCollectionEquality().hash(_targetServers));

  @override
  String toString() {
    return 'VotingSharePlanItem(submitAt: $submitAt, targetCount: $targetCount, targetServers: $targetServers)';
  }
}

/// @nodoc
abstract mixin class _$VotingSharePlanItemCopyWith<$Res>
    implements $VotingSharePlanItemCopyWith<$Res> {
  factory _$VotingSharePlanItemCopyWith(_VotingSharePlanItem value,
          $Res Function(_VotingSharePlanItem) _then) =
      __$VotingSharePlanItemCopyWithImpl;
  @override
  @useResult
  $Res call({BigInt submitAt, int targetCount, List<String> targetServers});
}

/// @nodoc
class __$VotingSharePlanItemCopyWithImpl<$Res>
    implements _$VotingSharePlanItemCopyWith<$Res> {
  __$VotingSharePlanItemCopyWithImpl(this._self, this._then);

  final _VotingSharePlanItem _self;
  final $Res Function(_VotingSharePlanItem) _then;

  /// Create a copy of VotingSharePlanItem
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $Res call({
    Object? submitAt = null,
    Object? targetCount = null,
    Object? targetServers = null,
  }) {
    return _then(_VotingSharePlanItem(
      submitAt: null == submitAt
          ? _self.submitAt
          : submitAt // ignore: cast_nullable_to_non_nullable
              as BigInt,
      targetCount: null == targetCount
          ? _self.targetCount
          : targetCount // ignore: cast_nullable_to_non_nullable
              as int,
      targetServers: null == targetServers
          ? _self._targetServers
          : targetServers // ignore: cast_nullable_to_non_nullable
              as List<String>,
    ));
  }
}

/// @nodoc
mixin _$VotingShareTrackingSummary {
  BigInt get total;
  BigInt get confirmed;
  BigInt get waiting;
  BigInt get ready;
  BigInt get overdue;

  /// Create a copy of VotingShareTrackingSummary
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $VotingShareTrackingSummaryCopyWith<VotingShareTrackingSummary>
      get copyWith =>
          _$VotingShareTrackingSummaryCopyWithImpl<VotingShareTrackingSummary>(
              this as VotingShareTrackingSummary, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is VotingShareTrackingSummary &&
            (identical(other.total, total) || other.total == total) &&
            (identical(other.confirmed, confirmed) ||
                other.confirmed == confirmed) &&
            (identical(other.waiting, waiting) || other.waiting == waiting) &&
            (identical(other.ready, ready) || other.ready == ready) &&
            (identical(other.overdue, overdue) || other.overdue == overdue));
  }

  @override
  int get hashCode =>
      Object.hash(runtimeType, total, confirmed, waiting, ready, overdue);

  @override
  String toString() {
    return 'VotingShareTrackingSummary(total: $total, confirmed: $confirmed, waiting: $waiting, ready: $ready, overdue: $overdue)';
  }
}

/// @nodoc
abstract mixin class $VotingShareTrackingSummaryCopyWith<$Res> {
  factory $VotingShareTrackingSummaryCopyWith(VotingShareTrackingSummary value,
          $Res Function(VotingShareTrackingSummary) _then) =
      _$VotingShareTrackingSummaryCopyWithImpl;
  @useResult
  $Res call(
      {BigInt total,
      BigInt confirmed,
      BigInt waiting,
      BigInt ready,
      BigInt overdue});
}

/// @nodoc
class _$VotingShareTrackingSummaryCopyWithImpl<$Res>
    implements $VotingShareTrackingSummaryCopyWith<$Res> {
  _$VotingShareTrackingSummaryCopyWithImpl(this._self, this._then);

  final VotingShareTrackingSummary _self;
  final $Res Function(VotingShareTrackingSummary) _then;

  /// Create a copy of VotingShareTrackingSummary
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? total = null,
    Object? confirmed = null,
    Object? waiting = null,
    Object? ready = null,
    Object? overdue = null,
  }) {
    return _then(_self.copyWith(
      total: null == total
          ? _self.total
          : total // ignore: cast_nullable_to_non_nullable
              as BigInt,
      confirmed: null == confirmed
          ? _self.confirmed
          : confirmed // ignore: cast_nullable_to_non_nullable
              as BigInt,
      waiting: null == waiting
          ? _self.waiting
          : waiting // ignore: cast_nullable_to_non_nullable
              as BigInt,
      ready: null == ready
          ? _self.ready
          : ready // ignore: cast_nullable_to_non_nullable
              as BigInt,
      overdue: null == overdue
          ? _self.overdue
          : overdue // ignore: cast_nullable_to_non_nullable
              as BigInt,
    ));
  }
}

/// Adds pattern-matching-related methods to [VotingShareTrackingSummary].
extension VotingShareTrackingSummaryPatterns on VotingShareTrackingSummary {
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
    TResult Function(_VotingShareTrackingSummary value)? $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _VotingShareTrackingSummary() when $default != null:
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
    TResult Function(_VotingShareTrackingSummary value) $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingShareTrackingSummary():
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
    TResult? Function(_VotingShareTrackingSummary value)? $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingShareTrackingSummary() when $default != null:
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
    TResult Function(BigInt total, BigInt confirmed, BigInt waiting,
            BigInt ready, BigInt overdue)?
        $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _VotingShareTrackingSummary() when $default != null:
        return $default(_that.total, _that.confirmed, _that.waiting,
            _that.ready, _that.overdue);
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
    TResult Function(BigInt total, BigInt confirmed, BigInt waiting,
            BigInt ready, BigInt overdue)
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingShareTrackingSummary():
        return $default(_that.total, _that.confirmed, _that.waiting,
            _that.ready, _that.overdue);
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
    TResult? Function(BigInt total, BigInt confirmed, BigInt waiting,
            BigInt ready, BigInt overdue)?
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingShareTrackingSummary() when $default != null:
        return $default(_that.total, _that.confirmed, _that.waiting,
            _that.ready, _that.overdue);
      case _:
        return null;
    }
  }
}

/// @nodoc

class _VotingShareTrackingSummary implements VotingShareTrackingSummary {
  const _VotingShareTrackingSummary(
      {required this.total,
      required this.confirmed,
      required this.waiting,
      required this.ready,
      required this.overdue});

  @override
  final BigInt total;
  @override
  final BigInt confirmed;
  @override
  final BigInt waiting;
  @override
  final BigInt ready;
  @override
  final BigInt overdue;

  /// Create a copy of VotingShareTrackingSummary
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  _$VotingShareTrackingSummaryCopyWith<_VotingShareTrackingSummary>
      get copyWith => __$VotingShareTrackingSummaryCopyWithImpl<
          _VotingShareTrackingSummary>(this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _VotingShareTrackingSummary &&
            (identical(other.total, total) || other.total == total) &&
            (identical(other.confirmed, confirmed) ||
                other.confirmed == confirmed) &&
            (identical(other.waiting, waiting) || other.waiting == waiting) &&
            (identical(other.ready, ready) || other.ready == ready) &&
            (identical(other.overdue, overdue) || other.overdue == overdue));
  }

  @override
  int get hashCode =>
      Object.hash(runtimeType, total, confirmed, waiting, ready, overdue);

  @override
  String toString() {
    return 'VotingShareTrackingSummary(total: $total, confirmed: $confirmed, waiting: $waiting, ready: $ready, overdue: $overdue)';
  }
}

/// @nodoc
abstract mixin class _$VotingShareTrackingSummaryCopyWith<$Res>
    implements $VotingShareTrackingSummaryCopyWith<$Res> {
  factory _$VotingShareTrackingSummaryCopyWith(
          _VotingShareTrackingSummary value,
          $Res Function(_VotingShareTrackingSummary) _then) =
      __$VotingShareTrackingSummaryCopyWithImpl;
  @override
  @useResult
  $Res call(
      {BigInt total,
      BigInt confirmed,
      BigInt waiting,
      BigInt ready,
      BigInt overdue});
}

/// @nodoc
class __$VotingShareTrackingSummaryCopyWithImpl<$Res>
    implements _$VotingShareTrackingSummaryCopyWith<$Res> {
  __$VotingShareTrackingSummaryCopyWithImpl(this._self, this._then);

  final _VotingShareTrackingSummary _self;
  final $Res Function(_VotingShareTrackingSummary) _then;

  /// Create a copy of VotingShareTrackingSummary
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $Res call({
    Object? total = null,
    Object? confirmed = null,
    Object? waiting = null,
    Object? ready = null,
    Object? overdue = null,
  }) {
    return _then(_VotingShareTrackingSummary(
      total: null == total
          ? _self.total
          : total // ignore: cast_nullable_to_non_nullable
              as BigInt,
      confirmed: null == confirmed
          ? _self.confirmed
          : confirmed // ignore: cast_nullable_to_non_nullable
              as BigInt,
      waiting: null == waiting
          ? _self.waiting
          : waiting // ignore: cast_nullable_to_non_nullable
              as BigInt,
      ready: null == ready
          ? _self.ready
          : ready // ignore: cast_nullable_to_non_nullable
              as BigInt,
      overdue: null == overdue
          ? _self.overdue
          : overdue // ignore: cast_nullable_to_non_nullable
              as BigInt,
    ));
  }
}

/// @nodoc
mixin _$VotingShareWorkflow {
  int get bundleIndex;
  int get proposalId;
  int get shareIndex;
  String get phase;

  /// Create a copy of VotingShareWorkflow
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $VotingShareWorkflowCopyWith<VotingShareWorkflow> get copyWith =>
      _$VotingShareWorkflowCopyWithImpl<VotingShareWorkflow>(
          this as VotingShareWorkflow, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is VotingShareWorkflow &&
            (identical(other.bundleIndex, bundleIndex) ||
                other.bundleIndex == bundleIndex) &&
            (identical(other.proposalId, proposalId) ||
                other.proposalId == proposalId) &&
            (identical(other.shareIndex, shareIndex) ||
                other.shareIndex == shareIndex) &&
            (identical(other.phase, phase) || other.phase == phase));
  }

  @override
  int get hashCode =>
      Object.hash(runtimeType, bundleIndex, proposalId, shareIndex, phase);

  @override
  String toString() {
    return 'VotingShareWorkflow(bundleIndex: $bundleIndex, proposalId: $proposalId, shareIndex: $shareIndex, phase: $phase)';
  }
}

/// @nodoc
abstract mixin class $VotingShareWorkflowCopyWith<$Res> {
  factory $VotingShareWorkflowCopyWith(
          VotingShareWorkflow value, $Res Function(VotingShareWorkflow) _then) =
      _$VotingShareWorkflowCopyWithImpl;
  @useResult
  $Res call({int bundleIndex, int proposalId, int shareIndex, String phase});
}

/// @nodoc
class _$VotingShareWorkflowCopyWithImpl<$Res>
    implements $VotingShareWorkflowCopyWith<$Res> {
  _$VotingShareWorkflowCopyWithImpl(this._self, this._then);

  final VotingShareWorkflow _self;
  final $Res Function(VotingShareWorkflow) _then;

  /// Create a copy of VotingShareWorkflow
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? bundleIndex = null,
    Object? proposalId = null,
    Object? shareIndex = null,
    Object? phase = null,
  }) {
    return _then(_self.copyWith(
      bundleIndex: null == bundleIndex
          ? _self.bundleIndex
          : bundleIndex // ignore: cast_nullable_to_non_nullable
              as int,
      proposalId: null == proposalId
          ? _self.proposalId
          : proposalId // ignore: cast_nullable_to_non_nullable
              as int,
      shareIndex: null == shareIndex
          ? _self.shareIndex
          : shareIndex // ignore: cast_nullable_to_non_nullable
              as int,
      phase: null == phase
          ? _self.phase
          : phase // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// Adds pattern-matching-related methods to [VotingShareWorkflow].
extension VotingShareWorkflowPatterns on VotingShareWorkflow {
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
    TResult Function(_VotingShareWorkflow value)? $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _VotingShareWorkflow() when $default != null:
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
    TResult Function(_VotingShareWorkflow value) $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingShareWorkflow():
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
    TResult? Function(_VotingShareWorkflow value)? $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingShareWorkflow() when $default != null:
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
    TResult Function(
            int bundleIndex, int proposalId, int shareIndex, String phase)?
        $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _VotingShareWorkflow() when $default != null:
        return $default(
            _that.bundleIndex, _that.proposalId, _that.shareIndex, _that.phase);
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
    TResult Function(
            int bundleIndex, int proposalId, int shareIndex, String phase)
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingShareWorkflow():
        return $default(
            _that.bundleIndex, _that.proposalId, _that.shareIndex, _that.phase);
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
    TResult? Function(
            int bundleIndex, int proposalId, int shareIndex, String phase)?
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingShareWorkflow() when $default != null:
        return $default(
            _that.bundleIndex, _that.proposalId, _that.shareIndex, _that.phase);
      case _:
        return null;
    }
  }
}

/// @nodoc

class _VotingShareWorkflow implements VotingShareWorkflow {
  const _VotingShareWorkflow(
      {required this.bundleIndex,
      required this.proposalId,
      required this.shareIndex,
      required this.phase});

  @override
  final int bundleIndex;
  @override
  final int proposalId;
  @override
  final int shareIndex;
  @override
  final String phase;

  /// Create a copy of VotingShareWorkflow
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  _$VotingShareWorkflowCopyWith<_VotingShareWorkflow> get copyWith =>
      __$VotingShareWorkflowCopyWithImpl<_VotingShareWorkflow>(
          this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _VotingShareWorkflow &&
            (identical(other.bundleIndex, bundleIndex) ||
                other.bundleIndex == bundleIndex) &&
            (identical(other.proposalId, proposalId) ||
                other.proposalId == proposalId) &&
            (identical(other.shareIndex, shareIndex) ||
                other.shareIndex == shareIndex) &&
            (identical(other.phase, phase) || other.phase == phase));
  }

  @override
  int get hashCode =>
      Object.hash(runtimeType, bundleIndex, proposalId, shareIndex, phase);

  @override
  String toString() {
    return 'VotingShareWorkflow(bundleIndex: $bundleIndex, proposalId: $proposalId, shareIndex: $shareIndex, phase: $phase)';
  }
}

/// @nodoc
abstract mixin class _$VotingShareWorkflowCopyWith<$Res>
    implements $VotingShareWorkflowCopyWith<$Res> {
  factory _$VotingShareWorkflowCopyWith(_VotingShareWorkflow value,
          $Res Function(_VotingShareWorkflow) _then) =
      __$VotingShareWorkflowCopyWithImpl;
  @override
  @useResult
  $Res call({int bundleIndex, int proposalId, int shareIndex, String phase});
}

/// @nodoc
class __$VotingShareWorkflowCopyWithImpl<$Res>
    implements _$VotingShareWorkflowCopyWith<$Res> {
  __$VotingShareWorkflowCopyWithImpl(this._self, this._then);

  final _VotingShareWorkflow _self;
  final $Res Function(_VotingShareWorkflow) _then;

  /// Create a copy of VotingShareWorkflow
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $Res call({
    Object? bundleIndex = null,
    Object? proposalId = null,
    Object? shareIndex = null,
    Object? phase = null,
  }) {
    return _then(_VotingShareWorkflow(
      bundleIndex: null == bundleIndex
          ? _self.bundleIndex
          : bundleIndex // ignore: cast_nullable_to_non_nullable
              as int,
      proposalId: null == proposalId
          ? _self.proposalId
          : proposalId // ignore: cast_nullable_to_non_nullable
              as int,
      shareIndex: null == shareIndex
          ? _self.shareIndex
          : shareIndex // ignore: cast_nullable_to_non_nullable
              as int,
      phase: null == phase
          ? _self.phase
          : phase // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// @nodoc
mixin _$VotingSignedVoteCommitment {
  int get proposalId;
  int get choice;
  String get voteRoundId;
  Uint8List get vanNullifier;
  Uint8List get voteAuthorityNoteNew;
  Uint8List get voteCommitment;
  Uint8List get proof;
  int get anchorHeight;
  Uint8List get rVpk;
  Uint8List get voteAuthSig;
  String get commitmentBundleJson;

  /// Create a copy of VotingSignedVoteCommitment
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $VotingSignedVoteCommitmentCopyWith<VotingSignedVoteCommitment>
      get copyWith =>
          _$VotingSignedVoteCommitmentCopyWithImpl<VotingSignedVoteCommitment>(
              this as VotingSignedVoteCommitment, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is VotingSignedVoteCommitment &&
            (identical(other.proposalId, proposalId) ||
                other.proposalId == proposalId) &&
            (identical(other.choice, choice) || other.choice == choice) &&
            (identical(other.voteRoundId, voteRoundId) ||
                other.voteRoundId == voteRoundId) &&
            const DeepCollectionEquality()
                .equals(other.vanNullifier, vanNullifier) &&
            const DeepCollectionEquality()
                .equals(other.voteAuthorityNoteNew, voteAuthorityNoteNew) &&
            const DeepCollectionEquality()
                .equals(other.voteCommitment, voteCommitment) &&
            const DeepCollectionEquality().equals(other.proof, proof) &&
            (identical(other.anchorHeight, anchorHeight) ||
                other.anchorHeight == anchorHeight) &&
            const DeepCollectionEquality().equals(other.rVpk, rVpk) &&
            const DeepCollectionEquality()
                .equals(other.voteAuthSig, voteAuthSig) &&
            (identical(other.commitmentBundleJson, commitmentBundleJson) ||
                other.commitmentBundleJson == commitmentBundleJson));
  }

  @override
  int get hashCode => Object.hash(
      runtimeType,
      proposalId,
      choice,
      voteRoundId,
      const DeepCollectionEquality().hash(vanNullifier),
      const DeepCollectionEquality().hash(voteAuthorityNoteNew),
      const DeepCollectionEquality().hash(voteCommitment),
      const DeepCollectionEquality().hash(proof),
      anchorHeight,
      const DeepCollectionEquality().hash(rVpk),
      const DeepCollectionEquality().hash(voteAuthSig),
      commitmentBundleJson);

  @override
  String toString() {
    return 'VotingSignedVoteCommitment(proposalId: $proposalId, choice: $choice, voteRoundId: $voteRoundId, vanNullifier: $vanNullifier, voteAuthorityNoteNew: $voteAuthorityNoteNew, voteCommitment: $voteCommitment, proof: $proof, anchorHeight: $anchorHeight, rVpk: $rVpk, voteAuthSig: $voteAuthSig, commitmentBundleJson: $commitmentBundleJson)';
  }
}

/// @nodoc
abstract mixin class $VotingSignedVoteCommitmentCopyWith<$Res> {
  factory $VotingSignedVoteCommitmentCopyWith(VotingSignedVoteCommitment value,
          $Res Function(VotingSignedVoteCommitment) _then) =
      _$VotingSignedVoteCommitmentCopyWithImpl;
  @useResult
  $Res call(
      {int proposalId,
      int choice,
      String voteRoundId,
      Uint8List vanNullifier,
      Uint8List voteAuthorityNoteNew,
      Uint8List voteCommitment,
      Uint8List proof,
      int anchorHeight,
      Uint8List rVpk,
      Uint8List voteAuthSig,
      String commitmentBundleJson});
}

/// @nodoc
class _$VotingSignedVoteCommitmentCopyWithImpl<$Res>
    implements $VotingSignedVoteCommitmentCopyWith<$Res> {
  _$VotingSignedVoteCommitmentCopyWithImpl(this._self, this._then);

  final VotingSignedVoteCommitment _self;
  final $Res Function(VotingSignedVoteCommitment) _then;

  /// Create a copy of VotingSignedVoteCommitment
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? proposalId = null,
    Object? choice = null,
    Object? voteRoundId = null,
    Object? vanNullifier = null,
    Object? voteAuthorityNoteNew = null,
    Object? voteCommitment = null,
    Object? proof = null,
    Object? anchorHeight = null,
    Object? rVpk = null,
    Object? voteAuthSig = null,
    Object? commitmentBundleJson = null,
  }) {
    return _then(_self.copyWith(
      proposalId: null == proposalId
          ? _self.proposalId
          : proposalId // ignore: cast_nullable_to_non_nullable
              as int,
      choice: null == choice
          ? _self.choice
          : choice // ignore: cast_nullable_to_non_nullable
              as int,
      voteRoundId: null == voteRoundId
          ? _self.voteRoundId
          : voteRoundId // ignore: cast_nullable_to_non_nullable
              as String,
      vanNullifier: null == vanNullifier
          ? _self.vanNullifier
          : vanNullifier // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      voteAuthorityNoteNew: null == voteAuthorityNoteNew
          ? _self.voteAuthorityNoteNew
          : voteAuthorityNoteNew // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      voteCommitment: null == voteCommitment
          ? _self.voteCommitment
          : voteCommitment // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      proof: null == proof
          ? _self.proof
          : proof // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      anchorHeight: null == anchorHeight
          ? _self.anchorHeight
          : anchorHeight // ignore: cast_nullable_to_non_nullable
              as int,
      rVpk: null == rVpk
          ? _self.rVpk
          : rVpk // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      voteAuthSig: null == voteAuthSig
          ? _self.voteAuthSig
          : voteAuthSig // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      commitmentBundleJson: null == commitmentBundleJson
          ? _self.commitmentBundleJson
          : commitmentBundleJson // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// Adds pattern-matching-related methods to [VotingSignedVoteCommitment].
extension VotingSignedVoteCommitmentPatterns on VotingSignedVoteCommitment {
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
    TResult Function(_VotingSignedVoteCommitment value)? $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _VotingSignedVoteCommitment() when $default != null:
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
    TResult Function(_VotingSignedVoteCommitment value) $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingSignedVoteCommitment():
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
    TResult? Function(_VotingSignedVoteCommitment value)? $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingSignedVoteCommitment() when $default != null:
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
    TResult Function(
            int proposalId,
            int choice,
            String voteRoundId,
            Uint8List vanNullifier,
            Uint8List voteAuthorityNoteNew,
            Uint8List voteCommitment,
            Uint8List proof,
            int anchorHeight,
            Uint8List rVpk,
            Uint8List voteAuthSig,
            String commitmentBundleJson)?
        $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _VotingSignedVoteCommitment() when $default != null:
        return $default(
            _that.proposalId,
            _that.choice,
            _that.voteRoundId,
            _that.vanNullifier,
            _that.voteAuthorityNoteNew,
            _that.voteCommitment,
            _that.proof,
            _that.anchorHeight,
            _that.rVpk,
            _that.voteAuthSig,
            _that.commitmentBundleJson);
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
    TResult Function(
            int proposalId,
            int choice,
            String voteRoundId,
            Uint8List vanNullifier,
            Uint8List voteAuthorityNoteNew,
            Uint8List voteCommitment,
            Uint8List proof,
            int anchorHeight,
            Uint8List rVpk,
            Uint8List voteAuthSig,
            String commitmentBundleJson)
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingSignedVoteCommitment():
        return $default(
            _that.proposalId,
            _that.choice,
            _that.voteRoundId,
            _that.vanNullifier,
            _that.voteAuthorityNoteNew,
            _that.voteCommitment,
            _that.proof,
            _that.anchorHeight,
            _that.rVpk,
            _that.voteAuthSig,
            _that.commitmentBundleJson);
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
    TResult? Function(
            int proposalId,
            int choice,
            String voteRoundId,
            Uint8List vanNullifier,
            Uint8List voteAuthorityNoteNew,
            Uint8List voteCommitment,
            Uint8List proof,
            int anchorHeight,
            Uint8List rVpk,
            Uint8List voteAuthSig,
            String commitmentBundleJson)?
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingSignedVoteCommitment() when $default != null:
        return $default(
            _that.proposalId,
            _that.choice,
            _that.voteRoundId,
            _that.vanNullifier,
            _that.voteAuthorityNoteNew,
            _that.voteCommitment,
            _that.proof,
            _that.anchorHeight,
            _that.rVpk,
            _that.voteAuthSig,
            _that.commitmentBundleJson);
      case _:
        return null;
    }
  }
}

/// @nodoc

class _VotingSignedVoteCommitment implements VotingSignedVoteCommitment {
  const _VotingSignedVoteCommitment(
      {required this.proposalId,
      required this.choice,
      required this.voteRoundId,
      required this.vanNullifier,
      required this.voteAuthorityNoteNew,
      required this.voteCommitment,
      required this.proof,
      required this.anchorHeight,
      required this.rVpk,
      required this.voteAuthSig,
      required this.commitmentBundleJson});

  @override
  final int proposalId;
  @override
  final int choice;
  @override
  final String voteRoundId;
  @override
  final Uint8List vanNullifier;
  @override
  final Uint8List voteAuthorityNoteNew;
  @override
  final Uint8List voteCommitment;
  @override
  final Uint8List proof;
  @override
  final int anchorHeight;
  @override
  final Uint8List rVpk;
  @override
  final Uint8List voteAuthSig;
  @override
  final String commitmentBundleJson;

  /// Create a copy of VotingSignedVoteCommitment
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  _$VotingSignedVoteCommitmentCopyWith<_VotingSignedVoteCommitment>
      get copyWith => __$VotingSignedVoteCommitmentCopyWithImpl<
          _VotingSignedVoteCommitment>(this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _VotingSignedVoteCommitment &&
            (identical(other.proposalId, proposalId) ||
                other.proposalId == proposalId) &&
            (identical(other.choice, choice) || other.choice == choice) &&
            (identical(other.voteRoundId, voteRoundId) ||
                other.voteRoundId == voteRoundId) &&
            const DeepCollectionEquality()
                .equals(other.vanNullifier, vanNullifier) &&
            const DeepCollectionEquality()
                .equals(other.voteAuthorityNoteNew, voteAuthorityNoteNew) &&
            const DeepCollectionEquality()
                .equals(other.voteCommitment, voteCommitment) &&
            const DeepCollectionEquality().equals(other.proof, proof) &&
            (identical(other.anchorHeight, anchorHeight) ||
                other.anchorHeight == anchorHeight) &&
            const DeepCollectionEquality().equals(other.rVpk, rVpk) &&
            const DeepCollectionEquality()
                .equals(other.voteAuthSig, voteAuthSig) &&
            (identical(other.commitmentBundleJson, commitmentBundleJson) ||
                other.commitmentBundleJson == commitmentBundleJson));
  }

  @override
  int get hashCode => Object.hash(
      runtimeType,
      proposalId,
      choice,
      voteRoundId,
      const DeepCollectionEquality().hash(vanNullifier),
      const DeepCollectionEquality().hash(voteAuthorityNoteNew),
      const DeepCollectionEquality().hash(voteCommitment),
      const DeepCollectionEquality().hash(proof),
      anchorHeight,
      const DeepCollectionEquality().hash(rVpk),
      const DeepCollectionEquality().hash(voteAuthSig),
      commitmentBundleJson);

  @override
  String toString() {
    return 'VotingSignedVoteCommitment(proposalId: $proposalId, choice: $choice, voteRoundId: $voteRoundId, vanNullifier: $vanNullifier, voteAuthorityNoteNew: $voteAuthorityNoteNew, voteCommitment: $voteCommitment, proof: $proof, anchorHeight: $anchorHeight, rVpk: $rVpk, voteAuthSig: $voteAuthSig, commitmentBundleJson: $commitmentBundleJson)';
  }
}

/// @nodoc
abstract mixin class _$VotingSignedVoteCommitmentCopyWith<$Res>
    implements $VotingSignedVoteCommitmentCopyWith<$Res> {
  factory _$VotingSignedVoteCommitmentCopyWith(
          _VotingSignedVoteCommitment value,
          $Res Function(_VotingSignedVoteCommitment) _then) =
      __$VotingSignedVoteCommitmentCopyWithImpl;
  @override
  @useResult
  $Res call(
      {int proposalId,
      int choice,
      String voteRoundId,
      Uint8List vanNullifier,
      Uint8List voteAuthorityNoteNew,
      Uint8List voteCommitment,
      Uint8List proof,
      int anchorHeight,
      Uint8List rVpk,
      Uint8List voteAuthSig,
      String commitmentBundleJson});
}

/// @nodoc
class __$VotingSignedVoteCommitmentCopyWithImpl<$Res>
    implements _$VotingSignedVoteCommitmentCopyWith<$Res> {
  __$VotingSignedVoteCommitmentCopyWithImpl(this._self, this._then);

  final _VotingSignedVoteCommitment _self;
  final $Res Function(_VotingSignedVoteCommitment) _then;

  /// Create a copy of VotingSignedVoteCommitment
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $Res call({
    Object? proposalId = null,
    Object? choice = null,
    Object? voteRoundId = null,
    Object? vanNullifier = null,
    Object? voteAuthorityNoteNew = null,
    Object? voteCommitment = null,
    Object? proof = null,
    Object? anchorHeight = null,
    Object? rVpk = null,
    Object? voteAuthSig = null,
    Object? commitmentBundleJson = null,
  }) {
    return _then(_VotingSignedVoteCommitment(
      proposalId: null == proposalId
          ? _self.proposalId
          : proposalId // ignore: cast_nullable_to_non_nullable
              as int,
      choice: null == choice
          ? _self.choice
          : choice // ignore: cast_nullable_to_non_nullable
              as int,
      voteRoundId: null == voteRoundId
          ? _self.voteRoundId
          : voteRoundId // ignore: cast_nullable_to_non_nullable
              as String,
      vanNullifier: null == vanNullifier
          ? _self.vanNullifier
          : vanNullifier // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      voteAuthorityNoteNew: null == voteAuthorityNoteNew
          ? _self.voteAuthorityNoteNew
          : voteAuthorityNoteNew // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      voteCommitment: null == voteCommitment
          ? _self.voteCommitment
          : voteCommitment // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      proof: null == proof
          ? _self.proof
          : proof // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      anchorHeight: null == anchorHeight
          ? _self.anchorHeight
          : anchorHeight // ignore: cast_nullable_to_non_nullable
              as int,
      rVpk: null == rVpk
          ? _self.rVpk
          : rVpk // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      voteAuthSig: null == voteAuthSig
          ? _self.voteAuthSig
          : voteAuthSig // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      commitmentBundleJson: null == commitmentBundleJson
          ? _self.commitmentBundleJson
          : commitmentBundleJson // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// @nodoc
mixin _$VotingVanWitness {
  List<Uint8List> get authPath;
  int get position;
  int get anchorHeight;

  /// Create a copy of VotingVanWitness
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $VotingVanWitnessCopyWith<VotingVanWitness> get copyWith =>
      _$VotingVanWitnessCopyWithImpl<VotingVanWitness>(
          this as VotingVanWitness, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is VotingVanWitness &&
            const DeepCollectionEquality().equals(other.authPath, authPath) &&
            (identical(other.position, position) ||
                other.position == position) &&
            (identical(other.anchorHeight, anchorHeight) ||
                other.anchorHeight == anchorHeight));
  }

  @override
  int get hashCode => Object.hash(runtimeType,
      const DeepCollectionEquality().hash(authPath), position, anchorHeight);

  @override
  String toString() {
    return 'VotingVanWitness(authPath: $authPath, position: $position, anchorHeight: $anchorHeight)';
  }
}

/// @nodoc
abstract mixin class $VotingVanWitnessCopyWith<$Res> {
  factory $VotingVanWitnessCopyWith(
          VotingVanWitness value, $Res Function(VotingVanWitness) _then) =
      _$VotingVanWitnessCopyWithImpl;
  @useResult
  $Res call({List<Uint8List> authPath, int position, int anchorHeight});
}

/// @nodoc
class _$VotingVanWitnessCopyWithImpl<$Res>
    implements $VotingVanWitnessCopyWith<$Res> {
  _$VotingVanWitnessCopyWithImpl(this._self, this._then);

  final VotingVanWitness _self;
  final $Res Function(VotingVanWitness) _then;

  /// Create a copy of VotingVanWitness
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? authPath = null,
    Object? position = null,
    Object? anchorHeight = null,
  }) {
    return _then(_self.copyWith(
      authPath: null == authPath
          ? _self.authPath
          : authPath // ignore: cast_nullable_to_non_nullable
              as List<Uint8List>,
      position: null == position
          ? _self.position
          : position // ignore: cast_nullable_to_non_nullable
              as int,
      anchorHeight: null == anchorHeight
          ? _self.anchorHeight
          : anchorHeight // ignore: cast_nullable_to_non_nullable
              as int,
    ));
  }
}

/// Adds pattern-matching-related methods to [VotingVanWitness].
extension VotingVanWitnessPatterns on VotingVanWitness {
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
    TResult Function(_VotingVanWitness value)? $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _VotingVanWitness() when $default != null:
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
    TResult Function(_VotingVanWitness value) $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingVanWitness():
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
    TResult? Function(_VotingVanWitness value)? $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingVanWitness() when $default != null:
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
    TResult Function(List<Uint8List> authPath, int position, int anchorHeight)?
        $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _VotingVanWitness() when $default != null:
        return $default(_that.authPath, _that.position, _that.anchorHeight);
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
    TResult Function(List<Uint8List> authPath, int position, int anchorHeight)
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingVanWitness():
        return $default(_that.authPath, _that.position, _that.anchorHeight);
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
    TResult? Function(List<Uint8List> authPath, int position, int anchorHeight)?
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingVanWitness() when $default != null:
        return $default(_that.authPath, _that.position, _that.anchorHeight);
      case _:
        return null;
    }
  }
}

/// @nodoc

class _VotingVanWitness implements VotingVanWitness {
  const _VotingVanWitness(
      {required final List<Uint8List> authPath,
      required this.position,
      required this.anchorHeight})
      : _authPath = authPath;

  final List<Uint8List> _authPath;
  @override
  List<Uint8List> get authPath {
    if (_authPath is EqualUnmodifiableListView) return _authPath;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_authPath);
  }

  @override
  final int position;
  @override
  final int anchorHeight;

  /// Create a copy of VotingVanWitness
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  _$VotingVanWitnessCopyWith<_VotingVanWitness> get copyWith =>
      __$VotingVanWitnessCopyWithImpl<_VotingVanWitness>(this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _VotingVanWitness &&
            const DeepCollectionEquality().equals(other._authPath, _authPath) &&
            (identical(other.position, position) ||
                other.position == position) &&
            (identical(other.anchorHeight, anchorHeight) ||
                other.anchorHeight == anchorHeight));
  }

  @override
  int get hashCode => Object.hash(runtimeType,
      const DeepCollectionEquality().hash(_authPath), position, anchorHeight);

  @override
  String toString() {
    return 'VotingVanWitness(authPath: $authPath, position: $position, anchorHeight: $anchorHeight)';
  }
}

/// @nodoc
abstract mixin class _$VotingVanWitnessCopyWith<$Res>
    implements $VotingVanWitnessCopyWith<$Res> {
  factory _$VotingVanWitnessCopyWith(
          _VotingVanWitness value, $Res Function(_VotingVanWitness) _then) =
      __$VotingVanWitnessCopyWithImpl;
  @override
  @useResult
  $Res call({List<Uint8List> authPath, int position, int anchorHeight});
}

/// @nodoc
class __$VotingVanWitnessCopyWithImpl<$Res>
    implements _$VotingVanWitnessCopyWith<$Res> {
  __$VotingVanWitnessCopyWithImpl(this._self, this._then);

  final _VotingVanWitness _self;
  final $Res Function(_VotingVanWitness) _then;

  /// Create a copy of VotingVanWitness
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $Res call({
    Object? authPath = null,
    Object? position = null,
    Object? anchorHeight = null,
  }) {
    return _then(_VotingVanWitness(
      authPath: null == authPath
          ? _self._authPath
          : authPath // ignore: cast_nullable_to_non_nullable
              as List<Uint8List>,
      position: null == position
          ? _self.position
          : position // ignore: cast_nullable_to_non_nullable
              as int,
      anchorHeight: null == anchorHeight
          ? _self.anchorHeight
          : anchorHeight // ignore: cast_nullable_to_non_nullable
              as int,
    ));
  }
}

/// @nodoc
mixin _$VotingVoteCommitStage {
  int get proposalId;
  int get bundleIndex;

  /// Create a copy of VotingVoteCommitStage
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $VotingVoteCommitStageCopyWith<VotingVoteCommitStage> get copyWith =>
      _$VotingVoteCommitStageCopyWithImpl<VotingVoteCommitStage>(
          this as VotingVoteCommitStage, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is VotingVoteCommitStage &&
            (identical(other.proposalId, proposalId) ||
                other.proposalId == proposalId) &&
            (identical(other.bundleIndex, bundleIndex) ||
                other.bundleIndex == bundleIndex));
  }

  @override
  int get hashCode => Object.hash(runtimeType, proposalId, bundleIndex);

  @override
  String toString() {
    return 'VotingVoteCommitStage(proposalId: $proposalId, bundleIndex: $bundleIndex)';
  }
}

/// @nodoc
abstract mixin class $VotingVoteCommitStageCopyWith<$Res> {
  factory $VotingVoteCommitStageCopyWith(VotingVoteCommitStage value,
          $Res Function(VotingVoteCommitStage) _then) =
      _$VotingVoteCommitStageCopyWithImpl;
  @useResult
  $Res call({int proposalId, int bundleIndex});
}

/// @nodoc
class _$VotingVoteCommitStageCopyWithImpl<$Res>
    implements $VotingVoteCommitStageCopyWith<$Res> {
  _$VotingVoteCommitStageCopyWithImpl(this._self, this._then);

  final VotingVoteCommitStage _self;
  final $Res Function(VotingVoteCommitStage) _then;

  /// Create a copy of VotingVoteCommitStage
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? proposalId = null,
    Object? bundleIndex = null,
  }) {
    return _then(_self.copyWith(
      proposalId: null == proposalId
          ? _self.proposalId
          : proposalId // ignore: cast_nullable_to_non_nullable
              as int,
      bundleIndex: null == bundleIndex
          ? _self.bundleIndex
          : bundleIndex // ignore: cast_nullable_to_non_nullable
              as int,
    ));
  }
}

/// Adds pattern-matching-related methods to [VotingVoteCommitStage].
extension VotingVoteCommitStagePatterns on VotingVoteCommitStage {
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
    TResult Function(VotingVoteCommitStage_ProofStarting value)? proofStarting,
    TResult Function(VotingVoteCommitStage_ProofProgress value)? proofProgress,
    TResult Function(VotingVoteCommitStage_SharePayloadsBuilding value)?
        sharePayloadsBuilding,
    TResult Function(VotingVoteCommitStage_Signing value)? signing,
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case VotingVoteCommitStage_ProofStarting() when proofStarting != null:
        return proofStarting(_that);
      case VotingVoteCommitStage_ProofProgress() when proofProgress != null:
        return proofProgress(_that);
      case VotingVoteCommitStage_SharePayloadsBuilding()
          when sharePayloadsBuilding != null:
        return sharePayloadsBuilding(_that);
      case VotingVoteCommitStage_Signing() when signing != null:
        return signing(_that);
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
    required TResult Function(VotingVoteCommitStage_ProofStarting value)
        proofStarting,
    required TResult Function(VotingVoteCommitStage_ProofProgress value)
        proofProgress,
    required TResult Function(VotingVoteCommitStage_SharePayloadsBuilding value)
        sharePayloadsBuilding,
    required TResult Function(VotingVoteCommitStage_Signing value) signing,
  }) {
    final _that = this;
    switch (_that) {
      case VotingVoteCommitStage_ProofStarting():
        return proofStarting(_that);
      case VotingVoteCommitStage_ProofProgress():
        return proofProgress(_that);
      case VotingVoteCommitStage_SharePayloadsBuilding():
        return sharePayloadsBuilding(_that);
      case VotingVoteCommitStage_Signing():
        return signing(_that);
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
    TResult? Function(VotingVoteCommitStage_ProofStarting value)? proofStarting,
    TResult? Function(VotingVoteCommitStage_ProofProgress value)? proofProgress,
    TResult? Function(VotingVoteCommitStage_SharePayloadsBuilding value)?
        sharePayloadsBuilding,
    TResult? Function(VotingVoteCommitStage_Signing value)? signing,
  }) {
    final _that = this;
    switch (_that) {
      case VotingVoteCommitStage_ProofStarting() when proofStarting != null:
        return proofStarting(_that);
      case VotingVoteCommitStage_ProofProgress() when proofProgress != null:
        return proofProgress(_that);
      case VotingVoteCommitStage_SharePayloadsBuilding()
          when sharePayloadsBuilding != null:
        return sharePayloadsBuilding(_that);
      case VotingVoteCommitStage_Signing() when signing != null:
        return signing(_that);
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
    TResult Function(int proposalId, int bundleIndex)? proofStarting,
    TResult Function(int proposalId, int bundleIndex, double progress)?
        proofProgress,
    TResult Function(int proposalId, int bundleIndex)? sharePayloadsBuilding,
    TResult Function(int proposalId, int bundleIndex)? signing,
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case VotingVoteCommitStage_ProofStarting() when proofStarting != null:
        return proofStarting(_that.proposalId, _that.bundleIndex);
      case VotingVoteCommitStage_ProofProgress() when proofProgress != null:
        return proofProgress(
            _that.proposalId, _that.bundleIndex, _that.progress);
      case VotingVoteCommitStage_SharePayloadsBuilding()
          when sharePayloadsBuilding != null:
        return sharePayloadsBuilding(_that.proposalId, _that.bundleIndex);
      case VotingVoteCommitStage_Signing() when signing != null:
        return signing(_that.proposalId, _that.bundleIndex);
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
    required TResult Function(int proposalId, int bundleIndex) proofStarting,
    required TResult Function(int proposalId, int bundleIndex, double progress)
        proofProgress,
    required TResult Function(int proposalId, int bundleIndex)
        sharePayloadsBuilding,
    required TResult Function(int proposalId, int bundleIndex) signing,
  }) {
    final _that = this;
    switch (_that) {
      case VotingVoteCommitStage_ProofStarting():
        return proofStarting(_that.proposalId, _that.bundleIndex);
      case VotingVoteCommitStage_ProofProgress():
        return proofProgress(
            _that.proposalId, _that.bundleIndex, _that.progress);
      case VotingVoteCommitStage_SharePayloadsBuilding():
        return sharePayloadsBuilding(_that.proposalId, _that.bundleIndex);
      case VotingVoteCommitStage_Signing():
        return signing(_that.proposalId, _that.bundleIndex);
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
    TResult? Function(int proposalId, int bundleIndex)? proofStarting,
    TResult? Function(int proposalId, int bundleIndex, double progress)?
        proofProgress,
    TResult? Function(int proposalId, int bundleIndex)? sharePayloadsBuilding,
    TResult? Function(int proposalId, int bundleIndex)? signing,
  }) {
    final _that = this;
    switch (_that) {
      case VotingVoteCommitStage_ProofStarting() when proofStarting != null:
        return proofStarting(_that.proposalId, _that.bundleIndex);
      case VotingVoteCommitStage_ProofProgress() when proofProgress != null:
        return proofProgress(
            _that.proposalId, _that.bundleIndex, _that.progress);
      case VotingVoteCommitStage_SharePayloadsBuilding()
          when sharePayloadsBuilding != null:
        return sharePayloadsBuilding(_that.proposalId, _that.bundleIndex);
      case VotingVoteCommitStage_Signing() when signing != null:
        return signing(_that.proposalId, _that.bundleIndex);
      case _:
        return null;
    }
  }
}

/// @nodoc

class VotingVoteCommitStage_ProofStarting extends VotingVoteCommitStage {
  const VotingVoteCommitStage_ProofStarting(
      {required this.proposalId, required this.bundleIndex})
      : super._();

  @override
  final int proposalId;
  @override
  final int bundleIndex;

  /// Create a copy of VotingVoteCommitStage
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $VotingVoteCommitStage_ProofStartingCopyWith<
          VotingVoteCommitStage_ProofStarting>
      get copyWith => _$VotingVoteCommitStage_ProofStartingCopyWithImpl<
          VotingVoteCommitStage_ProofStarting>(this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is VotingVoteCommitStage_ProofStarting &&
            (identical(other.proposalId, proposalId) ||
                other.proposalId == proposalId) &&
            (identical(other.bundleIndex, bundleIndex) ||
                other.bundleIndex == bundleIndex));
  }

  @override
  int get hashCode => Object.hash(runtimeType, proposalId, bundleIndex);

  @override
  String toString() {
    return 'VotingVoteCommitStage.proofStarting(proposalId: $proposalId, bundleIndex: $bundleIndex)';
  }
}

/// @nodoc
abstract mixin class $VotingVoteCommitStage_ProofStartingCopyWith<$Res>
    implements $VotingVoteCommitStageCopyWith<$Res> {
  factory $VotingVoteCommitStage_ProofStartingCopyWith(
          VotingVoteCommitStage_ProofStarting value,
          $Res Function(VotingVoteCommitStage_ProofStarting) _then) =
      _$VotingVoteCommitStage_ProofStartingCopyWithImpl;
  @override
  @useResult
  $Res call({int proposalId, int bundleIndex});
}

/// @nodoc
class _$VotingVoteCommitStage_ProofStartingCopyWithImpl<$Res>
    implements $VotingVoteCommitStage_ProofStartingCopyWith<$Res> {
  _$VotingVoteCommitStage_ProofStartingCopyWithImpl(this._self, this._then);

  final VotingVoteCommitStage_ProofStarting _self;
  final $Res Function(VotingVoteCommitStage_ProofStarting) _then;

  /// Create a copy of VotingVoteCommitStage
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $Res call({
    Object? proposalId = null,
    Object? bundleIndex = null,
  }) {
    return _then(VotingVoteCommitStage_ProofStarting(
      proposalId: null == proposalId
          ? _self.proposalId
          : proposalId // ignore: cast_nullable_to_non_nullable
              as int,
      bundleIndex: null == bundleIndex
          ? _self.bundleIndex
          : bundleIndex // ignore: cast_nullable_to_non_nullable
              as int,
    ));
  }
}

/// @nodoc

class VotingVoteCommitStage_ProofProgress extends VotingVoteCommitStage {
  const VotingVoteCommitStage_ProofProgress(
      {required this.proposalId,
      required this.bundleIndex,
      required this.progress})
      : super._();

  @override
  final int proposalId;
  @override
  final int bundleIndex;
  final double progress;

  /// Create a copy of VotingVoteCommitStage
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $VotingVoteCommitStage_ProofProgressCopyWith<
          VotingVoteCommitStage_ProofProgress>
      get copyWith => _$VotingVoteCommitStage_ProofProgressCopyWithImpl<
          VotingVoteCommitStage_ProofProgress>(this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is VotingVoteCommitStage_ProofProgress &&
            (identical(other.proposalId, proposalId) ||
                other.proposalId == proposalId) &&
            (identical(other.bundleIndex, bundleIndex) ||
                other.bundleIndex == bundleIndex) &&
            (identical(other.progress, progress) ||
                other.progress == progress));
  }

  @override
  int get hashCode =>
      Object.hash(runtimeType, proposalId, bundleIndex, progress);

  @override
  String toString() {
    return 'VotingVoteCommitStage.proofProgress(proposalId: $proposalId, bundleIndex: $bundleIndex, progress: $progress)';
  }
}

/// @nodoc
abstract mixin class $VotingVoteCommitStage_ProofProgressCopyWith<$Res>
    implements $VotingVoteCommitStageCopyWith<$Res> {
  factory $VotingVoteCommitStage_ProofProgressCopyWith(
          VotingVoteCommitStage_ProofProgress value,
          $Res Function(VotingVoteCommitStage_ProofProgress) _then) =
      _$VotingVoteCommitStage_ProofProgressCopyWithImpl;
  @override
  @useResult
  $Res call({int proposalId, int bundleIndex, double progress});
}

/// @nodoc
class _$VotingVoteCommitStage_ProofProgressCopyWithImpl<$Res>
    implements $VotingVoteCommitStage_ProofProgressCopyWith<$Res> {
  _$VotingVoteCommitStage_ProofProgressCopyWithImpl(this._self, this._then);

  final VotingVoteCommitStage_ProofProgress _self;
  final $Res Function(VotingVoteCommitStage_ProofProgress) _then;

  /// Create a copy of VotingVoteCommitStage
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $Res call({
    Object? proposalId = null,
    Object? bundleIndex = null,
    Object? progress = null,
  }) {
    return _then(VotingVoteCommitStage_ProofProgress(
      proposalId: null == proposalId
          ? _self.proposalId
          : proposalId // ignore: cast_nullable_to_non_nullable
              as int,
      bundleIndex: null == bundleIndex
          ? _self.bundleIndex
          : bundleIndex // ignore: cast_nullable_to_non_nullable
              as int,
      progress: null == progress
          ? _self.progress
          : progress // ignore: cast_nullable_to_non_nullable
              as double,
    ));
  }
}

/// @nodoc

class VotingVoteCommitStage_SharePayloadsBuilding
    extends VotingVoteCommitStage {
  const VotingVoteCommitStage_SharePayloadsBuilding(
      {required this.proposalId, required this.bundleIndex})
      : super._();

  @override
  final int proposalId;
  @override
  final int bundleIndex;

  /// Create a copy of VotingVoteCommitStage
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $VotingVoteCommitStage_SharePayloadsBuildingCopyWith<
          VotingVoteCommitStage_SharePayloadsBuilding>
      get copyWith => _$VotingVoteCommitStage_SharePayloadsBuildingCopyWithImpl<
          VotingVoteCommitStage_SharePayloadsBuilding>(this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is VotingVoteCommitStage_SharePayloadsBuilding &&
            (identical(other.proposalId, proposalId) ||
                other.proposalId == proposalId) &&
            (identical(other.bundleIndex, bundleIndex) ||
                other.bundleIndex == bundleIndex));
  }

  @override
  int get hashCode => Object.hash(runtimeType, proposalId, bundleIndex);

  @override
  String toString() {
    return 'VotingVoteCommitStage.sharePayloadsBuilding(proposalId: $proposalId, bundleIndex: $bundleIndex)';
  }
}

/// @nodoc
abstract mixin class $VotingVoteCommitStage_SharePayloadsBuildingCopyWith<$Res>
    implements $VotingVoteCommitStageCopyWith<$Res> {
  factory $VotingVoteCommitStage_SharePayloadsBuildingCopyWith(
          VotingVoteCommitStage_SharePayloadsBuilding value,
          $Res Function(VotingVoteCommitStage_SharePayloadsBuilding) _then) =
      _$VotingVoteCommitStage_SharePayloadsBuildingCopyWithImpl;
  @override
  @useResult
  $Res call({int proposalId, int bundleIndex});
}

/// @nodoc
class _$VotingVoteCommitStage_SharePayloadsBuildingCopyWithImpl<$Res>
    implements $VotingVoteCommitStage_SharePayloadsBuildingCopyWith<$Res> {
  _$VotingVoteCommitStage_SharePayloadsBuildingCopyWithImpl(
      this._self, this._then);

  final VotingVoteCommitStage_SharePayloadsBuilding _self;
  final $Res Function(VotingVoteCommitStage_SharePayloadsBuilding) _then;

  /// Create a copy of VotingVoteCommitStage
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $Res call({
    Object? proposalId = null,
    Object? bundleIndex = null,
  }) {
    return _then(VotingVoteCommitStage_SharePayloadsBuilding(
      proposalId: null == proposalId
          ? _self.proposalId
          : proposalId // ignore: cast_nullable_to_non_nullable
              as int,
      bundleIndex: null == bundleIndex
          ? _self.bundleIndex
          : bundleIndex // ignore: cast_nullable_to_non_nullable
              as int,
    ));
  }
}

/// @nodoc

class VotingVoteCommitStage_Signing extends VotingVoteCommitStage {
  const VotingVoteCommitStage_Signing(
      {required this.proposalId, required this.bundleIndex})
      : super._();

  @override
  final int proposalId;
  @override
  final int bundleIndex;

  /// Create a copy of VotingVoteCommitStage
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $VotingVoteCommitStage_SigningCopyWith<VotingVoteCommitStage_Signing>
      get copyWith => _$VotingVoteCommitStage_SigningCopyWithImpl<
          VotingVoteCommitStage_Signing>(this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is VotingVoteCommitStage_Signing &&
            (identical(other.proposalId, proposalId) ||
                other.proposalId == proposalId) &&
            (identical(other.bundleIndex, bundleIndex) ||
                other.bundleIndex == bundleIndex));
  }

  @override
  int get hashCode => Object.hash(runtimeType, proposalId, bundleIndex);

  @override
  String toString() {
    return 'VotingVoteCommitStage.signing(proposalId: $proposalId, bundleIndex: $bundleIndex)';
  }
}

/// @nodoc
abstract mixin class $VotingVoteCommitStage_SigningCopyWith<$Res>
    implements $VotingVoteCommitStageCopyWith<$Res> {
  factory $VotingVoteCommitStage_SigningCopyWith(
          VotingVoteCommitStage_Signing value,
          $Res Function(VotingVoteCommitStage_Signing) _then) =
      _$VotingVoteCommitStage_SigningCopyWithImpl;
  @override
  @useResult
  $Res call({int proposalId, int bundleIndex});
}

/// @nodoc
class _$VotingVoteCommitStage_SigningCopyWithImpl<$Res>
    implements $VotingVoteCommitStage_SigningCopyWith<$Res> {
  _$VotingVoteCommitStage_SigningCopyWithImpl(this._self, this._then);

  final VotingVoteCommitStage_Signing _self;
  final $Res Function(VotingVoteCommitStage_Signing) _then;

  /// Create a copy of VotingVoteCommitStage
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $Res call({
    Object? proposalId = null,
    Object? bundleIndex = null,
  }) {
    return _then(VotingVoteCommitStage_Signing(
      proposalId: null == proposalId
          ? _self.proposalId
          : proposalId // ignore: cast_nullable_to_non_nullable
              as int,
      bundleIndex: null == bundleIndex
          ? _self.bundleIndex
          : bundleIndex // ignore: cast_nullable_to_non_nullable
              as int,
    ));
  }
}

/// @nodoc
mixin _$VotingVoteCommitments {
  int get bundleIndex;
  List<VotingSignedVoteCommitment> get commitments;

  /// Create a copy of VotingVoteCommitments
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $VotingVoteCommitmentsCopyWith<VotingVoteCommitments> get copyWith =>
      _$VotingVoteCommitmentsCopyWithImpl<VotingVoteCommitments>(
          this as VotingVoteCommitments, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is VotingVoteCommitments &&
            (identical(other.bundleIndex, bundleIndex) ||
                other.bundleIndex == bundleIndex) &&
            const DeepCollectionEquality()
                .equals(other.commitments, commitments));
  }

  @override
  int get hashCode => Object.hash(runtimeType, bundleIndex,
      const DeepCollectionEquality().hash(commitments));

  @override
  String toString() {
    return 'VotingVoteCommitments(bundleIndex: $bundleIndex, commitments: $commitments)';
  }
}

/// @nodoc
abstract mixin class $VotingVoteCommitmentsCopyWith<$Res> {
  factory $VotingVoteCommitmentsCopyWith(VotingVoteCommitments value,
          $Res Function(VotingVoteCommitments) _then) =
      _$VotingVoteCommitmentsCopyWithImpl;
  @useResult
  $Res call({int bundleIndex, List<VotingSignedVoteCommitment> commitments});
}

/// @nodoc
class _$VotingVoteCommitmentsCopyWithImpl<$Res>
    implements $VotingVoteCommitmentsCopyWith<$Res> {
  _$VotingVoteCommitmentsCopyWithImpl(this._self, this._then);

  final VotingVoteCommitments _self;
  final $Res Function(VotingVoteCommitments) _then;

  /// Create a copy of VotingVoteCommitments
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? bundleIndex = null,
    Object? commitments = null,
  }) {
    return _then(_self.copyWith(
      bundleIndex: null == bundleIndex
          ? _self.bundleIndex
          : bundleIndex // ignore: cast_nullable_to_non_nullable
              as int,
      commitments: null == commitments
          ? _self.commitments
          : commitments // ignore: cast_nullable_to_non_nullable
              as List<VotingSignedVoteCommitment>,
    ));
  }
}

/// Adds pattern-matching-related methods to [VotingVoteCommitments].
extension VotingVoteCommitmentsPatterns on VotingVoteCommitments {
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
    TResult Function(_VotingVoteCommitments value)? $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _VotingVoteCommitments() when $default != null:
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
    TResult Function(_VotingVoteCommitments value) $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingVoteCommitments():
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
    TResult? Function(_VotingVoteCommitments value)? $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingVoteCommitments() when $default != null:
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
    TResult Function(
            int bundleIndex, List<VotingSignedVoteCommitment> commitments)?
        $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _VotingVoteCommitments() when $default != null:
        return $default(_that.bundleIndex, _that.commitments);
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
    TResult Function(
            int bundleIndex, List<VotingSignedVoteCommitment> commitments)
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingVoteCommitments():
        return $default(_that.bundleIndex, _that.commitments);
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
    TResult? Function(
            int bundleIndex, List<VotingSignedVoteCommitment> commitments)?
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingVoteCommitments() when $default != null:
        return $default(_that.bundleIndex, _that.commitments);
      case _:
        return null;
    }
  }
}

/// @nodoc

class _VotingVoteCommitments implements VotingVoteCommitments {
  const _VotingVoteCommitments(
      {required this.bundleIndex,
      required final List<VotingSignedVoteCommitment> commitments})
      : _commitments = commitments;

  @override
  final int bundleIndex;
  final List<VotingSignedVoteCommitment> _commitments;
  @override
  List<VotingSignedVoteCommitment> get commitments {
    if (_commitments is EqualUnmodifiableListView) return _commitments;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_commitments);
  }

  /// Create a copy of VotingVoteCommitments
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  _$VotingVoteCommitmentsCopyWith<_VotingVoteCommitments> get copyWith =>
      __$VotingVoteCommitmentsCopyWithImpl<_VotingVoteCommitments>(
          this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _VotingVoteCommitments &&
            (identical(other.bundleIndex, bundleIndex) ||
                other.bundleIndex == bundleIndex) &&
            const DeepCollectionEquality()
                .equals(other._commitments, _commitments));
  }

  @override
  int get hashCode => Object.hash(runtimeType, bundleIndex,
      const DeepCollectionEquality().hash(_commitments));

  @override
  String toString() {
    return 'VotingVoteCommitments(bundleIndex: $bundleIndex, commitments: $commitments)';
  }
}

/// @nodoc
abstract mixin class _$VotingVoteCommitmentsCopyWith<$Res>
    implements $VotingVoteCommitmentsCopyWith<$Res> {
  factory _$VotingVoteCommitmentsCopyWith(_VotingVoteCommitments value,
          $Res Function(_VotingVoteCommitments) _then) =
      __$VotingVoteCommitmentsCopyWithImpl;
  @override
  @useResult
  $Res call({int bundleIndex, List<VotingSignedVoteCommitment> commitments});
}

/// @nodoc
class __$VotingVoteCommitmentsCopyWithImpl<$Res>
    implements _$VotingVoteCommitmentsCopyWith<$Res> {
  __$VotingVoteCommitmentsCopyWithImpl(this._self, this._then);

  final _VotingVoteCommitments _self;
  final $Res Function(_VotingVoteCommitments) _then;

  /// Create a copy of VotingVoteCommitments
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $Res call({
    Object? bundleIndex = null,
    Object? commitments = null,
  }) {
    return _then(_VotingVoteCommitments(
      bundleIndex: null == bundleIndex
          ? _self.bundleIndex
          : bundleIndex // ignore: cast_nullable_to_non_nullable
              as int,
      commitments: null == commitments
          ? _self._commitments
          : commitments // ignore: cast_nullable_to_non_nullable
              as List<VotingSignedVoteCommitment>,
    ));
  }
}

/// @nodoc
mixin _$VotingVoteConfirmation {
  String get txHash;
  int get vanLeafPosition;
  BigInt get vcTreePosition;

  /// Create a copy of VotingVoteConfirmation
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $VotingVoteConfirmationCopyWith<VotingVoteConfirmation> get copyWith =>
      _$VotingVoteConfirmationCopyWithImpl<VotingVoteConfirmation>(
          this as VotingVoteConfirmation, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is VotingVoteConfirmation &&
            (identical(other.txHash, txHash) || other.txHash == txHash) &&
            (identical(other.vanLeafPosition, vanLeafPosition) ||
                other.vanLeafPosition == vanLeafPosition) &&
            (identical(other.vcTreePosition, vcTreePosition) ||
                other.vcTreePosition == vcTreePosition));
  }

  @override
  int get hashCode =>
      Object.hash(runtimeType, txHash, vanLeafPosition, vcTreePosition);

  @override
  String toString() {
    return 'VotingVoteConfirmation(txHash: $txHash, vanLeafPosition: $vanLeafPosition, vcTreePosition: $vcTreePosition)';
  }
}

/// @nodoc
abstract mixin class $VotingVoteConfirmationCopyWith<$Res> {
  factory $VotingVoteConfirmationCopyWith(VotingVoteConfirmation value,
          $Res Function(VotingVoteConfirmation) _then) =
      _$VotingVoteConfirmationCopyWithImpl;
  @useResult
  $Res call({String txHash, int vanLeafPosition, BigInt vcTreePosition});
}

/// @nodoc
class _$VotingVoteConfirmationCopyWithImpl<$Res>
    implements $VotingVoteConfirmationCopyWith<$Res> {
  _$VotingVoteConfirmationCopyWithImpl(this._self, this._then);

  final VotingVoteConfirmation _self;
  final $Res Function(VotingVoteConfirmation) _then;

  /// Create a copy of VotingVoteConfirmation
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? txHash = null,
    Object? vanLeafPosition = null,
    Object? vcTreePosition = null,
  }) {
    return _then(_self.copyWith(
      txHash: null == txHash
          ? _self.txHash
          : txHash // ignore: cast_nullable_to_non_nullable
              as String,
      vanLeafPosition: null == vanLeafPosition
          ? _self.vanLeafPosition
          : vanLeafPosition // ignore: cast_nullable_to_non_nullable
              as int,
      vcTreePosition: null == vcTreePosition
          ? _self.vcTreePosition
          : vcTreePosition // ignore: cast_nullable_to_non_nullable
              as BigInt,
    ));
  }
}

/// Adds pattern-matching-related methods to [VotingVoteConfirmation].
extension VotingVoteConfirmationPatterns on VotingVoteConfirmation {
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
    TResult Function(_VotingVoteConfirmation value)? $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _VotingVoteConfirmation() when $default != null:
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
    TResult Function(_VotingVoteConfirmation value) $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingVoteConfirmation():
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
    TResult? Function(_VotingVoteConfirmation value)? $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingVoteConfirmation() when $default != null:
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
    TResult Function(String txHash, int vanLeafPosition, BigInt vcTreePosition)?
        $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _VotingVoteConfirmation() when $default != null:
        return $default(
            _that.txHash, _that.vanLeafPosition, _that.vcTreePosition);
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
    TResult Function(String txHash, int vanLeafPosition, BigInt vcTreePosition)
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingVoteConfirmation():
        return $default(
            _that.txHash, _that.vanLeafPosition, _that.vcTreePosition);
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
    TResult? Function(
            String txHash, int vanLeafPosition, BigInt vcTreePosition)?
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingVoteConfirmation() when $default != null:
        return $default(
            _that.txHash, _that.vanLeafPosition, _that.vcTreePosition);
      case _:
        return null;
    }
  }
}

/// @nodoc

class _VotingVoteConfirmation implements VotingVoteConfirmation {
  const _VotingVoteConfirmation(
      {required this.txHash,
      required this.vanLeafPosition,
      required this.vcTreePosition});

  @override
  final String txHash;
  @override
  final int vanLeafPosition;
  @override
  final BigInt vcTreePosition;

  /// Create a copy of VotingVoteConfirmation
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  _$VotingVoteConfirmationCopyWith<_VotingVoteConfirmation> get copyWith =>
      __$VotingVoteConfirmationCopyWithImpl<_VotingVoteConfirmation>(
          this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _VotingVoteConfirmation &&
            (identical(other.txHash, txHash) || other.txHash == txHash) &&
            (identical(other.vanLeafPosition, vanLeafPosition) ||
                other.vanLeafPosition == vanLeafPosition) &&
            (identical(other.vcTreePosition, vcTreePosition) ||
                other.vcTreePosition == vcTreePosition));
  }

  @override
  int get hashCode =>
      Object.hash(runtimeType, txHash, vanLeafPosition, vcTreePosition);

  @override
  String toString() {
    return 'VotingVoteConfirmation(txHash: $txHash, vanLeafPosition: $vanLeafPosition, vcTreePosition: $vcTreePosition)';
  }
}

/// @nodoc
abstract mixin class _$VotingVoteConfirmationCopyWith<$Res>
    implements $VotingVoteConfirmationCopyWith<$Res> {
  factory _$VotingVoteConfirmationCopyWith(_VotingVoteConfirmation value,
          $Res Function(_VotingVoteConfirmation) _then) =
      __$VotingVoteConfirmationCopyWithImpl;
  @override
  @useResult
  $Res call({String txHash, int vanLeafPosition, BigInt vcTreePosition});
}

/// @nodoc
class __$VotingVoteConfirmationCopyWithImpl<$Res>
    implements _$VotingVoteConfirmationCopyWith<$Res> {
  __$VotingVoteConfirmationCopyWithImpl(this._self, this._then);

  final _VotingVoteConfirmation _self;
  final $Res Function(_VotingVoteConfirmation) _then;

  /// Create a copy of VotingVoteConfirmation
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $Res call({
    Object? txHash = null,
    Object? vanLeafPosition = null,
    Object? vcTreePosition = null,
  }) {
    return _then(_VotingVoteConfirmation(
      txHash: null == txHash
          ? _self.txHash
          : txHash // ignore: cast_nullable_to_non_nullable
              as String,
      vanLeafPosition: null == vanLeafPosition
          ? _self.vanLeafPosition
          : vanLeafPosition // ignore: cast_nullable_to_non_nullable
              as int,
      vcTreePosition: null == vcTreePosition
          ? _self.vcTreePosition
          : vcTreePosition // ignore: cast_nullable_to_non_nullable
              as BigInt,
    ));
  }
}

/// @nodoc
mixin _$VotingVotePayloads {
  VotingVoteSubmission get submission;
  List<VotingSharePayload> get sharePayloads;

  /// Create a copy of VotingVotePayloads
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $VotingVotePayloadsCopyWith<VotingVotePayloads> get copyWith =>
      _$VotingVotePayloadsCopyWithImpl<VotingVotePayloads>(
          this as VotingVotePayloads, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is VotingVotePayloads &&
            (identical(other.submission, submission) ||
                other.submission == submission) &&
            const DeepCollectionEquality()
                .equals(other.sharePayloads, sharePayloads));
  }

  @override
  int get hashCode => Object.hash(runtimeType, submission,
      const DeepCollectionEquality().hash(sharePayloads));

  @override
  String toString() {
    return 'VotingVotePayloads(submission: $submission, sharePayloads: $sharePayloads)';
  }
}

/// @nodoc
abstract mixin class $VotingVotePayloadsCopyWith<$Res> {
  factory $VotingVotePayloadsCopyWith(
          VotingVotePayloads value, $Res Function(VotingVotePayloads) _then) =
      _$VotingVotePayloadsCopyWithImpl;
  @useResult
  $Res call(
      {VotingVoteSubmission submission,
      List<VotingSharePayload> sharePayloads});

  $VotingVoteSubmissionCopyWith<$Res> get submission;
}

/// @nodoc
class _$VotingVotePayloadsCopyWithImpl<$Res>
    implements $VotingVotePayloadsCopyWith<$Res> {
  _$VotingVotePayloadsCopyWithImpl(this._self, this._then);

  final VotingVotePayloads _self;
  final $Res Function(VotingVotePayloads) _then;

  /// Create a copy of VotingVotePayloads
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? submission = null,
    Object? sharePayloads = null,
  }) {
    return _then(_self.copyWith(
      submission: null == submission
          ? _self.submission
          : submission // ignore: cast_nullable_to_non_nullable
              as VotingVoteSubmission,
      sharePayloads: null == sharePayloads
          ? _self.sharePayloads
          : sharePayloads // ignore: cast_nullable_to_non_nullable
              as List<VotingSharePayload>,
    ));
  }

  /// Create a copy of VotingVotePayloads
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $VotingVoteSubmissionCopyWith<$Res> get submission {
    return $VotingVoteSubmissionCopyWith<$Res>(_self.submission, (value) {
      return _then(_self.copyWith(submission: value));
    });
  }
}

/// Adds pattern-matching-related methods to [VotingVotePayloads].
extension VotingVotePayloadsPatterns on VotingVotePayloads {
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
    TResult Function(_VotingVotePayloads value)? $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _VotingVotePayloads() when $default != null:
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
    TResult Function(_VotingVotePayloads value) $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingVotePayloads():
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
    TResult? Function(_VotingVotePayloads value)? $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingVotePayloads() when $default != null:
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
    TResult Function(VotingVoteSubmission submission,
            List<VotingSharePayload> sharePayloads)?
        $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _VotingVotePayloads() when $default != null:
        return $default(_that.submission, _that.sharePayloads);
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
    TResult Function(VotingVoteSubmission submission,
            List<VotingSharePayload> sharePayloads)
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingVotePayloads():
        return $default(_that.submission, _that.sharePayloads);
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
    TResult? Function(VotingVoteSubmission submission,
            List<VotingSharePayload> sharePayloads)?
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingVotePayloads() when $default != null:
        return $default(_that.submission, _that.sharePayloads);
      case _:
        return null;
    }
  }
}

/// @nodoc

class _VotingVotePayloads implements VotingVotePayloads {
  const _VotingVotePayloads(
      {required this.submission,
      required final List<VotingSharePayload> sharePayloads})
      : _sharePayloads = sharePayloads;

  @override
  final VotingVoteSubmission submission;
  final List<VotingSharePayload> _sharePayloads;
  @override
  List<VotingSharePayload> get sharePayloads {
    if (_sharePayloads is EqualUnmodifiableListView) return _sharePayloads;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_sharePayloads);
  }

  /// Create a copy of VotingVotePayloads
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  _$VotingVotePayloadsCopyWith<_VotingVotePayloads> get copyWith =>
      __$VotingVotePayloadsCopyWithImpl<_VotingVotePayloads>(this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _VotingVotePayloads &&
            (identical(other.submission, submission) ||
                other.submission == submission) &&
            const DeepCollectionEquality()
                .equals(other._sharePayloads, _sharePayloads));
  }

  @override
  int get hashCode => Object.hash(runtimeType, submission,
      const DeepCollectionEquality().hash(_sharePayloads));

  @override
  String toString() {
    return 'VotingVotePayloads(submission: $submission, sharePayloads: $sharePayloads)';
  }
}

/// @nodoc
abstract mixin class _$VotingVotePayloadsCopyWith<$Res>
    implements $VotingVotePayloadsCopyWith<$Res> {
  factory _$VotingVotePayloadsCopyWith(
          _VotingVotePayloads value, $Res Function(_VotingVotePayloads) _then) =
      __$VotingVotePayloadsCopyWithImpl;
  @override
  @useResult
  $Res call(
      {VotingVoteSubmission submission,
      List<VotingSharePayload> sharePayloads});

  @override
  $VotingVoteSubmissionCopyWith<$Res> get submission;
}

/// @nodoc
class __$VotingVotePayloadsCopyWithImpl<$Res>
    implements _$VotingVotePayloadsCopyWith<$Res> {
  __$VotingVotePayloadsCopyWithImpl(this._self, this._then);

  final _VotingVotePayloads _self;
  final $Res Function(_VotingVotePayloads) _then;

  /// Create a copy of VotingVotePayloads
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $Res call({
    Object? submission = null,
    Object? sharePayloads = null,
  }) {
    return _then(_VotingVotePayloads(
      submission: null == submission
          ? _self.submission
          : submission // ignore: cast_nullable_to_non_nullable
              as VotingVoteSubmission,
      sharePayloads: null == sharePayloads
          ? _self._sharePayloads
          : sharePayloads // ignore: cast_nullable_to_non_nullable
              as List<VotingSharePayload>,
    ));
  }

  /// Create a copy of VotingVotePayloads
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $VotingVoteSubmissionCopyWith<$Res> get submission {
    return $VotingVoteSubmissionCopyWith<$Res>(_self.submission, (value) {
      return _then(_self.copyWith(submission: value));
    });
  }
}

/// @nodoc
mixin _$VotingVoteRecovery {
  int get bundleIndex;
  int get proposalId;
  int get choice;
  String get phase;
  String get workflowPhase;
  String? get txHash;
  BigInt? get vcTreePosition;
  bool get hasCommitmentBundle;

  /// Create a copy of VotingVoteRecovery
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $VotingVoteRecoveryCopyWith<VotingVoteRecovery> get copyWith =>
      _$VotingVoteRecoveryCopyWithImpl<VotingVoteRecovery>(
          this as VotingVoteRecovery, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is VotingVoteRecovery &&
            (identical(other.bundleIndex, bundleIndex) ||
                other.bundleIndex == bundleIndex) &&
            (identical(other.proposalId, proposalId) ||
                other.proposalId == proposalId) &&
            (identical(other.choice, choice) || other.choice == choice) &&
            (identical(other.phase, phase) || other.phase == phase) &&
            (identical(other.workflowPhase, workflowPhase) ||
                other.workflowPhase == workflowPhase) &&
            (identical(other.txHash, txHash) || other.txHash == txHash) &&
            (identical(other.vcTreePosition, vcTreePosition) ||
                other.vcTreePosition == vcTreePosition) &&
            (identical(other.hasCommitmentBundle, hasCommitmentBundle) ||
                other.hasCommitmentBundle == hasCommitmentBundle));
  }

  @override
  int get hashCode => Object.hash(runtimeType, bundleIndex, proposalId, choice,
      phase, workflowPhase, txHash, vcTreePosition, hasCommitmentBundle);

  @override
  String toString() {
    return 'VotingVoteRecovery(bundleIndex: $bundleIndex, proposalId: $proposalId, choice: $choice, phase: $phase, workflowPhase: $workflowPhase, txHash: $txHash, vcTreePosition: $vcTreePosition, hasCommitmentBundle: $hasCommitmentBundle)';
  }
}

/// @nodoc
abstract mixin class $VotingVoteRecoveryCopyWith<$Res> {
  factory $VotingVoteRecoveryCopyWith(
          VotingVoteRecovery value, $Res Function(VotingVoteRecovery) _then) =
      _$VotingVoteRecoveryCopyWithImpl;
  @useResult
  $Res call(
      {int bundleIndex,
      int proposalId,
      int choice,
      String phase,
      String workflowPhase,
      String? txHash,
      BigInt? vcTreePosition,
      bool hasCommitmentBundle});
}

/// @nodoc
class _$VotingVoteRecoveryCopyWithImpl<$Res>
    implements $VotingVoteRecoveryCopyWith<$Res> {
  _$VotingVoteRecoveryCopyWithImpl(this._self, this._then);

  final VotingVoteRecovery _self;
  final $Res Function(VotingVoteRecovery) _then;

  /// Create a copy of VotingVoteRecovery
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? bundleIndex = null,
    Object? proposalId = null,
    Object? choice = null,
    Object? phase = null,
    Object? workflowPhase = null,
    Object? txHash = freezed,
    Object? vcTreePosition = freezed,
    Object? hasCommitmentBundle = null,
  }) {
    return _then(_self.copyWith(
      bundleIndex: null == bundleIndex
          ? _self.bundleIndex
          : bundleIndex // ignore: cast_nullable_to_non_nullable
              as int,
      proposalId: null == proposalId
          ? _self.proposalId
          : proposalId // ignore: cast_nullable_to_non_nullable
              as int,
      choice: null == choice
          ? _self.choice
          : choice // ignore: cast_nullable_to_non_nullable
              as int,
      phase: null == phase
          ? _self.phase
          : phase // ignore: cast_nullable_to_non_nullable
              as String,
      workflowPhase: null == workflowPhase
          ? _self.workflowPhase
          : workflowPhase // ignore: cast_nullable_to_non_nullable
              as String,
      txHash: freezed == txHash
          ? _self.txHash
          : txHash // ignore: cast_nullable_to_non_nullable
              as String?,
      vcTreePosition: freezed == vcTreePosition
          ? _self.vcTreePosition
          : vcTreePosition // ignore: cast_nullable_to_non_nullable
              as BigInt?,
      hasCommitmentBundle: null == hasCommitmentBundle
          ? _self.hasCommitmentBundle
          : hasCommitmentBundle // ignore: cast_nullable_to_non_nullable
              as bool,
    ));
  }
}

/// Adds pattern-matching-related methods to [VotingVoteRecovery].
extension VotingVoteRecoveryPatterns on VotingVoteRecovery {
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
    TResult Function(_VotingVoteRecovery value)? $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _VotingVoteRecovery() when $default != null:
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
    TResult Function(_VotingVoteRecovery value) $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingVoteRecovery():
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
    TResult? Function(_VotingVoteRecovery value)? $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingVoteRecovery() when $default != null:
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
    TResult Function(
            int bundleIndex,
            int proposalId,
            int choice,
            String phase,
            String workflowPhase,
            String? txHash,
            BigInt? vcTreePosition,
            bool hasCommitmentBundle)?
        $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _VotingVoteRecovery() when $default != null:
        return $default(
            _that.bundleIndex,
            _that.proposalId,
            _that.choice,
            _that.phase,
            _that.workflowPhase,
            _that.txHash,
            _that.vcTreePosition,
            _that.hasCommitmentBundle);
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
    TResult Function(
            int bundleIndex,
            int proposalId,
            int choice,
            String phase,
            String workflowPhase,
            String? txHash,
            BigInt? vcTreePosition,
            bool hasCommitmentBundle)
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingVoteRecovery():
        return $default(
            _that.bundleIndex,
            _that.proposalId,
            _that.choice,
            _that.phase,
            _that.workflowPhase,
            _that.txHash,
            _that.vcTreePosition,
            _that.hasCommitmentBundle);
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
    TResult? Function(
            int bundleIndex,
            int proposalId,
            int choice,
            String phase,
            String workflowPhase,
            String? txHash,
            BigInt? vcTreePosition,
            bool hasCommitmentBundle)?
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingVoteRecovery() when $default != null:
        return $default(
            _that.bundleIndex,
            _that.proposalId,
            _that.choice,
            _that.phase,
            _that.workflowPhase,
            _that.txHash,
            _that.vcTreePosition,
            _that.hasCommitmentBundle);
      case _:
        return null;
    }
  }
}

/// @nodoc

class _VotingVoteRecovery implements VotingVoteRecovery {
  const _VotingVoteRecovery(
      {required this.bundleIndex,
      required this.proposalId,
      required this.choice,
      required this.phase,
      required this.workflowPhase,
      this.txHash,
      this.vcTreePosition,
      required this.hasCommitmentBundle});

  @override
  final int bundleIndex;
  @override
  final int proposalId;
  @override
  final int choice;
  @override
  final String phase;
  @override
  final String workflowPhase;
  @override
  final String? txHash;
  @override
  final BigInt? vcTreePosition;
  @override
  final bool hasCommitmentBundle;

  /// Create a copy of VotingVoteRecovery
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  _$VotingVoteRecoveryCopyWith<_VotingVoteRecovery> get copyWith =>
      __$VotingVoteRecoveryCopyWithImpl<_VotingVoteRecovery>(this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _VotingVoteRecovery &&
            (identical(other.bundleIndex, bundleIndex) ||
                other.bundleIndex == bundleIndex) &&
            (identical(other.proposalId, proposalId) ||
                other.proposalId == proposalId) &&
            (identical(other.choice, choice) || other.choice == choice) &&
            (identical(other.phase, phase) || other.phase == phase) &&
            (identical(other.workflowPhase, workflowPhase) ||
                other.workflowPhase == workflowPhase) &&
            (identical(other.txHash, txHash) || other.txHash == txHash) &&
            (identical(other.vcTreePosition, vcTreePosition) ||
                other.vcTreePosition == vcTreePosition) &&
            (identical(other.hasCommitmentBundle, hasCommitmentBundle) ||
                other.hasCommitmentBundle == hasCommitmentBundle));
  }

  @override
  int get hashCode => Object.hash(runtimeType, bundleIndex, proposalId, choice,
      phase, workflowPhase, txHash, vcTreePosition, hasCommitmentBundle);

  @override
  String toString() {
    return 'VotingVoteRecovery(bundleIndex: $bundleIndex, proposalId: $proposalId, choice: $choice, phase: $phase, workflowPhase: $workflowPhase, txHash: $txHash, vcTreePosition: $vcTreePosition, hasCommitmentBundle: $hasCommitmentBundle)';
  }
}

/// @nodoc
abstract mixin class _$VotingVoteRecoveryCopyWith<$Res>
    implements $VotingVoteRecoveryCopyWith<$Res> {
  factory _$VotingVoteRecoveryCopyWith(
          _VotingVoteRecovery value, $Res Function(_VotingVoteRecovery) _then) =
      __$VotingVoteRecoveryCopyWithImpl;
  @override
  @useResult
  $Res call(
      {int bundleIndex,
      int proposalId,
      int choice,
      String phase,
      String workflowPhase,
      String? txHash,
      BigInt? vcTreePosition,
      bool hasCommitmentBundle});
}

/// @nodoc
class __$VotingVoteRecoveryCopyWithImpl<$Res>
    implements _$VotingVoteRecoveryCopyWith<$Res> {
  __$VotingVoteRecoveryCopyWithImpl(this._self, this._then);

  final _VotingVoteRecovery _self;
  final $Res Function(_VotingVoteRecovery) _then;

  /// Create a copy of VotingVoteRecovery
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $Res call({
    Object? bundleIndex = null,
    Object? proposalId = null,
    Object? choice = null,
    Object? phase = null,
    Object? workflowPhase = null,
    Object? txHash = freezed,
    Object? vcTreePosition = freezed,
    Object? hasCommitmentBundle = null,
  }) {
    return _then(_VotingVoteRecovery(
      bundleIndex: null == bundleIndex
          ? _self.bundleIndex
          : bundleIndex // ignore: cast_nullable_to_non_nullable
              as int,
      proposalId: null == proposalId
          ? _self.proposalId
          : proposalId // ignore: cast_nullable_to_non_nullable
              as int,
      choice: null == choice
          ? _self.choice
          : choice // ignore: cast_nullable_to_non_nullable
              as int,
      phase: null == phase
          ? _self.phase
          : phase // ignore: cast_nullable_to_non_nullable
              as String,
      workflowPhase: null == workflowPhase
          ? _self.workflowPhase
          : workflowPhase // ignore: cast_nullable_to_non_nullable
              as String,
      txHash: freezed == txHash
          ? _self.txHash
          : txHash // ignore: cast_nullable_to_non_nullable
              as String?,
      vcTreePosition: freezed == vcTreePosition
          ? _self.vcTreePosition
          : vcTreePosition // ignore: cast_nullable_to_non_nullable
              as BigInt?,
      hasCommitmentBundle: null == hasCommitmentBundle
          ? _self.hasCommitmentBundle
          : hasCommitmentBundle // ignore: cast_nullable_to_non_nullable
              as bool,
    ));
  }
}

/// @nodoc
mixin _$VotingVoteSubmission {
  String get voteRoundId;
  int get proposalId;
  Uint8List get vanNullifier;
  Uint8List get voteAuthorityNoteNew;
  Uint8List get voteCommitment;
  Uint8List get proof;
  Uint8List get rVpk;
  Uint8List get voteAuthSig;
  int get anchorHeight;

  /// Create a copy of VotingVoteSubmission
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $VotingVoteSubmissionCopyWith<VotingVoteSubmission> get copyWith =>
      _$VotingVoteSubmissionCopyWithImpl<VotingVoteSubmission>(
          this as VotingVoteSubmission, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is VotingVoteSubmission &&
            (identical(other.voteRoundId, voteRoundId) ||
                other.voteRoundId == voteRoundId) &&
            (identical(other.proposalId, proposalId) ||
                other.proposalId == proposalId) &&
            const DeepCollectionEquality()
                .equals(other.vanNullifier, vanNullifier) &&
            const DeepCollectionEquality()
                .equals(other.voteAuthorityNoteNew, voteAuthorityNoteNew) &&
            const DeepCollectionEquality()
                .equals(other.voteCommitment, voteCommitment) &&
            const DeepCollectionEquality().equals(other.proof, proof) &&
            const DeepCollectionEquality().equals(other.rVpk, rVpk) &&
            const DeepCollectionEquality()
                .equals(other.voteAuthSig, voteAuthSig) &&
            (identical(other.anchorHeight, anchorHeight) ||
                other.anchorHeight == anchorHeight));
  }

  @override
  int get hashCode => Object.hash(
      runtimeType,
      voteRoundId,
      proposalId,
      const DeepCollectionEquality().hash(vanNullifier),
      const DeepCollectionEquality().hash(voteAuthorityNoteNew),
      const DeepCollectionEquality().hash(voteCommitment),
      const DeepCollectionEquality().hash(proof),
      const DeepCollectionEquality().hash(rVpk),
      const DeepCollectionEquality().hash(voteAuthSig),
      anchorHeight);

  @override
  String toString() {
    return 'VotingVoteSubmission(voteRoundId: $voteRoundId, proposalId: $proposalId, vanNullifier: $vanNullifier, voteAuthorityNoteNew: $voteAuthorityNoteNew, voteCommitment: $voteCommitment, proof: $proof, rVpk: $rVpk, voteAuthSig: $voteAuthSig, anchorHeight: $anchorHeight)';
  }
}

/// @nodoc
abstract mixin class $VotingVoteSubmissionCopyWith<$Res> {
  factory $VotingVoteSubmissionCopyWith(VotingVoteSubmission value,
          $Res Function(VotingVoteSubmission) _then) =
      _$VotingVoteSubmissionCopyWithImpl;
  @useResult
  $Res call(
      {String voteRoundId,
      int proposalId,
      Uint8List vanNullifier,
      Uint8List voteAuthorityNoteNew,
      Uint8List voteCommitment,
      Uint8List proof,
      Uint8List rVpk,
      Uint8List voteAuthSig,
      int anchorHeight});
}

/// @nodoc
class _$VotingVoteSubmissionCopyWithImpl<$Res>
    implements $VotingVoteSubmissionCopyWith<$Res> {
  _$VotingVoteSubmissionCopyWithImpl(this._self, this._then);

  final VotingVoteSubmission _self;
  final $Res Function(VotingVoteSubmission) _then;

  /// Create a copy of VotingVoteSubmission
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? voteRoundId = null,
    Object? proposalId = null,
    Object? vanNullifier = null,
    Object? voteAuthorityNoteNew = null,
    Object? voteCommitment = null,
    Object? proof = null,
    Object? rVpk = null,
    Object? voteAuthSig = null,
    Object? anchorHeight = null,
  }) {
    return _then(_self.copyWith(
      voteRoundId: null == voteRoundId
          ? _self.voteRoundId
          : voteRoundId // ignore: cast_nullable_to_non_nullable
              as String,
      proposalId: null == proposalId
          ? _self.proposalId
          : proposalId // ignore: cast_nullable_to_non_nullable
              as int,
      vanNullifier: null == vanNullifier
          ? _self.vanNullifier
          : vanNullifier // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      voteAuthorityNoteNew: null == voteAuthorityNoteNew
          ? _self.voteAuthorityNoteNew
          : voteAuthorityNoteNew // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      voteCommitment: null == voteCommitment
          ? _self.voteCommitment
          : voteCommitment // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      proof: null == proof
          ? _self.proof
          : proof // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      rVpk: null == rVpk
          ? _self.rVpk
          : rVpk // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      voteAuthSig: null == voteAuthSig
          ? _self.voteAuthSig
          : voteAuthSig // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      anchorHeight: null == anchorHeight
          ? _self.anchorHeight
          : anchorHeight // ignore: cast_nullable_to_non_nullable
              as int,
    ));
  }
}

/// Adds pattern-matching-related methods to [VotingVoteSubmission].
extension VotingVoteSubmissionPatterns on VotingVoteSubmission {
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
    TResult Function(_VotingVoteSubmission value)? $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _VotingVoteSubmission() when $default != null:
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
    TResult Function(_VotingVoteSubmission value) $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingVoteSubmission():
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
    TResult? Function(_VotingVoteSubmission value)? $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingVoteSubmission() when $default != null:
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
    TResult Function(
            String voteRoundId,
            int proposalId,
            Uint8List vanNullifier,
            Uint8List voteAuthorityNoteNew,
            Uint8List voteCommitment,
            Uint8List proof,
            Uint8List rVpk,
            Uint8List voteAuthSig,
            int anchorHeight)?
        $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _VotingVoteSubmission() when $default != null:
        return $default(
            _that.voteRoundId,
            _that.proposalId,
            _that.vanNullifier,
            _that.voteAuthorityNoteNew,
            _that.voteCommitment,
            _that.proof,
            _that.rVpk,
            _that.voteAuthSig,
            _that.anchorHeight);
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
    TResult Function(
            String voteRoundId,
            int proposalId,
            Uint8List vanNullifier,
            Uint8List voteAuthorityNoteNew,
            Uint8List voteCommitment,
            Uint8List proof,
            Uint8List rVpk,
            Uint8List voteAuthSig,
            int anchorHeight)
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingVoteSubmission():
        return $default(
            _that.voteRoundId,
            _that.proposalId,
            _that.vanNullifier,
            _that.voteAuthorityNoteNew,
            _that.voteCommitment,
            _that.proof,
            _that.rVpk,
            _that.voteAuthSig,
            _that.anchorHeight);
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
    TResult? Function(
            String voteRoundId,
            int proposalId,
            Uint8List vanNullifier,
            Uint8List voteAuthorityNoteNew,
            Uint8List voteCommitment,
            Uint8List proof,
            Uint8List rVpk,
            Uint8List voteAuthSig,
            int anchorHeight)?
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingVoteSubmission() when $default != null:
        return $default(
            _that.voteRoundId,
            _that.proposalId,
            _that.vanNullifier,
            _that.voteAuthorityNoteNew,
            _that.voteCommitment,
            _that.proof,
            _that.rVpk,
            _that.voteAuthSig,
            _that.anchorHeight);
      case _:
        return null;
    }
  }
}

/// @nodoc

class _VotingVoteSubmission implements VotingVoteSubmission {
  const _VotingVoteSubmission(
      {required this.voteRoundId,
      required this.proposalId,
      required this.vanNullifier,
      required this.voteAuthorityNoteNew,
      required this.voteCommitment,
      required this.proof,
      required this.rVpk,
      required this.voteAuthSig,
      required this.anchorHeight});

  @override
  final String voteRoundId;
  @override
  final int proposalId;
  @override
  final Uint8List vanNullifier;
  @override
  final Uint8List voteAuthorityNoteNew;
  @override
  final Uint8List voteCommitment;
  @override
  final Uint8List proof;
  @override
  final Uint8List rVpk;
  @override
  final Uint8List voteAuthSig;
  @override
  final int anchorHeight;

  /// Create a copy of VotingVoteSubmission
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  _$VotingVoteSubmissionCopyWith<_VotingVoteSubmission> get copyWith =>
      __$VotingVoteSubmissionCopyWithImpl<_VotingVoteSubmission>(
          this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _VotingVoteSubmission &&
            (identical(other.voteRoundId, voteRoundId) ||
                other.voteRoundId == voteRoundId) &&
            (identical(other.proposalId, proposalId) ||
                other.proposalId == proposalId) &&
            const DeepCollectionEquality()
                .equals(other.vanNullifier, vanNullifier) &&
            const DeepCollectionEquality()
                .equals(other.voteAuthorityNoteNew, voteAuthorityNoteNew) &&
            const DeepCollectionEquality()
                .equals(other.voteCommitment, voteCommitment) &&
            const DeepCollectionEquality().equals(other.proof, proof) &&
            const DeepCollectionEquality().equals(other.rVpk, rVpk) &&
            const DeepCollectionEquality()
                .equals(other.voteAuthSig, voteAuthSig) &&
            (identical(other.anchorHeight, anchorHeight) ||
                other.anchorHeight == anchorHeight));
  }

  @override
  int get hashCode => Object.hash(
      runtimeType,
      voteRoundId,
      proposalId,
      const DeepCollectionEquality().hash(vanNullifier),
      const DeepCollectionEquality().hash(voteAuthorityNoteNew),
      const DeepCollectionEquality().hash(voteCommitment),
      const DeepCollectionEquality().hash(proof),
      const DeepCollectionEquality().hash(rVpk),
      const DeepCollectionEquality().hash(voteAuthSig),
      anchorHeight);

  @override
  String toString() {
    return 'VotingVoteSubmission(voteRoundId: $voteRoundId, proposalId: $proposalId, vanNullifier: $vanNullifier, voteAuthorityNoteNew: $voteAuthorityNoteNew, voteCommitment: $voteCommitment, proof: $proof, rVpk: $rVpk, voteAuthSig: $voteAuthSig, anchorHeight: $anchorHeight)';
  }
}

/// @nodoc
abstract mixin class _$VotingVoteSubmissionCopyWith<$Res>
    implements $VotingVoteSubmissionCopyWith<$Res> {
  factory _$VotingVoteSubmissionCopyWith(_VotingVoteSubmission value,
          $Res Function(_VotingVoteSubmission) _then) =
      __$VotingVoteSubmissionCopyWithImpl;
  @override
  @useResult
  $Res call(
      {String voteRoundId,
      int proposalId,
      Uint8List vanNullifier,
      Uint8List voteAuthorityNoteNew,
      Uint8List voteCommitment,
      Uint8List proof,
      Uint8List rVpk,
      Uint8List voteAuthSig,
      int anchorHeight});
}

/// @nodoc
class __$VotingVoteSubmissionCopyWithImpl<$Res>
    implements _$VotingVoteSubmissionCopyWith<$Res> {
  __$VotingVoteSubmissionCopyWithImpl(this._self, this._then);

  final _VotingVoteSubmission _self;
  final $Res Function(_VotingVoteSubmission) _then;

  /// Create a copy of VotingVoteSubmission
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $Res call({
    Object? voteRoundId = null,
    Object? proposalId = null,
    Object? vanNullifier = null,
    Object? voteAuthorityNoteNew = null,
    Object? voteCommitment = null,
    Object? proof = null,
    Object? rVpk = null,
    Object? voteAuthSig = null,
    Object? anchorHeight = null,
  }) {
    return _then(_VotingVoteSubmission(
      voteRoundId: null == voteRoundId
          ? _self.voteRoundId
          : voteRoundId // ignore: cast_nullable_to_non_nullable
              as String,
      proposalId: null == proposalId
          ? _self.proposalId
          : proposalId // ignore: cast_nullable_to_non_nullable
              as int,
      vanNullifier: null == vanNullifier
          ? _self.vanNullifier
          : vanNullifier // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      voteAuthorityNoteNew: null == voteAuthorityNoteNew
          ? _self.voteAuthorityNoteNew
          : voteAuthorityNoteNew // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      voteCommitment: null == voteCommitment
          ? _self.voteCommitment
          : voteCommitment // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      proof: null == proof
          ? _self.proof
          : proof // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      rVpk: null == rVpk
          ? _self.rVpk
          : rVpk // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      voteAuthSig: null == voteAuthSig
          ? _self.voteAuthSig
          : voteAuthSig // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      anchorHeight: null == anchorHeight
          ? _self.anchorHeight
          : anchorHeight // ignore: cast_nullable_to_non_nullable
              as int,
    ));
  }
}

// dart format on
