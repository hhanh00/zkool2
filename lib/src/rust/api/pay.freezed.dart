// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'pay.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;

/// @nodoc
mixin _$PcztPackage {
  Uint8List get pczt;
  UsizeArray3 get nSpends;
  Uint64List get saplingIndices;
  Uint64List get orchardIndices;
  bool get canSign;
  bool get canBroadcast;
  double? get price;
  int? get category;

  /// Create a copy of PcztPackage
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $PcztPackageCopyWith<PcztPackage> get copyWith =>
      _$PcztPackageCopyWithImpl<PcztPackage>(this as PcztPackage, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is PcztPackage &&
            const DeepCollectionEquality().equals(other.pczt, pczt) &&
            const DeepCollectionEquality().equals(other.nSpends, nSpends) &&
            const DeepCollectionEquality()
                .equals(other.saplingIndices, saplingIndices) &&
            const DeepCollectionEquality()
                .equals(other.orchardIndices, orchardIndices) &&
            (identical(other.canSign, canSign) || other.canSign == canSign) &&
            (identical(other.canBroadcast, canBroadcast) ||
                other.canBroadcast == canBroadcast) &&
            (identical(other.price, price) || other.price == price) &&
            (identical(other.category, category) ||
                other.category == category));
  }

  @override
  int get hashCode => Object.hash(
      runtimeType,
      const DeepCollectionEquality().hash(pczt),
      const DeepCollectionEquality().hash(nSpends),
      const DeepCollectionEquality().hash(saplingIndices),
      const DeepCollectionEquality().hash(orchardIndices),
      canSign,
      canBroadcast,
      price,
      category);

  @override
  String toString() {
    return 'PcztPackage(pczt: $pczt, nSpends: $nSpends, saplingIndices: $saplingIndices, orchardIndices: $orchardIndices, canSign: $canSign, canBroadcast: $canBroadcast, price: $price, category: $category)';
  }
}

/// @nodoc
abstract mixin class $PcztPackageCopyWith<$Res> {
  factory $PcztPackageCopyWith(
          PcztPackage value, $Res Function(PcztPackage) _then) =
      _$PcztPackageCopyWithImpl;
  @useResult
  $Res call(
      {Uint8List pczt,
      UsizeArray3 nSpends,
      Uint64List saplingIndices,
      Uint64List orchardIndices,
      bool canSign,
      bool canBroadcast,
      double? price,
      int? category});
}

/// @nodoc
class _$PcztPackageCopyWithImpl<$Res> implements $PcztPackageCopyWith<$Res> {
  _$PcztPackageCopyWithImpl(this._self, this._then);

  final PcztPackage _self;
  final $Res Function(PcztPackage) _then;

  /// Create a copy of PcztPackage
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? pczt = null,
    Object? nSpends = null,
    Object? saplingIndices = null,
    Object? orchardIndices = null,
    Object? canSign = null,
    Object? canBroadcast = null,
    Object? price = freezed,
    Object? category = freezed,
  }) {
    return _then(_self.copyWith(
      pczt: null == pczt
          ? _self.pczt
          : pczt // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      nSpends: null == nSpends
          ? _self.nSpends
          : nSpends // ignore: cast_nullable_to_non_nullable
              as UsizeArray3,
      saplingIndices: null == saplingIndices
          ? _self.saplingIndices
          : saplingIndices // ignore: cast_nullable_to_non_nullable
              as Uint64List,
      orchardIndices: null == orchardIndices
          ? _self.orchardIndices
          : orchardIndices // ignore: cast_nullable_to_non_nullable
              as Uint64List,
      canSign: null == canSign
          ? _self.canSign
          : canSign // ignore: cast_nullable_to_non_nullable
              as bool,
      canBroadcast: null == canBroadcast
          ? _self.canBroadcast
          : canBroadcast // ignore: cast_nullable_to_non_nullable
              as bool,
      price: freezed == price
          ? _self.price
          : price // ignore: cast_nullable_to_non_nullable
              as double?,
      category: freezed == category
          ? _self.category
          : category // ignore: cast_nullable_to_non_nullable
              as int?,
    ));
  }
}

