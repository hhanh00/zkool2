// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'voting_drive.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;

/// @nodoc
mixin _$VotingDriveStatus {
  String get roundId;
  bool get running;
  BigInt get dispatches;
  int get completedProposals;
  int get totalProposals;
  int get remainingObligations;
  int get sharesConfirmed;
  int get sharesTotal;
  String? get quiescence;
  List<String> get failures;

  /// Create a copy of VotingDriveStatus
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $VotingDriveStatusCopyWith<VotingDriveStatus> get copyWith =>
      _$VotingDriveStatusCopyWithImpl<VotingDriveStatus>(
          this as VotingDriveStatus, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is VotingDriveStatus &&
            (identical(other.roundId, roundId) || other.roundId == roundId) &&
            (identical(other.running, running) || other.running == running) &&
            (identical(other.dispatches, dispatches) ||
                other.dispatches == dispatches) &&
            (identical(other.completedProposals, completedProposals) ||
                other.completedProposals == completedProposals) &&
            (identical(other.totalProposals, totalProposals) ||
                other.totalProposals == totalProposals) &&
            (identical(other.remainingObligations, remainingObligations) ||
                other.remainingObligations == remainingObligations) &&
            (identical(other.sharesConfirmed, sharesConfirmed) ||
                other.sharesConfirmed == sharesConfirmed) &&
            (identical(other.sharesTotal, sharesTotal) ||
                other.sharesTotal == sharesTotal) &&
            (identical(other.quiescence, quiescence) ||
                other.quiescence == quiescence) &&
            const DeepCollectionEquality().equals(other.failures, failures));
  }

  @override
  int get hashCode => Object.hash(
      runtimeType,
      roundId,
      running,
      dispatches,
      completedProposals,
      totalProposals,
      remainingObligations,
      sharesConfirmed,
      sharesTotal,
      quiescence,
      const DeepCollectionEquality().hash(failures));

  @override
  String toString() {
    return 'VotingDriveStatus(roundId: $roundId, running: $running, dispatches: $dispatches, completedProposals: $completedProposals, totalProposals: $totalProposals, remainingObligations: $remainingObligations, sharesConfirmed: $sharesConfirmed, sharesTotal: $sharesTotal, quiescence: $quiescence, failures: $failures)';
  }
}

/// @nodoc
abstract mixin class $VotingDriveStatusCopyWith<$Res> {
  factory $VotingDriveStatusCopyWith(
          VotingDriveStatus value, $Res Function(VotingDriveStatus) _then) =
      _$VotingDriveStatusCopyWithImpl;
  @useResult
  $Res call(
      {String roundId,
      bool running,
      BigInt dispatches,
      int completedProposals,
      int totalProposals,
      int remainingObligations,
      int sharesConfirmed,
      int sharesTotal,
      String? quiescence,
      List<String> failures});
}

/// @nodoc
class _$VotingDriveStatusCopyWithImpl<$Res>
    implements $VotingDriveStatusCopyWith<$Res> {
  _$VotingDriveStatusCopyWithImpl(this._self, this._then);

  final VotingDriveStatus _self;
  final $Res Function(VotingDriveStatus) _then;

  /// Create a copy of VotingDriveStatus
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? roundId = null,
    Object? running = null,
    Object? dispatches = null,
    Object? completedProposals = null,
    Object? totalProposals = null,
    Object? remainingObligations = null,
    Object? sharesConfirmed = null,
    Object? sharesTotal = null,
    Object? quiescence = freezed,
    Object? failures = null,
  }) {
    return _then(_self.copyWith(
      roundId: null == roundId
          ? _self.roundId
          : roundId // ignore: cast_nullable_to_non_nullable
              as String,
      running: null == running
          ? _self.running
          : running // ignore: cast_nullable_to_non_nullable
              as bool,
      dispatches: null == dispatches
          ? _self.dispatches
          : dispatches // ignore: cast_nullable_to_non_nullable
              as BigInt,
      completedProposals: null == completedProposals
          ? _self.completedProposals
          : completedProposals // ignore: cast_nullable_to_non_nullable
              as int,
      totalProposals: null == totalProposals
          ? _self.totalProposals
          : totalProposals // ignore: cast_nullable_to_non_nullable
              as int,
      remainingObligations: null == remainingObligations
          ? _self.remainingObligations
          : remainingObligations // ignore: cast_nullable_to_non_nullable
              as int,
      sharesConfirmed: null == sharesConfirmed
          ? _self.sharesConfirmed
          : sharesConfirmed // ignore: cast_nullable_to_non_nullable
              as int,
      sharesTotal: null == sharesTotal
          ? _self.sharesTotal
          : sharesTotal // ignore: cast_nullable_to_non_nullable
              as int,
      quiescence: freezed == quiescence
          ? _self.quiescence
          : quiescence // ignore: cast_nullable_to_non_nullable
              as String?,
      failures: null == failures
          ? _self.failures
          : failures // ignore: cast_nullable_to_non_nullable
              as List<String>,
    ));
  }
}

