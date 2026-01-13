// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'account.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;

/// @nodoc
mixin _$Account {
  int get coin;
  int get id;
  String get name;
  String? get seed;
  String? get passphrase;
  int get aindex;
  int get dindex;
  Uint8List? get icon;
  int get birth;
  Folder get folder;
  int get position;
  bool get hidden;
  bool get saved;
  bool get enabled;
  bool get internal;
  int get hw;
  int get height;
  int get time;
  BigInt get balance;

  /// Create a copy of Account
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $AccountCopyWith<Account> get copyWith =>
      _$AccountCopyWithImpl<Account>(this as Account, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is Account &&
            (identical(other.coin, coin) || other.coin == coin) &&
            (identical(other.id, id) || other.id == id) &&
            (identical(other.name, name) || other.name == name) &&
            (identical(other.seed, seed) || other.seed == seed) &&
            (identical(other.passphrase, passphrase) ||
                other.passphrase == passphrase) &&
            (identical(other.aindex, aindex) || other.aindex == aindex) &&
            (identical(other.dindex, dindex) || other.dindex == dindex) &&
            const DeepCollectionEquality().equals(other.icon, icon) &&
            (identical(other.birth, birth) || other.birth == birth) &&
            (identical(other.folder, folder) || other.folder == folder) &&
            (identical(other.position, position) ||
                other.position == position) &&
            (identical(other.hidden, hidden) || other.hidden == hidden) &&
            (identical(other.saved, saved) || other.saved == saved) &&
            (identical(other.enabled, enabled) || other.enabled == enabled) &&
            (identical(other.internal, internal) ||
                other.internal == internal) &&
            (identical(other.hw, hw) || other.hw == hw) &&
            (identical(other.height, height) || other.height == height) &&
            (identical(other.time, time) || other.time == time) &&
            (identical(other.balance, balance) || other.balance == balance));
  }

  @override
  int get hashCode => Object.hashAll([
        runtimeType,
        coin,
        id,
        name,
        seed,
        passphrase,
        aindex,
        dindex,
        const DeepCollectionEquality().hash(icon),
        birth,
        folder,
        position,
        hidden,
        saved,
        enabled,
        internal,
        hw,
        height,
        time,
        balance
      ]);

  @override
  String toString() {
    return 'Account(coin: $coin, id: $id, name: $name, seed: $seed, passphrase: $passphrase, aindex: $aindex, dindex: $dindex, icon: $icon, birth: $birth, folder: $folder, position: $position, hidden: $hidden, saved: $saved, enabled: $enabled, internal: $internal, hw: $hw, height: $height, time: $time, balance: $balance)';
  }
}

/// @nodoc
abstract mixin class $AccountCopyWith<$Res> {
  factory $AccountCopyWith(Account value, $Res Function(Account) _then) =
      _$AccountCopyWithImpl;
  @useResult
  $Res call(
      {int coin,
      int id,
      String name,
      String? seed,
      String? passphrase,
      int aindex,
      int dindex,
      Uint8List? icon,
      int birth,
      Folder folder,
      int position,
      bool hidden,
      bool saved,
      bool enabled,
      bool internal,
      int hw,
      int height,
      int time,
      BigInt balance});

  $FolderCopyWith<$Res> get folder;
}

/// @nodoc
class _$AccountCopyWithImpl<$Res> implements $AccountCopyWith<$Res> {
  _$AccountCopyWithImpl(this._self, this._then);

  final Account _self;
  final $Res Function(Account) _then;

  /// Create a copy of Account
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? coin = null,
    Object? id = null,
    Object? name = null,
    Object? seed = freezed,
    Object? passphrase = freezed,
    Object? aindex = null,
    Object? dindex = null,
    Object? icon = freezed,
    Object? birth = null,
    Object? folder = null,
    Object? position = null,
    Object? hidden = null,
    Object? saved = null,
    Object? enabled = null,
    Object? internal = null,
    Object? hw = null,
    Object? height = null,
    Object? time = null,
    Object? balance = null,
  }) {
    return _then(_self.copyWith(
      coin: null == coin
          ? _self.coin
          : coin // ignore: cast_nullable_to_non_nullable
              as int,
      id: null == id
          ? _self.id
          : id // ignore: cast_nullable_to_non_nullable
              as int,
      name: null == name
          ? _self.name
          : name // ignore: cast_nullable_to_non_nullable
              as String,
      seed: freezed == seed
          ? _self.seed
          : seed // ignore: cast_nullable_to_non_nullable
              as String?,
      passphrase: freezed == passphrase
          ? _self.passphrase
          : passphrase // ignore: cast_nullable_to_non_nullable
              as String?,
      aindex: null == aindex
          ? _self.aindex
          : aindex // ignore: cast_nullable_to_non_nullable
              as int,
      dindex: null == dindex
          ? _self.dindex
          : dindex // ignore: cast_nullable_to_non_nullable
              as int,
      icon: freezed == icon
          ? _self.icon
          : icon // ignore: cast_nullable_to_non_nullable
              as Uint8List?,
      birth: null == birth
          ? _self.birth
          : birth // ignore: cast_nullable_to_non_nullable
              as int,
      folder: null == folder
          ? _self.folder
          : folder // ignore: cast_nullable_to_non_nullable
              as Folder,
      position: null == position
          ? _self.position
          : position // ignore: cast_nullable_to_non_nullable
              as int,
      hidden: null == hidden
          ? _self.hidden
          : hidden // ignore: cast_nullable_to_non_nullable
              as bool,
      saved: null == saved
          ? _self.saved
          : saved // ignore: cast_nullable_to_non_nullable
              as bool,
      enabled: null == enabled
          ? _self.enabled
          : enabled // ignore: cast_nullable_to_non_nullable
              as bool,
      internal: null == internal
          ? _self.internal
          : internal // ignore: cast_nullable_to_non_nullable
              as bool,
      hw: null == hw
          ? _self.hw
          : hw // ignore: cast_nullable_to_non_nullable
              as int,
      height: null == height
          ? _self.height
          : height // ignore: cast_nullable_to_non_nullable
              as int,
      time: null == time
          ? _self.time
          : time // ignore: cast_nullable_to_non_nullable
              as int,
      balance: null == balance
          ? _self.balance
          : balance // ignore: cast_nullable_to_non_nullable
              as BigInt,
    ));
  }

  /// Create a copy of Account
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $FolderCopyWith<$Res> get folder {
    return $FolderCopyWith<$Res>(_self.folder, (value) {
      return _then(_self.copyWith(folder: value));
    });
  }
}

/// Adds pattern-matching-related methods to [Account].
extension AccountPatterns on Account {
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
    TResult Function(_Account value)? $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _Account() when $default != null:
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
    TResult Function(_Account value) $default,
  ) {
    final _that = this;
    switch (_that) {
      case _Account():
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
    TResult? Function(_Account value)? $default,
  ) {
    final _that = this;
    switch (_that) {
      case _Account() when $default != null:
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
            int coin,
            int id,
            String name,
            String? seed,
            String? passphrase,
            int aindex,
            int dindex,
            Uint8List? icon,
            int birth,
            Folder folder,
            int position,
            bool hidden,
            bool saved,
            bool enabled,
            bool internal,
            int hw,
            int height,
            int time,
            BigInt balance)?
        $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _Account() when $default != null:
        return $default(
            _that.coin,
            _that.id,
            _that.name,
            _that.seed,
            _that.passphrase,
            _that.aindex,
            _that.dindex,
            _that.icon,
            _that.birth,
            _that.folder,
            _that.position,
            _that.hidden,
            _that.saved,
            _that.enabled,
            _that.internal,
            _that.hw,
            _that.height,
            _that.time,
            _that.balance);
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
            int coin,
            int id,
            String name,
            String? seed,
            String? passphrase,
            int aindex,
            int dindex,
            Uint8List? icon,
            int birth,
            Folder folder,
            int position,
            bool hidden,
            bool saved,
            bool enabled,
            bool internal,
            int hw,
            int height,
            int time,
            BigInt balance)
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _Account():
        return $default(
            _that.coin,
            _that.id,
            _that.name,
            _that.seed,
            _that.passphrase,
            _that.aindex,
            _that.dindex,
            _that.icon,
            _that.birth,
            _that.folder,
            _that.position,
            _that.hidden,
            _that.saved,
            _that.enabled,
            _that.internal,
            _that.hw,
            _that.height,
            _that.time,
            _that.balance);
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
            int coin,
            int id,
            String name,
            String? seed,
            String? passphrase,
            int aindex,
            int dindex,
            Uint8List? icon,
            int birth,
            Folder folder,
            int position,
            bool hidden,
            bool saved,
            bool enabled,
            bool internal,
            int hw,
            int height,
            int time,
            BigInt balance)?
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _Account() when $default != null:
        return $default(
            _that.coin,
            _that.id,
            _that.name,
            _that.seed,
            _that.passphrase,
            _that.aindex,
            _that.dindex,
            _that.icon,
            _that.birth,
            _that.folder,
            _that.position,
            _that.hidden,
            _that.saved,
            _that.enabled,
            _that.internal,
            _that.hw,
            _that.height,
            _that.time,
            _that.balance);
      case _:
        return null;
    }
  }
}