/// Adds pattern-matching-related methods to [PcztPackage].
extension PcztPackagePatterns on PcztPackage {
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
    TResult Function(_PcztPackage value)? $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _PcztPackage() when $default != null:
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
    TResult Function(_PcztPackage value) $default,
  ) {
    final _that = this;
    switch (_that) {
      case _PcztPackage():
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
    TResult? Function(_PcztPackage value)? $default,
  ) {
    final _that = this;
    switch (_that) {
      case _PcztPackage() when $default != null:
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
            Uint8List pczt,
            UsizeArray3 nSpends,
            Uint64List saplingIndices,
            Uint64List orchardIndices,
            bool canSign,
            bool canBroadcast,
            double? price,
            int? category)?
        $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _PcztPackage() when $default != null:
        return $default(
            _that.pczt,
            _that.nSpends,
            _that.saplingIndices,
            _that.orchardIndices,
            _that.canSign,
            _that.canBroadcast,
            _that.price,
            _that.category);
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
            Uint8List pczt,
            UsizeArray3 nSpends,
            Uint64List saplingIndices,
            Uint64List orchardIndices,
            bool canSign,
            bool canBroadcast,
            double? price,
            int? category)
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _PcztPackage():
        return $default(
            _that.pczt,
            _that.nSpends,
            _that.saplingIndices,
            _that.orchardIndices,
            _that.canSign,
            _that.canBroadcast,
            _that.price,
            _that.category);
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
            Uint8List pczt,
            UsizeArray3 nSpends,
            Uint64List saplingIndices,
            Uint64List orchardIndices,
            bool canSign,
            bool canBroadcast,
            double? price,
            int? category)?
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _PcztPackage() when $default != null:
        return $default(
            _that.pczt,
            _that.nSpends,
            _that.saplingIndices,
            _that.orchardIndices,
            _that.canSign,
            _that.canBroadcast,
            _that.price,
            _that.category);
      case _:
        return null;
    }
  }
}

/// @nodoc

class _PcztPackage implements PcztPackage {
  const _PcztPackage(
      {required this.pczt,
      required this.nSpends,
      required this.saplingIndices,
      required this.orchardIndices,
      required this.canSign,
      required this.canBroadcast,
      this.price,
      this.category});

  @override
  final Uint8List pczt;
  @override
  final UsizeArray3 nSpends;
  @override
  final Uint64List saplingIndices;
  @override
  final Uint64List orchardIndices;
  @override
  final bool canSign;
  @override
  final bool canBroadcast;
  @override
  final double? price;
  @override
  final int? category;

  /// Create a copy of PcztPackage
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  _$PcztPackageCopyWith<_PcztPackage> get copyWith =>
      __$PcztPackageCopyWithImpl<_PcztPackage>(this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _PcztPackage &&
            const DeepCollectionEquality().equals(other.pczt, pczt) &&
            const DeepCollectionEquality().equals(other.nSpends, nSpends) &&
            const DeepCollectionEquality()
                .equals(other.saplingIndices, saplingIndices) &&
            const DeepCollectionEquality()
                .equals(other.orchardIndices, orchardIndices) &&
            (identical(other.canSign, canSign) || other.canSign == canSign) &&
            (identical(other.canBroadcast, canBroadcast) ||
                other.canBroadcast == canBroadcast) &&
            (identical(other.price, price) || other.price == price) &&
            (identical(other.category, category) ||
                other.category == category));
  }

  @override
  int get hashCode => Object.hash(
      runtimeType,
      const DeepCollectionEquality().hash(pczt),
      const DeepCollectionEquality().hash(nSpends),
      const DeepCollectionEquality().hash(saplingIndices),
      const DeepCollectionEquality().hash(orchardIndices),
      canSign,
      canBroadcast,
      price,
      category);

  @override
  String toString() {
    return 'PcztPackage(pczt: $pczt, nSpends: $nSpends, saplingIndices: $saplingIndices, orchardIndices: $orchardIndices, canSign: $canSign, canBroadcast: $canBroadcast, price: $price, category: $category)';
  }
}

