// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'store.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;

/// @nodoc
mixin _$SyncState {
  int get start;
  int get end;
  int get height;
  int get time;
  List<Account> get accounts;

  /// Create a copy of SyncState
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $SyncStateCopyWith<SyncState> get copyWith =>
      _$SyncStateCopyWithImpl<SyncState>(this as SyncState, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is SyncState &&
            (identical(other.start, start) || other.start == start) &&
            (identical(other.end, end) || other.end == end) &&
            (identical(other.height, height) || other.height == height) &&
            (identical(other.time, time) || other.time == time) &&
            const DeepCollectionEquality().equals(other.accounts, accounts));
  }

  @override
  int get hashCode => Object.hash(runtimeType, start, end, height, time,
      const DeepCollectionEquality().hash(accounts));

  @override
  String toString() {
    return 'SyncState(start: $start, end: $end, height: $height, time: $time, accounts: $accounts)';
  }
}

/// @nodoc
abstract mixin class $SyncStateCopyWith<$Res> {
  factory $SyncStateCopyWith(SyncState value, $Res Function(SyncState) _then) =
      _$SyncStateCopyWithImpl;
  @useResult
  $Res call({int start, int end, int height, int time, List<Account> accounts});
}

/// @nodoc
class _$SyncStateCopyWithImpl<$Res> implements $SyncStateCopyWith<$Res> {
  _$SyncStateCopyWithImpl(this._self, this._then);

  final SyncState _self;
  final $Res Function(SyncState) _then;

  /// Create a copy of SyncState
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? start = null,
    Object? end = null,
    Object? height = null,
    Object? time = null,
    Object? accounts = null,
  }) {
    return _then(_self.copyWith(
      start: null == start
          ? _self.start
          : start // ignore: cast_nullable_to_non_nullable
              as int,
      end: null == end
          ? _self.end
          : end // ignore: cast_nullable_to_non_nullable
              as int,
      height: null == height
          ? _self.height
          : height // ignore: cast_nullable_to_non_nullable
              as int,
      time: null == time
          ? _self.time
          : time // ignore: cast_nullable_to_non_nullable
              as int,
      accounts: null == accounts
          ? _self.accounts
          : accounts // ignore: cast_nullable_to_non_nullable
              as List<Account>,
    ));
  }
}

/// Adds pattern-matching-related methods to [SyncState].
extension SyncStatePatterns on SyncState {
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
    TResult Function(_SyncState value)? $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _SyncState() when $default != null:
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
    TResult Function(_SyncState value) $default,
  ) {
    final _that = this;
    switch (_that) {
      case _SyncState():
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
    TResult? Function(_SyncState value)? $default,
  ) {
    final _that = this;
    switch (_that) {
      case _SyncState() when $default != null:
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
            int start, int end, int height, int time, List<Account> accounts)?
        $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _SyncState() when $default != null:
        return $default(
            _that.start, _that.end, _that.height, _that.time, _that.accounts);
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
            int start, int end, int height, int time, List<Account> accounts)
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _SyncState():
        return $default(
            _that.start, _that.end, _that.height, _that.time, _that.accounts);
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
            int start, int end, int height, int time, List<Account> accounts)?
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _SyncState() when $default != null:
        return $default(
            _that.start, _that.end, _that.height, _that.time, _that.accounts);
      case _:
        return null;
    }
  }
}

/// @nodoc

class _SyncState implements SyncState {
  _SyncState(
      {required this.start,
      required this.end,
      required this.height,
      required this.time,
      required final List<Account> accounts})
      : _accounts = accounts;

  @override
  final int start;
  @override
  final int end;
  @override
  final int height;
  @override
  final int time;
  final List<Account> _accounts;
  @override
  List<Account> get accounts {
    if (_accounts is EqualUnmodifiableListView) return _accounts;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_accounts);
  }

  /// Create a copy of SyncState
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  _$SyncStateCopyWith<_SyncState> get copyWith =>
      __$SyncStateCopyWithImpl<_SyncState>(this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _SyncState &&
            (identical(other.start, start) || other.start == start) &&
            (identical(other.end, end) || other.end == end) &&
            (identical(other.height, height) || other.height == height) &&
            (identical(other.time, time) || other.time == time) &&
            const DeepCollectionEquality().equals(other._accounts, _accounts));
  }

  @override
  int get hashCode => Object.hash(runtimeType, start, end, height, time,
      const DeepCollectionEquality().hash(_accounts));

  @override
  String toString() {
    return 'SyncState(start: $start, end: $end, height: $height, time: $time, accounts: $accounts)';
  }
}

/// @nodoc
abstract mixin class _$SyncStateCopyWith<$Res>
    implements $SyncStateCopyWith<$Res> {
  factory _$SyncStateCopyWith(
          _SyncState value, $Res Function(_SyncState) _then) =
      __$SyncStateCopyWithImpl;
  @override
  @useResult
  $Res call({int start, int end, int height, int time, List<Account> accounts});
}

/// @nodoc
class __$SyncStateCopyWithImpl<$Res> implements _$SyncStateCopyWith<$Res> {
  __$SyncStateCopyWithImpl(this._self, this._then);

  final _SyncState _self;
  final $Res Function(_SyncState) _then;

  /// Create a copy of SyncState
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $Res call({
    Object? start = null,
    Object? end = null,
    Object? height = null,
    Object? time = null,
    Object? accounts = null,
  }) {
    return _then(_SyncState(
      start: null == start
          ? _self.start
          : start // ignore: cast_nullable_to_non_nullable
              as int,
      end: null == end
          ? _self.end
          : end // ignore: cast_nullable_to_non_nullable
              as int,
      height: null == height
          ? _self.height
          : height // ignore: cast_nullable_to_non_nullable
              as int,
      time: null == time
          ? _self.time
          : time // ignore: cast_nullable_to_non_nullable
              as int,
      accounts: null == accounts
          ? _self._accounts
          : accounts // ignore: cast_nullable_to_non_nullable
              as List<Account>,
    ));
  }
}

/// @nodoc
mixin _$SyncProgressAccount {
  Account get account;
  int get start;
  int get end;
  int get height;
  int get time;

  /// Create a copy of SyncProgressAccount
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $SyncProgressAccountCopyWith<SyncProgressAccount> get copyWith =>
      _$SyncProgressAccountCopyWithImpl<SyncProgressAccount>(
          this as SyncProgressAccount, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is SyncProgressAccount &&
            (identical(other.account, account) || other.account == account) &&
            (identical(other.start, start) || other.start == start) &&
            (identical(other.end, end) || other.end == end) &&
            (identical(other.height, height) || other.height == height) &&
            (identical(other.time, time) || other.time == time));
  }

  @override
  int get hashCode =>
      Object.hash(runtimeType, account, start, end, height, time);

  @override
  String toString() {
    return 'SyncProgressAccount(account: $account, start: $start, end: $end, height: $height, time: $time)';
  }
}

/// @nodoc
abstract mixin class $SyncProgressAccountCopyWith<$Res> {
  factory $SyncProgressAccountCopyWith(
          SyncProgressAccount value, $Res Function(SyncProgressAccount) _then) =
      _$SyncProgressAccountCopyWithImpl;
  @useResult
  $Res call({Account account, int start, int end, int height, int time});

  $AccountCopyWith<$Res> get account;
}

/// @nodoc
class _$SyncProgressAccountCopyWithImpl<$Res>
    implements $SyncProgressAccountCopyWith<$Res> {
  _$SyncProgressAccountCopyWithImpl(this._self, this._then);

  final SyncProgressAccount _self;
  final $Res Function(SyncProgressAccount) _then;

  /// Create a copy of SyncProgressAccount
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? account = null,
    Object? start = null,
    Object? end = null,
    Object? height = null,
    Object? time = null,
  }) {
    return _then(_self.copyWith(
      account: null == account
          ? _self.account
          : account // ignore: cast_nullable_to_non_nullable
              as Account,
      start: null == start
          ? _self.start
          : start // ignore: cast_nullable_to_non_nullable
              as int,
      end: null == end
          ? _self.end
          : end // ignore: cast_nullable_to_non_nullable
              as int,
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

  /// Create a copy of SyncProgressAccount
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $AccountCopyWith<$Res> get account {
    return $AccountCopyWith<$Res>(_self.account, (value) {
      return _then(_self.copyWith(account: value));
    });
  }
}

/// Adds pattern-matching-related methods to [SyncProgressAccount].
extension SyncProgressAccountPatterns on SyncProgressAccount {
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
    TResult Function(_SyncProgressAccount value)? $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _SyncProgressAccount() when $default != null:
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
    TResult Function(_SyncProgressAccount value) $default,
  ) {
    final _that = this;
    switch (_that) {
      case _SyncProgressAccount():
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
    TResult? Function(_SyncProgressAccount value)? $default,
  ) {
    final _that = this;
    switch (_that) {
      case _SyncProgressAccount() when $default != null:
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
    TResult Function(Account account, int start, int end, int height, int time)?
        $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _SyncProgressAccount() when $default != null:
        return $default(
            _that.account, _that.start, _that.end, _that.height, _that.time);
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
    TResult Function(Account account, int start, int end, int height, int time)
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _SyncProgressAccount():
        return $default(
            _that.account, _that.start, _that.end, _that.height, _that.time);
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
            Account account, int start, int end, int height, int time)?
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _SyncProgressAccount() when $default != null:
        return $default(
            _that.account, _that.start, _that.end, _that.height, _that.time);
      case _:
        return null;
    }
  }
}

/// @nodoc

class _SyncProgressAccount extends SyncProgressAccount {
  _SyncProgressAccount(
      {required this.account,
      required this.start,
      required this.end,
      required this.height,
      required this.time})
      : super._();

  @override
  final Account account;
  @override
  final int start;
  @override
  final int end;
  @override
  final int height;
  @override
  final int time;

  /// Create a copy of SyncProgressAccount
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  _$SyncProgressAccountCopyWith<_SyncProgressAccount> get copyWith =>
      __$SyncProgressAccountCopyWithImpl<_SyncProgressAccount>(
          this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _SyncProgressAccount &&
            (identical(other.account, account) || other.account == account) &&
            (identical(other.start, start) || other.start == start) &&
            (identical(other.end, end) || other.end == end) &&
            (identical(other.height, height) || other.height == height) &&
            (identical(other.time, time) || other.time == time));
  }

  @override
  int get hashCode =>
      Object.hash(runtimeType, account, start, end, height, time);

  @override
  String toString() {
    return 'SyncProgressAccount(account: $account, start: $start, end: $end, height: $height, time: $time)';
  }
}

/// @nodoc
abstract mixin class _$SyncProgressAccountCopyWith<$Res>
    implements $SyncProgressAccountCopyWith<$Res> {
  factory _$SyncProgressAccountCopyWith(_SyncProgressAccount value,
          $Res Function(_SyncProgressAccount) _then) =
      __$SyncProgressAccountCopyWithImpl;
  @override
  @useResult
  $Res call({Account account, int start, int end, int height, int time});

  @override
  $AccountCopyWith<$Res> get account;
}

