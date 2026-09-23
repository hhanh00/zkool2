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
mixin _$Decision {
  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType && other is Decision);
  }

  @override
  int get hashCode => runtimeType.hashCode;

  @override
  String toString() {
    return 'Decision()';
  }
}

/// @nodoc
class $DecisionCopyWith<$Res> {
  $DecisionCopyWith(Decision _, $Res Function(Decision) __);
}

/// Adds pattern-matching-related methods to [Decision].
extension DecisionPatterns on Decision {
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
    TResult Function(Decision_Choice value)? choice,
    TResult Function(Decision_Skipped value)? skipped,
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case Decision_Choice() when choice != null:
        return choice(_that);
      case Decision_Skipped() when skipped != null:
        return skipped(_that);
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
    required TResult Function(Decision_Choice value) choice,
    required TResult Function(Decision_Skipped value) skipped,
  }) {
    final _that = this;
    switch (_that) {
      case Decision_Choice():
        return choice(_that);
      case Decision_Skipped():
        return skipped(_that);
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
    TResult? Function(Decision_Choice value)? choice,
    TResult? Function(Decision_Skipped value)? skipped,
  }) {
    final _that = this;
    switch (_that) {
      case Decision_Choice() when choice != null:
        return choice(_that);
      case Decision_Skipped() when skipped != null:
        return skipped(_that);
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
    TResult Function(int choice)? choice,
    TResult Function()? skipped,
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case Decision_Choice() when choice != null:
        return choice(_that.choice);
      case Decision_Skipped() when skipped != null:
        return skipped();
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
    required TResult Function(int choice) choice,
    required TResult Function() skipped,
  }) {
    final _that = this;
    switch (_that) {
      case Decision_Choice():
        return choice(_that.choice);
      case Decision_Skipped():
        return skipped();
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
    TResult? Function(int choice)? choice,
    TResult? Function()? skipped,
  }) {
    final _that = this;
    switch (_that) {
      case Decision_Choice() when choice != null:
        return choice(_that.choice);
      case Decision_Skipped() when skipped != null:
        return skipped();
      case _:
        return null;
    }
  }
}

/// @nodoc

class Decision_Choice extends Decision {
  const Decision_Choice({required this.choice}) : super._();

  final int choice;

  /// Create a copy of Decision
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $Decision_ChoiceCopyWith<Decision_Choice> get copyWith =>
      _$Decision_ChoiceCopyWithImpl<Decision_Choice>(this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is Decision_Choice &&
            (identical(other.choice, choice) || other.choice == choice));
  }

  @override
  int get hashCode => Object.hash(runtimeType, choice);

  @override
  String toString() {
    return 'Decision.choice(choice: $choice)';
  }
}

/// @nodoc
abstract mixin class $Decision_ChoiceCopyWith<$Res>
    implements $DecisionCopyWith<$Res> {
  factory $Decision_ChoiceCopyWith(
          Decision_Choice value, $Res Function(Decision_Choice) _then) =
      _$Decision_ChoiceCopyWithImpl;
  @useResult
  $Res call({int choice});
}

/// @nodoc
class _$Decision_ChoiceCopyWithImpl<$Res>
    implements $Decision_ChoiceCopyWith<$Res> {
  _$Decision_ChoiceCopyWithImpl(this._self, this._then);

  final Decision_Choice _self;
  final $Res Function(Decision_Choice) _then;

  /// Create a copy of Decision
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  $Res call({
    Object? choice = null,
  }) {
    return _then(Decision_Choice(
      choice: null == choice
          ? _self.choice
          : choice // ignore: cast_nullable_to_non_nullable
              as int,
    ));
  }
}

/// @nodoc

class Decision_Skipped extends Decision {
  const Decision_Skipped() : super._();

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType && other is Decision_Skipped);
  }

  @override
  int get hashCode => runtimeType.hashCode;

  @override
  String toString() {
    return 'Decision.skipped()';
  }
}

/// @nodoc
mixin _$VotingProposalListItem {
  int get proposalId;
  String get title;
  List<String> get options;

  /// Create a copy of VotingProposalListItem
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $VotingProposalListItemCopyWith<VotingProposalListItem> get copyWith =>
      _$VotingProposalListItemCopyWithImpl<VotingProposalListItem>(
          this as VotingProposalListItem, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is VotingProposalListItem &&
            (identical(other.proposalId, proposalId) ||
                other.proposalId == proposalId) &&
            (identical(other.title, title) || other.title == title) &&
            const DeepCollectionEquality().equals(other.options, options));
  }

  @override
  int get hashCode => Object.hash(runtimeType, proposalId, title,
      const DeepCollectionEquality().hash(options));

  @override
  String toString() {
    return 'VotingProposalListItem(proposalId: $proposalId, title: $title, options: $options)';
  }
}

