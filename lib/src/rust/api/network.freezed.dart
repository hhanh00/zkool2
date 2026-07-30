// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'network.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;

/// @nodoc
mixin _$ExchangeRate {
  double get fromPrice;
  double get toPrice;
  String get fromCurrency;
  String get toCurrency;

  /// Create a copy of ExchangeRate
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $ExchangeRateCopyWith<ExchangeRate> get copyWith => _$ExchangeRateCopyWithImpl<ExchangeRate>(this as ExchangeRate, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is ExchangeRate &&
            (identical(other.fromPrice, fromPrice) || other.fromPrice == fromPrice) &&
            (identical(other.toPrice, toPrice) || other.toPrice == toPrice) &&
            (identical(other.fromCurrency, fromCurrency) || other.fromCurrency == fromCurrency) &&
            (identical(other.toCurrency, toCurrency) || other.toCurrency == toCurrency));
  }

  @override
  int get hashCode => Object.hash(runtimeType, fromPrice, toPrice, fromCurrency, toCurrency);

  @override
  String toString() {
    return 'ExchangeRate(fromPrice: $fromPrice, toPrice: $toPrice, fromCurrency: $fromCurrency, toCurrency: $toCurrency)';
  }
}

/// @nodoc
abstract mixin class $ExchangeRateCopyWith<$Res> {
  factory $ExchangeRateCopyWith(ExchangeRate value, $Res Function(ExchangeRate) _then) = _$ExchangeRateCopyWithImpl;
  @useResult
  $Res call({double fromPrice, double toPrice, String fromCurrency, String toCurrency});
}

/// @nodoc
class _$ExchangeRateCopyWithImpl<$Res> implements $ExchangeRateCopyWith<$Res> {
  _$ExchangeRateCopyWithImpl(this._self, this._then);

  final ExchangeRate _self;
  final $Res Function(ExchangeRate) _then;