/// Adds pattern-matching-related methods to [VotingDriveStatus].
extension VotingDriveStatusPatterns on VotingDriveStatus {
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
    TResult Function(_VotingDriveStatus value)? $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _VotingDriveStatus() when $default != null:
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
    TResult Function(_VotingDriveStatus value) $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingDriveStatus():
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
    TResult? Function(_VotingDriveStatus value)? $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingDriveStatus() when $default != null:
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
            bool running,
            BigInt dispatches,
            int completedProposals,
            int totalProposals,
            int remainingObligations,
            int sharesConfirmed,
            int sharesTotal,
            String? quiescence,
            List<String> failures)?
        $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _VotingDriveStatus() when $default != null:
        return $default(
            _that.roundId,
            _that.running,
            _that.dispatches,
            _that.completedProposals,
            _that.totalProposals,
            _that.remainingObligations,
            _that.sharesConfirmed,
            _that.sharesTotal,
            _that.quiescence,
            _that.failures);
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
            bool running,
            BigInt dispatches,
            int completedProposals,
            int totalProposals,
            int remainingObligations,
            int sharesConfirmed,
            int sharesTotal,
            String? quiescence,
            List<String> failures)
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingDriveStatus():
        return $default(
            _that.roundId,
            _that.running,
            _that.dispatches,
            _that.completedProposals,
            _that.totalProposals,
            _that.remainingObligations,
            _that.sharesConfirmed,
            _that.sharesTotal,
            _that.quiescence,
            _that.failures);
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
            bool running,
            BigInt dispatches,
            int completedProposals,
            int totalProposals,
            int remainingObligations,
            int sharesConfirmed,
            int sharesTotal,
            String? quiescence,
            List<String> failures)?
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingDriveStatus() when $default != null:
        return $default(
            _that.roundId,
            _that.running,
            _that.dispatches,
            _that.completedProposals,
            _that.totalProposals,
            _that.remainingObligations,
            _that.sharesConfirmed,
            _that.sharesTotal,
            _that.quiescence,
            _that.failures);
      case _:
        return null;
    }
  }
}

/// @nodoc

class _VotingDriveStatus implements VotingDriveStatus {
  const _VotingDriveStatus(
      {required this.roundId,
      required this.running,
      required this.dispatches,
      required this.completedProposals,
      required this.totalProposals,
      required this.remainingObligations,
      required this.sharesConfirmed,
      required this.sharesTotal,
      this.quiescence,
      required final List<String> failures})
      : _failures = failures;

  @override
  final String roundId;
  @override
  final bool running;
  @override
  final BigInt dispatches;
  @override
  final int completedProposals;
  @override
  final int totalProposals;
  @override
  final int remainingObligations;
  @override
  final int sharesConfirmed;
  @override
  final int sharesTotal;
  @override
  final String? quiescence;
  final List<String> _failures;
  @override
  List<String> get failures {
    if (_failures is EqualUnmodifiableListView) return _failures;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_failures);
  }

  /// Create a copy of VotingDriveStatus
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  _$VotingDriveStatusCopyWith<_VotingDriveStatus> get copyWith =>
      __$VotingDriveStatusCopyWithImpl<_VotingDriveStatus>(this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _VotingDriveStatus &&
            (identical(other.roundId, roundId) || other.roundId == roundId) &&
            (identical(other.running, running) || other.running == running) &&
            (identical(other.dispatches, dispatches) ||
                other.dispatches == dispatches) &&
            (identical(other.completedProposals, completedProposals) ||
                other.completedProposals == completedProposals) &&
            (identical(other.totalProposals, totalProposals) ||
                other.totalProposals == totalProposals) &&
            (identical(other.remainingObligations, remainingObligations) ||
                other.remainingObligations == remainingObligations) &&
            (identical(other.sharesConfirmed, sharesConfirmed) ||
                other.sharesConfirmed == sharesConfirmed) &&
            (identical(other.sharesTotal, sharesTotal) ||
                other.sharesTotal == sharesTotal) &&
            (identical(other.quiescence, quiescence) ||
                other.quiescence == quiescence) &&
            const DeepCollectionEquality().equals(other._failures, _failures));
  }

  @override
  int get hashCode => Object.hash(
      runtimeType,
      roundId,
      running,
      dispatches,
      completedProposals,
      totalProposals,
      remainingObligations,
      sharesConfirmed,
      sharesTotal,
      quiescence,
      const DeepCollectionEquality().hash(_failures));

  @override
  String toString() {
    return 'VotingDriveStatus(roundId: $roundId, running: $running, dispatches: $dispatches, completedProposals: $completedProposals, totalProposals: $totalProposals, remainingObligations: $remainingObligations, sharesConfirmed: $sharesConfirmed, sharesTotal: $sharesTotal, quiescence: $quiescence, failures: $failures)';
  }
}