/// @nodoc
abstract mixin class $VotingProposalListItemCopyWith<$Res> {
  factory $VotingProposalListItemCopyWith(VotingProposalListItem value,
          $Res Function(VotingProposalListItem) _then) =
      _$VotingProposalListItemCopyWithImpl;
  @useResult
  $Res call({int proposalId, String title, List<String> options});
}

/// @nodoc
class _$VotingProposalListItemCopyWithImpl<$Res>
    implements $VotingProposalListItemCopyWith<$Res> {
  _$VotingProposalListItemCopyWithImpl(this._self, this._then);

  final VotingProposalListItem _self;
  final $Res Function(VotingProposalListItem) _then;

  /// Create a copy of VotingProposalListItem
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? proposalId = null,
    Object? title = null,
    Object? options = null,
  }) {
    return _then(_self.copyWith(
      proposalId: null == proposalId
          ? _self.proposalId
          : proposalId // ignore: cast_nullable_to_non_nullable
              as int,
      title: null == title
          ? _self.title
          : title // ignore: cast_nullable_to_non_nullable
              as String,
      options: null == options
          ? _self.options
          : options // ignore: cast_nullable_to_non_nullable
              as List<String>,
    ));
  }
}

/// Adds pattern-matching-related methods to [VotingProposalListItem].
extension VotingProposalListItemPatterns on VotingProposalListItem {
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
    TResult Function(_VotingProposalListItem value)? $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _VotingProposalListItem() when $default != null:
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
    TResult Function(_VotingProposalListItem value) $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingProposalListItem():
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
    TResult? Function(_VotingProposalListItem value)? $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingProposalListItem() when $default != null:
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
    TResult Function(int proposalId, String title, List<String> options)?
        $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _VotingProposalListItem() when $default != null:
        return $default(_that.proposalId, _that.title, _that.options);
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
    TResult Function(int proposalId, String title, List<String> options)
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingProposalListItem():
        return $default(_that.proposalId, _that.title, _that.options);
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
    TResult? Function(int proposalId, String title, List<String> options)?
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingProposalListItem() when $default != null:
        return $default(_that.proposalId, _that.title, _that.options);
      case _:
        return null;
    }
  }
}

/// @nodoc

class _VotingProposalListItem implements VotingProposalListItem {
  const _VotingProposalListItem(
      {required this.proposalId,
      required this.title,
      required final List<String> options})
      : _options = options;

  @override
  final int proposalId;
  @override
  final String title;
  final List<String> _options;
  @override
  List<String> get options {
    if (_options is EqualUnmodifiableListView) return _options;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_options);
  }

  /// Create a copy of VotingProposalListItem
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  _$VotingProposalListItemCopyWith<_VotingProposalListItem> get copyWith =>
      __$VotingProposalListItemCopyWithImpl<_VotingProposalListItem>(
          this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _VotingProposalListItem &&
            (identical(other.proposalId, proposalId) ||
                other.proposalId == proposalId) &&
            (identical(other.title, title) || other.title == title) &&
            const DeepCollectionEquality().equals(other._options, _options));
  }

  @override
  int get hashCode => Object.hash(runtimeType, proposalId, title,
      const DeepCollectionEquality().hash(_options));

  @override
  String toString() {
    return 'VotingProposalListItem(proposalId: $proposalId, title: $title, options: $options)';
  }
}

/// @nodoc
abstract mixin class _$VotingProposalListItemCopyWith<$Res>
    implements $VotingProposalListItemCopyWith<$Res> {
  factory _$VotingProposalListItemCopyWith(_VotingProposalListItem value,
          $Res Function(_VotingProposalListItem) _then) =
      __$VotingProposalListItemCopyWithImpl;
  @override
  @useResult
  $Res call({int proposalId, String title, List<String> options});
}

