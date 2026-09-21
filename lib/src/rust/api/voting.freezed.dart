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
            const DeepCollectionEquality().equals(other.proposals, proposals));
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
      const DeepCollectionEquality().hash(proposals));

  @override
  String toString() {
    return 'VotingRoundListItem(roundId: $roundId, title: $title, status: $status, snapshotHeight: $snapshotHeight, bundleCount: $bundleCount, action: $action, proposals: $proposals)';
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
      List<VotingProposalListItem> proposals});
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
            List<VotingProposalListItem> proposals)?
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
            _that.proposals);
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
            List<VotingProposalListItem> proposals)
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
            _that.proposals);
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
            List<VotingProposalListItem> proposals)?
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
            _that.proposals);
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
      required final List<VotingProposalListItem> proposals})
      : _proposals = proposals;

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
                .equals(other._proposals, _proposals));
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
      const DeepCollectionEquality().hash(_proposals));

  @override
  String toString() {
    return 'VotingRoundListItem(roundId: $roundId, title: $title, status: $status, snapshotHeight: $snapshotHeight, bundleCount: $bundleCount, action: $action, proposals: $proposals)';
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
      List<VotingProposalListItem> proposals});
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
    ));
  }
}

// dart format on