/// @nodoc
abstract mixin class _$PcztPackageCopyWith<$Res>
    implements $PcztPackageCopyWith<$Res> {
  factory _$PcztPackageCopyWith(
          _PcztPackage value, $Res Function(_PcztPackage) _then) =
      __$PcztPackageCopyWithImpl;
  @override
  @useResult
  $Res call(
      {Uint8List pczt,
      UsizeArray3 nSpends,
      Uint64List saplingIndices,
      Uint64List orchardIndices,
      bool canSign,
      bool canBroadcast,
      double? price,
      int? category});
}

/// @nodoc
class __$PcztPackageCopyWithImpl<$Res> implements _$PcztPackageCopyWith<$Res> {
  __$PcztPackageCopyWithImpl(this._self, this._then);

  final _PcztPackage _self;
  final $Res Function(_PcztPackage) _then;

  /// Create a copy of PcztPackage
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $Res call({
    Object? pczt = null,
    Object? nSpends = null,
    Object? saplingIndices = null,
    Object? orchardIndices = null,
    Object? canSign = null,
    Object? canBroadcast = null,
    Object? price = freezed,
    Object? category = freezed,
  }) {
    return _then(_PcztPackage(
      pczt: null == pczt
          ? _self.pczt
          : pczt // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      nSpends: null == nSpends
          ? _self.nSpends
          : nSpends // ignore: cast_nullable_to_non_nullable
              as UsizeArray3,
      saplingIndices: null == saplingIndices
          ? _self.saplingIndices
          : saplingIndices // ignore: cast_nullable_to_non_nullable
              as Uint64List,
      orchardIndices: null == orchardIndices
          ? _self.orchardIndices
          : orchardIndices // ignore: cast_nullable_to_non_nullable
              as Uint64List,
      canSign: null == canSign
          ? _self.canSign
          : canSign // ignore: cast_nullable_to_non_nullable
              as bool,
      canBroadcast: null == canBroadcast
          ? _self.canBroadcast
          : canBroadcast // ignore: cast_nullable_to_non_nullable
              as bool,
      price: freezed == price
          ? _self.price
          : price // ignore: cast_nullable_to_non_nullable
              as double?,
      category: freezed == category
          ? _self.category
          : category // ignore: cast_nullable_to_non_nullable
              as int?,
    ));
  }
}

/// @nodoc
mixin _$SigningEvent {
  Object get field0;

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is SigningEvent &&
            const DeepCollectionEquality().equals(other.field0, field0));
  }

  @override
  int get hashCode =>
      Object.hash(runtimeType, const DeepCollectionEquality().hash(field0));

  @override
  String toString() {
    return 'SigningEvent(field0: $field0)';
  }
}

/// @nodoc
class $SigningEventCopyWith<$Res> {
  $SigningEventCopyWith(SigningEvent _, $Res Function(SigningEvent) __);
}

/// Adds pattern-matching-related methods to [SigningEvent].
extension SigningEventPatterns on SigningEvent {
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
    TResult Function(SigningEvent_Progress value)? progress,
    TResult Function(SigningEvent_Result value)? result,
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case SigningEvent_Progress() when progress != null:
        return progress(_that);
      case SigningEvent_Result() when result != null:
        return result(_that);
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
    required TResult Function(SigningEvent_Progress value) progress,
    required TResult Function(SigningEvent_Result value) result,
  }) {
    final _that = this;
    switch (_that) {
      case SigningEvent_Progress():
        return progress(_that);
      case SigningEvent_Result():
        return result(_that);
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
    TResult? Function(SigningEvent_Progress value)? progress,
    TResult? Function(SigningEvent_Result value)? result,
  }) {
    final _that = this;
    switch (_that) {
      case SigningEvent_Progress() when progress != null:
        return progress(_that);
      case SigningEvent_Result() when result != null:
        return result(_that);
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
    TResult Function(String field0)? progress,
    TResult Function(PcztPackage field0)? result,
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case SigningEvent_Progress() when progress != null:
        return progress(_that.field0);
      case SigningEvent_Result() when result != null:
        return result(_that.field0);
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
    required TResult Function(String field0) progress,
    required TResult Function(PcztPackage field0) result,
  }) {
    final _that = this;
    switch (_that) {
      case SigningEvent_Progress():
        return progress(_that.field0);
      case SigningEvent_Result():
        return result(_that.field0);
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
    TResult? Function(String field0)? progress,
    TResult? Function(PcztPackage field0)? result,
  }) {
    final _that = this;
    switch (_that) {
      case SigningEvent_Progress() when progress != null:
        return progress(_that.field0);
      case SigningEvent_Result() when result != null:
        return result(_that.field0);
      case _:
        return null;
    }
  }
}