/// @nodoc
class __$VotingProposalListItemCopyWithImpl<$Res>
    implements _$VotingProposalListItemCopyWith<$Res> {
  __$VotingProposalListItemCopyWithImpl(this._self, this._then);

  final _VotingProposalListItem _self;
  final $Res Function(_VotingProposalListItem) _then;

  /// Create a copy of VotingProposalListItem
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $Res call({
    Object? proposalId = null,
    Object? title = null,
    Object? options = null,
  }) {
    return _then(_VotingProposalListItem(
      proposalId: null == proposalId
          ? _self.proposalId
          : proposalId // ignore: cast_nullable_to_non_nullable
              as int,
      title: null == title
          ? _self.title
          : title // ignore: cast_nullable_to_non_nullable
              as String,
      options: null == options
          ? _self._options
          : options // ignore: cast_nullable_to_non_nullable
              as List<String>,
    ));
  }
}

/// @nodoc
mixin _$VotingRoundListItem {
  String get roundId;
  String get title;
  String get status;
  BigInt? get snapshotHeight;
  int get bundleCount;
  String get action;
  List<VotingProposalListItem> get proposals;
  List<String> get helperUrls;
  BigInt? get voteEndTime;

  /// Create a copy of VotingRoundListItem
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $VotingRoundListItemCopyWith<VotingRoundListItem> get copyWith =>
      _$VotingRoundListItemCopyWithImpl<VotingRoundListItem>(
          this as VotingRoundListItem, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is VotingRoundListItem &&
            (identical(other.roundId, roundId) || other.roundId == roundId) &&
            (identical(other.title, title) || other.title == title) &&
            (identical(other.status, status) || other.status == status) &&
            (identical(other.snapshotHeight, snapshotHeight) ||
                other.snapshotHeight == snapshotHeight) &&
            (identical(other.bundleCount, bundleCount) ||
                other.bundleCount == bundleCount) &&
            (identical(other.action, action) || other.action == action) &&
            const DeepCollectionEquality().equals(other.proposals, proposals) &&
            const DeepCollectionEquality()
                .equals(other.helperUrls, helperUrls) &&
            (identical(other.voteEndTime, voteEndTime) ||
                other.voteEndTime == voteEndTime));
  }

  @override
  int get hashCode => Object.hash(
      runtimeType,
      roundId,
      title,
      status,
      snapshotHeight,
      bundleCount,
      action,
      const DeepCollectionEquality().hash(proposals),
      const DeepCollectionEquality().hash(helperUrls),
      voteEndTime);

  @override
  String toString() {
    return 'VotingRoundListItem(roundId: $roundId, title: $title, status: $status, snapshotHeight: $snapshotHeight, bundleCount: $bundleCount, action: $action, proposals: $proposals, helperUrls: $helperUrls, voteEndTime: $voteEndTime)';
  }
}

/// @nodoc
abstract mixin class $VotingRoundListItemCopyWith<$Res> {
  factory $VotingRoundListItemCopyWith(
          VotingRoundListItem value, $Res Function(VotingRoundListItem) _then) =
      _$VotingRoundListItemCopyWithImpl;
  @useResult
  $Res call(
      {String roundId,
      String title,
      String status,
      BigInt? snapshotHeight,
      int bundleCount,
      String action,
      List<VotingProposalListItem> proposals,
      List<String> helperUrls,
      BigInt? voteEndTime});
}

/// @nodoc
class _$VotingRoundListItemCopyWithImpl<$Res>
    implements $VotingRoundListItemCopyWith<$Res> {
  _$VotingRoundListItemCopyWithImpl(this._self, this._then);

  final VotingRoundListItem _self;
  final $Res Function(VotingRoundListItem) _then;

  /// Create a copy of VotingRoundListItem
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? roundId = null,
    Object? title = null,
    Object? status = null,
    Object? snapshotHeight = freezed,
    Object? bundleCount = null,
    Object? action = null,
    Object? proposals = null,
    Object? helperUrls = null,
    Object? voteEndTime = freezed,
  }) {
    return _then(_self.copyWith(
      roundId: null == roundId
          ? _self.roundId
          : roundId // ignore: cast_nullable_to_non_nullable
              as String,
      title: null == title
          ? _self.title
          : title // ignore: cast_nullable_to_non_nullable
              as String,
      status: null == status
          ? _self.status
          : status // ignore: cast_nullable_to_non_nullable
              as String,
      snapshotHeight: freezed == snapshotHeight
          ? _self.snapshotHeight
          : snapshotHeight // ignore: cast_nullable_to_non_nullable
              as BigInt?,
      bundleCount: null == bundleCount
          ? _self.bundleCount
          : bundleCount // ignore: cast_nullable_to_non_nullable
              as int,
      action: null == action
          ? _self.action
          : action // ignore: cast_nullable_to_non_nullable
              as String,
      proposals: null == proposals
          ? _self.proposals
          : proposals // ignore: cast_nullable_to_non_nullable
              as List<VotingProposalListItem>,
      helperUrls: null == helperUrls
          ? _self.helperUrls
          : helperUrls // ignore: cast_nullable_to_non_nullable
              as List<String>,
      voteEndTime: freezed == voteEndTime
          ? _self.voteEndTime
          : voteEndTime // ignore: cast_nullable_to_non_nullable
              as BigInt?,
    ));
  }
}

