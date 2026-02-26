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
mixin _$ElectionId {
  String? get url;
  Uint8List get hash;

  /// Create a copy of ElectionId
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $ElectionIdCopyWith<ElectionId> get copyWith =>
      _$ElectionIdCopyWithImpl<ElectionId>(this as ElectionId, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is ElectionId &&
            (identical(other.url, url) || other.url == url) &&
            const DeepCollectionEquality().equals(other.hash, hash));
  }

  @override
  int get hashCode =>
      Object.hash(runtimeType, url, const DeepCollectionEquality().hash(hash));

  @override
  String toString() {
    return 'ElectionId(url: $url, hash: $hash)';
  }
}

/// @nodoc
abstract mixin class $ElectionIdCopyWith<$Res> {
  factory $ElectionIdCopyWith(
          ElectionId value, $Res Function(ElectionId) _then) =
      _$ElectionIdCopyWithImpl;
  @useResult
  $Res call({String? url, Uint8List hash});
}

/// @nodoc
class _$ElectionIdCopyWithImpl<$Res> implements $ElectionIdCopyWith<$Res> {
  _$ElectionIdCopyWithImpl(this._self, this._then);

  final ElectionId _self;
  final $Res Function(ElectionId) _then;

  /// Create a copy of ElectionId
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? url = freezed,
    Object? hash = null,
  }) {
    return _then(_self.copyWith(
      url: freezed == url
          ? _self.url
          : url // ignore: cast_nullable_to_non_nullable
              as String?,
      hash: null == hash
          ? _self.hash
          : hash // ignore: cast_nullable_to_non_nullable
              as Uint8List,
    ));
  }
}

/// Adds pattern-matching-related methods to [ElectionId].
extension ElectionIdPatterns on ElectionId {
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
    TResult Function(_ElectionId value)? $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _ElectionId() when $default != null:
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
    TResult Function(_ElectionId value) $default,
  ) {
    final _that = this;
    switch (_that) {
      case _ElectionId():
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
    TResult? Function(_ElectionId value)? $default,
  ) {
    final _that = this;
    switch (_that) {
      case _ElectionId() when $default != null:
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
    TResult Function(String? url, Uint8List hash)? $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _ElectionId() when $default != null:
        return $default(_that.url, _that.hash);
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
    TResult Function(String? url, Uint8List hash) $default,
  ) {
    final _that = this;
    switch (_that) {
      case _ElectionId():
        return $default(_that.url, _that.hash);
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
    TResult? Function(String? url, Uint8List hash)? $default,
  ) {
    final _that = this;
    switch (_that) {
      case _ElectionId() when $default != null:
        return $default(_that.url, _that.hash);
      case _:
        return null;
    }
  }
}

/// @nodoc

class _ElectionId implements ElectionId {
  const _ElectionId({this.url, required this.hash});

  @override
  final String? url;
  @override
  final Uint8List hash;

  /// Create a copy of ElectionId
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  _$ElectionIdCopyWith<_ElectionId> get copyWith =>
      __$ElectionIdCopyWithImpl<_ElectionId>(this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _ElectionId &&
            (identical(other.url, url) || other.url == url) &&
            const DeepCollectionEquality().equals(other.hash, hash));
  }

  @override
  int get hashCode =>
      Object.hash(runtimeType, url, const DeepCollectionEquality().hash(hash));

  @override
  String toString() {
    return 'ElectionId(url: $url, hash: $hash)';
  }
}

/// @nodoc
abstract mixin class _$ElectionIdCopyWith<$Res>
    implements $ElectionIdCopyWith<$Res> {
  factory _$ElectionIdCopyWith(
          _ElectionId value, $Res Function(_ElectionId) _then) =
      __$ElectionIdCopyWithImpl;
  @override
  @useResult
  $Res call({String? url, Uint8List hash});
}

/// @nodoc
class __$ElectionIdCopyWithImpl<$Res> implements _$ElectionIdCopyWith<$Res> {
  __$ElectionIdCopyWithImpl(this._self, this._then);

  final _ElectionId _self;
  final $Res Function(_ElectionId) _then;

  /// Create a copy of ElectionId
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $Res call({
    Object? url = freezed,
    Object? hash = null,
  }) {
    return _then(_ElectionId(
      url: freezed == url
          ? _self.url
          : url // ignore: cast_nullable_to_non_nullable
              as String?,
      hash: null == hash
          ? _self.hash
          : hash // ignore: cast_nullable_to_non_nullable
              as Uint8List,
    ));
  }
}

// dart format on
