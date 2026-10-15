// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'sync.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;

/// @nodoc
mixin _$SyncEvent {
  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType && other is SyncEvent);
  }

  @override
  int get hashCode => runtimeType.hashCode;

  @override
  String toString() {
    return 'SyncEvent()';
  }
}

/// @nodoc
class $SyncEventCopyWith<$Res> {
  $SyncEventCopyWith(SyncEvent _, $Res Function(SyncEvent) __);
}

/// Adds pattern-matching-related methods to [SyncEvent].
extension SyncEventPatterns on SyncEvent {
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
    TResult Function(SyncEvent_Progress value)? progress,
    TResult Function(SyncEvent_Finished value)? finished,
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case SyncEvent_Progress() when progress != null:
        return progress(_that);
      case SyncEvent_Finished() when finished != null:
        return finished(_that);
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
    required TResult Function(SyncEvent_Progress value) progress,
    required TResult Function(SyncEvent_Finished value) finished,
  }) {
    final _that = this;
    switch (_that) {
      case SyncEvent_Progress():
        return progress(_that);
      case SyncEvent_Finished():
        return finished(_that);
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
    TResult? Function(SyncEvent_Progress value)? progress,
    TResult? Function(SyncEvent_Finished value)? finished,
  }) {
    final _that = this;
    switch (_that) {
      case SyncEvent_Progress() when progress != null:
        return progress(_that);
      case SyncEvent_Finished() when finished != null:
        return finished(_that);
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
    TResult Function(int height, int time)? progress,
    TResult Function(SyncOutcome outcome)? finished,
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case SyncEvent_Progress() when progress != null:
        return progress(_that.height, _that.time);
      case SyncEvent_Finished() when finished != null:
        return finished(_that.outcome);
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
    required TResult Function(int height, int time) progress,
    required TResult Function(SyncOutcome outcome) finished,
  }) {
    final _that = this;
    switch (_that) {
      case SyncEvent_Progress():
        return progress(_that.height, _that.time);
      case SyncEvent_Finished():
        return finished(_that.outcome);
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
    TResult? Function(int height, int time)? progress,
    TResult? Function(SyncOutcome outcome)? finished,
  }) {
    final _that = this;
    switch (_that) {
      case SyncEvent_Progress() when progress != null:
        return progress(_that.height, _that.time);
      case SyncEvent_Finished() when finished != null:
        return finished(_that.outcome);
      case _:
        return null;
    }
  }
}

/// @nodoc

class SyncEvent_Progress extends SyncEvent {
  const SyncEvent_Progress({required this.height, required this.time})
      : super._();

  final int height;
  final int time;

  /// Create a copy of SyncEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $SyncEvent_ProgressCopyWith<SyncEvent_Progress> get copyWith =>
      _$SyncEvent_ProgressCopyWithImpl<SyncEvent_Progress>(this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is SyncEvent_Progress &&
            (identical(other.height, height) || other.height == height) &&
            (identical(other.time, time) || other.time == time));
  }

  @override
  int get hashCode => Object.hash(runtimeType, height, time);

  @override
  String toString() {
    return 'SyncEvent.progress(height: $height, time: $time)';
  }
}

/// @nodoc
abstract mixin class $SyncEvent_ProgressCopyWith<$Res>
    implements $SyncEventCopyWith<$Res> {
  factory $SyncEvent_ProgressCopyWith(
          SyncEvent_Progress value, $Res Function(SyncEvent_Progress) _then) =
      _$SyncEvent_ProgressCopyWithImpl;
  @useResult
  $Res call({int height, int time});
}

/// @nodoc
class _$SyncEvent_ProgressCopyWithImpl<$Res>
    implements $SyncEvent_ProgressCopyWith<$Res> {
  _$SyncEvent_ProgressCopyWithImpl(this._self, this._then);

  final SyncEvent_Progress _self;
  final $Res Function(SyncEvent_Progress) _then;

  /// Create a copy of SyncEvent
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  $Res call({
    Object? height = null,
    Object? time = null,
  }) {
    return _then(SyncEvent_Progress(
      height: null == height
          ? _self.height
          : height // ignore: cast_nullable_to_non_nullable
              as int,
      time: null == time
          ? _self.time
          : time // ignore: cast_nullable_to_non_nullable
              as int,
    ));
  }
}