/// Adds pattern-matching-related methods to [VotingRoundListItem].
extension VotingRoundListItemPatterns on VotingRoundListItem {
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
    TResult Function(_VotingRoundListItem value)? $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _VotingRoundListItem() when $default != null:
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
    TResult Function(_VotingRoundListItem value) $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingRoundListItem():
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
    TResult? Function(_VotingRoundListItem value)? $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingRoundListItem() when $default != null:
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
            String title,
            String status,
            BigInt? snapshotHeight,
            int bundleCount,
            String action,
            List<VotingProposalListItem> proposals,
            List<String> helperUrls,
            BigInt? voteEndTime)?
        $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _VotingRoundListItem() when $default != null:
        return $default(
            _that.roundId,
            _that.title,
            _that.status,
            _that.snapshotHeight,
            _that.bundleCount,
            _that.action,
            _that.proposals,
            _that.helperUrls,
            _that.voteEndTime);
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
            String title,
            String status,
            BigInt? snapshotHeight,
            int bundleCount,
            String action,
            List<VotingProposalListItem> proposals,
            List<String> helperUrls,
            BigInt? voteEndTime)
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingRoundListItem():
        return $default(
            _that.roundId,
            _that.title,
            _that.status,
            _that.snapshotHeight,
            _that.bundleCount,
            _that.action,
            _that.proposals,
            _that.helperUrls,
            _that.voteEndTime);
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
            String title,
            String status,
            BigInt? snapshotHeight,
            int bundleCount,
            String action,
            List<VotingProposalListItem> proposals,
            List<String> helperUrls,
            BigInt? voteEndTime)?
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingRoundListItem() when $default != null:
        return $default(
            _that.roundId,
            _that.title,
            _that.status,
            _that.snapshotHeight,
            _that.bundleCount,
            _that.action,
            _that.proposals,
            _that.helperUrls,
            _that.voteEndTime);
      case _:
        return null;
    }
  }
}

/// @nodoc

class _VotingRoundListItem implements VotingRoundListItem {
  const _VotingRoundListItem(
      {required this.roundId,
      required this.title,
      required this.status,
      this.snapshotHeight,
      required this.bundleCount,
      required this.action,
      required final List<VotingProposalListItem> proposals,
      required final List<String> helperUrls,
      this.voteEndTime})
      : _proposals = proposals,
        _helperUrls = helperUrls;

  @override
  final String roundId;
  @override
  final String title;
  @override
  final String status;
  @override
  final BigInt? snapshotHeight;
  @override
  final int bundleCount;
  @override
  final String action;
  final List<VotingProposalListItem> _proposals;
  @override
  List<VotingProposalListItem> get proposals {
    if (_proposals is EqualUnmodifiableListView) return _proposals;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_proposals);
  }

  final List<String> _helperUrls;
  @override
  List<String> get helperUrls {
    if (_helperUrls is EqualUnmodifiableListView) return _helperUrls;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_helperUrls);
  }

  @override
  final BigInt? voteEndTime;

  /// Create a copy of VotingRoundListItem
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  _$VotingRoundListItemCopyWith<_VotingRoundListItem> get copyWith =>
      __$VotingRoundListItemCopyWithImpl<_VotingRoundListItem>(
          this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _VotingRoundListItem &&
            (identical(other.roundId, roundId) || other.roundId == roundId) &&
            (identical(other.title, title) || other.title == title) &&
            (identical(other.status, status) || other.status == status) &&
            (identical(other.snapshotHeight, snapshotHeight) ||
                other.snapshotHeight == snapshotHeight) &&
            (identical(other.bundleCount, bundleCount) ||
                other.bundleCount == bundleCount) &&
            (identical(other.action, action) || other.action == action) &&
            const DeepCollectionEquality()
                .equals(other._proposals, _proposals) &&
            const DeepCollectionEquality()
                .equals(other._helperUrls, _helperUrls) &&
            (identical(other.voteEndTime, voteEndTime) ||
                other.voteEndTime == voteEndTime));
  }

  @override
  int get hashCode => Object.hash(
      runtimeType,
      roundId,
      title,
      status,
      snapshotHeight,
      bundleCount,
      action,
      const DeepCollectionEquality().hash(_proposals),
      const DeepCollectionEquality().hash(_helperUrls),
      voteEndTime);

  @override
  String toString() {
    return 'VotingRoundListItem(roundId: $roundId, title: $title, status: $status, snapshotHeight: $snapshotHeight, bundleCount: $bundleCount, action: $action, proposals: $proposals, helperUrls: $helperUrls, voteEndTime: $voteEndTime)';
  }
}