/// @nodoc

class _Account implements Account {
  const _Account(
      {required this.coin,
      required this.id,
      required this.name,
      this.seed,
      this.passphrase,
      required this.aindex,
      required this.dindex,
      this.icon,
      required this.birth,
      required this.folder,
      required this.position,
      required this.hidden,
      required this.saved,
      required this.enabled,
      required this.internal,
      required this.hw,
      required this.height,
      required this.time,
      required this.balance});

  @override
  final int coin;
  @override
  final int id;
  @override
  final String name;
  @override
  final String? seed;
  @override
  final String? passphrase;
  @override
  final int aindex;
  @override
  final int dindex;
  @override
  final Uint8List? icon;
  @override
  final int birth;
  @override
  final Folder folder;
  @override
  final int position;
  @override
  final bool hidden;
  @override
  final bool saved;
  @override
  final bool enabled;
  @override
  final bool internal;
  @override
  final int hw;
  @override
  final int height;
  @override
  final int time;
  @override
  final BigInt balance;

  /// Create a copy of Account
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  _$AccountCopyWith<_Account> get copyWith =>
      __$AccountCopyWithImpl<_Account>(this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _Account &&
            (identical(other.coin, coin) || other.coin == coin) &&
            (identical(other.id, id) || other.id == id) &&
            (identical(other.name, name) || other.name == name) &&
            (identical(other.seed, seed) || other.seed == seed) &&
            (identical(other.passphrase, passphrase) ||
                other.passphrase == passphrase) &&
            (identical(other.aindex, aindex) || other.aindex == aindex) &&
            (identical(other.dindex, dindex) || other.dindex == dindex) &&
            const DeepCollectionEquality().equals(other.icon, icon) &&
            (identical(other.birth, birth) || other.birth == birth) &&
            (identical(other.folder, folder) || other.folder == folder) &&
            (identical(other.position, position) ||
                other.position == position) &&
            (identical(other.hidden, hidden) || other.hidden == hidden) &&
            (identical(other.saved, saved) || other.saved == saved) &&
            (identical(other.enabled, enabled) || other.enabled == enabled) &&
            (identical(other.internal, internal) ||
                other.internal == internal) &&
            (identical(other.hw, hw) || other.hw == hw) &&
            (identical(other.height, height) || other.height == height) &&
            (identical(other.time, time) || other.time == time) &&
            (identical(other.balance, balance) || other.balance == balance));
  }

  @override
  int get hashCode => Object.hashAll([
        runtimeType,
        coin,
        id,
        name,
        seed,
        passphrase,
        aindex,
        dindex,
        const DeepCollectionEquality().hash(icon),
        birth,
        folder,
        position,
        hidden,
        saved,
        enabled,
        internal,
        hw,
        height,
        time,
        balance
      ]);

  @override
  String toString() {
    return 'Account(coin: $coin, id: $id, name: $name, seed: $seed, passphrase: $passphrase, aindex: $aindex, dindex: $dindex, icon: $icon, birth: $birth, folder: $folder, position: $position, hidden: $hidden, saved: $saved, enabled: $enabled, internal: $internal, hw: $hw, height: $height, time: $time, balance: $balance)';
  }
}

/// @nodoc
abstract mixin class _$AccountCopyWith<$Res> implements $AccountCopyWith<$Res> {
  factory _$AccountCopyWith(_Account value, $Res Function(_Account) _then) =
      __$AccountCopyWithImpl;
  @override
  @useResult
  $Res call(
      {int coin,
      int id,
      String name,
      String? seed,
      String? passphrase,
      int aindex,
      int dindex,
      Uint8List? icon,
      int birth,
      Folder folder,
      int position,
      bool hidden,
      bool saved,
      bool enabled,
      bool internal,
      int hw,
      int height,
      int time,
      BigInt balance});

  @override
  $FolderCopyWith<$Res> get folder;
}

/// @nodoc
class __$AccountCopyWithImpl<$Res> implements _$AccountCopyWith<$Res> {
  __$AccountCopyWithImpl(this._self, this._then);

  final _Account _self;
  final $Res Function(_Account) _then;

  /// Create a copy of Account
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $Res call({
    Object? coin = null,
    Object? id = null,
    Object? name = null,
    Object? seed = freezed,
    Object? passphrase = freezed,
    Object? aindex = null,
    Object? dindex = null,
    Object? icon = freezed,
    Object? birth = null,
    Object? folder = null,
    Object? position = null,
    Object? hidden = null,
    Object? saved = null,
    Object? enabled = null,
    Object? internal = null,
    Object? hw = null,
    Object? height = null,
    Object? time = null,
    Object? balance = null,
  }) {
    return _then(_Account(
      coin: null == coin
          ? _self.coin
          : coin // ignore: cast_nullable_to_non_nullable
              as int,
      id: null == id
          ? _self.id
          : id // ignore: cast_nullable_to_non_nullable
              as int,
      name: null == name
          ? _self.name
          : name // ignore: cast_nullable_to_non_nullable
              as String,
      seed: freezed == seed
          ? _self.seed
          : seed // ignore: cast_nullable_to_non_nullable
              as String?,
      passphrase: freezed == passphrase
          ? _self.passphrase
          : passphrase // ignore: cast_nullable_to_non_nullable
              as String?,
      aindex: null == aindex
          ? _self.aindex
          : aindex // ignore: cast_nullable_to_non_nullable
              as int,
      dindex: null == dindex
          ? _self.dindex
          : dindex // ignore: cast_nullable_to_non_nullable
              as int,
      icon: freezed == icon
          ? _self.icon
          : icon // ignore: cast_nullable_to_non_nullable
              as Uint8List?,
      birth: null == birth
          ? _self.birth
          : birth // ignore: cast_nullable_to_non_nullable
              as int,
      folder: null == folder
          ? _self.folder
          : folder // ignore: cast_nullable_to_non_nullable
              as Folder,
      position: null == position
          ? _self.position
          : position // ignore: cast_nullable_to_non_nullable
              as int,
      hidden: null == hidden
          ? _self.hidden
          : hidden // ignore: cast_nullable_to_non_nullable
              as bool,
      saved: null == saved
          ? _self.saved
          : saved // ignore: cast_nullable_to_non_nullable
              as bool,
      enabled: null == enabled
          ? _self.enabled
          : enabled // ignore: cast_nullable_to_non_nullable
              as bool,
      internal: null == internal
          ? _self.internal
          : internal // ignore: cast_nullable_to_non_nullable
              as bool,
      hw: null == hw
          ? _self.hw
          : hw // ignore: cast_nullable_to_non_nullable
              as int,
      height: null == height
          ? _self.height
          : height // ignore: cast_nullable_to_non_nullable
              as int,
      time: null == time
          ? _self.time
          : time // ignore: cast_nullable_to_non_nullable
              as int,
      balance: null == balance
          ? _self.balance
          : balance // ignore: cast_nullable_to_non_nullable
              as BigInt,
    ));
  }

  /// Create a copy of Account
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $FolderCopyWith<$Res> get folder {
    return $FolderCopyWith<$Res>(_self.folder, (value) {
      return _then(_self.copyWith(folder: value));
    });
  }
}

/// @nodoc
mixin _$AccountUpdate {
  int get coin;
  int get id;
  String? get name;
  Uint8List? get icon;
  int? get birth;
  int get folder;
  bool? get hidden;
  bool? get enabled;

  /// Create a copy of AccountUpdate
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $AccountUpdateCopyWith<AccountUpdate> get copyWith =>
      _$AccountUpdateCopyWithImpl<AccountUpdate>(
          this as AccountUpdate, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is AccountUpdate &&
            (identical(other.coin, coin) || other.coin == coin) &&
            (identical(other.id, id) || other.id == id) &&
            (identical(other.name, name) || other.name == name) &&
            const DeepCollectionEquality().equals(other.icon, icon) &&
            (identical(other.birth, birth) || other.birth == birth) &&
            (identical(other.folder, folder) || other.folder == folder) &&
            (identical(other.hidden, hidden) || other.hidden == hidden) &&
            (identical(other.enabled, enabled) || other.enabled == enabled));
  }

  @override
  int get hashCode => Object.hash(
      runtimeType,
      coin,
      id,
      name,
      const DeepCollectionEquality().hash(icon),
      birth,
      folder,
      hidden,
      enabled);

  @override
  String toString() {
    return 'AccountUpdate(coin: $coin, id: $id, name: $name, icon: $icon, birth: $birth, folder: $folder, hidden: $hidden, enabled: $enabled)';
  }
}