/// @nodoc

class SyncEvent_Finished extends SyncEvent {
  const SyncEvent_Finished({required this.outcome}) : super._();

  final SyncOutcome outcome;

  /// Create a copy of SyncEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $SyncEvent_FinishedCopyWith<SyncEvent_Finished> get copyWith =>
      _$SyncEvent_FinishedCopyWithImpl<SyncEvent_Finished>(this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is SyncEvent_Finished &&
            (identical(other.outcome, outcome) || other.outcome == outcome));
  }

  @override
  int get hashCode => Object.hash(runtimeType, outcome);

  @override
  String toString() {
    return 'SyncEvent.finished(outcome: $outcome)';
  }
}

/// @nodoc
abstract mixin class $SyncEvent_FinishedCopyWith<$Res>
    implements $SyncEventCopyWith<$Res> {
  factory $SyncEvent_FinishedCopyWith(
          SyncEvent_Finished value, $Res Function(SyncEvent_Finished) _then) =
      _$SyncEvent_FinishedCopyWithImpl;
  @useResult
  $Res call({SyncOutcome outcome});

  $SyncOutcomeCopyWith<$Res> get outcome;
}

/// @nodoc
class _$SyncEvent_FinishedCopyWithImpl<$Res>
    implements $SyncEvent_FinishedCopyWith<$Res> {
  _$SyncEvent_FinishedCopyWithImpl(this._self, this._then);

  final SyncEvent_Finished _self;
  final $Res Function(SyncEvent_Finished) _then;

  /// Create a copy of SyncEvent
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  $Res call({
    Object? outcome = null,
  }) {
    return _then(SyncEvent_Finished(
      outcome: null == outcome
          ? _self.outcome
          : outcome // ignore: cast_nullable_to_non_nullable
              as SyncOutcome,
    ));
  }

  /// Create a copy of SyncEvent
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $SyncOutcomeCopyWith<$Res> get outcome {
    return $SyncOutcomeCopyWith<$Res>(_self.outcome, (value) {
      return _then(_self.copyWith(outcome: value));
    });
  }
}

/// @nodoc
mixin _$SyncOutcome {
  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType && other is SyncOutcome);
  }

  @override
  int get hashCode => runtimeType.hashCode;

  @override
  String toString() {
    return 'SyncOutcome()';
  }
}

/// @nodoc
class $SyncOutcomeCopyWith<$Res> {
  $SyncOutcomeCopyWith(SyncOutcome _, $Res Function(SyncOutcome) __);
}