/// @nodoc
class __$SyncProgressAccountCopyWithImpl<$Res>
    implements _$SyncProgressAccountCopyWith<$Res> {
  __$SyncProgressAccountCopyWithImpl(this._self, this._then);

  final _SyncProgressAccount _self;
  final $Res Function(_SyncProgressAccount) _then;

  /// Create a copy of SyncProgressAccount
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $Res call({
    Object? account = null,
    Object? start = null,
    Object? end = null,
    Object? height = null,
    Object? time = null,
  }) {
    return _then(_SyncProgressAccount(
      account: null == account
          ? _self.account
          : account // ignore: cast_nullable_to_non_nullable
              as Account,
      start: null == start
          ? _self.start
          : start // ignore: cast_nullable_to_non_nullable
              as int,
      end: null == end
          ? _self.end
          : end // ignore: cast_nullable_to_non_nullable
              as int,
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

  /// Create a copy of SyncProgressAccount
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $AccountCopyWith<$Res> get account {
    return $AccountCopyWith<$Res>(_self.account, (value) {
      return _then(_self.copyWith(account: value));
    });
  }
}

/// @nodoc
mixin _$AccountData {
  Account get account;
  int get pool;
  PoolBalance get balance;
  List<Tx> get transactions;
  List<Memo> get memos;
  List<TxNote> get notes;
  FrostParams? get frostParams;

  /// Create a copy of AccountData
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $AccountDataCopyWith<AccountData> get copyWith =>
      _$AccountDataCopyWithImpl<AccountData>(this as AccountData, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is AccountData &&
            (identical(other.account, account) || other.account == account) &&
            (identical(other.pool, pool) || other.pool == pool) &&
            (identical(other.balance, balance) || other.balance == balance) &&
            const DeepCollectionEquality()
                .equals(other.transactions, transactions) &&
            const DeepCollectionEquality().equals(other.memos, memos) &&
            const DeepCollectionEquality().equals(other.notes, notes) &&
            (identical(other.frostParams, frostParams) ||
                other.frostParams == frostParams));
  }

  @override
  int get hashCode => Object.hash(
      runtimeType,
      account,
      pool,
      balance,
      const DeepCollectionEquality().hash(transactions),
      const DeepCollectionEquality().hash(memos),
      const DeepCollectionEquality().hash(notes),
      frostParams);

  @override
  String toString() {
    return 'AccountData(account: $account, pool: $pool, balance: $balance, transactions: $transactions, memos: $memos, notes: $notes, frostParams: $frostParams)';
  }
}

/// @nodoc
abstract mixin class $AccountDataCopyWith<$Res> {
  factory $AccountDataCopyWith(
          AccountData value, $Res Function(AccountData) _then) =
      _$AccountDataCopyWithImpl;
  @useResult
  $Res call(
      {Account account,
      int pool,
      PoolBalance balance,
      List<Tx> transactions,
      List<Memo> memos,
      List<TxNote> notes,
      FrostParams? frostParams});

  $AccountCopyWith<$Res> get account;
  $FrostParamsCopyWith<$Res>? get frostParams;
}

/// @nodoc
class _$AccountDataCopyWithImpl<$Res> implements $AccountDataCopyWith<$Res> {
  _$AccountDataCopyWithImpl(this._self, this._then);

  final AccountData _self;
  final $Res Function(AccountData) _then;

  /// Create a copy of AccountData
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? account = null,
    Object? pool = null,
    Object? balance = null,
    Object? transactions = null,
    Object? memos = null,
    Object? notes = null,
    Object? frostParams = freezed,
  }) {
    return _then(_self.copyWith(
      account: null == account
          ? _self.account
          : account // ignore: cast_nullable_to_non_nullable
              as Account,
      pool: null == pool
          ? _self.pool
          : pool // ignore: cast_nullable_to_non_nullable
              as int,
      balance: null == balance
          ? _self.balance
          : balance // ignore: cast_nullable_to_non_nullable
              as PoolBalance,
      transactions: null == transactions
          ? _self.transactions
          : transactions // ignore: cast_nullable_to_non_nullable
              as List<Tx>,
      memos: null == memos
          ? _self.memos
          : memos // ignore: cast_nullable_to_non_nullable
              as List<Memo>,
      notes: null == notes
          ? _self.notes
          : notes // ignore: cast_nullable_to_non_nullable
              as List<TxNote>,
      frostParams: freezed == frostParams
          ? _self.frostParams
          : frostParams // ignore: cast_nullable_to_non_nullable
              as FrostParams?,
    ));
  }

  /// Create a copy of AccountData
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $AccountCopyWith<$Res> get account {
    return $AccountCopyWith<$Res>(_self.account, (value) {
      return _then(_self.copyWith(account: value));
    });
  }

  /// Create a copy of AccountData
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $FrostParamsCopyWith<$Res>? get frostParams {
    if (_self.frostParams == null) {
      return null;
    }

    return $FrostParamsCopyWith<$Res>(_self.frostParams!, (value) {
      return _then(_self.copyWith(frostParams: value));
    });
  }
}

/// Adds pattern-matching-related methods to [AccountData].
extension AccountDataPatterns on AccountData {
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
    TResult Function(_AccountData value)? $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _AccountData() when $default != null:
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
    TResult Function(_AccountData value) $default,
  ) {
    final _that = this;
    switch (_that) {
      case _AccountData():
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
    TResult? Function(_AccountData value)? $default,
  ) {
    final _that = this;
    switch (_that) {
      case _AccountData() when $default != null:
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
            Account account,
            int pool,
            PoolBalance balance,
            List<Tx> transactions,
            List<Memo> memos,
            List<TxNote> notes,
            FrostParams? frostParams)?
        $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _AccountData() when $default != null:
        return $default(_that.account, _that.pool, _that.balance,
            _that.transactions, _that.memos, _that.notes, _that.frostParams);
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
            Account account,
            int pool,
            PoolBalance balance,
            List<Tx> transactions,
            List<Memo> memos,
            List<TxNote> notes,
            FrostParams? frostParams)
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _AccountData():
        return $default(_that.account, _that.pool, _that.balance,
            _that.transactions, _that.memos, _that.notes, _that.frostParams);
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
            Account account,
            int pool,
            PoolBalance balance,
            List<Tx> transactions,
            List<Memo> memos,
            List<TxNote> notes,
            FrostParams? frostParams)?
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _AccountData() when $default != null:
        return $default(_that.account, _that.pool, _that.balance,
            _that.transactions, _that.memos, _that.notes, _that.frostParams);
      case _:
        return null;
    }
  }
}

/// @nodoc

class _AccountData implements AccountData {
  _AccountData(
      {required this.account,
      required this.pool,
      required this.balance,
      required final List<Tx> transactions,
      required final List<Memo> memos,
      required final List<TxNote> notes,
      this.frostParams})
      : _transactions = transactions,
        _memos = memos,
        _notes = notes;

  @override
  final Account account;
  @override
  final int pool;
  @override
  final PoolBalance balance;
  final List<Tx> _transactions;
  @override
  List<Tx> get transactions {
    if (_transactions is EqualUnmodifiableListView) return _transactions;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_transactions);
  }

  final List<Memo> _memos;
  @override
  List<Memo> get memos {
    if (_memos is EqualUnmodifiableListView) return _memos;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_memos);
  }

  final List<TxNote> _notes;
  @override
  List<TxNote> get notes {
    if (_notes is EqualUnmodifiableListView) return _notes;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_notes);
  }

  @override
  final FrostParams? frostParams;

  /// Create a copy of AccountData
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  _$AccountDataCopyWith<_AccountData> get copyWith =>
      __$AccountDataCopyWithImpl<_AccountData>(this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _AccountData &&
            (identical(other.account, account) || other.account == account) &&
            (identical(other.pool, pool) || other.pool == pool) &&
            (identical(other.balance, balance) || other.balance == balance) &&
            const DeepCollectionEquality()
                .equals(other._transactions, _transactions) &&
            const DeepCollectionEquality().equals(other._memos, _memos) &&
            const DeepCollectionEquality().equals(other._notes, _notes) &&
            (identical(other.frostParams, frostParams) ||
                other.frostParams == frostParams));
  }

  @override
  int get hashCode => Object.hash(
      runtimeType,
      account,
      pool,
      balance,
      const DeepCollectionEquality().hash(_transactions),
      const DeepCollectionEquality().hash(_memos),
      const DeepCollectionEquality().hash(_notes),
      frostParams);

  @override
  String toString() {
    return 'AccountData(account: $account, pool: $pool, balance: $balance, transactions: $transactions, memos: $memos, notes: $notes, frostParams: $frostParams)';
  }
}

/// @nodoc
abstract mixin class _$AccountDataCopyWith<$Res>
    implements $AccountDataCopyWith<$Res> {
  factory _$AccountDataCopyWith(
          _AccountData value, $Res Function(_AccountData) _then) =
      __$AccountDataCopyWithImpl;
  @override
  @useResult
  $Res call(
      {Account account,
      int pool,
      PoolBalance balance,
      List<Tx> transactions,
      List<Memo> memos,
      List<TxNote> notes,
      FrostParams? frostParams});

  @override
  $AccountCopyWith<$Res> get account;
  @override
  $FrostParamsCopyWith<$Res>? get frostParams;
}

/// @nodoc
class __$AccountDataCopyWithImpl<$Res> implements _$AccountDataCopyWith<$Res> {
  __$AccountDataCopyWithImpl(this._self, this._then);

  final _AccountData _self;
  final $Res Function(_AccountData) _then;

  /// Create a copy of AccountData
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $Res call({
    Object? account = null,
    Object? pool = null,
    Object? balance = null,
    Object? transactions = null,
    Object? memos = null,
    Object? notes = null,
    Object? frostParams = freezed,
  }) {
    return _then(_AccountData(
      account: null == account
          ? _self.account
          : account // ignore: cast_nullable_to_non_nullable
              as Account,
      pool: null == pool
          ? _self.pool
          : pool // ignore: cast_nullable_to_non_nullable
              as int,
      balance: null == balance
          ? _self.balance
          : balance // ignore: cast_nullable_to_non_nullable
              as PoolBalance,
      transactions: null == transactions
          ? _self._transactions
          : transactions // ignore: cast_nullable_to_non_nullable
              as List<Tx>,
      memos: null == memos
          ? _self._memos
          : memos // ignore: cast_nullable_to_non_nullable
              as List<Memo>,
      notes: null == notes
          ? _self._notes
          : notes // ignore: cast_nullable_to_non_nullable
              as List<TxNote>,
      frostParams: freezed == frostParams
          ? _self.frostParams
          : frostParams // ignore: cast_nullable_to_non_nullable
              as FrostParams?,
    ));
  }

  /// Create a copy of AccountData
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $AccountCopyWith<$Res> get account {
    return $AccountCopyWith<$Res>(_self.account, (value) {
      return _then(_self.copyWith(account: value));
    });
  }

  /// Create a copy of AccountData
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $FrostParamsCopyWith<$Res>? get frostParams {
    if (_self.frostParams == null) {
      return null;
    }

    return $FrostParamsCopyWith<$Res>(_self.frostParams!, (value) {
      return _then(_self.copyWith(frostParams: value));
    });
  }
}

