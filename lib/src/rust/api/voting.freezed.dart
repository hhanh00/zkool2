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
mixin _$VotingRoundListItem {
  String get roundId;
  String get title;
  String get status;
  BigInt? get snapshotHeight;
  int get bundleCount;
  String get action;

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
            (identical(other.action, action) || other.action == action));
  }

  @override
  int get hashCode => Object.hash(
      runtimeType, roundId, title, status, snapshotHeight, bundleCount, action);

  @override
  String toString() {
    return 'VotingRoundListItem(roundId: $roundId, title: $title, status: $status, snapshotHeight: $snapshotHeight, bundleCount: $bundleCount, action: $action)';
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
      String action});
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
    TResult Function(String roundId, String title, String status,
            BigInt? snapshotHeight, int bundleCount, String action)?
        $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _VotingRoundListItem() when $default != null:
        return $default(_that.roundId, _that.title, _that.status,
            _that.snapshotHeight, _that.bundleCount, _that.action);
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
    TResult Function(String roundId, String title, String status,
            BigInt? snapshotHeight, int bundleCount, String action)
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingRoundListItem():
        return $default(_that.roundId, _that.title, _that.status,
            _that.snapshotHeight, _that.bundleCount, _that.action);
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
    TResult? Function(String roundId, String title, String status,
            BigInt? snapshotHeight, int bundleCount, String action)?
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingRoundListItem() when $default != null:
        return $default(_that.roundId, _that.title, _that.status,
            _that.snapshotHeight, _that.bundleCount, _that.action);
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
      required this.action});

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
            (identical(other.action, action) || other.action == action));
  }

  @override
  int get hashCode => Object.hash(
      runtimeType, roundId, title, status, snapshotHeight, bundleCount, action);

  @override
  String toString() {
    return 'VotingRoundListItem(roundId: $roundId, title: $title, status: $status, snapshotHeight: $snapshotHeight, bundleCount: $bundleCount, action: $action)';
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
      String action});
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
    ));
  }
}

// dart format on