/// Adds pattern-matching-related methods to [SyncOutcome].
extension SyncOutcomePatterns on SyncOutcome {
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
    TResult Function(SyncOutcome_Synced value)? synced,
    TResult Function(SyncOutcome_Skipped value)? skipped,
    TResult Function(SyncOutcome_Cancelled value)? cancelled,
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case SyncOutcome_Synced() when synced != null:
        return synced(_that);
      case SyncOutcome_Skipped() when skipped != null:
        return skipped(_that);
      case SyncOutcome_Cancelled() when cancelled != null:
        return cancelled(_that);
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
    required TResult Function(SyncOutcome_Synced value) synced,
    required TResult Function(SyncOutcome_Skipped value) skipped,
    required TResult Function(SyncOutcome_Cancelled value) cancelled,
  }) {
    final _that = this;
    switch (_that) {
      case SyncOutcome_Synced():
        return synced(_that);
      case SyncOutcome_Skipped():
        return skipped(_that);
      case SyncOutcome_Cancelled():
        return cancelled(_that);
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
    TResult? Function(SyncOutcome_Synced value)? synced,
    TResult? Function(SyncOutcome_Skipped value)? skipped,
    TResult? Function(SyncOutcome_Cancelled value)? cancelled,
  }) {
    final _that = this;
    switch (_that) {
      case SyncOutcome_Synced() when synced != null:
        return synced(_that);
      case SyncOutcome_Skipped() when skipped != null:
        return skipped(_that);
      case SyncOutcome_Cancelled() when cancelled != null:
        return cancelled(_that);
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
    TResult Function(int height)? synced,
    TResult Function()? skipped,
    TResult Function()? cancelled,
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case SyncOutcome_Synced() when synced != null:
        return synced(_that.height);
      case SyncOutcome_Skipped() when skipped != null:
        return skipped();
      case SyncOutcome_Cancelled() when cancelled != null:
        return cancelled();
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
    required TResult Function(int height) synced,
    required TResult Function() skipped,
    required TResult Function() cancelled,
  }) {
    final _that = this;
    switch (_that) {
      case SyncOutcome_Synced():
        return synced(_that.height);
      case SyncOutcome_Skipped():
        return skipped();
      case SyncOutcome_Cancelled():
        return cancelled();
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
    TResult? Function(int height)? synced,
    TResult? Function()? skipped,
    TResult? Function()? cancelled,
  }) {
    final _that = this;
    switch (_that) {
      case SyncOutcome_Synced() when synced != null:
        return synced(_that.height);
      case SyncOutcome_Skipped() when skipped != null:
        return skipped();
      case SyncOutcome_Cancelled() when cancelled != null:
        return cancelled();
      case _:
        return null;
    }
  }
}

/// @nodoc

class SyncOutcome_Synced extends SyncOutcome {
  const SyncOutcome_Synced({required this.height}) : super._();

  final int height;

  /// Create a copy of SyncOutcome
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $SyncOutcome_SyncedCopyWith<SyncOutcome_Synced> get copyWith =>
      _$SyncOutcome_SyncedCopyWithImpl<SyncOutcome_Synced>(this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is SyncOutcome_Synced &&
            (identical(other.height, height) || other.height == height));
  }

  @override
  int get hashCode => Object.hash(runtimeType, height);

  @override
  String toString() {
    return 'SyncOutcome.synced(height: $height)';
  }
}

/// @nodoc
abstract mixin class $SyncOutcome_SyncedCopyWith<$Res>
    implements $SyncOutcomeCopyWith<$Res> {
  factory $SyncOutcome_SyncedCopyWith(
          SyncOutcome_Synced value, $Res Function(SyncOutcome_Synced) _then) =
      _$SyncOutcome_SyncedCopyWithImpl;
  @useResult
  $Res call({int height});
}

/// @nodoc
class _$SyncOutcome_SyncedCopyWithImpl<$Res>
    implements $SyncOutcome_SyncedCopyWith<$Res> {
  _$SyncOutcome_SyncedCopyWithImpl(this._self, this._then);

  final SyncOutcome_Synced _self;
  final $Res Function(SyncOutcome_Synced) _then;

  /// Create a copy of SyncOutcome
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  $Res call({
    Object? height = null,
  }) {
    return _then(SyncOutcome_Synced(
      height: null == height
          ? _self.height
          : height // ignore: cast_nullable_to_non_nullable
              as int,
    ));
  }
}

/// @nodoc

class SyncOutcome_Skipped extends SyncOutcome {
  const SyncOutcome_Skipped() : super._();

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType && other is SyncOutcome_Skipped);
  }

  @override
  int get hashCode => runtimeType.hashCode;

  @override
  String toString() {
    return 'SyncOutcome.skipped()';
  }
}

/// @nodoc

class SyncOutcome_Cancelled extends SyncOutcome {
  const SyncOutcome_Cancelled() : super._();

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType && other is SyncOutcome_Cancelled);
  }

  @override
  int get hashCode => runtimeType.hashCode;

  @override
  String toString() {
    return 'SyncOutcome.cancelled()';
  }
}

// dart format on