/// @nodoc
abstract mixin class $AccountUpdateCopyWith<$Res> {
  factory $AccountUpdateCopyWith(
          AccountUpdate value, $Res Function(AccountUpdate) _then) =
      _$AccountUpdateCopyWithImpl;
  @useResult
  $Res call(
      {int coin,
      int id,
      String? name,
      Uint8List? icon,
      int? birth,
      int folder,
      bool? hidden,
      bool? enabled});
}

/// @nodoc
class _$AccountUpdateCopyWithImpl<$Res>
    implements $AccountUpdateCopyWith<$Res> {
  _$AccountUpdateCopyWithImpl(this._self, this._then);

  final AccountUpdate _self;
  final $Res Function(AccountUpdate) _then;

  /// Create a copy of AccountUpdate
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? coin = null,
    Object? id = null,
    Object? name = freezed,
    Object? icon = freezed,
    Object? birth = freezed,
    Object? folder = null,
    Object? hidden = freezed,
    Object? enabled = freezed,
  }) {
    return _then(_self.copyWith(
      coin: null == coin
          ? _self.coin
          : coin // ignore: cast_nullable_to_non_nullable
              as int,
      id: null == id
          ? _self.id
          : id // ignore: cast_nullable_to_non_nullable
              as int,
      name: freezed == name
          ? _self.name
          : name // ignore: cast_nullable_to_non_nullable
              as String?,
      icon: freezed == icon
          ? _self.icon
          : icon // ignore: cast_nullable_to_non_nullable
              as Uint8List?,
      birth: freezed == birth
          ? _self.birth
          : birth // ignore: cast_nullable_to_non_nullable
              as int?,
      folder: null == folder
          ? _self.folder
          : folder // ignore: cast_nullable_to_non_nullable
              as int,
      hidden: freezed == hidden
          ? _self.hidden
          : hidden // ignore: cast_nullable_to_non_nullable
              as bool?,
      enabled: freezed == enabled
          ? _self.enabled
          : enabled // ignore: cast_nullable_to_non_nullable
              as bool?,
    ));
  }
}

/// Adds pattern-matching-related methods to [AccountUpdate].
extension AccountUpdatePatterns on AccountUpdate {
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
    TResult Function(_AccountUpdate value)? $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _AccountUpdate() when $default != null:
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
    TResult Function(_AccountUpdate value) $default,
  ) {
    final _that = this;
    switch (_that) {
      case _AccountUpdate():
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
    TResult? Function(_AccountUpdate value)? $default,
  ) {
    final _that = this;
    switch (_that) {
      case _AccountUpdate() when $default != null:
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
    TResult Function(int coin, int id, String? name, Uint8List? icon,
            int? birth, int folder, bool? hidden, bool? enabled)?
        $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _AccountUpdate() when $default != null:
        return $default(_that.coin, _that.id, _that.name, _that.icon,
            _that.birth, _that.folder, _that.hidden, _that.enabled);
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
    TResult Function(int coin, int id, String? name, Uint8List? icon,
            int? birth, int folder, bool? hidden, bool? enabled)
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _AccountUpdate():
        return $default(_that.coin, _that.id, _that.name, _that.icon,
            _that.birth, _that.folder, _that.hidden, _that.enabled);
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
    TResult? Function(int coin, int id, String? name, Uint8List? icon,
            int? birth, int folder, bool? hidden, bool? enabled)?
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _AccountUpdate() when $default != null:
        return $default(_that.coin, _that.id, _that.name, _that.icon,
            _that.birth, _that.folder, _that.hidden, _that.enabled);
      case _:
        return null;
    }
  }
}

/// @nodoc

class _AccountUpdate implements AccountUpdate {
  const _AccountUpdate(
      {required this.coin,
      required this.id,
      this.name,
      this.icon,
      this.birth,
      required this.folder,
      this.hidden,
      this.enabled});

  @override
  final int coin;
  @override
  final int id;
  @override
  final String? name;
  @override
  final Uint8List? icon;
  @override
  final int? birth;
  @override
  final int folder;
  @override
  final bool? hidden;
  @override
  final bool? enabled;

  /// Create a copy of AccountUpdate
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  _$AccountUpdateCopyWith<_AccountUpdate> get copyWith =>
      __$AccountUpdateCopyWithImpl<_AccountUpdate>(this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _AccountUpdate &&
            (identical(other.coin, coin) || other.coin == coin) &&
            (identical(other.id, id) || other.id == id) &&
            (identical(other.name, name) || other.name == name) &&
            const DeepCollectionEquality().equals(other.icon, icon) &&
            (identical(other.birth, birth) || other.birth == birth) &&
            (identical(other.folder, folder) || other.folder == folder) &&
            (identical(other.hidden, hidden) || other.hidden == hidden) &&
            (identical(other.enabled, enabled) || other.enabled == enabled));
  }

  @override
  int get hashCode => Object.hash(
      runtimeType,
      coin,
      id,
      name,
      const DeepCollectionEquality().hash(icon),
      birth,
      folder,
      hidden,
      enabled);

  @override
  String toString() {
    return 'AccountUpdate(coin: $coin, id: $id, name: $name, icon: $icon, birth: $birth, folder: $folder, hidden: $hidden, enabled: $enabled)';
  }
}

/// @nodoc
abstract mixin class _$AccountUpdateCopyWith<$Res>
    implements $AccountUpdateCopyWith<$Res> {
  factory _$AccountUpdateCopyWith(
          _AccountUpdate value, $Res Function(_AccountUpdate) _then) =
      __$AccountUpdateCopyWithImpl;
  @override
  @useResult
  $Res call(
      {int coin,
      int id,
      String? name,
      Uint8List? icon,
      int? birth,
      int folder,
      bool? hidden,
      bool? enabled});
}

/// @nodoc
class __$AccountUpdateCopyWithImpl<$Res>
    implements _$AccountUpdateCopyWith<$Res> {
  __$AccountUpdateCopyWithImpl(this._self, this._then);

  final _AccountUpdate _self;
  final $Res Function(_AccountUpdate) _then;

  /// Create a copy of AccountUpdate
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $Res call({
    Object? coin = null,
    Object? id = null,
    Object? name = freezed,
    Object? icon = freezed,
    Object? birth = freezed,
    Object? folder = null,
    Object? hidden = freezed,
    Object? enabled = freezed,
  }) {
    return _then(_AccountUpdate(
      coin: null == coin
          ? _self.coin
          : coin // ignore: cast_nullable_to_non_nullable
              as int,
      id: null == id
          ? _self.id
          : id // ignore: cast_nullable_to_non_nullable
              as int,
      name: freezed == name
          ? _self.name
          : name // ignore: cast_nullable_to_non_nullable
              as String?,
      icon: freezed == icon
          ? _self.icon
          : icon // ignore: cast_nullable_to_non_nullable
              as Uint8List?,
      birth: freezed == birth
          ? _self.birth
          : birth // ignore: cast_nullable_to_non_nullable
              as int?,
      folder: null == folder
          ? _self.folder
          : folder // ignore: cast_nullable_to_non_nullable
              as int,
      hidden: freezed == hidden
          ? _self.hidden
          : hidden // ignore: cast_nullable_to_non_nullable
              as bool?,
      enabled: freezed == enabled
          ? _self.enabled
          : enabled // ignore: cast_nullable_to_non_nullable
              as bool?,
    ));
  }
}

/// @nodoc
mixin _$Category {
  int get id;
  String get name;
  bool get isIncome;

  /// Create a copy of Category
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $CategoryCopyWith<Category> get copyWith =>
      _$CategoryCopyWithImpl<Category>(this as Category, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is Category &&
            (identical(other.id, id) || other.id == id) &&
            (identical(other.name, name) || other.name == name) &&
            (identical(other.isIncome, isIncome) ||
                other.isIncome == isIncome));
  }

  @override
  int get hashCode => Object.hash(runtimeType, id, name, isIncome);

  @override
  String toString() {
    return 'Category(id: $id, name: $name, isIncome: $isIncome)';
  }
}

/// @nodoc
abstract mixin class $CategoryCopyWith<$Res> {
  factory $CategoryCopyWith(Category value, $Res Function(Category) _then) =
      _$CategoryCopyWithImpl;
  @useResult
  $Res call({int id, String name, bool isIncome});
}

/// @nodoc
class _$CategoryCopyWithImpl<$Res> implements $CategoryCopyWith<$Res> {
  _$CategoryCopyWithImpl(this._self, this._then);

  final Category _self;
  final $Res Function(Category) _then;

  /// Create a copy of Category
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? id = null,
    Object? name = null,
    Object? isIncome = null,
  }) {
    return _then(_self.copyWith(
      id: null == id
          ? _self.id
          : id // ignore: cast_nullable_to_non_nullable
              as int,
      name: null == name
          ? _self.name
          : name // ignore: cast_nullable_to_non_nullable
              as String,
      isIncome: null == isIncome
          ? _self.isIncome
          : isIncome // ignore: cast_nullable_to_non_nullable
              as bool,
    ));
  }
}