  /// Create a copy of ExchangeRate
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? fromPrice = null,
    Object? toPrice = null,
    Object? fromCurrency = null,
    Object? toCurrency = null,
  }) {
    return _then(_self.copyWith(
      fromPrice: null == fromPrice
          ? _self.fromPrice
          : fromPrice // ignore: cast_nullable_to_non_nullable
              as double,
      toPrice: null == toPrice
          ? _self.toPrice
          : toPrice // ignore: cast_nullable_to_non_nullable
              as double,
      fromCurrency: null == fromCurrency
          ? _self.fromCurrency
          : fromCurrency // ignore: cast_nullable_to_non_nullable
              as String,
      toCurrency: null == toCurrency
          ? _self.toCurrency
          : toCurrency // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// Adds pattern-matching-related methods to [ExchangeRate].
extension ExchangeRatePatterns on ExchangeRate {
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
    TResult Function(_ExchangeRate value)? $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _ExchangeRate() when $default != null:
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
    TResult Function(_ExchangeRate value) $default,
  ) {
    final _that = this;
    switch (_that) {
      case _ExchangeRate():
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
    TResult? Function(_ExchangeRate value)? $default,
  ) {
    final _that = this;
    switch (_that) {
      case _ExchangeRate() when $default != null:
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
    TResult Function(double fromPrice, double toPrice, String fromCurrency, String toCurrency)? $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _ExchangeRate() when $default != null:
        return $default(_that.fromPrice, _that.toPrice, _that.fromCurrency, _that.toCurrency);
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
    TResult Function(double fromPrice, double toPrice, String fromCurrency, String toCurrency) $default,
  ) {
    final _that = this;
    switch (_that) {
      case _ExchangeRate():
        return $default(_that.fromPrice, _that.toPrice, _that.fromCurrency, _that.toCurrency);
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
    TResult? Function(double fromPrice, double toPrice, String fromCurrency, String toCurrency)? $default,
  ) {
    final _that = this;
    switch (_that) {
      case _ExchangeRate() when $default != null:
        return $default(_that.fromPrice, _that.toPrice, _that.fromCurrency, _that.toCurrency);
      case _:
        return null;
    }
  }
}

/// @nodoc

class _ExchangeRate implements ExchangeRate {
  const _ExchangeRate({required this.fromPrice, required this.toPrice, required this.fromCurrency, required this.toCurrency});

  @override
  final double fromPrice;
  @override
  final double toPrice;
  @override
  final String fromCurrency;
  @override
  final String toCurrency;

  /// Create a copy of ExchangeRate
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  _$ExchangeRateCopyWith<_ExchangeRate> get copyWith => __$ExchangeRateCopyWithImpl<_ExchangeRate>(this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _ExchangeRate &&
            (identical(other.fromPrice, fromPrice) || other.fromPrice == fromPrice) &&
            (identical(other.toPrice, toPrice) || other.toPrice == toPrice) &&
            (identical(other.fromCurrency, fromCurrency) || other.fromCurrency == fromCurrency) &&
            (identical(other.toCurrency, toCurrency) || other.toCurrency == toCurrency));
  }

  @override
  int get hashCode => Object.hash(runtimeType, fromPrice, toPrice, fromCurrency, toCurrency);

  @override
  String toString() {
    return 'ExchangeRate(fromPrice: $fromPrice, toPrice: $toPrice, fromCurrency: $fromCurrency, toCurrency: $toCurrency)';
  }
}

/// @nodoc
abstract mixin class _$ExchangeRateCopyWith<$Res> implements $ExchangeRateCopyWith<$Res> {
  factory _$ExchangeRateCopyWith(_ExchangeRate value, $Res Function(_ExchangeRate) _then) = __$ExchangeRateCopyWithImpl;
  @override
  @useResult
  $Res call({double fromPrice, double toPrice, String fromCurrency, String toCurrency});
}

/// @nodoc
class __$ExchangeRateCopyWithImpl<$Res> implements _$ExchangeRateCopyWith<$Res> {
  __$ExchangeRateCopyWithImpl(this._self, this._then);

  final _ExchangeRate _self;
  final $Res Function(_ExchangeRate) _then;

  /// Create a copy of ExchangeRate
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $Res call({
    Object? fromPrice = null,
    Object? toPrice = null,
    Object? fromCurrency = null,
    Object? toCurrency = null,
  }) {
    return _then(_ExchangeRate(
      fromPrice: null == fromPrice
          ? _self.fromPrice
          : fromPrice // ignore: cast_nullable_to_non_nullable
              as double,
      toPrice: null == toPrice
          ? _self.toPrice
          : toPrice // ignore: cast_nullable_to_non_nullable
              as double,
      fromCurrency: null == fromCurrency
          ? _self.fromCurrency
          : fromCurrency // ignore: cast_nullable_to_non_nullable
              as String,
      toCurrency: null == toCurrency
          ? _self.toCurrency
          : toCurrency // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// @nodoc
mixin _$LWDInfo {
  String get url;
  bool get isTor;
  int get height;
  String get status;
  int get uptime;
  String get version;
  int get ping;

  /// Create a copy of LWDInfo
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $LWDInfoCopyWith<LWDInfo> get copyWith => _$LWDInfoCopyWithImpl<LWDInfo>(this as LWDInfo, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is LWDInfo &&
            (identical(other.url, url) || other.url == url) &&
            (identical(other.isTor, isTor) || other.isTor == isTor) &&
            (identical(other.height, height) || other.height == height) &&
            (identical(other.status, status) || other.status == status) &&
            (identical(other.uptime, uptime) || other.uptime == uptime) &&
            (identical(other.version, version) || other.version == version) &&
            (identical(other.ping, ping) || other.ping == ping));
  }

  @override
  int get hashCode => Object.hash(runtimeType, url, isTor, height, status, uptime, version, ping);

  @override
  String toString() {
    return 'LWDInfo(url: $url, isTor: $isTor, height: $height, status: $status, uptime: $uptime, version: $version, ping: $ping)';
  }
}

/// @nodoc
abstract mixin class $LWDInfoCopyWith<$Res> {
  factory $LWDInfoCopyWith(LWDInfo value, $Res Function(LWDInfo) _then) = _$LWDInfoCopyWithImpl;
  @useResult
  $Res call({String url, bool isTor, int height, String status, int uptime, String version, int ping});
}

/// @nodoc
class _$LWDInfoCopyWithImpl<$Res> implements $LWDInfoCopyWith<$Res> {
  _$LWDInfoCopyWithImpl(this._self, this._then);

  final LWDInfo _self;
  final $Res Function(LWDInfo) _then;

  /// Create a copy of LWDInfo
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? url = null,
    Object? isTor = null,
    Object? height = null,
    Object? status = null,
    Object? uptime = null,
    Object? version = null,
    Object? ping = null,
  }) {
    return _then(_self.copyWith(
      url: null == url
          ? _self.url
          : url // ignore: cast_nullable_to_non_nullable
              as String,
      isTor: null == isTor
          ? _self.isTor
          : isTor // ignore: cast_nullable_to_non_nullable
              as bool,
      height: null == height
          ? _self.height
          : height // ignore: cast_nullable_to_non_nullable
              as int,
      status: null == status
          ? _self.status
          : status // ignore: cast_nullable_to_non_nullable
              as String,
      uptime: null == uptime
          ? _self.uptime
          : uptime // ignore: cast_nullable_to_non_nullable
              as int,
      version: null == version
          ? _self.version
          : version // ignore: cast_nullable_to_non_nullable
              as String,
      ping: null == ping
          ? _self.ping
          : ping // ignore: cast_nullable_to_non_nullable
              as int,
    ));
  }
}

/// Adds pattern-matching-related methods to [LWDInfo].
extension LWDInfoPatterns on LWDInfo {
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
    TResult Function(_LWDInfo value)? $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _LWDInfo() when $default != null:
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
    TResult Function(_LWDInfo value) $default,
  ) {
    final _that = this;
    switch (_that) {
      case _LWDInfo():
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
    TResult? Function(_LWDInfo value)? $default,
  ) {
    final _that = this;
    switch (_that) {
      case _LWDInfo() when $default != null:
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
    TResult Function(String url, bool isTor, int height, String status, int uptime, String version, int ping)? $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _LWDInfo() when $default != null:
        return $default(_that.url, _that.isTor, _that.height, _that.status, _that.uptime, _that.version, _that.ping);
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
    TResult Function(String url, bool isTor, int height, String status, int uptime, String version, int ping) $default,
  ) {
    final _that = this;
    switch (_that) {
      case _LWDInfo():
        return $default(_that.url, _that.isTor, _that.height, _that.status, _that.uptime, _that.version, _that.ping);
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
    TResult? Function(String url, bool isTor, int height, String status, int uptime, String version, int ping)? $default,
  ) {
    final _that = this;
    switch (_that) {
      case _LWDInfo() when $default != null:
        return $default(_that.url, _that.isTor, _that.height, _that.status, _that.uptime, _that.version, _that.ping);
      case _:
        return null;
    }
  }
}

/// @nodoc

class _LWDInfo implements LWDInfo {
  const _LWDInfo(
      {required this.url, required this.isTor, required this.height, required this.status, required this.uptime, required this.version, required this.ping});

  @override
  final String url;
  @override
  final bool isTor;
  @override
  final int height;
  @override
  final String status;
  @override
  final int uptime;
  @override
  final String version;
  @override
  final int ping;

  /// Create a copy of LWDInfo
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  _$LWDInfoCopyWith<_LWDInfo> get copyWith => __$LWDInfoCopyWithImpl<_LWDInfo>(this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _LWDInfo &&
            (identical(other.url, url) || other.url == url) &&
            (identical(other.isTor, isTor) || other.isTor == isTor) &&
            (identical(other.height, height) || other.height == height) &&
            (identical(other.status, status) || other.status == status) &&
            (identical(other.uptime, uptime) || other.uptime == uptime) &&
            (identical(other.version, version) || other.version == version) &&
            (identical(other.ping, ping) || other.ping == ping));
  }

  @override
  int get hashCode => Object.hash(runtimeType, url, isTor, height, status, uptime, version, ping);

  @override
  String toString() {
    return 'LWDInfo(url: $url, isTor: $isTor, height: $height, status: $status, uptime: $uptime, version: $version, ping: $ping)';
  }
}

/// @nodoc
abstract mixin class _$LWDInfoCopyWith<$Res> implements $LWDInfoCopyWith<$Res> {
  factory _$LWDInfoCopyWith(_LWDInfo value, $Res Function(_LWDInfo) _then) = __$LWDInfoCopyWithImpl;
  @override
  @useResult
  $Res call({String url, bool isTor, int height, String status, int uptime, String version, int ping});
}

/// @nodoc
class __$LWDInfoCopyWithImpl<$Res> implements _$LWDInfoCopyWith<$Res> {
  __$LWDInfoCopyWithImpl(this._self, this._then);

  final _LWDInfo _self;
  final $Res Function(_LWDInfo) _then;

  /// Create a copy of LWDInfo
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $Res call({
    Object? url = null,
    Object? isTor = null,
    Object? height = null,
    Object? status = null,
    Object? uptime = null,
    Object? version = null,
    Object? ping = null,
  }) {
    return _then(_LWDInfo(
      url: null == url
          ? _self.url
          : url // ignore: cast_nullable_to_non_nullable
              as String,
      isTor: null == isTor
          ? _self.isTor
          : isTor // ignore: cast_nullable_to_non_nullable
              as bool,
      height: null == height
          ? _self.height
          : height // ignore: cast_nullable_to_non_nullable
              as int,
      status: null == status
          ? _self.status
          : status // ignore: cast_nullable_to_non_nullable
              as String,
      uptime: null == uptime
          ? _self.uptime
          : uptime // ignore: cast_nullable_to_non_nullable
              as int,
      version: null == version
          ? _self.version
          : version // ignore: cast_nullable_to_non_nullable
              as String,
      ping: null == ping
          ? _self.ping
          : ping // ignore: cast_nullable_to_non_nullable
              as int,
    ));
  }
}

// dart format on