/// @nodoc
mixin _$AppSettings {
  String get dbName;
  String get net;
  bool get isLightNode;
  String get lwd;
  String get blockExplorer;
  String get syncInterval; // in blocks
  String get actionsPerSync;
  bool get useTor;
  String get coingecko;
  bool get recovery;
  bool get needPin;
  DateTime get pinUnlockedAt;
  bool get offline;
  bool get getFx;
  QRSettings get qrSettings;
  bool get vault;

  /// Create a copy of AppSettings
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $AppSettingsCopyWith<AppSettings> get copyWith =>
      _$AppSettingsCopyWithImpl<AppSettings>(this as AppSettings, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is AppSettings &&
            (identical(other.dbName, dbName) || other.dbName == dbName) &&
            (identical(other.net, net) || other.net == net) &&
            (identical(other.isLightNode, isLightNode) ||
                other.isLightNode == isLightNode) &&
            (identical(other.lwd, lwd) || other.lwd == lwd) &&
            (identical(other.blockExplorer, blockExplorer) ||
                other.blockExplorer == blockExplorer) &&
            (identical(other.syncInterval, syncInterval) ||
                other.syncInterval == syncInterval) &&
            (identical(other.actionsPerSync, actionsPerSync) ||
                other.actionsPerSync == actionsPerSync) &&
            (identical(other.useTor, useTor) || other.useTor == useTor) &&
            (identical(other.coingecko, coingecko) ||
                other.coingecko == coingecko) &&
            (identical(other.recovery, recovery) ||
                other.recovery == recovery) &&
            (identical(other.needPin, needPin) || other.needPin == needPin) &&
            (identical(other.pinUnlockedAt, pinUnlockedAt) ||
                other.pinUnlockedAt == pinUnlockedAt) &&
            (identical(other.offline, offline) || other.offline == offline) &&
            (identical(other.getFx, getFx) || other.getFx == getFx) &&
            (identical(other.qrSettings, qrSettings) ||
                other.qrSettings == qrSettings) &&
            (identical(other.vault, vault) || other.vault == vault));
  }

  @override
  int get hashCode => Object.hash(
      runtimeType,
      dbName,
      net,
      isLightNode,
      lwd,
      blockExplorer,
      syncInterval,
      actionsPerSync,
      useTor,
      coingecko,
      recovery,
      needPin,
      pinUnlockedAt,
      offline,
      getFx,
      qrSettings,
      vault);

  @override
  String toString() {
    return 'AppSettings(dbName: $dbName, net: $net, isLightNode: $isLightNode, lwd: $lwd, blockExplorer: $blockExplorer, syncInterval: $syncInterval, actionsPerSync: $actionsPerSync, useTor: $useTor, coingecko: $coingecko, recovery: $recovery, needPin: $needPin, pinUnlockedAt: $pinUnlockedAt, offline: $offline, getFx: $getFx, qrSettings: $qrSettings, vault: $vault)';
  }
}

/// @nodoc
abstract mixin class $AppSettingsCopyWith<$Res> {
  factory $AppSettingsCopyWith(
          AppSettings value, $Res Function(AppSettings) _then) =
      _$AppSettingsCopyWithImpl;
  @useResult
  $Res call(
      {String dbName,
      String net,
      bool isLightNode,
      String lwd,
      String blockExplorer,
      String syncInterval,
      String actionsPerSync,
      bool useTor,
      String coingecko,
      bool recovery,
      bool needPin,
      DateTime pinUnlockedAt,
      bool offline,
      bool getFx,
      QRSettings qrSettings,
      bool vault});

  $QRSettingsCopyWith<$Res> get qrSettings;
}

/// @nodoc
class _$AppSettingsCopyWithImpl<$Res> implements $AppSettingsCopyWith<$Res> {
  _$AppSettingsCopyWithImpl(this._self, this._then);

  final AppSettings _self;
  final $Res Function(AppSettings) _then;

  /// Create a copy of AppSettings
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? dbName = null,
    Object? net = null,
    Object? isLightNode = null,
    Object? lwd = null,
    Object? blockExplorer = null,
    Object? syncInterval = null,
    Object? actionsPerSync = null,
    Object? useTor = null,
    Object? coingecko = null,
    Object? recovery = null,
    Object? needPin = null,
    Object? pinUnlockedAt = null,
    Object? offline = null,
    Object? getFx = null,
    Object? qrSettings = null,
    Object? vault = null,
  }) {
    return _then(_self.copyWith(
      dbName: null == dbName
          ? _self.dbName
          : dbName // ignore: cast_nullable_to_non_nullable
              as String,
      net: null == net
          ? _self.net
          : net // ignore: cast_nullable_to_non_nullable
              as String,
      isLightNode: null == isLightNode
          ? _self.isLightNode
          : isLightNode // ignore: cast_nullable_to_non_nullable
              as bool,
      lwd: null == lwd
          ? _self.lwd
          : lwd // ignore: cast_nullable_to_non_nullable
              as String,
      blockExplorer: null == blockExplorer
          ? _self.blockExplorer
          : blockExplorer // ignore: cast_nullable_to_non_nullable
              as String,
      syncInterval: null == syncInterval
          ? _self.syncInterval
          : syncInterval // ignore: cast_nullable_to_non_nullable
              as String,
      actionsPerSync: null == actionsPerSync
          ? _self.actionsPerSync
          : actionsPerSync // ignore: cast_nullable_to_non_nullable
              as String,
      useTor: null == useTor
          ? _self.useTor
          : useTor // ignore: cast_nullable_to_non_nullable
              as bool,
      coingecko: null == coingecko
          ? _self.coingecko
          : coingecko // ignore: cast_nullable_to_non_nullable
              as String,
      recovery: null == recovery
          ? _self.recovery
          : recovery // ignore: cast_nullable_to_non_nullable
              as bool,
      needPin: null == needPin
          ? _self.needPin
          : needPin // ignore: cast_nullable_to_non_nullable
              as bool,
      pinUnlockedAt: null == pinUnlockedAt
          ? _self.pinUnlockedAt
          : pinUnlockedAt // ignore: cast_nullable_to_non_nullable
              as DateTime,
      offline: null == offline
          ? _self.offline
          : offline // ignore: cast_nullable_to_non_nullable
              as bool,
      getFx: null == getFx
          ? _self.getFx
          : getFx // ignore: cast_nullable_to_non_nullable
              as bool,
      qrSettings: null == qrSettings
          ? _self.qrSettings
          : qrSettings // ignore: cast_nullable_to_non_nullable
              as QRSettings,
      vault: null == vault
          ? _self.vault
          : vault // ignore: cast_nullable_to_non_nullable
              as bool,
    ));
  }

  /// Create a copy of AppSettings
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $QRSettingsCopyWith<$Res> get qrSettings {
    return $QRSettingsCopyWith<$Res>(_self.qrSettings, (value) {
      return _then(_self.copyWith(qrSettings: value));
    });
  }
}

/// Adds pattern-matching-related methods to [AppSettings].
extension AppSettingsPatterns on AppSettings {
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
    TResult Function(_AppSettings value)? $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _AppSettings() when $default != null:
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
    TResult Function(_AppSettings value) $default,
  ) {
    final _that = this;
    switch (_that) {
      case _AppSettings():
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
    TResult? Function(_AppSettings value)? $default,
  ) {
    final _that = this;
    switch (_that) {
      case _AppSettings() when $default != null:
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
            String dbName,
            String net,
            bool isLightNode,
            String lwd,
            String blockExplorer,
            String syncInterval,
            String actionsPerSync,
            bool useTor,
            String coingecko,
            bool recovery,
            bool needPin,
            DateTime pinUnlockedAt,
            bool offline,
            bool getFx,
            QRSettings qrSettings,
            bool vault)?
        $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _AppSettings() when $default != null:
        return $default(
            _that.dbName,
            _that.net,
            _that.isLightNode,
            _that.lwd,
            _that.blockExplorer,
            _that.syncInterval,
            _that.actionsPerSync,
            _that.useTor,
            _that.coingecko,
            _that.recovery,
            _that.needPin,
            _that.pinUnlockedAt,
            _that.offline,
            _that.getFx,
            _that.qrSettings,
            _that.vault);
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
            String dbName,
            String net,
            bool isLightNode,
            String lwd,
            String blockExplorer,
            String syncInterval,
            String actionsPerSync,
            bool useTor,
            String coingecko,
            bool recovery,
            bool needPin,
            DateTime pinUnlockedAt,
            bool offline,
            bool getFx,
            QRSettings qrSettings,
            bool vault)
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _AppSettings():
        return $default(
            _that.dbName,
            _that.net,
            _that.isLightNode,
            _that.lwd,
            _that.blockExplorer,
            _that.syncInterval,
            _that.actionsPerSync,
            _that.useTor,
            _that.coingecko,
            _that.recovery,
            _that.needPin,
            _that.pinUnlockedAt,
            _that.offline,
            _that.getFx,
            _that.qrSettings,
            _that.vault);
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
            String dbName,
            String net,
            bool isLightNode,
            String lwd,
            String blockExplorer,
            String syncInterval,
            String actionsPerSync,
            bool useTor,
            String coingecko,
            bool recovery,
            bool needPin,
            DateTime pinUnlockedAt,
            bool offline,
            bool getFx,
            QRSettings qrSettings,
            bool vault)?
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _AppSettings() when $default != null:
        return $default(
            _that.dbName,
            _that.net,
            _that.isLightNode,
            _that.lwd,
            _that.blockExplorer,
            _that.syncInterval,
            _that.actionsPerSync,
            _that.useTor,
            _that.coingecko,
            _that.recovery,
            _that.needPin,
            _that.pinUnlockedAt,
            _that.offline,
            _that.getFx,
            _that.qrSettings,
            _that.vault);
      case _:
        return null;
    }
  }
}

/// @nodoc

class _AppSettings implements AppSettings {
  _AppSettings(
      {required this.dbName,
      required this.net,
      required this.isLightNode,
      required this.lwd,
      required this.blockExplorer,
      required this.syncInterval,
      required this.actionsPerSync,
      required this.useTor,
      required this.coingecko,
      required this.recovery,
      required this.needPin,
      required this.pinUnlockedAt,
      required this.offline,
      required this.getFx,
      required this.qrSettings,
      required this.vault});

  @override
  final String dbName;
  @override
  final String net;
  @override
  final bool isLightNode;
  @override
  final String lwd;
  @override
  final String blockExplorer;
  @override
  final String syncInterval;
// in blocks
  @override
  final String actionsPerSync;
  @override
  final bool useTor;
  @override
  final String coingecko;
  @override
  final bool recovery;
  @override
  final bool needPin;
  @override
  final DateTime pinUnlockedAt;
  @override
  final bool offline;
  @override
  final bool getFx;
  @override
  final QRSettings qrSettings;
  @override
  final bool vault;