/// Adds pattern-matching-related methods to [Category].
extension CategoryPatterns on Category {
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
    TResult Function(_Category value)? $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _Category() when $default != null:
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
    TResult Function(_Category value) $default,
  ) {
    final _that = this;
    switch (_that) {
      case _Category():
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
    TResult? Function(_Category value)? $default,
  ) {
    final _that = this;
    switch (_that) {
      case _Category() when $default != null:
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
    TResult Function(int id, String name, bool isIncome)? $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _Category() when $default != null:
        return $default(_that.id, _that.name, _that.isIncome);
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
    TResult Function(int id, String name, bool isIncome) $default,
  ) {
    final _that = this;
    switch (_that) {
      case _Category():
        return $default(_that.id, _that.name, _that.isIncome);
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
    TResult? Function(int id, String name, bool isIncome)? $default,
  ) {
    final _that = this;
    switch (_that) {
      case _Category() when $default != null:
        return $default(_that.id, _that.name, _that.isIncome);
      case _:
        return null;
    }
  }
}

/// @nodoc

class _Category implements Category {
  const _Category(
      {required this.id, required this.name, required this.isIncome});

  @override
  final int id;
  @override
  final String name;
  @override
  final bool isIncome;

  /// Create a copy of Category
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  _$CategoryCopyWith<_Category> get copyWith =>
      __$CategoryCopyWithImpl<_Category>(this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _Category &&
            (identical(other.id, id) || other.id == id) &&
            (identical(other.name, name) || other.name == name) &&
            (identical(other.isIncome, isIncome) ||
                other.isIncome == isIncome));
  }

  @override
  int get hashCode => Object.hash(runtimeType, id, name, isIncome);

  @override
  String toString() {
    return 'Category(id: $id, name: $name, isIncome: $isIncome)';
  }
}

/// @nodoc
abstract mixin class _$CategoryCopyWith<$Res>
    implements $CategoryCopyWith<$Res> {
  factory _$CategoryCopyWith(_Category value, $Res Function(_Category) _then) =
      __$CategoryCopyWithImpl;
  @override
  @useResult
  $Res call({int id, String name, bool isIncome});
}

/// @nodoc
class __$CategoryCopyWithImpl<$Res> implements _$CategoryCopyWith<$Res> {
  __$CategoryCopyWithImpl(this._self, this._then);

  final _Category _self;
  final $Res Function(_Category) _then;

  /// Create a copy of Category
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $Res call({
    Object? id = null,
    Object? name = null,
    Object? isIncome = null,
  }) {
    return _then(_Category(
      id: null == id
          ? _self.id
          : id // ignore: cast_nullable_to_non_nullable
              as int,
      name: null == name
          ? _self.name
          : name // ignore: cast_nullable_to_non_nullable
              as String,
      isIncome: null == isIncome
          ? _self.isIncome
          : isIncome // ignore: cast_nullable_to_non_nullable
              as bool,
    ));
  }
}

/// @nodoc
mixin _$Folder {
  int get id;
  String get name;

  /// Create a copy of Folder
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $FolderCopyWith<Folder> get copyWith =>
      _$FolderCopyWithImpl<Folder>(this as Folder, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is Folder &&
            (identical(other.id, id) || other.id == id) &&
            (identical(other.name, name) || other.name == name));
  }

  @override
  int get hashCode => Object.hash(runtimeType, id, name);

  @override
  String toString() {
    return 'Folder(id: $id, name: $name)';
  }
}

/// @nodoc
abstract mixin class $FolderCopyWith<$Res> {
  factory $FolderCopyWith(Folder value, $Res Function(Folder) _then) =
      _$FolderCopyWithImpl;
  @useResult
  $Res call({int id, String name});
}

/// @nodoc
class _$FolderCopyWithImpl<$Res> implements $FolderCopyWith<$Res> {
  _$FolderCopyWithImpl(this._self, this._then);

  final Folder _self;
  final $Res Function(Folder) _then;

