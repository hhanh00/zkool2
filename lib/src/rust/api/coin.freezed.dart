// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'coin.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;

/// @nodoc
mixin _$Coin {
  int get coin;
  int get account;
  String get dbFilepath;
  String get url;
  int get serverType;
  bool get useTor;

  /// Create a copy of Coin
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $CoinCopyWith<Coin> get copyWith =>
      _$CoinCopyWithImpl<Coin>(this as Coin, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is Coin &&
            (identical(other.coin, coin) || other.coin == coin) &&
            (identical(other.account, account) || other.account == account) &&
            (identical(other.dbFilepath, dbFilepath) ||
                other.dbFilepath == dbFilepath) &&
            (identical(other.url, url) || other.url == url) &&
            (identical(other.serverType, serverType) ||
                other.serverType == serverType) &&
            (identical(other.useTor, useTor) || other.useTor == useTor));
  }

  @override
  int get hashCode => Object.hash(
      runtimeType, coin, account, dbFilepath, url, serverType, useTor);

  @override
  String toString() {
    return 'Coin(coin: $coin, account: $account, dbFilepath: $dbFilepath, url: $url, serverType: $serverType, useTor: $useTor)';
  }
}

/// @nodoc
abstract mixin class $CoinCopyWith<$Res> {
  factory $CoinCopyWith(Coin value, $Res Function(Coin) _then) =
      _$CoinCopyWithImpl;
  @useResult
  $Res call(
      {int coin,
      int account,
      String dbFilepath,
      String url,
      int serverType,
      bool useTor});
}

/// @nodoc
class _$CoinCopyWithImpl<$Res> implements $CoinCopyWith<$Res> {
  _$CoinCopyWithImpl(this._self, this._then);

  final Coin _self;
  final $Res Function(Coin) _then;

  /// Create a copy of Coin
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? coin = null,
    Object? account = null,
    Object? dbFilepath = null,
    Object? url = null,
    Object? serverType = null,
    Object? useTor = null,
  }) {
    return _then(_self.copyWith(
      coin: null == coin
          ? _self.coin
          : coin // ignore: cast_nullable_to_non_nullable
              as int,
      account: null == account
          ? _self.account
          : account // ignore: cast_nullable_to_non_nullable
              as int,
      dbFilepath: null == dbFilepath
          ? _self.dbFilepath
          : dbFilepath // ignore: cast_nullable_to_non_nullable
              as String,
      url: null == url
          ? _self.url
          : url // ignore: cast_nullable_to_non_nullable
              as String,
      serverType: null == serverType
          ? _self.serverType
          : serverType // ignore: cast_nullable_to_non_nullable
              as int,
      useTor: null == useTor
          ? _self.useTor
          : useTor // ignore: cast_nullable_to_non_nullable
              as bool,
    ));
  }
}

/// Adds pattern-matching-related methods to [Coin].
extension CoinPatterns on Coin {
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
    TResult Function(_Coin value)? raw,
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _Coin() when raw != null:
        return raw(_that);
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
    required TResult Function(_Coin value) raw,
  }) {
    final _that = this;
    switch (_that) {
      case _Coin():
        return raw(_that);
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
    TResult? Function(_Coin value)? raw,
  }) {
    final _that = this;
    switch (_that) {
      case _Coin() when raw != null:
        return raw(_that);
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
    TResult Function(int coin, int account, String dbFilepath, String url,
            int serverType, bool useTor)?
        raw,
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _Coin() when raw != null:
        return raw(_that.coin, _that.account, _that.dbFilepath, _that.url,
            _that.serverType, _that.useTor);
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
    required TResult Function(int coin, int account, String dbFilepath,
            String url, int serverType, bool useTor)
        raw,
  }) {
    final _that = this;
    switch (_that) {
      case _Coin():
        return raw(_that.coin, _that.account, _that.dbFilepath, _that.url,
            _that.serverType, _that.useTor);
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
    TResult? Function(int coin, int account, String dbFilepath, String url,
            int serverType, bool useTor)?
        raw,
  }) {
    final _that = this;
    switch (_that) {
      case _Coin() when raw != null:
        return raw(_that.coin, _that.account, _that.dbFilepath, _that.url,
            _that.serverType, _that.useTor);
      case _:
        return null;
    }
  }
}

/// @nodoc

class _Coin extends Coin {
  const _Coin(
      {required this.coin,
      required this.account,
      required this.dbFilepath,
      required this.url,
      required this.serverType,
      required this.useTor})
      : super._();

  @override
  final int coin;
  @override
  final int account;
  @override
  final String dbFilepath;
  @override
  final String url;
  @override
  final int serverType;
  @override
  final bool useTor;

  /// Create a copy of Coin
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  _$CoinCopyWith<_Coin> get copyWith =>
      __$CoinCopyWithImpl<_Coin>(this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _Coin &&
            (identical(other.coin, coin) || other.coin == coin) &&
            (identical(other.account, account) || other.account == account) &&
            (identical(other.dbFilepath, dbFilepath) ||
                other.dbFilepath == dbFilepath) &&
            (identical(other.url, url) || other.url == url) &&
            (identical(other.serverType, serverType) ||
                other.serverType == serverType) &&
            (identical(other.useTor, useTor) || other.useTor == useTor));
  }

  @override
  int get hashCode => Object.hash(
      runtimeType, coin, account, dbFilepath, url, serverType, useTor);

  @override
  String toString() {
    return 'Coin.raw(coin: $coin, account: $account, dbFilepath: $dbFilepath, url: $url, serverType: $serverType, useTor: $useTor)';
  }
}

/// @nodoc
abstract mixin class _$CoinCopyWith<$Res> implements $CoinCopyWith<$Res> {
  factory _$CoinCopyWith(_Coin value, $Res Function(_Coin) _then) =
      __$CoinCopyWithImpl;
  @override
  @useResult
  $Res call(
      {int coin,
      int account,
      String dbFilepath,
      String url,
      int serverType,
      bool useTor});
}

/// @nodoc
class __$CoinCopyWithImpl<$Res> implements _$CoinCopyWith<$Res> {
  __$CoinCopyWithImpl(this._self, this._then);

  final _Coin _self;
  final $Res Function(_Coin) _then;

  /// Create a copy of Coin
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $Res call({
    Object? coin = null,
    Object? account = null,
    Object? dbFilepath = null,
    Object? url = null,
    Object? serverType = null,
    Object? useTor = null,
  }) {
    return _then(_Coin(
      coin: null == coin
          ? _self.coin
          : coin // ignore: cast_nullable_to_non_nullable
              as int,
      account: null == account
          ? _self.account
          : account // ignore: cast_nullable_to_non_nullable
              as int,
      dbFilepath: null == dbFilepath
          ? _self.dbFilepath
          : dbFilepath // ignore: cast_nullable_to_non_nullable
              as String,
      url: null == url
          ? _self.url
          : url // ignore: cast_nullable_to_non_nullable
              as String,
      serverType: null == serverType
          ? _self.serverType
          : serverType // ignore: cast_nullable_to_non_nullable
              as int,
      useTor: null == useTor
          ? _self.useTor
          : useTor // ignore: cast_nullable_to_non_nullable
              as bool,
    ));
  }
}

// dart format on