/// @nodoc
abstract mixin class _$VotingDriveStatusCopyWith<$Res>
    implements $VotingDriveStatusCopyWith<$Res> {
  factory _$VotingDriveStatusCopyWith(
          _VotingDriveStatus value, $Res Function(_VotingDriveStatus) _then) =
      __$VotingDriveStatusCopyWithImpl;
  @override
  @useResult
  $Res call(
      {String roundId,
      bool running,
      BigInt dispatches,
      int completedProposals,
      int totalProposals,
      int remainingObligations,
      int sharesConfirmed,
      int sharesTotal,
      String? quiescence,
      List<String> failures});
}

/// @nodoc
class __$VotingDriveStatusCopyWithImpl<$Res>
    implements _$VotingDriveStatusCopyWith<$Res> {
  __$VotingDriveStatusCopyWithImpl(this._self, this._then);

  final _VotingDriveStatus _self;
  final $Res Function(_VotingDriveStatus) _then;

  /// Create a copy of VotingDriveStatus
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $Res call({
    Object? roundId = null,
    Object? running = null,
    Object? dispatches = null,
    Object? completedProposals = null,
    Object? totalProposals = null,
    Object? remainingObligations = null,
    Object? sharesConfirmed = null,
    Object? sharesTotal = null,
    Object? quiescence = freezed,
    Object? failures = null,
  }) {
    return _then(_VotingDriveStatus(
      roundId: null == roundId
          ? _self.roundId
          : roundId // ignore: cast_nullable_to_non_nullable
              as String,
      running: null == running
          ? _self.running
          : running // ignore: cast_nullable_to_non_nullable
              as bool,
      dispatches: null == dispatches
          ? _self.dispatches
          : dispatches // ignore: cast_nullable_to_non_nullable
              as BigInt,
      completedProposals: null == completedProposals
          ? _self.completedProposals
          : completedProposals // ignore: cast_nullable_to_non_nullable
              as int,
      totalProposals: null == totalProposals
          ? _self.totalProposals
          : totalProposals // ignore: cast_nullable_to_non_nullable
              as int,
      remainingObligations: null == remainingObligations
          ? _self.remainingObligations
          : remainingObligations // ignore: cast_nullable_to_non_nullable
              as int,
      sharesConfirmed: null == sharesConfirmed
          ? _self.sharesConfirmed
          : sharesConfirmed // ignore: cast_nullable_to_non_nullable
              as int,
      sharesTotal: null == sharesTotal
          ? _self.sharesTotal
          : sharesTotal // ignore: cast_nullable_to_non_nullable
              as int,
      quiescence: freezed == quiescence
          ? _self.quiescence
          : quiescence // ignore: cast_nullable_to_non_nullable
              as String?,
      failures: null == failures
          ? _self._failures
          : failures // ignore: cast_nullable_to_non_nullable
              as List<String>,
    ));
  }
}