  /// Create a copy of AppSettings
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  _$AppSettingsCopyWith<_AppSettings> get copyWith =>
      __$AppSettingsCopyWithImpl<_AppSettings>(this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _AppSettings &&
            (identical(other.dbName, dbName) || other.dbName == dbName) &&
            (identical(other.net, net) || other.net == net) &&
            (identical(other.isLightNode, isLightNode) ||
                other.isLightNode == isLightNode) &&
            (identical(other.lwd, lwd) || other.lwd == lwd) &&
            (identical(other.blockExplorer, blockExplorer) ||
                other.blockExplorer == blockExplorer) &&
            (identical(other.syncInterval, syncInterval) ||
                other.syncInterval == syncInterval) &&
            (identical(other.actionsPerSync, actionsPerSync) ||
                other.actionsPerSync == actionsPerSync) &&
            (identical(other.useTor, useTor) || other.useTor == useTor) &&
            (identical(other.coingecko, coingecko) ||
                other.coingecko == coingecko) &&
            (identical(other.recovery, recovery) ||
                other.recovery == recovery) &&
            (identical(other.needPin, needPin) || other.needPin == needPin) &&
            (identical(other.pinUnlockedAt, pinUnlockedAt) ||
                other.pinUnlockedAt == pinUnlockedAt) &&
            (identical(other.offline, offline) || other.offline == offline) &&
            (identical(other.getFx, getFx) || other.getFx == getFx) &&
            (identical(other.qrSettings, qrSettings) ||
                other.qrSettings == qrSettings) &&
            (identical(other.vault, vault) || other.vault == vault));
  }

  @override
  int get hashCode => Object.hash(
      runtimeType,
      dbName,
      net,
      isLightNode,
      lwd,
      blockExplorer,
      syncInterval,
      actionsPerSync,
      useTor,
      coingecko,
      recovery,
      needPin,
      pinUnlockedAt,
      offline,
      getFx,
      qrSettings,
      vault);

  @override
  String toString() {
    return 'AppSettings(dbName: $dbName, net: $net, isLightNode: $isLightNode, lwd: $lwd, blockExplorer: $blockExplorer, syncInterval: $syncInterval, actionsPerSync: $actionsPerSync, useTor: $useTor, coingecko: $coingecko, recovery: $recovery, needPin: $needPin, pinUnlockedAt: $pinUnlockedAt, offline: $offline, getFx: $getFx, qrSettings: $qrSettings, vault: $vault)';
  }
}

/// @nodoc
abstract mixin class _$AppSettingsCopyWith<$Res>
    implements $AppSettingsCopyWith<$Res> {
  factory _$AppSettingsCopyWith(
          _AppSettings value, $Res Function(_AppSettings) _then) =
      __$AppSettingsCopyWithImpl;
  @override
  @useResult
  $Res call(
      {String dbName,
      String net,
      bool isLightNode,
      String lwd,
      String blockExplorer,
      String syncInterval,
      String actionsPerSync,
      bool useTor,
      String coingecko,
      bool recovery,
      bool needPin,
      DateTime pinUnlockedAt,
      bool offline,
      bool getFx,
      QRSettings qrSettings,
      bool vault});

  @override
  $QRSettingsCopyWith<$Res> get qrSettings;
}

/// @nodoc
class __$AppSettingsCopyWithImpl<$Res> implements _$AppSettingsCopyWith<$Res> {
  __$AppSettingsCopyWithImpl(this._self, this._then);

  final _AppSettings _self;
  final $Res Function(_AppSettings) _then;

  /// Create a copy of AppSettings
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $Res call({
    Object? dbName = null,
    Object? net = null,
    Object? isLightNode = null,
    Object? lwd = null,
    Object? blockExplorer = null,
    Object? syncInterval = null,
    Object? actionsPerSync = null,
    Object? useTor = null,
    Object? coingecko = null,
    Object? recovery = null,
    Object? needPin = null,
    Object? pinUnlockedAt = null,
    Object? offline = null,
    Object? getFx = null,
    Object? qrSettings = null,
    Object? vault = null,
  }) {
    return _then(_AppSettings(
      dbName: null == dbName
          ? _self.dbName
          : dbName // ignore: cast_nullable_to_non_nullable
              as String,
      net: null == net
          ? _self.net
          : net // ignore: cast_nullable_to_non_nullable
              as String,
      isLightNode: null == isLightNode
          ? _self.isLightNode
          : isLightNode // ignore: cast_nullable_to_non_nullable
              as bool,
      lwd: null == lwd
          ? _self.lwd
          : lwd // ignore: cast_nullable_to_non_nullable
              as String,
      blockExplorer: null == blockExplorer
          ? _self.blockExplorer
          : blockExplorer // ignore: cast_nullable_to_non_nullable
              as String,
      syncInterval: null == syncInterval
          ? _self.syncInterval
          : syncInterval // ignore: cast_nullable_to_non_nullable
              as String,
      actionsPerSync: null == actionsPerSync
          ? _self.actionsPerSync
          : actionsPerSync // ignore: cast_nullable_to_non_nullable
              as String,
      useTor: null == useTor
          ? _self.useTor
          : useTor // ignore: cast_nullable_to_non_nullable
              as bool,
      coingecko: null == coingecko
          ? _self.coingecko
          : coingecko // ignore: cast_nullable_to_non_nullable
              as String,
      recovery: null == recovery
          ? _self.recovery
          : recovery // ignore: cast_nullable_to_non_nullable
              as bool,
      needPin: null == needPin
          ? _self.needPin
          : needPin // ignore: cast_nullable_to_non_nullable
              as bool,
      pinUnlockedAt: null == pinUnlockedAt
          ? _self.pinUnlockedAt
          : pinUnlockedAt // ignore: cast_nullable_to_non_nullable
              as DateTime,
      offline: null == offline
          ? _self.offline
          : offline // ignore: cast_nullable_to_non_nullable
              as bool,
      getFx: null == getFx
          ? _self.getFx
          : getFx // ignore: cast_nullable_to_non_nullable
              as bool,
      qrSettings: null == qrSettings
          ? _self.qrSettings
          : qrSettings // ignore: cast_nullable_to_non_nullable
              as QRSettings,
      vault: null == vault
          ? _self.vault
          : vault // ignore: cast_nullable_to_non_nullable
              as bool,
    ));
  }

  /// Create a copy of AppSettings
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $QRSettingsCopyWith<$Res> get qrSettings {
    return $QRSettingsCopyWith<$Res>(_self.qrSettings, (value) {
      return _then(_self.copyWith(qrSettings: value));
    });
  }
}

/// @nodoc
mixin _$ElectionData {
  ElectionPropsPub? get election;
  String get url;
  int get account;

  /// Create a copy of ElectionData
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $ElectionDataCopyWith<ElectionData> get copyWith =>
      _$ElectionDataCopyWithImpl<ElectionData>(
          this as ElectionData, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is ElectionData &&
            (identical(other.election, election) ||
                other.election == election) &&
            (identical(other.url, url) || other.url == url) &&
            (identical(other.account, account) || other.account == account));
  }

  @override
  int get hashCode => Object.hash(runtimeType, election, url, account);

  @override
  String toString() {
    return 'ElectionData(election: $election, url: $url, account: $account)';
  }
}

/// @nodoc
abstract mixin class $ElectionDataCopyWith<$Res> {
  factory $ElectionDataCopyWith(
          ElectionData value, $Res Function(ElectionData) _then) =
      _$ElectionDataCopyWithImpl;
  @useResult
  $Res call({ElectionPropsPub? election, String url, int account});
}

/// @nodoc
class _$ElectionDataCopyWithImpl<$Res> implements $ElectionDataCopyWith<$Res> {
  _$ElectionDataCopyWithImpl(this._self, this._then);

  final ElectionData _self;
  final $Res Function(ElectionData) _then;

  /// Create a copy of ElectionData
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? election = freezed,
    Object? url = null,
    Object? account = null,
  }) {
    return _then(_self.copyWith(
      election: freezed == election
          ? _self.election
          : election // ignore: cast_nullable_to_non_nullable
              as ElectionPropsPub?,
      url: null == url
          ? _self.url
          : url // ignore: cast_nullable_to_non_nullable
              as String,
      account: null == account
          ? _self.account
          : account // ignore: cast_nullable_to_non_nullable
              as int,
    ));
  }
}

/// Adds pattern-matching-related methods to [ElectionData].
extension ElectionDataPatterns on ElectionData {
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
    TResult Function(_ElectionData value)? $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _ElectionData() when $default != null:
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
    TResult Function(_ElectionData value) $default,
  ) {
    final _that = this;
    switch (_that) {
      case _ElectionData():
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
    TResult? Function(_ElectionData value)? $default,
  ) {
    final _that = this;
    switch (_that) {
      case _ElectionData() when $default != null:
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
    TResult Function(ElectionPropsPub? election, String url, int account)?
        $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _ElectionData() when $default != null:
        return $default(_that.election, _that.url, _that.account);
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
    TResult Function(ElectionPropsPub? election, String url, int account)
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _ElectionData():
        return $default(_that.election, _that.url, _that.account);
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
    TResult? Function(ElectionPropsPub? election, String url, int account)?
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _ElectionData() when $default != null:
        return $default(_that.election, _that.url, _that.account);
      case _:
        return null;
    }
  }
}

/// @nodoc

class _ElectionData implements ElectionData {
  _ElectionData(
      {required this.election, required this.url, required this.account});

  @override
  final ElectionPropsPub? election;
  @override
  final String url;
  @override
  final int account;

  /// Create a copy of ElectionData
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  _$ElectionDataCopyWith<_ElectionData> get copyWith =>
      __$ElectionDataCopyWithImpl<_ElectionData>(this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _ElectionData &&
            (identical(other.election, election) ||
                other.election == election) &&
            (identical(other.url, url) || other.url == url) &&
            (identical(other.account, account) || other.account == account));
  }

  @override
  int get hashCode => Object.hash(runtimeType, election, url, account);

  @override
  String toString() {
    return 'ElectionData(election: $election, url: $url, account: $account)';
  }
}

/// @nodoc
abstract mixin class _$ElectionDataCopyWith<$Res>
    implements $ElectionDataCopyWith<$Res> {
  factory _$ElectionDataCopyWith(
          _ElectionData value, $Res Function(_ElectionData) _then) =
      __$ElectionDataCopyWithImpl;
  @override
  @useResult
  $Res call({ElectionPropsPub? election, String url, int account});
}

/// @nodoc
class __$ElectionDataCopyWithImpl<$Res>
    implements _$ElectionDataCopyWith<$Res> {
  __$ElectionDataCopyWithImpl(this._self, this._then);

  final _ElectionData _self;
  final $Res Function(_ElectionData) _then;

  /// Create a copy of ElectionData
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $Res call({
    Object? election = freezed,
    Object? url = null,
    Object? account = null,
  }) {
    return _then(_ElectionData(
      election: freezed == election
          ? _self.election
          : election // ignore: cast_nullable_to_non_nullable
              as ElectionPropsPub?,
      url: null == url
          ? _self.url
          : url // ignore: cast_nullable_to_non_nullable
              as String,
      account: null == account
          ? _self.account
          : account // ignore: cast_nullable_to_non_nullable
              as int,
    ));
  }
}