  /// Create a copy of Folder
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? id = null,
    Object? name = null,
  }) {
    return _then(_self.copyWith(
      id: null == id
          ? _self.id
          : id // ignore: cast_nullable_to_non_nullable
              as int,
      name: null == name
          ? _self.name
          : name // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// Adds pattern-matching-related methods to [Folder].
extension FolderPatterns on Folder {
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
    TResult Function(_Folder value)? $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _Folder() when $default != null:
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
    TResult Function(_Folder value) $default,
  ) {
    final _that = this;
    switch (_that) {
      case _Folder():
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
    TResult? Function(_Folder value)? $default,
  ) {
    final _that = this;
    switch (_that) {
      case _Folder() when $default != null:
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
    TResult Function(int id, String name)? $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _Folder() when $default != null:
        return $default(_that.id, _that.name);
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
    TResult Function(int id, String name) $default,
  ) {
    final _that = this;
    switch (_that) {
      case _Folder():
        return $default(_that.id, _that.name);
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
    TResult? Function(int id, String name)? $default,
  ) {
    final _that = this;
    switch (_that) {
      case _Folder() when $default != null:
        return $default(_that.id, _that.name);
      case _:
        return null;
    }
  }
}

/// @nodoc

class _Folder implements Folder {
  const _Folder({required this.id, required this.name});

  @override
  final int id;
  @override
  final String name;

  /// Create a copy of Folder
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  _$FolderCopyWith<_Folder> get copyWith =>
      __$FolderCopyWithImpl<_Folder>(this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _Folder &&
            (identical(other.id, id) || other.id == id) &&
            (identical(other.name, name) || other.name == name));
  }

  @override
  int get hashCode => Object.hash(runtimeType, id, name);

  @override
  String toString() {
    return 'Folder(id: $id, name: $name)';
  }
}

/// @nodoc
abstract mixin class _$FolderCopyWith<$Res> implements $FolderCopyWith<$Res> {
  factory _$FolderCopyWith(_Folder value, $Res Function(_Folder) _then) =
      __$FolderCopyWithImpl;
  @override
  @useResult
  $Res call({int id, String name});
}

/// @nodoc
class __$FolderCopyWithImpl<$Res> implements _$FolderCopyWith<$Res> {
  __$FolderCopyWithImpl(this._self, this._then);

  final _Folder _self;
  final $Res Function(_Folder) _then;

  /// Create a copy of Folder
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $Res call({
    Object? id = null,
    Object? name = null,
  }) {
    return _then(_Folder(
      id: null == id
          ? _self.id
          : id // ignore: cast_nullable_to_non_nullable
              as int,
      name: null == name
          ? _self.name
          : name // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// @nodoc
mixin _$FrostParams {
  int get id;
  int get n;
  int get t;

  /// Create a copy of FrostParams
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $FrostParamsCopyWith<FrostParams> get copyWith =>
      _$FrostParamsCopyWithImpl<FrostParams>(this as FrostParams, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is FrostParams &&
            (identical(other.id, id) || other.id == id) &&
            (identical(other.n, n) || other.n == n) &&
            (identical(other.t, t) || other.t == t));
  }

  @override
  int get hashCode => Object.hash(runtimeType, id, n, t);

  @override
  String toString() {
    return 'FrostParams(id: $id, n: $n, t: $t)';
  }
}

/// @nodoc
abstract mixin class $FrostParamsCopyWith<$Res> {
  factory $FrostParamsCopyWith(
          FrostParams value, $Res Function(FrostParams) _then) =
      _$FrostParamsCopyWithImpl;
  @useResult
  $Res call({int id, int n, int t});
}

/// @nodoc
class _$FrostParamsCopyWithImpl<$Res> implements $FrostParamsCopyWith<$Res> {
  _$FrostParamsCopyWithImpl(this._self, this._then);

  final FrostParams _self;
  final $Res Function(FrostParams) _then;

  /// Create a copy of FrostParams
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? id = null,
    Object? n = null,
    Object? t = null,
  }) {
    return _then(_self.copyWith(
      id: null == id
          ? _self.id
          : id // ignore: cast_nullable_to_non_nullable
              as int,
      n: null == n
          ? _self.n
          : n // ignore: cast_nullable_to_non_nullable
              as int,
      t: null == t
          ? _self.t
          : t // ignore: cast_nullable_to_non_nullable
              as int,
    ));
  }
}

/// Adds pattern-matching-related methods to [FrostParams].
extension FrostParamsPatterns on FrostParams {
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
    TResult Function(_FrostParams value)? $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _FrostParams() when $default != null:
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
    TResult Function(_FrostParams value) $default,
  ) {
    final _that = this;
    switch (_that) {
      case _FrostParams():
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
    TResult? Function(_FrostParams value)? $default,
  ) {
    final _that = this;
    switch (_that) {
      case _FrostParams() when $default != null:
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
    TResult Function(int id, int n, int t)? $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _FrostParams() when $default != null:
        return $default(_that.id, _that.n, _that.t);
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
    TResult Function(int id, int n, int t) $default,
  ) {
    final _that = this;
    switch (_that) {
      case _FrostParams():
        return $default(_that.id, _that.n, _that.t);
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
    TResult? Function(int id, int n, int t)? $default,
  ) {
    final _that = this;
    switch (_that) {
      case _FrostParams() when $default != null:
        return $default(_that.id, _that.n, _that.t);
      case _:
        return null;
    }
  }
}

/// @nodoc

class _FrostParams implements FrostParams {
  const _FrostParams({required this.id, required this.n, required this.t});

  @override
  final int id;
  @override
  final int n;
  @override
  final int t;

  /// Create a copy of FrostParams
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  _$FrostParamsCopyWith<_FrostParams> get copyWith =>
      __$FrostParamsCopyWithImpl<_FrostParams>(this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _FrostParams &&
            (identical(other.id, id) || other.id == id) &&
            (identical(other.n, n) || other.n == n) &&
            (identical(other.t, t) || other.t == t));
  }

  @override
  int get hashCode => Object.hash(runtimeType, id, n, t);

  @override
  String toString() {
    return 'FrostParams(id: $id, n: $n, t: $t)';
  }
}

/// @nodoc
abstract mixin class _$FrostParamsCopyWith<$Res>
    implements $FrostParamsCopyWith<$Res> {
  factory _$FrostParamsCopyWith(
          _FrostParams value, $Res Function(_FrostParams) _then) =
      __$FrostParamsCopyWithImpl;
  @override
  @useResult
  $Res call({int id, int n, int t});
}

/// @nodoc
class __$FrostParamsCopyWithImpl<$Res> implements _$FrostParamsCopyWith<$Res> {
  __$FrostParamsCopyWithImpl(this._self, this._then);

  final _FrostParams _self;
  final $Res Function(_FrostParams) _then;

  /// Create a copy of FrostParams
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $Res call({
    Object? id = null,
    Object? n = null,
    Object? t = null,
  }) {
    return _then(_FrostParams(
      id: null == id
          ? _self.id
          : id // ignore: cast_nullable_to_non_nullable
              as int,
      n: null == n
          ? _self.n
          : n // ignore: cast_nullable_to_non_nullable
              as int,
      t: null == t
          ? _self.t
          : t // ignore: cast_nullable_to_non_nullable
              as int,
    ));
  }
}

/// @nodoc
mixin _$Memo {
  int get id;
  int get idTx;
  int? get idNote;
  int get pool;
  int get height;
  int get vout;
  int get time;
  Uint8List get memoBytes;
  String? get memo;

  /// Create a copy of Memo
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $MemoCopyWith<Memo> get copyWith =>
      _$MemoCopyWithImpl<Memo>(this as Memo, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is Memo &&
            (identical(other.id, id) || other.id == id) &&
            (identical(other.idTx, idTx) || other.idTx == idTx) &&
            (identical(other.idNote, idNote) || other.idNote == idNote) &&
            (identical(other.pool, pool) || other.pool == pool) &&
            (identical(other.height, height) || other.height == height) &&
            (identical(other.vout, vout) || other.vout == vout) &&
            (identical(other.time, time) || other.time == time) &&
            const DeepCollectionEquality().equals(other.memoBytes, memoBytes) &&
            (identical(other.memo, memo) || other.memo == memo));
  }

  @override
  int get hashCode => Object.hash(runtimeType, id, idTx, idNote, pool, height,
      vout, time, const DeepCollectionEquality().hash(memoBytes), memo);

  @override
  String toString() {
    return 'Memo(id: $id, idTx: $idTx, idNote: $idNote, pool: $pool, height: $height, vout: $vout, time: $time, memoBytes: $memoBytes, memo: $memo)';
  }
}

/// @nodoc
abstract mixin class $MemoCopyWith<$Res> {
  factory $MemoCopyWith(Memo value, $Res Function(Memo) _then) =
      _$MemoCopyWithImpl;
  @useResult
  $Res call(
      {int id,
      int idTx,
      int? idNote,
      int pool,
      int height,
      int vout,
      int time,
      Uint8List memoBytes,
      String? memo});
}

/// @nodoc
class _$MemoCopyWithImpl<$Res> implements $MemoCopyWith<$Res> {
  _$MemoCopyWithImpl(this._self, this._then);

  final Memo _self;
  final $Res Function(Memo) _then;

  /// Create a copy of Memo
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? id = null,
    Object? idTx = null,
    Object? idNote = freezed,
    Object? pool = null,
    Object? height = null,
    Object? vout = null,
    Object? time = null,
    Object? memoBytes = null,
    Object? memo = freezed,
  }) {
    return _then(_self.copyWith(
      id: null == id
          ? _self.id
          : id // ignore: cast_nullable_to_non_nullable
              as int,
      idTx: null == idTx
          ? _self.idTx
          : idTx // ignore: cast_nullable_to_non_nullable
              as int,
      idNote: freezed == idNote
          ? _self.idNote
          : idNote // ignore: cast_nullable_to_non_nullable
              as int?,
      pool: null == pool
          ? _self.pool
          : pool // ignore: cast_nullable_to_non_nullable
              as int,
      height: null == height
          ? _self.height
          : height // ignore: cast_nullable_to_non_nullable
              as int,
      vout: null == vout
          ? _self.vout
          : vout // ignore: cast_nullable_to_non_nullable
              as int,
      time: null == time
          ? _self.time
          : time // ignore: cast_nullable_to_non_nullable
              as int,
      memoBytes: null == memoBytes
          ? _self.memoBytes
          : memoBytes // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      memo: freezed == memo
          ? _self.memo
          : memo // ignore: cast_nullable_to_non_nullable
              as String?,
    ));
  }
}

/// Adds pattern-matching-related methods to [Memo].
extension MemoPatterns on Memo {
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
    TResult Function(_Memo value)? $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _Memo() when $default != null:
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
    TResult Function(_Memo value) $default,
  ) {
    final _that = this;
    switch (_that) {
      case _Memo():
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
    TResult? Function(_Memo value)? $default,
  ) {
    final _that = this;
    switch (_that) {
      case _Memo() when $default != null:
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
    TResult Function(int id, int idTx, int? idNote, int pool, int height,
            int vout, int time, Uint8List memoBytes, String? memo)?
        $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _Memo() when $default != null:
        return $default(_that.id, _that.idTx, _that.idNote, _that.pool,
            _that.height, _that.vout, _that.time, _that.memoBytes, _that.memo);
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
    TResult Function(int id, int idTx, int? idNote, int pool, int height,
            int vout, int time, Uint8List memoBytes, String? memo)
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _Memo():
        return $default(_that.id, _that.idTx, _that.idNote, _that.pool,
            _that.height, _that.vout, _that.time, _that.memoBytes, _that.memo);
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
    TResult? Function(int id, int idTx, int? idNote, int pool, int height,
            int vout, int time, Uint8List memoBytes, String? memo)?
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _Memo() when $default != null:
        return $default(_that.id, _that.idTx, _that.idNote, _that.pool,
            _that.height, _that.vout, _that.time, _that.memoBytes, _that.memo);
      case _:
        return null;
    }
  }
}

/// @nodoc

class _Memo implements Memo {
  const _Memo(
      {required this.id,
      required this.idTx,
      this.idNote,
      required this.pool,
      required this.height,
      required this.vout,
      required this.time,
      required this.memoBytes,
      this.memo});

  @override
  final int id;
  @override
  final int idTx;
  @override
  final int? idNote;
  @override
  final int pool;
  @override
  final int height;
  @override
  final int vout;
  @override
  final int time;
  @override
  final Uint8List memoBytes;
  @override
  final String? memo;

  /// Create a copy of Memo
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  _$MemoCopyWith<_Memo> get copyWith =>
      __$MemoCopyWithImpl<_Memo>(this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _Memo &&
            (identical(other.id, id) || other.id == id) &&
            (identical(other.idTx, idTx) || other.idTx == idTx) &&
            (identical(other.idNote, idNote) || other.idNote == idNote) &&
            (identical(other.pool, pool) || other.pool == pool) &&
            (identical(other.height, height) || other.height == height) &&
            (identical(other.vout, vout) || other.vout == vout) &&
            (identical(other.time, time) || other.time == time) &&
            const DeepCollectionEquality().equals(other.memoBytes, memoBytes) &&
            (identical(other.memo, memo) || other.memo == memo));
  }

  @override
  int get hashCode => Object.hash(runtimeType, id, idTx, idNote, pool, height,
      vout, time, const DeepCollectionEquality().hash(memoBytes), memo);

  @override
  String toString() {
    return 'Memo(id: $id, idTx: $idTx, idNote: $idNote, pool: $pool, height: $height, vout: $vout, time: $time, memoBytes: $memoBytes, memo: $memo)';
  }
}

/// @nodoc
abstract mixin class _$MemoCopyWith<$Res> implements $MemoCopyWith<$Res> {
  factory _$MemoCopyWith(_Memo value, $Res Function(_Memo) _then) =
      __$MemoCopyWithImpl;
  @override
  @useResult
  $Res call(
      {int id,
      int idTx,
      int? idNote,
      int pool,
      int height,
      int vout,
      int time,
      Uint8List memoBytes,
      String? memo});
}

/// @nodoc
class __$MemoCopyWithImpl<$Res> implements _$MemoCopyWith<$Res> {
  __$MemoCopyWithImpl(this._self, this._then);

  final _Memo _self;
  final $Res Function(_Memo) _then;

  /// Create a copy of Memo
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $Res call({
    Object? id = null,
    Object? idTx = null,
    Object? idNote = freezed,
    Object? pool = null,
    Object? height = null,
    Object? vout = null,
    Object? time = null,
    Object? memoBytes = null,
    Object? memo = freezed,
  }) {
    return _then(_Memo(
      id: null == id
          ? _self.id
          : id // ignore: cast_nullable_to_non_nullable
              as int,
      idTx: null == idTx
          ? _self.idTx
          : idTx // ignore: cast_nullable_to_non_nullable
              as int,
      idNote: freezed == idNote
          ? _self.idNote
          : idNote // ignore: cast_nullable_to_non_nullable
              as int?,
      pool: null == pool
          ? _self.pool
          : pool // ignore: cast_nullable_to_non_nullable
              as int,
      height: null == height
          ? _self.height
          : height // ignore: cast_nullable_to_non_nullable
              as int,
      vout: null == vout
          ? _self.vout
          : vout // ignore: cast_nullable_to_non_nullable
              as int,
      time: null == time
          ? _self.time
          : time // ignore: cast_nullable_to_non_nullable
              as int,
      memoBytes: null == memoBytes
          ? _self.memoBytes
          : memoBytes // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      memo: freezed == memo
          ? _self.memo
          : memo // ignore: cast_nullable_to_non_nullable
              as String?,
    ));
  }
}

/// @nodoc
mixin _$NewAccount {
  Uint8List? get icon;
  String get name;
  bool get restore;
  String get key;
  String? get passphrase;
  Uint8List? get fingerprint;
  int get aindex;
  int? get birth;
  String get folder;
  int? get pools;
  bool get useInternal;
  bool get internal;
  bool get ledger;

  /// Create a copy of NewAccount
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $NewAccountCopyWith<NewAccount> get copyWith =>
      _$NewAccountCopyWithImpl<NewAccount>(this as NewAccount, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is NewAccount &&
            const DeepCollectionEquality().equals(other.icon, icon) &&
            (identical(other.name, name) || other.name == name) &&
            (identical(other.restore, restore) || other.restore == restore) &&
            (identical(other.key, key) || other.key == key) &&
            (identical(other.passphrase, passphrase) ||
                other.passphrase == passphrase) &&
            const DeepCollectionEquality()
                .equals(other.fingerprint, fingerprint) &&
            (identical(other.aindex, aindex) || other.aindex == aindex) &&
            (identical(other.birth, birth) || other.birth == birth) &&
            (identical(other.folder, folder) || other.folder == folder) &&
            (identical(other.pools, pools) || other.pools == pools) &&
            (identical(other.useInternal, useInternal) ||
                other.useInternal == useInternal) &&
            (identical(other.internal, internal) ||
                other.internal == internal) &&
            (identical(other.ledger, ledger) || other.ledger == ledger));
  }

  @override
  int get hashCode => Object.hash(
      runtimeType,
      const DeepCollectionEquality().hash(icon),
      name,
      restore,
      key,
      passphrase,
      const DeepCollectionEquality().hash(fingerprint),
      aindex,
      birth,
      folder,
      pools,
      useInternal,
      internal,
      ledger);

  @override
  String toString() {
    return 'NewAccount(icon: $icon, name: $name, restore: $restore, key: $key, passphrase: $passphrase, fingerprint: $fingerprint, aindex: $aindex, birth: $birth, folder: $folder, pools: $pools, useInternal: $useInternal, internal: $internal, ledger: $ledger)';
  }
}

/// @nodoc
abstract mixin class $NewAccountCopyWith<$Res> {
  factory $NewAccountCopyWith(
          NewAccount value, $Res Function(NewAccount) _then) =
      _$NewAccountCopyWithImpl;
  @useResult
  $Res call(
      {Uint8List? icon,
      String name,
      bool restore,
      String key,
      String? passphrase,
      Uint8List? fingerprint,
      int aindex,
      int? birth,
      String folder,
      int? pools,
      bool useInternal,
      bool internal,
      bool ledger});
}

/// @nodoc
class _$NewAccountCopyWithImpl<$Res> implements $NewAccountCopyWith<$Res> {
  _$NewAccountCopyWithImpl(this._self, this._then);

  final NewAccount _self;
  final $Res Function(NewAccount) _then;

  /// Create a copy of NewAccount
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? icon = freezed,
    Object? name = null,
    Object? restore = null,
    Object? key = null,
    Object? passphrase = freezed,
    Object? fingerprint = freezed,
    Object? aindex = null,
    Object? birth = freezed,
    Object? folder = null,
    Object? pools = freezed,
    Object? useInternal = null,
    Object? internal = null,
    Object? ledger = null,
  }) {
    return _then(_self.copyWith(
      icon: freezed == icon
          ? _self.icon
          : icon // ignore: cast_nullable_to_non_nullable
              as Uint8List?,
      name: null == name
          ? _self.name
          : name // ignore: cast_nullable_to_non_nullable
              as String,
      restore: null == restore
          ? _self.restore
          : restore // ignore: cast_nullable_to_non_nullable
              as bool,
      key: null == key
          ? _self.key
          : key // ignore: cast_nullable_to_non_nullable
              as String,
      passphrase: freezed == passphrase
          ? _self.passphrase
          : passphrase // ignore: cast_nullable_to_non_nullable
              as String?,
      fingerprint: freezed == fingerprint
          ? _self.fingerprint
          : fingerprint // ignore: cast_nullable_to_non_nullable
              as Uint8List?,
      aindex: null == aindex
          ? _self.aindex
          : aindex // ignore: cast_nullable_to_non_nullable
              as int,
      birth: freezed == birth
          ? _self.birth
          : birth // ignore: cast_nullable_to_non_nullable
              as int?,
      folder: null == folder
          ? _self.folder
          : folder // ignore: cast_nullable_to_non_nullable
              as String,
      pools: freezed == pools
          ? _self.pools
          : pools // ignore: cast_nullable_to_non_nullable
              as int?,
      useInternal: null == useInternal
          ? _self.useInternal
          : useInternal // ignore: cast_nullable_to_non_nullable
              as bool,
      internal: null == internal
          ? _self.internal
          : internal // ignore: cast_nullable_to_non_nullable
              as bool,
      ledger: null == ledger
          ? _self.ledger
          : ledger // ignore: cast_nullable_to_non_nullable
              as bool,
    ));
  }
}