/// @nodoc
mixin _$VotingEligibilityPreview {
  int get noteCount;
  BigInt get eligibleWeightZatoshi;
  bool get isEligible;
  BigInt get privacyTrimDroppedValueZatoshi;
  int get skippedSuffixBundles;
  int get skippedSuffixNotes;
  BigInt get skippedSuffixValueZatoshi;

  /// Create a copy of VotingEligibilityPreview
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $VotingEligibilityPreviewCopyWith<VotingEligibilityPreview> get copyWith =>
      _$VotingEligibilityPreviewCopyWithImpl<VotingEligibilityPreview>(
          this as VotingEligibilityPreview, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is VotingEligibilityPreview &&
            (identical(other.noteCount, noteCount) ||
                other.noteCount == noteCount) &&
            (identical(other.eligibleWeightZatoshi, eligibleWeightZatoshi) ||
                other.eligibleWeightZatoshi == eligibleWeightZatoshi) &&
            (identical(other.isEligible, isEligible) ||
                other.isEligible == isEligible) &&
            (identical(other.privacyTrimDroppedValueZatoshi,
                    privacyTrimDroppedValueZatoshi) ||
                other.privacyTrimDroppedValueZatoshi ==
                    privacyTrimDroppedValueZatoshi) &&
            (identical(other.skippedSuffixBundles, skippedSuffixBundles) ||
                other.skippedSuffixBundles == skippedSuffixBundles) &&
            (identical(other.skippedSuffixNotes, skippedSuffixNotes) ||
                other.skippedSuffixNotes == skippedSuffixNotes) &&
            (identical(other.skippedSuffixValueZatoshi,
                    skippedSuffixValueZatoshi) ||
                other.skippedSuffixValueZatoshi == skippedSuffixValueZatoshi));
  }

  @override
  int get hashCode => Object.hash(
      runtimeType,
      noteCount,
      eligibleWeightZatoshi,
      isEligible,
      privacyTrimDroppedValueZatoshi,
      skippedSuffixBundles,
      skippedSuffixNotes,
      skippedSuffixValueZatoshi);

  @override
  String toString() {
    return 'VotingEligibilityPreview(noteCount: $noteCount, eligibleWeightZatoshi: $eligibleWeightZatoshi, isEligible: $isEligible, privacyTrimDroppedValueZatoshi: $privacyTrimDroppedValueZatoshi, skippedSuffixBundles: $skippedSuffixBundles, skippedSuffixNotes: $skippedSuffixNotes, skippedSuffixValueZatoshi: $skippedSuffixValueZatoshi)';
  }
}

/// @nodoc
abstract mixin class $VotingEligibilityPreviewCopyWith<$Res> {
  factory $VotingEligibilityPreviewCopyWith(VotingEligibilityPreview value,
          $Res Function(VotingEligibilityPreview) _then) =
      _$VotingEligibilityPreviewCopyWithImpl;
  @useResult
  $Res call(
      {int noteCount,
      BigInt eligibleWeightZatoshi,
      bool isEligible,
      BigInt privacyTrimDroppedValueZatoshi,
      int skippedSuffixBundles,
      int skippedSuffixNotes,
      BigInt skippedSuffixValueZatoshi});
}

/// @nodoc
class _$VotingEligibilityPreviewCopyWithImpl<$Res>
    implements $VotingEligibilityPreviewCopyWith<$Res> {
  _$VotingEligibilityPreviewCopyWithImpl(this._self, this._then);

  final VotingEligibilityPreview _self;
  final $Res Function(VotingEligibilityPreview) _then;

  /// Create a copy of VotingEligibilityPreview
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? noteCount = null,
    Object? eligibleWeightZatoshi = null,
    Object? isEligible = null,
    Object? privacyTrimDroppedValueZatoshi = null,
    Object? skippedSuffixBundles = null,
    Object? skippedSuffixNotes = null,
    Object? skippedSuffixValueZatoshi = null,
  }) {
    return _then(_self.copyWith(
      noteCount: null == noteCount
          ? _self.noteCount
          : noteCount // ignore: cast_nullable_to_non_nullable
              as int,
      eligibleWeightZatoshi: null == eligibleWeightZatoshi
          ? _self.eligibleWeightZatoshi
          : eligibleWeightZatoshi // ignore: cast_nullable_to_non_nullable
              as BigInt,
      isEligible: null == isEligible
          ? _self.isEligible
          : isEligible // ignore: cast_nullable_to_non_nullable
              as bool,
      privacyTrimDroppedValueZatoshi: null == privacyTrimDroppedValueZatoshi
          ? _self.privacyTrimDroppedValueZatoshi
          : privacyTrimDroppedValueZatoshi // ignore: cast_nullable_to_non_nullable
              as BigInt,
      skippedSuffixBundles: null == skippedSuffixBundles
          ? _self.skippedSuffixBundles
          : skippedSuffixBundles // ignore: cast_nullable_to_non_nullable
              as int,
      skippedSuffixNotes: null == skippedSuffixNotes
          ? _self.skippedSuffixNotes
          : skippedSuffixNotes // ignore: cast_nullable_to_non_nullable
              as int,
      skippedSuffixValueZatoshi: null == skippedSuffixValueZatoshi
          ? _self.skippedSuffixValueZatoshi
          : skippedSuffixValueZatoshi // ignore: cast_nullable_to_non_nullable
              as BigInt,
    ));
  }
}