/// @nodoc
mixin _$MempoolState {
  bool get running;
  Map<int, int> get unconfirmedFunds;
  List<(String, String, int)> get unconfirmedTx;

  /// Create a copy of MempoolState
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $MempoolStateCopyWith<MempoolState> get copyWith =>
      _$MempoolStateCopyWithImpl<MempoolState>(
          this as MempoolState, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is MempoolState &&
            (identical(other.running, running) || other.running == running) &&
            const DeepCollectionEquality()
                .equals(other.unconfirmedFunds, unconfirmedFunds) &&
            const DeepCollectionEquality()
                .equals(other.unconfirmedTx, unconfirmedTx));
  }

  @override
  int get hashCode => Object.hash(
      runtimeType,
      running,
      const DeepCollectionEquality().hash(unconfirmedFunds),
      const DeepCollectionEquality().hash(unconfirmedTx));

  @override
  String toString() {
    return 'MempoolState(running: $running, unconfirmedFunds: $unconfirmedFunds, unconfirmedTx: $unconfirmedTx)';
  }
}

/// @nodoc
abstract mixin class $MempoolStateCopyWith<$Res> {
  factory $MempoolStateCopyWith(
          MempoolState value, $Res Function(MempoolState) _then) =
      _$MempoolStateCopyWithImpl;
  @useResult
  $Res call(
      {bool running,
      Map<int, int> unconfirmedFunds,
      List<(String, String, int)> unconfirmedTx});
}

/// @nodoc
class _$MempoolStateCopyWithImpl<$Res> implements $MempoolStateCopyWith<$Res> {
  _$MempoolStateCopyWithImpl(this._self, this._then);

  final MempoolState _self;
  final $Res Function(MempoolState) _then;

  /// Create a copy of MempoolState
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? running = null,
    Object? unconfirmedFunds = null,
    Object? unconfirmedTx = null,
  }) {
    return _then(_self.copyWith(
      running: null == running
          ? _self.running
          : running // ignore: cast_nullable_to_non_nullable
              as bool,
      unconfirmedFunds: null == unconfirmedFunds
          ? _self.unconfirmedFunds
          : unconfirmedFunds // ignore: cast_nullable_to_non_nullable
              as Map<int, int>,
      unconfirmedTx: null == unconfirmedTx
          ? _self.unconfirmedTx
          : unconfirmedTx // ignore: cast_nullable_to_non_nullable
              as List<(String, String, int)>,
    ));
  }
}

/// Adds pattern-matching-related methods to [MempoolState].
extension MempoolStatePatterns on MempoolState {
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
    TResult Function(_MempoolState value)? $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _MempoolState() when $default != null:
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
    TResult Function(_MempoolState value) $default,
  ) {
    final _that = this;
    switch (_that) {
      case _MempoolState():
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
    TResult? Function(_MempoolState value)? $default,
  ) {
    final _that = this;
    switch (_that) {
      case _MempoolState() when $default != null:
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
    TResult Function(bool running, Map<int, int> unconfirmedFunds,
            List<(String, String, int)> unconfirmedTx)?
        $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _MempoolState() when $default != null:
        return $default(
            _that.running, _that.unconfirmedFunds, _that.unconfirmedTx);
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
    TResult Function(bool running, Map<int, int> unconfirmedFunds,
            List<(String, String, int)> unconfirmedTx)
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _MempoolState():
        return $default(
            _that.running, _that.unconfirmedFunds, _that.unconfirmedTx);
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
    TResult? Function(bool running, Map<int, int> unconfirmedFunds,
            List<(String, String, int)> unconfirmedTx)?
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _MempoolState() when $default != null:
        return $default(
            _that.running, _that.unconfirmedFunds, _that.unconfirmedTx);
      case _:
        return null;
    }
  }
}

/// @nodoc

class _MempoolState implements MempoolState {
  _MempoolState(
      {required this.running,
      required this.unconfirmedFunds,
      required this.unconfirmedTx});

  @override
  final bool running;
  @override
  final Map<int, int> unconfirmedFunds;
  @override
  final List<(String, String, int)> unconfirmedTx;

  /// Create a copy of MempoolState
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  _$MempoolStateCopyWith<_MempoolState> get copyWith =>
      __$MempoolStateCopyWithImpl<_MempoolState>(this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _MempoolState &&
            (identical(other.running, running) || other.running == running) &&
            const DeepCollectionEquality()
                .equals(other.unconfirmedFunds, unconfirmedFunds) &&
            const DeepCollectionEquality()
                .equals(other.unconfirmedTx, unconfirmedTx));
  }

  @override
  int get hashCode => Object.hash(
      runtimeType,
      running,
      const DeepCollectionEquality().hash(unconfirmedFunds),
      const DeepCollectionEquality().hash(unconfirmedTx));

  @override
  String toString() {
    return 'MempoolState(running: $running, unconfirmedFunds: $unconfirmedFunds, unconfirmedTx: $unconfirmedTx)';
  }
}

/// @nodoc
abstract mixin class _$MempoolStateCopyWith<$Res>
    implements $MempoolStateCopyWith<$Res> {
  factory _$MempoolStateCopyWith(
          _MempoolState value, $Res Function(_MempoolState) _then) =
      __$MempoolStateCopyWithImpl;
  @override
  @useResult
  $Res call(
      {bool running,
      Map<int, int> unconfirmedFunds,
      List<(String, String, int)> unconfirmedTx});
}

/// @nodoc
class __$MempoolStateCopyWithImpl<$Res>
    implements _$MempoolStateCopyWith<$Res> {
  __$MempoolStateCopyWithImpl(this._self, this._then);

  final _MempoolState _self;
  final $Res Function(_MempoolState) _then;

  /// Create a copy of MempoolState
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $Res call({
    Object? running = null,
    Object? unconfirmedFunds = null,
    Object? unconfirmedTx = null,
  }) {
    return _then(_MempoolState(
      running: null == running
          ? _self.running
          : running // ignore: cast_nullable_to_non_nullable
              as bool,
      unconfirmedFunds: null == unconfirmedFunds
          ? _self.unconfirmedFunds
          : unconfirmedFunds // ignore: cast_nullable_to_non_nullable
              as Map<int, int>,
      unconfirmedTx: null == unconfirmedTx
          ? _self.unconfirmedTx
          : unconfirmedTx // ignore: cast_nullable_to_non_nullable
              as List<(String, String, int)>,
    ));
  }
}

/// @nodoc
mixin _$AccountsPageData {
  AppSettings get settings;
  List<Account> get accounts;
  double? get price;
  Folder? get selectedFolder;

  /// Create a copy of AccountsPageData
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $AccountsPageDataCopyWith<AccountsPageData> get copyWith =>
      _$AccountsPageDataCopyWithImpl<AccountsPageData>(
          this as AccountsPageData, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is AccountsPageData &&
            (identical(other.settings, settings) ||
                other.settings == settings) &&
            const DeepCollectionEquality().equals(other.accounts, accounts) &&
            (identical(other.price, price) || other.price == price) &&
            (identical(other.selectedFolder, selectedFolder) ||
                other.selectedFolder == selectedFolder));
  }

  @override
  int get hashCode => Object.hash(runtimeType, settings,
      const DeepCollectionEquality().hash(accounts), price, selectedFolder);

  @override
  String toString() {
    return 'AccountsPageData(settings: $settings, accounts: $accounts, price: $price, selectedFolder: $selectedFolder)';
  }
}

/// @nodoc
abstract mixin class $AccountsPageDataCopyWith<$Res> {
  factory $AccountsPageDataCopyWith(
          AccountsPageData value, $Res Function(AccountsPageData) _then) =
      _$AccountsPageDataCopyWithImpl;
  @useResult
  $Res call(
      {AppSettings settings,
      List<Account> accounts,
      double? price,
      Folder? selectedFolder});

  $AppSettingsCopyWith<$Res> get settings;
  $FolderCopyWith<$Res>? get selectedFolder;
}

/// @nodoc
class _$AccountsPageDataCopyWithImpl<$Res>
    implements $AccountsPageDataCopyWith<$Res> {
  _$AccountsPageDataCopyWithImpl(this._self, this._then);

  final AccountsPageData _self;
  final $Res Function(AccountsPageData) _then;

  /// Create a copy of AccountsPageData
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? settings = null,
    Object? accounts = null,
    Object? price = freezed,
    Object? selectedFolder = freezed,
  }) {
    return _then(_self.copyWith(
      settings: null == settings
          ? _self.settings
          : settings // ignore: cast_nullable_to_non_nullable
              as AppSettings,
      accounts: null == accounts
          ? _self.accounts
          : accounts // ignore: cast_nullable_to_non_nullable
              as List<Account>,
      price: freezed == price
          ? _self.price
          : price // ignore: cast_nullable_to_non_nullable
              as double?,
      selectedFolder: freezed == selectedFolder
          ? _self.selectedFolder
          : selectedFolder // ignore: cast_nullable_to_non_nullable
              as Folder?,
    ));
  }

  /// Create a copy of AccountsPageData
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $AppSettingsCopyWith<$Res> get settings {
    return $AppSettingsCopyWith<$Res>(_self.settings, (value) {
      return _then(_self.copyWith(settings: value));
    });
  }

  /// Create a copy of AccountsPageData
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $FolderCopyWith<$Res>? get selectedFolder {
    if (_self.selectedFolder == null) {
      return null;
    }

    return $FolderCopyWith<$Res>(_self.selectedFolder!, (value) {
      return _then(_self.copyWith(selectedFolder: value));
    });
  }
}

/// Adds pattern-matching-related methods to [AccountsPageData].
extension AccountsPageDataPatterns on AccountsPageData {
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
    TResult Function(_AccountsPageData value)? $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _AccountsPageData() when $default != null:
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
    TResult Function(_AccountsPageData value) $default,
  ) {
    final _that = this;
    switch (_that) {
      case _AccountsPageData():
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
    TResult? Function(_AccountsPageData value)? $default,
  ) {
    final _that = this;
    switch (_that) {
      case _AccountsPageData() when $default != null:
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
    TResult Function(AppSettings settings, List<Account> accounts,
            double? price, Folder? selectedFolder)?
        $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _AccountsPageData() when $default != null:
        return $default(
            _that.settings, _that.accounts, _that.price, _that.selectedFolder);
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
    TResult Function(AppSettings settings, List<Account> accounts,
            double? price, Folder? selectedFolder)
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _AccountsPageData():
        return $default(
            _that.settings, _that.accounts, _that.price, _that.selectedFolder);
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
    TResult? Function(AppSettings settings, List<Account> accounts,
            double? price, Folder? selectedFolder)?
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _AccountsPageData() when $default != null:
        return $default(
            _that.settings, _that.accounts, _that.price, _that.selectedFolder);
      case _:
        return null;
    }
  }
}

/// @nodoc

class _AccountsPageData implements AccountsPageData {
  const _AccountsPageData(
      {required this.settings,
      required final List<Account> accounts,
      required this.price,
      required this.selectedFolder})
      : _accounts = accounts;