/// @nodoc
abstract mixin class _$VotingRoundListItemCopyWith<$Res>
    implements $VotingRoundListItemCopyWith<$Res> {
  factory _$VotingRoundListItemCopyWith(_VotingRoundListItem value,
          $Res Function(_VotingRoundListItem) _then) =
      __$VotingRoundListItemCopyWithImpl;
  @override
  @useResult
  $Res call(
      {String roundId,
      String title,
      String status,
      BigInt? snapshotHeight,
      int bundleCount,
      String action,
      List<VotingProposalListItem> proposals,
      List<String> helperUrls,
      BigInt? voteEndTime});
}

/// @nodoc
class __$VotingRoundListItemCopyWithImpl<$Res>
    implements _$VotingRoundListItemCopyWith<$Res> {
  __$VotingRoundListItemCopyWithImpl(this._self, this._then);

  final _VotingRoundListItem _self;
  final $Res Function(_VotingRoundListItem) _then;

  /// Create a copy of VotingRoundListItem
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $Res call({
    Object? roundId = null,
    Object? title = null,
    Object? status = null,
    Object? snapshotHeight = freezed,
    Object? bundleCount = null,
    Object? action = null,
    Object? proposals = null,
    Object? helperUrls = null,
    Object? voteEndTime = freezed,
  }) {
    return _then(_VotingRoundListItem(
      roundId: null == roundId
          ? _self.roundId
          : roundId // ignore: cast_nullable_to_non_nullable
              as String,
      title: null == title
          ? _self.title
          : title // ignore: cast_nullable_to_non_nullable
              as String,
      status: null == status
          ? _self.status
          : status // ignore: cast_nullable_to_non_nullable
              as String,
      snapshotHeight: freezed == snapshotHeight
          ? _self.snapshotHeight
          : snapshotHeight // ignore: cast_nullable_to_non_nullable
              as BigInt?,
      bundleCount: null == bundleCount
          ? _self.bundleCount
          : bundleCount // ignore: cast_nullable_to_non_nullable
              as int,
      action: null == action
          ? _self.action
          : action // ignore: cast_nullable_to_non_nullable
              as String,
      proposals: null == proposals
          ? _self._proposals
          : proposals // ignore: cast_nullable_to_non_nullable
              as List<VotingProposalListItem>,
      helperUrls: null == helperUrls
          ? _self._helperUrls
          : helperUrls // ignore: cast_nullable_to_non_nullable
              as List<String>,
      voteEndTime: freezed == voteEndTime
          ? _self.voteEndTime
          : voteEndTime // ignore: cast_nullable_to_non_nullable
              as BigInt?,
    ));
  }
}

/// @nodoc
mixin _$VotingSelection {
  int get proposalId;
  Decision get decision;

  /// Create a copy of VotingSelection
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $VotingSelectionCopyWith<VotingSelection> get copyWith =>
      _$VotingSelectionCopyWithImpl<VotingSelection>(
          this as VotingSelection, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is VotingSelection &&
            (identical(other.proposalId, proposalId) ||
                other.proposalId == proposalId) &&
            (identical(other.decision, decision) ||
                other.decision == decision));
  }

  @override
  int get hashCode => Object.hash(runtimeType, proposalId, decision);

  @override
  String toString() {
    return 'VotingSelection(proposalId: $proposalId, decision: $decision)';
  }
}