/// Adds pattern-matching-related methods to [NewAccount].
extension NewAccountPatterns on NewAccount {
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
    TResult Function(_NewAccount value)? $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _NewAccount() when $default != null:
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
    TResult Function(_NewAccount value) $default,
  ) {
    final _that = this;
    switch (_that) {
      case _NewAccount():
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
    TResult? Function(_NewAccount value)? $default,
  ) {
    final _that = this;
    switch (_that) {
      case _NewAccount() when $default != null:
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
            Uint8List? icon,
            String name,
            bool restore,
            String key,
            String? passphrase,
            Uint8List? fingerprint,
            int aindex,
            int? birth,
            String folder,
            int? pools,
            bool useInternal,
            bool internal,
            bool ledger)?
        $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _NewAccount() when $default != null:
        return $default(
            _that.icon,
            _that.name,
            _that.restore,
            _that.key,
            _that.passphrase,
            _that.fingerprint,
            _that.aindex,
            _that.birth,
            _that.folder,
            _that.pools,
            _that.useInternal,
            _that.internal,
            _that.ledger);
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
            Uint8List? icon,
            String name,
            bool restore,
            String key,
            String? passphrase,
            Uint8List? fingerprint,
            int aindex,
            int? birth,
            String folder,
            int? pools,
            bool useInternal,
            bool internal,
            bool ledger)
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _NewAccount():
        return $default(
            _that.icon,
            _that.name,
            _that.restore,
            _that.key,
            _that.passphrase,
            _that.fingerprint,
            _that.aindex,
            _that.birth,
            _that.folder,
            _that.pools,
            _that.useInternal,
            _that.internal,
            _that.ledger);
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
            Uint8List? icon,
            String name,
            bool restore,
            String key,
            String? passphrase,
            Uint8List? fingerprint,
            int aindex,
            int? birth,
            String folder,
            int? pools,
            bool useInternal,
            bool internal,
            bool ledger)?
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _NewAccount() when $default != null:
        return $default(
            _that.icon,
            _that.name,
            _that.restore,
            _that.key,
            _that.passphrase,
            _that.fingerprint,
            _that.aindex,
            _that.birth,
            _that.folder,
            _that.pools,
            _that.useInternal,
            _that.internal,
            _that.ledger);
      case _:
        return null;
    }
  }
}