  @override
  final AppSettings settings;
  final List<Account> _accounts;
  @override
  List<Account> get accounts {
    if (_accounts is EqualUnmodifiableListView) return _accounts;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_accounts);
  }

  @override
  final double? price;
  @override
  final Folder? selectedFolder;

  /// Create a copy of AccountsPageData
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  _$AccountsPageDataCopyWith<_AccountsPageData> get copyWith =>
      __$AccountsPageDataCopyWithImpl<_AccountsPageData>(this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _AccountsPageData &&
            (identical(other.settings, settings) ||
                other.settings == settings) &&
            const DeepCollectionEquality().equals(other._accounts, _accounts) &&
            (identical(other.price, price) || other.price == price) &&
            (identical(other.selectedFolder, selectedFolder) ||
                other.selectedFolder == selectedFolder));
  }

  @override
  int get hashCode => Object.hash(runtimeType, settings,
      const DeepCollectionEquality().hash(_accounts), price, selectedFolder);

  @override
  String toString() {
    return 'AccountsPageData(settings: $settings, accounts: $accounts, price: $price, selectedFolder: $selectedFolder)';
  }
}

/// @nodoc
abstract mixin class _$AccountsPageDataCopyWith<$Res>
    implements $AccountsPageDataCopyWith<$Res> {
  factory _$AccountsPageDataCopyWith(
          _AccountsPageData value, $Res Function(_AccountsPageData) _then) =
      __$AccountsPageDataCopyWithImpl;
  @override
  @useResult
  $Res call(
      {AppSettings settings,
      List<Account> accounts,
      double? price,
      Folder? selectedFolder});

  @override
  $AppSettingsCopyWith<$Res> get settings;
  @override
  $FolderCopyWith<$Res>? get selectedFolder;
}

/// @nodoc
class __$AccountsPageDataCopyWithImpl<$Res>
    implements _$AccountsPageDataCopyWith<$Res> {
  __$AccountsPageDataCopyWithImpl(this._self, this._then);

  final _AccountsPageData _self;
  final $Res Function(_AccountsPageData) _then;

  /// Create a copy of AccountsPageData
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $Res call({
    Object? settings = null,
    Object? accounts = null,
    Object? price = freezed,
    Object? selectedFolder = freezed,
  }) {
    return _then(_AccountsPageData(
      settings: null == settings
          ? _self.settings
          : settings // ignore: cast_nullable_to_non_nullable
              as AppSettings,
      accounts: null == accounts
          ? _self._accounts
          : accounts // ignore: cast_nullable_to_non_nullable
              as List<Account>,
      price: freezed == price
          ? _self.price
          : price // ignore: cast_nullable_to_non_nullable
              as double?,
      selectedFolder: freezed == selectedFolder
          ? _self.selectedFolder
          : selectedFolder // ignore: cast_nullable_to_non_nullable
              as Folder?,
    ));
  }

  /// Create a copy of AccountsPageData
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $AppSettingsCopyWith<$Res> get settings {
    return $AppSettingsCopyWith<$Res>(_self.settings, (value) {
      return _then(_self.copyWith(settings: value));
    });
  }

  /// Create a copy of AccountsPageData
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $FolderCopyWith<$Res>? get selectedFolder {
    if (_self.selectedFolder == null) {
      return null;
    }

    return $FolderCopyWith<$Res>(_self.selectedFolder!, (value) {
      return _then(_self.copyWith(selectedFolder: value));
    });
  }
}

/// @nodoc
mixin _$BasicAccountData {
  List<Account> get allAccounts;
  AccountData? get currentAccount;

  /// Create a copy of BasicAccountData
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $BasicAccountDataCopyWith<BasicAccountData> get copyWith =>
      _$BasicAccountDataCopyWithImpl<BasicAccountData>(
          this as BasicAccountData, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is BasicAccountData &&
            const DeepCollectionEquality()
                .equals(other.allAccounts, allAccounts) &&
            (identical(other.currentAccount, currentAccount) ||
                other.currentAccount == currentAccount));
  }

  @override
  int get hashCode => Object.hash(runtimeType,
      const DeepCollectionEquality().hash(allAccounts), currentAccount);

  @override
  String toString() {
    return 'BasicAccountData(allAccounts: $allAccounts, currentAccount: $currentAccount)';
  }
}

/// @nodoc
abstract mixin class $BasicAccountDataCopyWith<$Res> {
  factory $BasicAccountDataCopyWith(
          BasicAccountData value, $Res Function(BasicAccountData) _then) =
      _$BasicAccountDataCopyWithImpl;
  @useResult
  $Res call({List<Account> allAccounts, AccountData? currentAccount});

  $AccountDataCopyWith<$Res>? get currentAccount;
}

/// @nodoc
class _$BasicAccountDataCopyWithImpl<$Res>
    implements $BasicAccountDataCopyWith<$Res> {
  _$BasicAccountDataCopyWithImpl(this._self, this._then);

  final BasicAccountData _self;
  final $Res Function(BasicAccountData) _then;

  /// Create a copy of BasicAccountData
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? allAccounts = null,
    Object? currentAccount = freezed,
  }) {
    return _then(_self.copyWith(
      allAccounts: null == allAccounts
          ? _self.allAccounts
          : allAccounts // ignore: cast_nullable_to_non_nullable
              as List<Account>,
      currentAccount: freezed == currentAccount
          ? _self.currentAccount
          : currentAccount // ignore: cast_nullable_to_non_nullable
              as AccountData?,
    ));
  }

  /// Create a copy of BasicAccountData
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $AccountDataCopyWith<$Res>? get currentAccount {
    if (_self.currentAccount == null) {
      return null;
    }

    return $AccountDataCopyWith<$Res>(_self.currentAccount!, (value) {
      return _then(_self.copyWith(currentAccount: value));
    });
  }
}

/// Adds pattern-matching-related methods to [BasicAccountData].
extension BasicAccountDataPatterns on BasicAccountData {
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
    TResult Function(_BasicAccountData value)? $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _BasicAccountData() when $default != null:
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
    TResult Function(_BasicAccountData value) $default,
  ) {
    final _that = this;
    switch (_that) {
      case _BasicAccountData():
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
    TResult? Function(_BasicAccountData value)? $default,
  ) {
    final _that = this;
    switch (_that) {
      case _BasicAccountData() when $default != null:
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
    TResult Function(List<Account> allAccounts, AccountData? currentAccount)?
        $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _BasicAccountData() when $default != null:
        return $default(_that.allAccounts, _that.currentAccount);
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
    TResult Function(List<Account> allAccounts, AccountData? currentAccount)
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _BasicAccountData():
        return $default(_that.allAccounts, _that.currentAccount);
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
    TResult? Function(List<Account> allAccounts, AccountData? currentAccount)?
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _BasicAccountData() when $default != null:
        return $default(_that.allAccounts, _that.currentAccount);
      case _:
        return null;
    }
  }
}

/// @nodoc

class _BasicAccountData implements BasicAccountData {
  const _BasicAccountData(
      {required final List<Account> allAccounts, required this.currentAccount})
      : _allAccounts = allAccounts;

  final List<Account> _allAccounts;
  @override
  List<Account> get allAccounts {
    if (_allAccounts is EqualUnmodifiableListView) return _allAccounts;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_allAccounts);
  }

  @override
  final AccountData? currentAccount;

  /// Create a copy of BasicAccountData
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  _$BasicAccountDataCopyWith<_BasicAccountData> get copyWith =>
      __$BasicAccountDataCopyWithImpl<_BasicAccountData>(this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _BasicAccountData &&
            const DeepCollectionEquality()
                .equals(other._allAccounts, _allAccounts) &&
            (identical(other.currentAccount, currentAccount) ||
                other.currentAccount == currentAccount));
  }

  @override
  int get hashCode => Object.hash(runtimeType,
      const DeepCollectionEquality().hash(_allAccounts), currentAccount);

  @override
  String toString() {
    return 'BasicAccountData(allAccounts: $allAccounts, currentAccount: $currentAccount)';
  }
}

/// @nodoc
abstract mixin class _$BasicAccountDataCopyWith<$Res>
    implements $BasicAccountDataCopyWith<$Res> {
  factory _$BasicAccountDataCopyWith(
          _BasicAccountData value, $Res Function(_BasicAccountData) _then) =
      __$BasicAccountDataCopyWithImpl;
  @override
  @useResult
  $Res call({List<Account> allAccounts, AccountData? currentAccount});

  @override
  $AccountDataCopyWith<$Res>? get currentAccount;
}

/// @nodoc
class __$BasicAccountDataCopyWithImpl<$Res>
    implements _$BasicAccountDataCopyWith<$Res> {
  __$BasicAccountDataCopyWithImpl(this._self, this._then);

  final _BasicAccountData _self;
  final $Res Function(_BasicAccountData) _then;

  /// Create a copy of BasicAccountData
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $Res call({
    Object? allAccounts = null,
    Object? currentAccount = freezed,
  }) {
    return _then(_BasicAccountData(
      allAccounts: null == allAccounts
          ? _self._allAccounts
          : allAccounts // ignore: cast_nullable_to_non_nullable
              as List<Account>,
      currentAccount: freezed == currentAccount
          ? _self.currentAccount
          : currentAccount // ignore: cast_nullable_to_non_nullable
              as AccountData?,
    ));
  }

  /// Create a copy of BasicAccountData
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $AccountDataCopyWith<$Res>? get currentAccount {
    if (_self.currentAccount == null) {
      return null;
    }

    return $AccountDataCopyWith<$Res>(_self.currentAccount!, (value) {
      return _then(_self.copyWith(currentAccount: value));
    });
  }
}

/// @nodoc
mixin _$AccountPageData {
  List<Account> get allAccounts;
  AccountData? get currentAccount;
  SyncProgressAccount? get syncState;

  /// Create a copy of AccountPageData
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $AccountPageDataCopyWith<AccountPageData> get copyWith =>
      _$AccountPageDataCopyWithImpl<AccountPageData>(
          this as AccountPageData, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is AccountPageData &&
            const DeepCollectionEquality()
                .equals(other.allAccounts, allAccounts) &&
            (identical(other.currentAccount, currentAccount) ||
                other.currentAccount == currentAccount) &&
            (identical(other.syncState, syncState) ||
                other.syncState == syncState));
  }

  @override
  int get hashCode => Object.hash(
      runtimeType,
      const DeepCollectionEquality().hash(allAccounts),
      currentAccount,
      syncState);

  @override
  String toString() {
    return 'AccountPageData(allAccounts: $allAccounts, currentAccount: $currentAccount, syncState: $syncState)';
  }
}

/// @nodoc
abstract mixin class $AccountPageDataCopyWith<$Res> {
  factory $AccountPageDataCopyWith(
          AccountPageData value, $Res Function(AccountPageData) _then) =
      _$AccountPageDataCopyWithImpl;
  @useResult
  $Res call(
      {List<Account> allAccounts,
      AccountData? currentAccount,
      SyncProgressAccount? syncState});

  $AccountDataCopyWith<$Res>? get currentAccount;
  $SyncProgressAccountCopyWith<$Res>? get syncState;
}