/// Adds pattern-matching-related methods to [VotingEligibilityPreview].
extension VotingEligibilityPreviewPatterns on VotingEligibilityPreview {
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
    TResult Function(_VotingEligibilityPreview value)? $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _VotingEligibilityPreview() when $default != null:
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
    TResult Function(_VotingEligibilityPreview value) $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingEligibilityPreview():
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
    TResult? Function(_VotingEligibilityPreview value)? $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingEligibilityPreview() when $default != null:
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
            int noteCount,
            BigInt eligibleWeightZatoshi,
            bool isEligible,
            BigInt privacyTrimDroppedValueZatoshi,
            int skippedSuffixBundles,
            int skippedSuffixNotes,
            BigInt skippedSuffixValueZatoshi)?
        $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _VotingEligibilityPreview() when $default != null:
        return $default(
            _that.noteCount,
            _that.eligibleWeightZatoshi,
            _that.isEligible,
            _that.privacyTrimDroppedValueZatoshi,
            _that.skippedSuffixBundles,
            _that.skippedSuffixNotes,
            _that.skippedSuffixValueZatoshi);
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
            int noteCount,
            BigInt eligibleWeightZatoshi,
            bool isEligible,
            BigInt privacyTrimDroppedValueZatoshi,
            int skippedSuffixBundles,
            int skippedSuffixNotes,
            BigInt skippedSuffixValueZatoshi)
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingEligibilityPreview():
        return $default(
            _that.noteCount,
            _that.eligibleWeightZatoshi,
            _that.isEligible,
            _that.privacyTrimDroppedValueZatoshi,
            _that.skippedSuffixBundles,
            _that.skippedSuffixNotes,
            _that.skippedSuffixValueZatoshi);
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
            int noteCount,
            BigInt eligibleWeightZatoshi,
            bool isEligible,
            BigInt privacyTrimDroppedValueZatoshi,
            int skippedSuffixBundles,
            int skippedSuffixNotes,
            BigInt skippedSuffixValueZatoshi)?
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingEligibilityPreview() when $default != null:
        return $default(
            _that.noteCount,
            _that.eligibleWeightZatoshi,
            _that.isEligible,
            _that.privacyTrimDroppedValueZatoshi,
            _that.skippedSuffixBundles,
            _that.skippedSuffixNotes,
            _that.skippedSuffixValueZatoshi);
      case _:
        return null;
    }
  }
}

/// @nodoc

class _VotingEligibilityPreview implements VotingEligibilityPreview {
  const _VotingEligibilityPreview(
      {required this.noteCount,
      required this.eligibleWeightZatoshi,
      required this.isEligible,
      required this.privacyTrimDroppedValueZatoshi,
      required this.skippedSuffixBundles,
      required this.skippedSuffixNotes,
      required this.skippedSuffixValueZatoshi});

  @override
  final int noteCount;
  @override
  final BigInt eligibleWeightZatoshi;
  @override
  final bool isEligible;
  @override
  final BigInt privacyTrimDroppedValueZatoshi;
  @override
  final int skippedSuffixBundles;
  @override
  final int skippedSuffixNotes;
  @override
  final BigInt skippedSuffixValueZatoshi;