/// @nodoc
abstract mixin class $VotingSelectionCopyWith<$Res> {
  factory $VotingSelectionCopyWith(
          VotingSelection value, $Res Function(VotingSelection) _then) =
      _$VotingSelectionCopyWithImpl;
  @useResult
  $Res call({int proposalId, Decision decision});

  $DecisionCopyWith<$Res> get decision;
}

/// @nodoc
class _$VotingSelectionCopyWithImpl<$Res>
    implements $VotingSelectionCopyWith<$Res> {
  _$VotingSelectionCopyWithImpl(this._self, this._then);

  final VotingSelection _self;
  final $Res Function(VotingSelection) _then;

  /// Create a copy of VotingSelection
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? proposalId = null,
    Object? decision = null,
  }) {
    return _then(_self.copyWith(
      proposalId: null == proposalId
          ? _self.proposalId
          : proposalId // ignore: cast_nullable_to_non_nullable
              as int,
      decision: null == decision
          ? _self.decision
          : decision // ignore: cast_nullable_to_non_nullable
              as Decision,
    ));
  }

  /// Create a copy of VotingSelection
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $DecisionCopyWith<$Res> get decision {
    return $DecisionCopyWith<$Res>(_self.decision, (value) {
      return _then(_self.copyWith(decision: value));
    });
  }
}

/// Adds pattern-matching-related methods to [VotingSelection].
extension VotingSelectionPatterns on VotingSelection {
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
    TResult Function(_VotingSelection value)? $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _VotingSelection() when $default != null:
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
    TResult Function(_VotingSelection value) $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingSelection():
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
    TResult? Function(_VotingSelection value)? $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingSelection() when $default != null:
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
    TResult Function(int proposalId, Decision decision)? $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _VotingSelection() when $default != null:
        return $default(_that.proposalId, _that.decision);
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
    TResult Function(int proposalId, Decision decision) $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingSelection():
        return $default(_that.proposalId, _that.decision);
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
    TResult? Function(int proposalId, Decision decision)? $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingSelection() when $default != null:
        return $default(_that.proposalId, _that.decision);
      case _:
        return null;
    }
  }
}

/// @nodoc

class _VotingSelection implements VotingSelection {
  const _VotingSelection({required this.proposalId, required this.decision});

  @override
  final int proposalId;
  @override
  final Decision decision;

  /// Create a copy of VotingSelection
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  _$VotingSelectionCopyWith<_VotingSelection> get copyWith =>
      __$VotingSelectionCopyWithImpl<_VotingSelection>(this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _VotingSelection &&
            (identical(other.proposalId, proposalId) ||
                other.proposalId == proposalId) &&
            (identical(other.decision, decision) ||
                other.decision == decision));
  }

  @override
  int get hashCode => Object.hash(runtimeType, proposalId, decision);

  @override
  String toString() {
    return 'VotingSelection(proposalId: $proposalId, decision: $decision)';
  }
}

/// @nodoc
abstract mixin class _$VotingSelectionCopyWith<$Res>
    implements $VotingSelectionCopyWith<$Res> {
  factory _$VotingSelectionCopyWith(
          _VotingSelection value, $Res Function(_VotingSelection) _then) =
      __$VotingSelectionCopyWithImpl;
  @override
  @useResult
  $Res call({int proposalId, Decision decision});

  @override
  $DecisionCopyWith<$Res> get decision;
}

/// @nodoc
class __$VotingSelectionCopyWithImpl<$Res>
    implements _$VotingSelectionCopyWith<$Res> {
  __$VotingSelectionCopyWithImpl(this._self, this._then);

  final _VotingSelection _self;
  final $Res Function(_VotingSelection) _then;

  /// Create a copy of VotingSelection
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $Res call({
    Object? proposalId = null,
    Object? decision = null,
  }) {
    return _then(_VotingSelection(
      proposalId: null == proposalId
          ? _self.proposalId
          : proposalId // ignore: cast_nullable_to_non_nullable
              as int,
      decision: null == decision
          ? _self.decision
          : decision // ignore: cast_nullable_to_non_nullable
              as Decision,
    ));
  }

  /// Create a copy of VotingSelection
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $DecisionCopyWith<$Res> get decision {
    return $DecisionCopyWith<$Res>(_self.decision, (value) {
      return _then(_self.copyWith(decision: value));
    });
  }
}

// dart format on