/// @nodoc

class SigningEvent_Progress extends SigningEvent {
  const SigningEvent_Progress(this.field0) : super._();

  @override
  final String field0;

  /// Create a copy of SigningEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $SigningEvent_ProgressCopyWith<SigningEvent_Progress> get copyWith =>
      _$SigningEvent_ProgressCopyWithImpl<SigningEvent_Progress>(
          this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is SigningEvent_Progress &&
            (identical(other.field0, field0) || other.field0 == field0));
  }

  @override
  int get hashCode => Object.hash(runtimeType, field0);

  @override
  String toString() {
    return 'SigningEvent.progress(field0: $field0)';
  }
}

/// @nodoc
abstract mixin class $SigningEvent_ProgressCopyWith<$Res>
    implements $SigningEventCopyWith<$Res> {
  factory $SigningEvent_ProgressCopyWith(SigningEvent_Progress value,
          $Res Function(SigningEvent_Progress) _then) =
      _$SigningEvent_ProgressCopyWithImpl;
  @useResult
  $Res call({String field0});
}

/// @nodoc
class _$SigningEvent_ProgressCopyWithImpl<$Res>
    implements $SigningEvent_ProgressCopyWith<$Res> {
  _$SigningEvent_ProgressCopyWithImpl(this._self, this._then);

  final SigningEvent_Progress _self;
  final $Res Function(SigningEvent_Progress) _then;

  /// Create a copy of SigningEvent
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  $Res call({
    Object? field0 = null,
  }) {
    return _then(SigningEvent_Progress(
      null == field0
          ? _self.field0
          : field0 // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// @nodoc

class SigningEvent_Result extends SigningEvent {
  const SigningEvent_Result(this.field0) : super._();

  @override
  final PcztPackage field0;

  /// Create a copy of SigningEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $SigningEvent_ResultCopyWith<SigningEvent_Result> get copyWith =>
      _$SigningEvent_ResultCopyWithImpl<SigningEvent_Result>(this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is SigningEvent_Result &&
            (identical(other.field0, field0) || other.field0 == field0));
  }

  @override
  int get hashCode => Object.hash(runtimeType, field0);

  @override
  String toString() {
    return 'SigningEvent.result(field0: $field0)';
  }
}

/// @nodoc
abstract mixin class $SigningEvent_ResultCopyWith<$Res>
    implements $SigningEventCopyWith<$Res> {
  factory $SigningEvent_ResultCopyWith(
          SigningEvent_Result value, $Res Function(SigningEvent_Result) _then) =
      _$SigningEvent_ResultCopyWithImpl;
  @useResult
  $Res call({PcztPackage field0});

  $PcztPackageCopyWith<$Res> get field0;
}

/// @nodoc
class _$SigningEvent_ResultCopyWithImpl<$Res>
    implements $SigningEvent_ResultCopyWith<$Res> {
  _$SigningEvent_ResultCopyWithImpl(this._self, this._then);

  final SigningEvent_Result _self;
  final $Res Function(SigningEvent_Result) _then;

  /// Create a copy of SigningEvent
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  $Res call({
    Object? field0 = null,
  }) {
    return _then(SigningEvent_Result(
      null == field0
          ? _self.field0
          : field0 // ignore: cast_nullable_to_non_nullable
              as PcztPackage,
    ));
  }

  /// Create a copy of SigningEvent
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $PcztPackageCopyWith<$Res> get field0 {
    return $PcztPackageCopyWith<$Res>(_self.field0, (value) {
      return _then(_self.copyWith(field0: value));
    });
  }
}

// dart format on