/// @nodoc
class _$AccountPageDataCopyWithImpl<$Res>
    implements $AccountPageDataCopyWith<$Res> {
  _$AccountPageDataCopyWithImpl(this._self, this._then);

  final AccountPageData _self;
  final $Res Function(AccountPageData) _then;

  /// Create a copy of AccountPageData
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? allAccounts = null,
    Object? currentAccount = freezed,
    Object? syncState = freezed,
  }) {
    return _then(_self.copyWith(
      allAccounts: null == allAccounts
          ? _self.allAccounts
          : allAccounts // ignore: cast_nullable_to_non_nullable
              as List<Account>,
      currentAccount: freezed == currentAccount
          ? _self.currentAccount
          : currentAccount // ignore: cast_nullable_to_non_nullable
              as AccountData?,
      syncState: freezed == syncState
          ? _self.syncState
          : syncState // ignore: cast_nullable_to_non_nullable
              as SyncProgressAccount?,
    ));
  }

  /// Create a copy of AccountPageData
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $AccountDataCopyWith<$Res>? get currentAccount {
    if (_self.currentAccount == null) {
      return null;
    }

    return $AccountDataCopyWith<$Res>(_self.currentAccount!, (value) {
      return _then(_self.copyWith(currentAccount: value));
    });
  }

  /// Create a copy of AccountPageData
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $SyncProgressAccountCopyWith<$Res>? get syncState {
    if (_self.syncState == null) {
      return null;
    }

    return $SyncProgressAccountCopyWith<$Res>(_self.syncState!, (value) {
      return _then(_self.copyWith(syncState: value));
    });
  }
}

/// Adds pattern-matching-related methods to [AccountPageData].
extension AccountPageDataPatterns on AccountPageData {
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
    TResult Function(_AccountPageData value)? $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _AccountPageData() when $default != null:
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
    TResult Function(_AccountPageData value) $default,
  ) {
    final _that = this;
    switch (_that) {
      case _AccountPageData():
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
    TResult? Function(_AccountPageData value)? $default,
  ) {
    final _that = this;
    switch (_that) {
      case _AccountPageData() when $default != null:
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
    TResult Function(List<Account> allAccounts, AccountData? currentAccount,
            SyncProgressAccount? syncState)?
        $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _AccountPageData() when $default != null:
        return $default(
            _that.allAccounts, _that.currentAccount, _that.syncState);
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
    TResult Function(List<Account> allAccounts, AccountData? currentAccount,
            SyncProgressAccount? syncState)
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _AccountPageData():
        return $default(
            _that.allAccounts, _that.currentAccount, _that.syncState);
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
    TResult? Function(List<Account> allAccounts, AccountData? currentAccount,
            SyncProgressAccount? syncState)?
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _AccountPageData() when $default != null:
        return $default(
            _that.allAccounts, _that.currentAccount, _that.syncState);
      case _:
        return null;
    }
  }
}

/// @nodoc

class _AccountPageData implements AccountPageData {
  const _AccountPageData(
      {required final List<Account> allAccounts,
      required this.currentAccount,
      required this.syncState})
      : _allAccounts = allAccounts;

  final List<Account> _allAccounts;
  @override
  List<Account> get allAccounts {
    if (_allAccounts is EqualUnmodifiableListView) return _allAccounts;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_allAccounts);
  }

  @override
  final AccountData? currentAccount;
  @override
  final SyncProgressAccount? syncState;

  /// Create a copy of AccountPageData
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  _$AccountPageDataCopyWith<_AccountPageData> get copyWith =>
      __$AccountPageDataCopyWithImpl<_AccountPageData>(this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _AccountPageData &&
            const DeepCollectionEquality()
                .equals(other._allAccounts, _allAccounts) &&
            (identical(other.currentAccount, currentAccount) ||
                other.currentAccount == currentAccount) &&
            (identical(other.syncState, syncState) ||
                other.syncState == syncState));
  }

  @override
  int get hashCode => Object.hash(
      runtimeType,
      const DeepCollectionEquality().hash(_allAccounts),
      currentAccount,
      syncState);

  @override
  String toString() {
    return 'AccountPageData(allAccounts: $allAccounts, currentAccount: $currentAccount, syncState: $syncState)';
  }
}

/// @nodoc
abstract mixin class _$AccountPageDataCopyWith<$Res>
    implements $AccountPageDataCopyWith<$Res> {
  factory _$AccountPageDataCopyWith(
          _AccountPageData value, $Res Function(_AccountPageData) _then) =
      __$AccountPageDataCopyWithImpl;
  @override
  @useResult
  $Res call(
      {List<Account> allAccounts,
      AccountData? currentAccount,
      SyncProgressAccount? syncState});

  @override
  $AccountDataCopyWith<$Res>? get currentAccount;
  @override
  $SyncProgressAccountCopyWith<$Res>? get syncState;
}

/// @nodoc
class __$AccountPageDataCopyWithImpl<$Res>
    implements _$AccountPageDataCopyWith<$Res> {
  __$AccountPageDataCopyWithImpl(this._self, this._then);

  final _AccountPageData _self;
  final $Res Function(_AccountPageData) _then;

  /// Create a copy of AccountPageData
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $Res call({
    Object? allAccounts = null,
    Object? currentAccount = freezed,
    Object? syncState = freezed,
  }) {
    return _then(_AccountPageData(
      allAccounts: null == allAccounts
          ? _self._allAccounts
          : allAccounts // ignore: cast_nullable_to_non_nullable
              as List<Account>,
      currentAccount: freezed == currentAccount
          ? _self.currentAccount
          : currentAccount // ignore: cast_nullable_to_non_nullable
              as AccountData?,
      syncState: freezed == syncState
          ? _self.syncState
          : syncState // ignore: cast_nullable_to_non_nullable
              as SyncProgressAccount?,
    ));
  }

  /// Create a copy of AccountPageData
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $AccountDataCopyWith<$Res>? get currentAccount {
    if (_self.currentAccount == null) {
      return null;
    }

    return $AccountDataCopyWith<$Res>(_self.currentAccount!, (value) {
      return _then(_self.copyWith(currentAccount: value));
    });
  }

  /// Create a copy of AccountPageData
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $SyncProgressAccountCopyWith<$Res>? get syncState {
    if (_self.syncState == null) {
      return null;
    }

    return $SyncProgressAccountCopyWith<$Res>(_self.syncState!, (value) {
      return _then(_self.copyWith(syncState: value));
    });
  }
}

/// @nodoc
mixin _$FullAccountPageData {
  List<Account> get allAccounts;
  AccountData? get currentAccount;
  SyncProgressAccount? get syncState;
  double? get price;
  MempoolState get mempool;

  /// Create a copy of FullAccountPageData
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $FullAccountPageDataCopyWith<FullAccountPageData> get copyWith =>
      _$FullAccountPageDataCopyWithImpl<FullAccountPageData>(
          this as FullAccountPageData, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is FullAccountPageData &&
            const DeepCollectionEquality()
                .equals(other.allAccounts, allAccounts) &&
            (identical(other.currentAccount, currentAccount) ||
                other.currentAccount == currentAccount) &&
            (identical(other.syncState, syncState) ||
                other.syncState == syncState) &&
            (identical(other.price, price) || other.price == price) &&
            (identical(other.mempool, mempool) || other.mempool == mempool));
  }

  @override
  int get hashCode => Object.hash(
      runtimeType,
      const DeepCollectionEquality().hash(allAccounts),
      currentAccount,
      syncState,
      price,
      mempool);

  @override
  String toString() {
    return 'FullAccountPageData(allAccounts: $allAccounts, currentAccount: $currentAccount, syncState: $syncState, price: $price, mempool: $mempool)';
  }
}

/// @nodoc
abstract mixin class $FullAccountPageDataCopyWith<$Res> {
  factory $FullAccountPageDataCopyWith(
          FullAccountPageData value, $Res Function(FullAccountPageData) _then) =
      _$FullAccountPageDataCopyWithImpl;
  @useResult
  $Res call(
      {List<Account> allAccounts,
      AccountData? currentAccount,
      SyncProgressAccount? syncState,
      double? price,
      MempoolState mempool});

  $AccountDataCopyWith<$Res>? get currentAccount;
  $SyncProgressAccountCopyWith<$Res>? get syncState;
  $MempoolStateCopyWith<$Res> get mempool;
}

/// @nodoc
class _$FullAccountPageDataCopyWithImpl<$Res>
    implements $FullAccountPageDataCopyWith<$Res> {
  _$FullAccountPageDataCopyWithImpl(this._self, this._then);

  final FullAccountPageData _self;
  final $Res Function(FullAccountPageData) _then;

  /// Create a copy of FullAccountPageData
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? allAccounts = null,
    Object? currentAccount = freezed,
    Object? syncState = freezed,
    Object? price = freezed,
    Object? mempool = null,
  }) {
    return _then(_self.copyWith(
      allAccounts: null == allAccounts
          ? _self.allAccounts
          : allAccounts // ignore: cast_nullable_to_non_nullable
              as List<Account>,
      currentAccount: freezed == currentAccount
          ? _self.currentAccount
          : currentAccount // ignore: cast_nullable_to_non_nullable
              as AccountData?,
      syncState: freezed == syncState
          ? _self.syncState
          : syncState // ignore: cast_nullable_to_non_nullable
              as SyncProgressAccount?,
      price: freezed == price
          ? _self.price
          : price // ignore: cast_nullable_to_non_nullable
              as double?,
      mempool: null == mempool
          ? _self.mempool
          : mempool // ignore: cast_nullable_to_non_nullable
              as MempoolState,
    ));
  }

  /// Create a copy of FullAccountPageData
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $AccountDataCopyWith<$Res>? get currentAccount {
    if (_self.currentAccount == null) {
      return null;
    }

    return $AccountDataCopyWith<$Res>(_self.currentAccount!, (value) {
      return _then(_self.copyWith(currentAccount: value));
    });
  }

  /// Create a copy of FullAccountPageData
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $SyncProgressAccountCopyWith<$Res>? get syncState {
    if (_self.syncState == null) {
      return null;
    }

    return $SyncProgressAccountCopyWith<$Res>(_self.syncState!, (value) {
      return _then(_self.copyWith(syncState: value));
    });
  }

  /// Create a copy of FullAccountPageData
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $MempoolStateCopyWith<$Res> get mempool {
    return $MempoolStateCopyWith<$Res>(_self.mempool, (value) {
      return _then(_self.copyWith(mempool: value));
    });
  }
}

/// Adds pattern-matching-related methods to [FullAccountPageData].
extension FullAccountPageDataPatterns on FullAccountPageData {
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
    TResult Function(_FullAccountPageData value)? $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _FullAccountPageData() when $default != null:
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
    TResult Function(_FullAccountPageData value) $default,
  ) {
    final _that = this;
    switch (_that) {
      case _FullAccountPageData():
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
    TResult? Function(_FullAccountPageData value)? $default,
  ) {
    final _that = this;
    switch (_that) {
      case _FullAccountPageData() when $default != null:
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
            List<Account> allAccounts,
            AccountData? currentAccount,
            SyncProgressAccount? syncState,
            double? price,
            MempoolState mempool)?
        $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _FullAccountPageData() when $default != null:
        return $default(_that.allAccounts, _that.currentAccount,
            _that.syncState, _that.price, _that.mempool);
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
    TResult Function(List<Account> allAccounts, AccountData? currentAccount,
            SyncProgressAccount? syncState, double? price, MempoolState mempool)
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _FullAccountPageData():
        return $default(_that.allAccounts, _that.currentAccount,
            _that.syncState, _that.price, _that.mempool);
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
            List<Account> allAccounts,
            AccountData? currentAccount,
            SyncProgressAccount? syncState,
            double? price,
            MempoolState mempool)?
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _FullAccountPageData() when $default != null:
        return $default(_that.allAccounts, _that.currentAccount,
            _that.syncState, _that.price, _that.mempool);
      case _:
        return null;
    }
  }
}