  /// Create a copy of VotingEligibilityPreview
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  _$VotingEligibilityPreviewCopyWith<_VotingEligibilityPreview> get copyWith =>
      __$VotingEligibilityPreviewCopyWithImpl<_VotingEligibilityPreview>(
          this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _VotingEligibilityPreview &&
            (identical(other.noteCount, noteCount) ||
                other.noteCount == noteCount) &&
            (identical(other.eligibleWeightZatoshi, eligibleWeightZatoshi) ||
                other.eligibleWeightZatoshi == eligibleWeightZatoshi) &&
            (identical(other.isEligible, isEligible) ||
                other.isEligible == isEligible) &&
            (identical(other.privacyTrimDroppedValueZatoshi,
                    privacyTrimDroppedValueZatoshi) ||
                other.privacyTrimDroppedValueZatoshi ==
                    privacyTrimDroppedValueZatoshi) &&
            (identical(other.skippedSuffixBundles, skippedSuffixBundles) ||
                other.skippedSuffixBundles == skippedSuffixBundles) &&
            (identical(other.skippedSuffixNotes, skippedSuffixNotes) ||
                other.skippedSuffixNotes == skippedSuffixNotes) &&
            (identical(other.skippedSuffixValueZatoshi,
                    skippedSuffixValueZatoshi) ||
                other.skippedSuffixValueZatoshi == skippedSuffixValueZatoshi));
  }

  @override
  int get hashCode => Object.hash(
      runtimeType,
      noteCount,
      eligibleWeightZatoshi,
      isEligible,
      privacyTrimDroppedValueZatoshi,
      skippedSuffixBundles,
      skippedSuffixNotes,
      skippedSuffixValueZatoshi);

  @override
  String toString() {
    return 'VotingEligibilityPreview(noteCount: $noteCount, eligibleWeightZatoshi: $eligibleWeightZatoshi, isEligible: $isEligible, privacyTrimDroppedValueZatoshi: $privacyTrimDroppedValueZatoshi, skippedSuffixBundles: $skippedSuffixBundles, skippedSuffixNotes: $skippedSuffixNotes, skippedSuffixValueZatoshi: $skippedSuffixValueZatoshi)';
  }
}

/// @nodoc
abstract mixin class _$VotingEligibilityPreviewCopyWith<$Res>
    implements $VotingEligibilityPreviewCopyWith<$Res> {
  factory _$VotingEligibilityPreviewCopyWith(_VotingEligibilityPreview value,
          $Res Function(_VotingEligibilityPreview) _then) =
      __$VotingEligibilityPreviewCopyWithImpl;
  @override
  @useResult
  $Res call(
      {int noteCount,
      BigInt eligibleWeightZatoshi,
      bool isEligible,
      BigInt privacyTrimDroppedValueZatoshi,
      int skippedSuffixBundles,
      int skippedSuffixNotes,
      BigInt skippedSuffixValueZatoshi});
}

/// @nodoc
class __$VotingEligibilityPreviewCopyWithImpl<$Res>
    implements _$VotingEligibilityPreviewCopyWith<$Res> {
  __$VotingEligibilityPreviewCopyWithImpl(this._self, this._then);

  final _VotingEligibilityPreview _self;
  final $Res Function(_VotingEligibilityPreview) _then;

  /// Create a copy of VotingEligibilityPreview
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $Res call({
    Object? noteCount = null,
    Object? eligibleWeightZatoshi = null,
    Object? isEligible = null,
    Object? privacyTrimDroppedValueZatoshi = null,
    Object? skippedSuffixBundles = null,
    Object? skippedSuffixNotes = null,
    Object? skippedSuffixValueZatoshi = null,
  }) {
    return _then(_VotingEligibilityPreview(
      noteCount: null == noteCount
          ? _self.noteCount
          : noteCount // ignore: cast_nullable_to_non_nullable
              as int,
      eligibleWeightZatoshi: null == eligibleWeightZatoshi
          ? _self.eligibleWeightZatoshi
          : eligibleWeightZatoshi // ignore: cast_nullable_to_non_nullable
              as BigInt,
      isEligible: null == isEligible
          ? _self.isEligible
          : isEligible // ignore: cast_nullable_to_non_nullable
              as bool,
      privacyTrimDroppedValueZatoshi: null == privacyTrimDroppedValueZatoshi
          ? _self.privacyTrimDroppedValueZatoshi
          : privacyTrimDroppedValueZatoshi // ignore: cast_nullable_to_non_nullable
              as BigInt,
      skippedSuffixBundles: null == skippedSuffixBundles
          ? _self.skippedSuffixBundles
          : skippedSuffixBundles // ignore: cast_nullable_to_non_nullable
              as int,
      skippedSuffixNotes: null == skippedSuffixNotes
          ? _self.skippedSuffixNotes
          : skippedSuffixNotes // ignore: cast_nullable_to_non_nullable
              as int,
      skippedSuffixValueZatoshi: null == skippedSuffixValueZatoshi
          ? _self.skippedSuffixValueZatoshi
          : skippedSuffixValueZatoshi // ignore: cast_nullable_to_non_nullable
              as BigInt,
    ));
  }
}

// dart format on