/// @nodoc

class _NewAccount implements NewAccount {
  const _NewAccount(
      {this.icon,
      required this.name,
      required this.restore,
      required this.key,
      this.passphrase,
      this.fingerprint,
      required this.aindex,
      this.birth,
      required this.folder,
      this.pools,
      required this.useInternal,
      required this.internal,
      required this.ledger});

  @override
  final Uint8List? icon;
  @override
  final String name;
  @override
  final bool restore;
  @override
  final String key;
  @override
  final String? passphrase;
  @override
  final Uint8List? fingerprint;
  @override
  final int aindex;
  @override
  final int? birth;
  @override
  final String folder;
  @override
  final int? pools;
  @override
  final bool useInternal;
  @override
  final bool internal;
  @override
  final bool ledger;

  /// Create a copy of NewAccount
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  _$NewAccountCopyWith<_NewAccount> get copyWith =>
      __$NewAccountCopyWithImpl<_NewAccount>(this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _NewAccount &&
            const DeepCollectionEquality().equals(other.icon, icon) &&
            (identical(other.name, name) || other.name == name) &&
            (identical(other.restore, restore) || other.restore == restore) &&
            (identical(other.key, key) || other.key == key) &&
            (identical(other.passphrase, passphrase) ||
                other.passphrase == passphrase) &&
            const DeepCollectionEquality()
                .equals(other.fingerprint, fingerprint) &&
            (identical(other.aindex, aindex) || other.aindex == aindex) &&
            (identical(other.birth, birth) || other.birth == birth) &&
            (identical(other.folder, folder) || other.folder == folder) &&
            (identical(other.pools, pools) || other.pools == pools) &&
            (identical(other.useInternal, useInternal) ||
                other.useInternal == useInternal) &&
            (identical(other.internal, internal) ||
                other.internal == internal) &&
            (identical(other.ledger, ledger) || other.ledger == ledger));
  }

  @override
  int get hashCode => Object.hash(
      runtimeType,
      const DeepCollectionEquality().hash(icon),
      name,
      restore,
      key,
      passphrase,
      const DeepCollectionEquality().hash(fingerprint),
      aindex,
      birth,
      folder,
      pools,
      useInternal,
      internal,
      ledger);

  @override
  String toString() {
    return 'NewAccount(icon: $icon, name: $name, restore: $restore, key: $key, passphrase: $passphrase, fingerprint: $fingerprint, aindex: $aindex, birth: $birth, folder: $folder, pools: $pools, useInternal: $useInternal, internal: $internal, ledger: $ledger)';
  }
}

/// @nodoc
abstract mixin class _$NewAccountCopyWith<$Res>
    implements $NewAccountCopyWith<$Res> {
  factory _$NewAccountCopyWith(
          _NewAccount value, $Res Function(_NewAccount) _then) =
      __$NewAccountCopyWithImpl;
  @override
  @useResult
  $Res call(
      {Uint8List? icon,
      String name,
      bool restore,
      String key,
      String? passphrase,
      Uint8List? fingerprint,
      int aindex,
      int? birth,
      String folder,
      int? pools,
      bool useInternal,
      bool internal,
      bool ledger});
}

/// @nodoc
class __$NewAccountCopyWithImpl<$Res> implements _$NewAccountCopyWith<$Res> {
  __$NewAccountCopyWithImpl(this._self, this._then);

  final _NewAccount _self;
  final $Res Function(_NewAccount) _then;

  /// Create a copy of NewAccount
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $Res call({
    Object? icon = freezed,
    Object? name = null,
    Object? restore = null,
    Object? key = null,
    Object? passphrase = freezed,
    Object? fingerprint = freezed,
    Object? aindex = null,
    Object? birth = freezed,
    Object? folder = null,
    Object? pools = freezed,
    Object? useInternal = null,
    Object? internal = null,
    Object? ledger = null,
  }) {
    return _then(_NewAccount(
      icon: freezed == icon
          ? _self.icon
          : icon // ignore: cast_nullable_to_non_nullable
              as Uint8List?,
      name: null == name
          ? _self.name
          : name // ignore: cast_nullable_to_non_nullable
              as String,
      restore: null == restore
          ? _self.restore
          : restore // ignore: cast_nullable_to_non_nullable
              as bool,
      key: null == key
          ? _self.key
          : key // ignore: cast_nullable_to_non_nullable
              as String,
      passphrase: freezed == passphrase
          ? _self.passphrase
          : passphrase // ignore: cast_nullable_to_non_nullable
              as String?,
      fingerprint: freezed == fingerprint
          ? _self.fingerprint
          : fingerprint // ignore: cast_nullable_to_non_nullable
              as Uint8List?,
      aindex: null == aindex
          ? _self.aindex
          : aindex // ignore: cast_nullable_to_non_nullable
              as int,
      birth: freezed == birth
          ? _self.birth
          : birth // ignore: cast_nullable_to_non_nullable
              as int?,
      folder: null == folder
          ? _self.folder
          : folder // ignore: cast_nullable_to_non_nullable
              as String,
      pools: freezed == pools
          ? _self.pools
          : pools // ignore: cast_nullable_to_non_nullable
              as int?,
      useInternal: null == useInternal
          ? _self.useInternal
          : useInternal // ignore: cast_nullable_to_non_nullable
              as bool,
      internal: null == internal
          ? _self.internal
          : internal // ignore: cast_nullable_to_non_nullable
              as bool,
      ledger: null == ledger
          ? _self.ledger
          : ledger // ignore: cast_nullable_to_non_nullable
              as bool,
    ));
  }
}

/// @nodoc
mixin _$Seed {
  String get mnemonic;
  String get phrase;
  int get aindex;

  /// Create a copy of Seed
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $SeedCopyWith<Seed> get copyWith =>
      _$SeedCopyWithImpl<Seed>(this as Seed, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is Seed &&
            (identical(other.mnemonic, mnemonic) ||
                other.mnemonic == mnemonic) &&
            (identical(other.phrase, phrase) || other.phrase == phrase) &&
            (identical(other.aindex, aindex) || other.aindex == aindex));
  }

  @override
  int get hashCode => Object.hash(runtimeType, mnemonic, phrase, aindex);

  @override
  String toString() {
    return 'Seed(mnemonic: $mnemonic, phrase: $phrase, aindex: $aindex)';
  }
}

/// @nodoc
abstract mixin class $SeedCopyWith<$Res> {
  factory $SeedCopyWith(Seed value, $Res Function(Seed) _then) =
      _$SeedCopyWithImpl;
  @useResult
  $Res call({String mnemonic, String phrase, int aindex});
}

/// @nodoc
class _$SeedCopyWithImpl<$Res> implements $SeedCopyWith<$Res> {
  _$SeedCopyWithImpl(this._self, this._then);

  final Seed _self;
  final $Res Function(Seed) _then;

  /// Create a copy of Seed
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? mnemonic = null,
    Object? phrase = null,
    Object? aindex = null,
  }) {
    return _then(_self.copyWith(
      mnemonic: null == mnemonic
          ? _self.mnemonic
          : mnemonic // ignore: cast_nullable_to_non_nullable
              as String,
      phrase: null == phrase
          ? _self.phrase
          : phrase // ignore: cast_nullable_to_non_nullable
              as String,
      aindex: null == aindex
          ? _self.aindex
          : aindex // ignore: cast_nullable_to_non_nullable
              as int,
    ));
  }
}