/// @nodoc

class _FullAccountPageData implements FullAccountPageData {
  const _FullAccountPageData(
      {required final List<Account> allAccounts,
      required this.currentAccount,
      required this.syncState,
      required this.price,
      required this.mempool})
      : _allAccounts = allAccounts;

  final List<Account> _allAccounts;
  @override
  List<Account> get allAccounts {
    if (_allAccounts is EqualUnmodifiableListView) return _allAccounts;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_allAccounts);
  }

  @override
  final AccountData? currentAccount;
  @override
  final SyncProgressAccount? syncState;
  @override
  final double? price;
  @override
  final MempoolState mempool;

  /// Create a copy of FullAccountPageData
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  _$FullAccountPageDataCopyWith<_FullAccountPageData> get copyWith =>
      __$FullAccountPageDataCopyWithImpl<_FullAccountPageData>(
          this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _FullAccountPageData &&
            const DeepCollectionEquality()
                .equals(other._allAccounts, _allAccounts) &&
            (identical(other.currentAccount, currentAccount) ||
                other.currentAccount == currentAccount) &&
            (identical(other.syncState, syncState) ||
                other.syncState == syncState) &&
            (identical(other.price, price) || other.price == price) &&
            (identical(other.mempool, mempool) || other.mempool == mempool));
  }

  @override
  int get hashCode => Object.hash(
      runtimeType,
      const DeepCollectionEquality().hash(_allAccounts),
      currentAccount,
      syncState,
      price,
      mempool);

  @override
  String toString() {
    return 'FullAccountPageData(allAccounts: $allAccounts, currentAccount: $currentAccount, syncState: $syncState, price: $price, mempool: $mempool)';
  }
}

/// @nodoc
abstract mixin class _$FullAccountPageDataCopyWith<$Res>
    implements $FullAccountPageDataCopyWith<$Res> {
  factory _$FullAccountPageDataCopyWith(_FullAccountPageData value,
          $Res Function(_FullAccountPageData) _then) =
      __$FullAccountPageDataCopyWithImpl;
  @override
  @useResult
  $Res call(
      {List<Account> allAccounts,
      AccountData? currentAccount,
      SyncProgressAccount? syncState,
      double? price,
      MempoolState mempool});

  @override
  $AccountDataCopyWith<$Res>? get currentAccount;
  @override
  $SyncProgressAccountCopyWith<$Res>? get syncState;
  @override
  $MempoolStateCopyWith<$Res> get mempool;
}

/// @nodoc
class __$FullAccountPageDataCopyWithImpl<$Res>
    implements _$FullAccountPageDataCopyWith<$Res> {
  __$FullAccountPageDataCopyWithImpl(this._self, this._then);

  final _FullAccountPageData _self;
  final $Res Function(_FullAccountPageData) _then;

  /// Create a copy of FullAccountPageData
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $Res call({
    Object? allAccounts = null,
    Object? currentAccount = freezed,
    Object? syncState = freezed,
    Object? price = freezed,
    Object? mempool = null,
  }) {
    return _then(_FullAccountPageData(
      allAccounts: null == allAccounts
          ? _self._allAccounts
          : allAccounts // ignore: cast_nullable_to_non_nullable
              as List<Account>,
      currentAccount: freezed == currentAccount
          ? _self.currentAccount
          : currentAccount // ignore: cast_nullable_to_non_nullable
              as AccountData?,
      syncState: freezed == syncState
          ? _self.syncState
          : syncState // ignore: cast_nullable_to_non_nullable
              as SyncProgressAccount?,
      price: freezed == price
          ? _self.price
          : price // ignore: cast_nullable_to_non_nullable
              as double?,
      mempool: null == mempool
          ? _self.mempool
          : mempool // ignore: cast_nullable_to_non_nullable
              as MempoolState,
    ));
  }

  /// Create a copy of FullAccountPageData
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $AccountDataCopyWith<$Res>? get currentAccount {
    if (_self.currentAccount == null) {
      return null;
    }

    return $AccountDataCopyWith<$Res>(_self.currentAccount!, (value) {
      return _then(_self.copyWith(currentAccount: value));
    });
  }

  /// Create a copy of FullAccountPageData
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $SyncProgressAccountCopyWith<$Res>? get syncState {
    if (_self.syncState == null) {
      return null;
    }

    return $SyncProgressAccountCopyWith<$Res>(_self.syncState!, (value) {
      return _then(_self.copyWith(syncState: value));
    });
  }

  /// Create a copy of FullAccountPageData
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $MempoolStateCopyWith<$Res> get mempool {
    return $MempoolStateCopyWith<$Res>(_self.mempool, (value) {
      return _then(_self.copyWith(mempool: value));
    });
  }
}

/// @nodoc
mixin _$QRSettings {
  bool get enabled;
  double get size;
  int get ecLevel;
  int get delay;
  int get repair;

  /// Create a copy of QRSettings
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $QRSettingsCopyWith<QRSettings> get copyWith =>
      _$QRSettingsCopyWithImpl<QRSettings>(this as QRSettings, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is QRSettings &&
            (identical(other.enabled, enabled) || other.enabled == enabled) &&
            (identical(other.size, size) || other.size == size) &&
            (identical(other.ecLevel, ecLevel) || other.ecLevel == ecLevel) &&
            (identical(other.delay, delay) || other.delay == delay) &&
            (identical(other.repair, repair) || other.repair == repair));
  }

  @override
  int get hashCode =>
      Object.hash(runtimeType, enabled, size, ecLevel, delay, repair);

  @override
  String toString() {
    return 'QRSettings(enabled: $enabled, size: $size, ecLevel: $ecLevel, delay: $delay, repair: $repair)';
  }
}

/// @nodoc
abstract mixin class $QRSettingsCopyWith<$Res> {
  factory $QRSettingsCopyWith(
          QRSettings value, $Res Function(QRSettings) _then) =
      _$QRSettingsCopyWithImpl;
  @useResult
  $Res call({bool enabled, double size, int ecLevel, int delay, int repair});
}

/// @nodoc
class _$QRSettingsCopyWithImpl<$Res> implements $QRSettingsCopyWith<$Res> {
  _$QRSettingsCopyWithImpl(this._self, this._then);

  final QRSettings _self;
  final $Res Function(QRSettings) _then;

  /// Create a copy of QRSettings
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? enabled = null,
    Object? size = null,
    Object? ecLevel = null,
    Object? delay = null,
    Object? repair = null,
  }) {
    return _then(_self.copyWith(
      enabled: null == enabled
          ? _self.enabled
          : enabled // ignore: cast_nullable_to_non_nullable
              as bool,
      size: null == size
          ? _self.size
          : size // ignore: cast_nullable_to_non_nullable
              as double,
      ecLevel: null == ecLevel
          ? _self.ecLevel
          : ecLevel // ignore: cast_nullable_to_non_nullable
              as int,
      delay: null == delay
          ? _self.delay
          : delay // ignore: cast_nullable_to_non_nullable
              as int,
      repair: null == repair
          ? _self.repair
          : repair // ignore: cast_nullable_to_non_nullable
              as int,
    ));
  }
}

/// Adds pattern-matching-related methods to [QRSettings].
extension QRSettingsPatterns on QRSettings {
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
    TResult Function(_QRSettings value)? $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _QRSettings() when $default != null:
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
    TResult Function(_QRSettings value) $default,
  ) {
    final _that = this;
    switch (_that) {
      case _QRSettings():
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
    TResult? Function(_QRSettings value)? $default,
  ) {
    final _that = this;
    switch (_that) {
      case _QRSettings() when $default != null:
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
            bool enabled, double size, int ecLevel, int delay, int repair)?
        $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _QRSettings() when $default != null:
        return $default(_that.enabled, _that.size, _that.ecLevel, _that.delay,
            _that.repair);
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
            bool enabled, double size, int ecLevel, int delay, int repair)
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _QRSettings():
        return $default(_that.enabled, _that.size, _that.ecLevel, _that.delay,
            _that.repair);
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
            bool enabled, double size, int ecLevel, int delay, int repair)?
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _QRSettings() when $default != null:
        return $default(_that.enabled, _that.size, _that.ecLevel, _that.delay,
            _that.repair);
      case _:
        return null;
    }
  }
}

/// @nodoc

class _QRSettings implements QRSettings {
  _QRSettings(
      {required this.enabled,
      required this.size,
      required this.ecLevel,
      required this.delay,
      required this.repair});

  @override
  final bool enabled;
  @override
  final double size;
  @override
  final int ecLevel;
  @override
  final int delay;
  @override
  final int repair;

  /// Create a copy of QRSettings
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  _$QRSettingsCopyWith<_QRSettings> get copyWith =>
      __$QRSettingsCopyWithImpl<_QRSettings>(this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _QRSettings &&
            (identical(other.enabled, enabled) || other.enabled == enabled) &&
            (identical(other.size, size) || other.size == size) &&
            (identical(other.ecLevel, ecLevel) || other.ecLevel == ecLevel) &&
            (identical(other.delay, delay) || other.delay == delay) &&
            (identical(other.repair, repair) || other.repair == repair));
  }

  @override
  int get hashCode =>
      Object.hash(runtimeType, enabled, size, ecLevel, delay, repair);

  @override
  String toString() {
    return 'QRSettings(enabled: $enabled, size: $size, ecLevel: $ecLevel, delay: $delay, repair: $repair)';
  }
}

/// @nodoc
abstract mixin class _$QRSettingsCopyWith<$Res>
    implements $QRSettingsCopyWith<$Res> {
  factory _$QRSettingsCopyWith(
          _QRSettings value, $Res Function(_QRSettings) _then) =
      __$QRSettingsCopyWithImpl;
  @override
  @useResult
  $Res call({bool enabled, double size, int ecLevel, int delay, int repair});
}

/// @nodoc
class __$QRSettingsCopyWithImpl<$Res> implements _$QRSettingsCopyWith<$Res> {
  __$QRSettingsCopyWithImpl(this._self, this._then);

  final _QRSettings _self;
  final $Res Function(_QRSettings) _then;

  /// Create a copy of QRSettings
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $Res call({
    Object? enabled = null,
    Object? size = null,
    Object? ecLevel = null,
    Object? delay = null,
    Object? repair = null,
  }) {
    return _then(_QRSettings(
      enabled: null == enabled
          ? _self.enabled
          : enabled // ignore: cast_nullable_to_non_nullable
              as bool,
      size: null == size
          ? _self.size
          : size // ignore: cast_nullable_to_non_nullable
              as double,
      ecLevel: null == ecLevel
          ? _self.ecLevel
          : ecLevel // ignore: cast_nullable_to_non_nullable
              as int,
      delay: null == delay
          ? _self.delay
          : delay // ignore: cast_nullable_to_non_nullable
              as int,
      repair: null == repair
          ? _self.repair
          : repair // ignore: cast_nullable_to_non_nullable
              as int,
    ));
  }
}

// dart format on