/// Adds pattern-matching-related methods to [Seed].
extension SeedPatterns on Seed {
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
    TResult Function(_Seed value)? $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _Seed() when $default != null:
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
    TResult Function(_Seed value) $default,
  ) {
    final _that = this;
    switch (_that) {
      case _Seed():
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
    TResult? Function(_Seed value)? $default,
  ) {
    final _that = this;
    switch (_that) {
      case _Seed() when $default != null:
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
    TResult Function(String mnemonic, String phrase, int aindex)? $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _Seed() when $default != null:
        return $default(_that.mnemonic, _that.phrase, _that.aindex);
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
    TResult Function(String mnemonic, String phrase, int aindex) $default,
  ) {
    final _that = this;
    switch (_that) {
      case _Seed():
        return $default(_that.mnemonic, _that.phrase, _that.aindex);
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
    TResult? Function(String mnemonic, String phrase, int aindex)? $default,
  ) {
    final _that = this;
    switch (_that) {
      case _Seed() when $default != null:
        return $default(_that.mnemonic, _that.phrase, _that.aindex);
      case _:
        return null;
    }
  }
}

/// @nodoc

class _Seed implements Seed {
  const _Seed(
      {required this.mnemonic, required this.phrase, required this.aindex});

  @override
  final String mnemonic;
  @override
  final String phrase;
  @override
  final int aindex;

  /// Create a copy of Seed
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  _$SeedCopyWith<_Seed> get copyWith =>
      __$SeedCopyWithImpl<_Seed>(this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _Seed &&
            (identical(other.mnemonic, mnemonic) ||
                other.mnemonic == mnemonic) &&
            (identical(other.phrase, phrase) || other.phrase == phrase) &&
            (identical(other.aindex, aindex) || other.aindex == aindex));
  }

  @override
  int get hashCode => Object.hash(runtimeType, mnemonic, phrase, aindex);

  @override
  String toString() {
    return 'Seed(mnemonic: $mnemonic, phrase: $phrase, aindex: $aindex)';
  }
}

/// @nodoc
abstract mixin class _$SeedCopyWith<$Res> implements $SeedCopyWith<$Res> {
  factory _$SeedCopyWith(_Seed value, $Res Function(_Seed) _then) =
      __$SeedCopyWithImpl;
  @override
  @useResult
  $Res call({String mnemonic, String phrase, int aindex});
}

/// @nodoc
class __$SeedCopyWithImpl<$Res> implements _$SeedCopyWith<$Res> {
  __$SeedCopyWithImpl(this._self, this._then);

  final _Seed _self;
  final $Res Function(_Seed) _then;

  /// Create a copy of Seed
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $Res call({
    Object? mnemonic = null,
    Object? phrase = null,
    Object? aindex = null,
  }) {
    return _then(_Seed(
      mnemonic: null == mnemonic
          ? _self.mnemonic
          : mnemonic // ignore: cast_nullable_to_non_nullable
              as String,
      phrase: null == phrase
          ? _self.phrase
          : phrase // ignore: cast_nullable_to_non_nullable
              as String,
      aindex: null == aindex
          ? _self.aindex
          : aindex // ignore: cast_nullable_to_non_nullable
              as int,
    ));
  }
}

/// @nodoc
mixin _$Tx {
  int get id;
  Uint8List get txid;
  int get height;
  int get time;
  PlatformInt64 get value;
  int? get tpe;
  String? get category;

  /// Create a copy of Tx
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $TxCopyWith<Tx> get copyWith => _$TxCopyWithImpl<Tx>(this as Tx, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is Tx &&
            (identical(other.id, id) || other.id == id) &&
            const DeepCollectionEquality().equals(other.txid, txid) &&
            (identical(other.height, height) || other.height == height) &&
            (identical(other.time, time) || other.time == time) &&
            (identical(other.value, value) || other.value == value) &&
            (identical(other.tpe, tpe) || other.tpe == tpe) &&
            (identical(other.category, category) ||
                other.category == category));
  }

  @override
  int get hashCode => Object.hash(
      runtimeType,
      id,
      const DeepCollectionEquality().hash(txid),
      height,
      time,
      value,
      tpe,
      category);

  @override
  String toString() {
    return 'Tx(id: $id, txid: $txid, height: $height, time: $time, value: $value, tpe: $tpe, category: $category)';
  }
}

/// @nodoc
abstract mixin class $TxCopyWith<$Res> {
  factory $TxCopyWith(Tx value, $Res Function(Tx) _then) = _$TxCopyWithImpl;
  @useResult
  $Res call(
      {int id,
      Uint8List txid,
      int height,
      int time,
      PlatformInt64 value,
      int? tpe,
      String? category});
}

/// @nodoc
class _$TxCopyWithImpl<$Res> implements $TxCopyWith<$Res> {
  _$TxCopyWithImpl(this._self, this._then);

  final Tx _self;
  final $Res Function(Tx) _then;

  /// Create a copy of Tx
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? id = null,
    Object? txid = null,
    Object? height = null,
    Object? time = null,
    Object? value = null,
    Object? tpe = freezed,
    Object? category = freezed,
  }) {
    return _then(_self.copyWith(
      id: null == id
          ? _self.id
          : id // ignore: cast_nullable_to_non_nullable
              as int,
      txid: null == txid
          ? _self.txid
          : txid // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      height: null == height
          ? _self.height
          : height // ignore: cast_nullable_to_non_nullable
              as int,
      time: null == time
          ? _self.time
          : time // ignore: cast_nullable_to_non_nullable
              as int,
      value: null == value
          ? _self.value
          : value // ignore: cast_nullable_to_non_nullable
              as PlatformInt64,
      tpe: freezed == tpe
          ? _self.tpe
          : tpe // ignore: cast_nullable_to_non_nullable
              as int?,
      category: freezed == category
          ? _self.category
          : category // ignore: cast_nullable_to_non_nullable
              as String?,
    ));
  }
}

/// Adds pattern-matching-related methods to [Tx].
extension TxPatterns on Tx {
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
    TResult Function(_Tx value)? $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _Tx() when $default != null:
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
    TResult Function(_Tx value) $default,
  ) {
    final _that = this;
    switch (_that) {
      case _Tx():
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
    TResult? Function(_Tx value)? $default,
  ) {
    final _that = this;
    switch (_that) {
      case _Tx() when $default != null:
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
    TResult Function(int id, Uint8List txid, int height, int time,
            PlatformInt64 value, int? tpe, String? category)?
        $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _Tx() when $default != null:
        return $default(_that.id, _that.txid, _that.height, _that.time,
            _that.value, _that.tpe, _that.category);
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
    TResult Function(int id, Uint8List txid, int height, int time,
            PlatformInt64 value, int? tpe, String? category)
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _Tx():
        return $default(_that.id, _that.txid, _that.height, _that.time,
            _that.value, _that.tpe, _that.category);
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
    TResult? Function(int id, Uint8List txid, int height, int time,
            PlatformInt64 value, int? tpe, String? category)?
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _Tx() when $default != null:
        return $default(_that.id, _that.txid, _that.height, _that.time,
            _that.value, _that.tpe, _that.category);
      case _:
        return null;
    }
  }
}

/// @nodoc

class _Tx implements Tx {
  const _Tx(
      {required this.id,
      required this.txid,
      required this.height,
      required this.time,
      required this.value,
      this.tpe,
      this.category});

  @override
  final int id;
  @override
  final Uint8List txid;
  @override
  final int height;
  @override
  final int time;
  @override
  final PlatformInt64 value;
  @override
  final int? tpe;
  @override
  final String? category;

  /// Create a copy of Tx
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  _$TxCopyWith<_Tx> get copyWith => __$TxCopyWithImpl<_Tx>(this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _Tx &&
            (identical(other.id, id) || other.id == id) &&
            const DeepCollectionEquality().equals(other.txid, txid) &&
            (identical(other.height, height) || other.height == height) &&
            (identical(other.time, time) || other.time == time) &&
            (identical(other.value, value) || other.value == value) &&
            (identical(other.tpe, tpe) || other.tpe == tpe) &&
            (identical(other.category, category) ||
                other.category == category));
  }

  @override
  int get hashCode => Object.hash(
      runtimeType,
      id,
      const DeepCollectionEquality().hash(txid),
      height,
      time,
      value,
      tpe,
      category);

  @override
  String toString() {
    return 'Tx(id: $id, txid: $txid, height: $height, time: $time, value: $value, tpe: $tpe, category: $category)';
  }
}

/// @nodoc
abstract mixin class _$TxCopyWith<$Res> implements $TxCopyWith<$Res> {
  factory _$TxCopyWith(_Tx value, $Res Function(_Tx) _then) = __$TxCopyWithImpl;
  @override
  @useResult
  $Res call(
      {int id,
      Uint8List txid,
      int height,
      int time,
      PlatformInt64 value,
      int? tpe,
      String? category});
}

/// @nodoc
class __$TxCopyWithImpl<$Res> implements _$TxCopyWith<$Res> {
  __$TxCopyWithImpl(this._self, this._then);

  final _Tx _self;
  final $Res Function(_Tx) _then;

  /// Create a copy of Tx
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $Res call({
    Object? id = null,
    Object? txid = null,
    Object? height = null,
    Object? time = null,
    Object? value = null,
    Object? tpe = freezed,
    Object? category = freezed,
  }) {
    return _then(_Tx(
      id: null == id
          ? _self.id
          : id // ignore: cast_nullable_to_non_nullable
              as int,
      txid: null == txid
          ? _self.txid
          : txid // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      height: null == height
          ? _self.height
          : height // ignore: cast_nullable_to_non_nullable
              as int,
      time: null == time
          ? _self.time
          : time // ignore: cast_nullable_to_non_nullable
              as int,
      value: null == value
          ? _self.value
          : value // ignore: cast_nullable_to_non_nullable
              as PlatformInt64,
      tpe: freezed == tpe
          ? _self.tpe
          : tpe // ignore: cast_nullable_to_non_nullable
              as int?,
      category: freezed == category
          ? _self.category
          : category // ignore: cast_nullable_to_non_nullable
              as String?,
    ));
  }
}

// dart format on
