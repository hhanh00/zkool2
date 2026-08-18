// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'voting_workflow.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;

/// @nodoc
mixin _$VoteRoundInput {
  String get roundId;
  int get coin;
  int get account;
  String get dbFilepath;
  String get url;
  int get serverType;
  int get transport;
  String get proxy;
  String get chainUrl;
  String get pirServerUrl;
  VotingPirLayout? get pirLayout;
  String? get roundParamsJson;
  String? get roundName;
  int? get maxRealNotesPerBundle;
  String? get lightwalletdUrl;
  String get voteNodeUrl;
  BigInt get ceremonyStart;
  BigInt? get voteEnd;
  List<String> get shareServerUrls;
  bool get singleShare;

  /// Create a copy of VoteRoundInput
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $VoteRoundInputCopyWith<VoteRoundInput> get copyWith =>
      _$VoteRoundInputCopyWithImpl<VoteRoundInput>(
          this as VoteRoundInput, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is VoteRoundInput &&
            (identical(other.roundId, roundId) || other.roundId == roundId) &&
            (identical(other.coin, coin) || other.coin == coin) &&
            (identical(other.account, account) || other.account == account) &&
            (identical(other.dbFilepath, dbFilepath) ||
                other.dbFilepath == dbFilepath) &&
            (identical(other.url, url) || other.url == url) &&
            (identical(other.serverType, serverType) ||
                other.serverType == serverType) &&
            (identical(other.transport, transport) ||
                other.transport == transport) &&
            (identical(other.proxy, proxy) || other.proxy == proxy) &&
            (identical(other.chainUrl, chainUrl) ||
                other.chainUrl == chainUrl) &&
            (identical(other.pirServerUrl, pirServerUrl) ||
                other.pirServerUrl == pirServerUrl) &&
            (identical(other.pirLayout, pirLayout) ||
                other.pirLayout == pirLayout) &&
            (identical(other.roundParamsJson, roundParamsJson) ||
                other.roundParamsJson == roundParamsJson) &&
            (identical(other.roundName, roundName) ||
                other.roundName == roundName) &&
            (identical(other.maxRealNotesPerBundle, maxRealNotesPerBundle) ||
                other.maxRealNotesPerBundle == maxRealNotesPerBundle) &&
            (identical(other.lightwalletdUrl, lightwalletdUrl) ||
                other.lightwalletdUrl == lightwalletdUrl) &&
            (identical(other.voteNodeUrl, voteNodeUrl) ||
                other.voteNodeUrl == voteNodeUrl) &&
            (identical(other.ceremonyStart, ceremonyStart) ||
                other.ceremonyStart == ceremonyStart) &&
            (identical(other.voteEnd, voteEnd) || other.voteEnd == voteEnd) &&
            const DeepCollectionEquality()
                .equals(other.shareServerUrls, shareServerUrls) &&
            (identical(other.singleShare, singleShare) ||
                other.singleShare == singleShare));
  }

  @override
  int get hashCode => Object.hashAll([
        runtimeType,
        roundId,
        coin,
        account,
        dbFilepath,
        url,
        serverType,
        transport,
        proxy,
        chainUrl,
        pirServerUrl,
        pirLayout,
        roundParamsJson,
        roundName,
        maxRealNotesPerBundle,
        lightwalletdUrl,
        voteNodeUrl,
        ceremonyStart,
        voteEnd,
        const DeepCollectionEquality().hash(shareServerUrls),
        singleShare
      ]);

  @override
  String toString() {
    return 'VoteRoundInput(roundId: $roundId, coin: $coin, account: $account, dbFilepath: $dbFilepath, url: $url, serverType: $serverType, transport: $transport, proxy: $proxy, chainUrl: $chainUrl, pirServerUrl: $pirServerUrl, pirLayout: $pirLayout, roundParamsJson: $roundParamsJson, roundName: $roundName, maxRealNotesPerBundle: $maxRealNotesPerBundle, lightwalletdUrl: $lightwalletdUrl, voteNodeUrl: $voteNodeUrl, ceremonyStart: $ceremonyStart, voteEnd: $voteEnd, shareServerUrls: $shareServerUrls, singleShare: $singleShare)';
  }
}

/// @nodoc
abstract mixin class $VoteRoundInputCopyWith<$Res> {
  factory $VoteRoundInputCopyWith(
          VoteRoundInput value, $Res Function(VoteRoundInput) _then) =
      _$VoteRoundInputCopyWithImpl;
  @useResult
  $Res call(
      {String roundId,
      int coin,
      int account,
      String dbFilepath,
      String url,
      int serverType,
      int transport,
      String proxy,
      String chainUrl,
      String pirServerUrl,
      VotingPirLayout? pirLayout,
      String? roundParamsJson,
      String? roundName,
      int? maxRealNotesPerBundle,
      String? lightwalletdUrl,
      String voteNodeUrl,
      BigInt ceremonyStart,
      BigInt? voteEnd,
      List<String> shareServerUrls,
      bool singleShare});

  $VotingPirLayoutCopyWith<$Res>? get pirLayout;
}

/// @nodoc
class _$VoteRoundInputCopyWithImpl<$Res>
    implements $VoteRoundInputCopyWith<$Res> {
  _$VoteRoundInputCopyWithImpl(this._self, this._then);

  final VoteRoundInput _self;
  final $Res Function(VoteRoundInput) _then;

  /// Create a copy of VoteRoundInput
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? roundId = null,
    Object? coin = null,
    Object? account = null,
    Object? dbFilepath = null,
    Object? url = null,
    Object? serverType = null,
    Object? transport = null,
    Object? proxy = null,
    Object? chainUrl = null,
    Object? pirServerUrl = null,
    Object? pirLayout = freezed,
    Object? roundParamsJson = freezed,
    Object? roundName = freezed,
    Object? maxRealNotesPerBundle = freezed,
    Object? lightwalletdUrl = freezed,
    Object? voteNodeUrl = null,
    Object? ceremonyStart = null,
    Object? voteEnd = freezed,
    Object? shareServerUrls = null,
    Object? singleShare = null,
  }) {
    return _then(_self.copyWith(
      roundId: null == roundId
          ? _self.roundId
          : roundId // ignore: cast_nullable_to_non_nullable
              as String,
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
      transport: null == transport
          ? _self.transport
          : transport // ignore: cast_nullable_to_non_nullable
              as int,
      proxy: null == proxy
          ? _self.proxy
          : proxy // ignore: cast_nullable_to_non_nullable
              as String,
      chainUrl: null == chainUrl
          ? _self.chainUrl
          : chainUrl // ignore: cast_nullable_to_non_nullable
              as String,
      pirServerUrl: null == pirServerUrl
          ? _self.pirServerUrl
          : pirServerUrl // ignore: cast_nullable_to_non_nullable
              as String,
      pirLayout: freezed == pirLayout
          ? _self.pirLayout
          : pirLayout // ignore: cast_nullable_to_non_nullable
              as VotingPirLayout?,
      roundParamsJson: freezed == roundParamsJson
          ? _self.roundParamsJson
          : roundParamsJson // ignore: cast_nullable_to_non_nullable
              as String?,
      roundName: freezed == roundName
          ? _self.roundName
          : roundName // ignore: cast_nullable_to_non_nullable
              as String?,
      maxRealNotesPerBundle: freezed == maxRealNotesPerBundle
          ? _self.maxRealNotesPerBundle
          : maxRealNotesPerBundle // ignore: cast_nullable_to_non_nullable
              as int?,
      lightwalletdUrl: freezed == lightwalletdUrl
          ? _self.lightwalletdUrl
          : lightwalletdUrl // ignore: cast_nullable_to_non_nullable
              as String?,
      voteNodeUrl: null == voteNodeUrl
          ? _self.voteNodeUrl
          : voteNodeUrl // ignore: cast_nullable_to_non_nullable
              as String,
      ceremonyStart: null == ceremonyStart
          ? _self.ceremonyStart
          : ceremonyStart // ignore: cast_nullable_to_non_nullable
              as BigInt,
      voteEnd: freezed == voteEnd
          ? _self.voteEnd
          : voteEnd // ignore: cast_nullable_to_non_nullable
              as BigInt?,
      shareServerUrls: null == shareServerUrls
          ? _self.shareServerUrls
          : shareServerUrls // ignore: cast_nullable_to_non_nullable
              as List<String>,
      singleShare: null == singleShare
          ? _self.singleShare
          : singleShare // ignore: cast_nullable_to_non_nullable
              as bool,
    ));
  }

  /// Create a copy of VoteRoundInput
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $VotingPirLayoutCopyWith<$Res>? get pirLayout {
    if (_self.pirLayout == null) {
      return null;
    }

    return $VotingPirLayoutCopyWith<$Res>(_self.pirLayout!, (value) {
      return _then(_self.copyWith(pirLayout: value));
    });
  }
}

/// Adds pattern-matching-related methods to [VoteRoundInput].
extension VoteRoundInputPatterns on VoteRoundInput {
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
    TResult Function(_VoteRoundInput value)? $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _VoteRoundInput() when $default != null:
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
    TResult Function(_VoteRoundInput value) $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VoteRoundInput():
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
    TResult? Function(_VoteRoundInput value)? $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VoteRoundInput() when $default != null:
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
            int coin,
            int account,
            String dbFilepath,
            String url,
            int serverType,
            int transport,
            String proxy,
            String chainUrl,
            String pirServerUrl,
            VotingPirLayout? pirLayout,
            String? roundParamsJson,
            String? roundName,
            int? maxRealNotesPerBundle,
            String? lightwalletdUrl,
            String voteNodeUrl,
            BigInt ceremonyStart,
            BigInt? voteEnd,
            List<String> shareServerUrls,
            bool singleShare)?
        $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _VoteRoundInput() when $default != null:
        return $default(
            _that.roundId,
            _that.coin,
            _that.account,
            _that.dbFilepath,
            _that.url,
            _that.serverType,
            _that.transport,
            _that.proxy,
            _that.chainUrl,
            _that.pirServerUrl,
            _that.pirLayout,
            _that.roundParamsJson,
            _that.roundName,
            _that.maxRealNotesPerBundle,
            _that.lightwalletdUrl,
            _that.voteNodeUrl,
            _that.ceremonyStart,
            _that.voteEnd,
            _that.shareServerUrls,
            _that.singleShare);
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
            int coin,
            int account,
            String dbFilepath,
            String url,
            int serverType,
            int transport,
            String proxy,
            String chainUrl,
            String pirServerUrl,
            VotingPirLayout? pirLayout,
            String? roundParamsJson,
            String? roundName,
            int? maxRealNotesPerBundle,
            String? lightwalletdUrl,
            String voteNodeUrl,
            BigInt ceremonyStart,
            BigInt? voteEnd,
            List<String> shareServerUrls,
            bool singleShare)
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VoteRoundInput():
        return $default(
            _that.roundId,
            _that.coin,
            _that.account,
            _that.dbFilepath,
            _that.url,
            _that.serverType,
            _that.transport,
            _that.proxy,
            _that.chainUrl,
            _that.pirServerUrl,
            _that.pirLayout,
            _that.roundParamsJson,
            _that.roundName,
            _that.maxRealNotesPerBundle,
            _that.lightwalletdUrl,
            _that.voteNodeUrl,
            _that.ceremonyStart,
            _that.voteEnd,
            _that.shareServerUrls,
            _that.singleShare);
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
            int coin,
            int account,
            String dbFilepath,
            String url,
            int serverType,
            int transport,
            String proxy,
            String chainUrl,
            String pirServerUrl,
            VotingPirLayout? pirLayout,
            String? roundParamsJson,
            String? roundName,
            int? maxRealNotesPerBundle,
            String? lightwalletdUrl,
            String voteNodeUrl,
            BigInt ceremonyStart,
            BigInt? voteEnd,
            List<String> shareServerUrls,
            bool singleShare)?
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VoteRoundInput() when $default != null:
        return $default(
            _that.roundId,
            _that.coin,
            _that.account,
            _that.dbFilepath,
            _that.url,
            _that.serverType,
            _that.transport,
            _that.proxy,
            _that.chainUrl,
            _that.pirServerUrl,
            _that.pirLayout,
            _that.roundParamsJson,
            _that.roundName,
            _that.maxRealNotesPerBundle,
            _that.lightwalletdUrl,
            _that.voteNodeUrl,
            _that.ceremonyStart,
            _that.voteEnd,
            _that.shareServerUrls,
            _that.singleShare);
      case _:
        return null;
    }
  }
}

/// @nodoc

class _VoteRoundInput implements VoteRoundInput {
  const _VoteRoundInput(
      {required this.roundId,
      required this.coin,
      required this.account,
      required this.dbFilepath,
      required this.url,
      required this.serverType,
      required this.transport,
      required this.proxy,
      required this.chainUrl,
      required this.pirServerUrl,
      this.pirLayout,
      this.roundParamsJson,
      this.roundName,
      this.maxRealNotesPerBundle,
      this.lightwalletdUrl,
      required this.voteNodeUrl,
      required this.ceremonyStart,
      this.voteEnd,
      required final List<String> shareServerUrls,
      required this.singleShare})
      : _shareServerUrls = shareServerUrls;

  @override
  final String roundId;
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
  final int transport;
  @override
  final String proxy;
  @override
  final String chainUrl;
  @override
  final String pirServerUrl;
  @override
  final VotingPirLayout? pirLayout;
  @override
  final String? roundParamsJson;
  @override
  final String? roundName;
  @override
  final int? maxRealNotesPerBundle;
  @override
  final String? lightwalletdUrl;
  @override
  final String voteNodeUrl;
  @override
  final BigInt ceremonyStart;
  @override
  final BigInt? voteEnd;
  final List<String> _shareServerUrls;
  @override
  List<String> get shareServerUrls {
    if (_shareServerUrls is EqualUnmodifiableListView) return _shareServerUrls;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_shareServerUrls);
  }

  @override
  final bool singleShare;

  /// Create a copy of VoteRoundInput
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  _$VoteRoundInputCopyWith<_VoteRoundInput> get copyWith =>
      __$VoteRoundInputCopyWithImpl<_VoteRoundInput>(this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _VoteRoundInput &&
            (identical(other.roundId, roundId) || other.roundId == roundId) &&
            (identical(other.coin, coin) || other.coin == coin) &&
            (identical(other.account, account) || other.account == account) &&
            (identical(other.dbFilepath, dbFilepath) ||
                other.dbFilepath == dbFilepath) &&
            (identical(other.url, url) || other.url == url) &&
            (identical(other.serverType, serverType) ||
                other.serverType == serverType) &&
            (identical(other.transport, transport) ||
                other.transport == transport) &&
            (identical(other.proxy, proxy) || other.proxy == proxy) &&
            (identical(other.chainUrl, chainUrl) ||
                other.chainUrl == chainUrl) &&
            (identical(other.pirServerUrl, pirServerUrl) ||
                other.pirServerUrl == pirServerUrl) &&
            (identical(other.pirLayout, pirLayout) ||
                other.pirLayout == pirLayout) &&
            (identical(other.roundParamsJson, roundParamsJson) ||
                other.roundParamsJson == roundParamsJson) &&
            (identical(other.roundName, roundName) ||
                other.roundName == roundName) &&
            (identical(other.maxRealNotesPerBundle, maxRealNotesPerBundle) ||
                other.maxRealNotesPerBundle == maxRealNotesPerBundle) &&
            (identical(other.lightwalletdUrl, lightwalletdUrl) ||
                other.lightwalletdUrl == lightwalletdUrl) &&
            (identical(other.voteNodeUrl, voteNodeUrl) ||
                other.voteNodeUrl == voteNodeUrl) &&
            (identical(other.ceremonyStart, ceremonyStart) ||
                other.ceremonyStart == ceremonyStart) &&
            (identical(other.voteEnd, voteEnd) || other.voteEnd == voteEnd) &&
            const DeepCollectionEquality()
                .equals(other._shareServerUrls, _shareServerUrls) &&
            (identical(other.singleShare, singleShare) ||
                other.singleShare == singleShare));
  }

  @override
  int get hashCode => Object.hashAll([
        runtimeType,
        roundId,
        coin,
        account,
        dbFilepath,
        url,
        serverType,
        transport,
        proxy,
        chainUrl,
        pirServerUrl,
        pirLayout,
        roundParamsJson,
        roundName,
        maxRealNotesPerBundle,
        lightwalletdUrl,
        voteNodeUrl,
        ceremonyStart,
        voteEnd,
        const DeepCollectionEquality().hash(_shareServerUrls),
        singleShare
      ]);

  @override
  String toString() {
    return 'VoteRoundInput(roundId: $roundId, coin: $coin, account: $account, dbFilepath: $dbFilepath, url: $url, serverType: $serverType, transport: $transport, proxy: $proxy, chainUrl: $chainUrl, pirServerUrl: $pirServerUrl, pirLayout: $pirLayout, roundParamsJson: $roundParamsJson, roundName: $roundName, maxRealNotesPerBundle: $maxRealNotesPerBundle, lightwalletdUrl: $lightwalletdUrl, voteNodeUrl: $voteNodeUrl, ceremonyStart: $ceremonyStart, voteEnd: $voteEnd, shareServerUrls: $shareServerUrls, singleShare: $singleShare)';
  }
}

/// @nodoc
abstract mixin class _$VoteRoundInputCopyWith<$Res>
    implements $VoteRoundInputCopyWith<$Res> {
  factory _$VoteRoundInputCopyWith(
          _VoteRoundInput value, $Res Function(_VoteRoundInput) _then) =
      __$VoteRoundInputCopyWithImpl;
  @override
  @useResult
  $Res call(
      {String roundId,
      int coin,
      int account,
      String dbFilepath,
      String url,
      int serverType,
      int transport,
      String proxy,
      String chainUrl,
      String pirServerUrl,
      VotingPirLayout? pirLayout,
      String? roundParamsJson,
      String? roundName,
      int? maxRealNotesPerBundle,
      String? lightwalletdUrl,
      String voteNodeUrl,
      BigInt ceremonyStart,
      BigInt? voteEnd,
      List<String> shareServerUrls,
      bool singleShare});

  @override
  $VotingPirLayoutCopyWith<$Res>? get pirLayout;
}

/// @nodoc
class __$VoteRoundInputCopyWithImpl<$Res>
    implements _$VoteRoundInputCopyWith<$Res> {
  __$VoteRoundInputCopyWithImpl(this._self, this._then);

  final _VoteRoundInput _self;
  final $Res Function(_VoteRoundInput) _then;

  /// Create a copy of VoteRoundInput
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $Res call({
    Object? roundId = null,
    Object? coin = null,
    Object? account = null,
    Object? dbFilepath = null,
    Object? url = null,
    Object? serverType = null,
    Object? transport = null,
    Object? proxy = null,
    Object? chainUrl = null,
    Object? pirServerUrl = null,
    Object? pirLayout = freezed,
    Object? roundParamsJson = freezed,
    Object? roundName = freezed,
    Object? maxRealNotesPerBundle = freezed,
    Object? lightwalletdUrl = freezed,
    Object? voteNodeUrl = null,
    Object? ceremonyStart = null,
    Object? voteEnd = freezed,
    Object? shareServerUrls = null,
    Object? singleShare = null,
  }) {
    return _then(_VoteRoundInput(
      roundId: null == roundId
          ? _self.roundId
          : roundId // ignore: cast_nullable_to_non_nullable
              as String,
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
      transport: null == transport
          ? _self.transport
          : transport // ignore: cast_nullable_to_non_nullable
              as int,
      proxy: null == proxy
          ? _self.proxy
          : proxy // ignore: cast_nullable_to_non_nullable
              as String,
      chainUrl: null == chainUrl
          ? _self.chainUrl
          : chainUrl // ignore: cast_nullable_to_non_nullable
              as String,
      pirServerUrl: null == pirServerUrl
          ? _self.pirServerUrl
          : pirServerUrl // ignore: cast_nullable_to_non_nullable
              as String,
      pirLayout: freezed == pirLayout
          ? _self.pirLayout
          : pirLayout // ignore: cast_nullable_to_non_nullable
              as VotingPirLayout?,
      roundParamsJson: freezed == roundParamsJson
          ? _self.roundParamsJson
          : roundParamsJson // ignore: cast_nullable_to_non_nullable
              as String?,
      roundName: freezed == roundName
          ? _self.roundName
          : roundName // ignore: cast_nullable_to_non_nullable
              as String?,
      maxRealNotesPerBundle: freezed == maxRealNotesPerBundle
          ? _self.maxRealNotesPerBundle
          : maxRealNotesPerBundle // ignore: cast_nullable_to_non_nullable
              as int?,
      lightwalletdUrl: freezed == lightwalletdUrl
          ? _self.lightwalletdUrl
          : lightwalletdUrl // ignore: cast_nullable_to_non_nullable
              as String?,
      voteNodeUrl: null == voteNodeUrl
          ? _self.voteNodeUrl
          : voteNodeUrl // ignore: cast_nullable_to_non_nullable
              as String,
      ceremonyStart: null == ceremonyStart
          ? _self.ceremonyStart
          : ceremonyStart // ignore: cast_nullable_to_non_nullable
              as BigInt,
      voteEnd: freezed == voteEnd
          ? _self.voteEnd
          : voteEnd // ignore: cast_nullable_to_non_nullable
              as BigInt?,
      shareServerUrls: null == shareServerUrls
          ? _self._shareServerUrls
          : shareServerUrls // ignore: cast_nullable_to_non_nullable
              as List<String>,
      singleShare: null == singleShare
          ? _self.singleShare
          : singleShare // ignore: cast_nullable_to_non_nullable
              as bool,
    ));
  }

  /// Create a copy of VoteRoundInput
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $VotingPirLayoutCopyWith<$Res>? get pirLayout {
    if (_self.pirLayout == null) {
      return null;
    }

    return $VotingPirLayoutCopyWith<$Res>(_self.pirLayout!, (value) {
      return _then(_self.copyWith(pirLayout: value));
    });
  }
}

/// @nodoc
mixin _$VotingWorkflowStatus {
  String get roundId;
  String get status;
  String get stage;
  double? get progress;
  String? get error;
  String? get doneLabel;
  String? get txHash;
  BigInt? get confirmHeight;
  BigInt? get eligibleWeightZatoshi;

  /// Create a copy of VotingWorkflowStatus
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $VotingWorkflowStatusCopyWith<VotingWorkflowStatus> get copyWith =>
      _$VotingWorkflowStatusCopyWithImpl<VotingWorkflowStatus>(
          this as VotingWorkflowStatus, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is VotingWorkflowStatus &&
            (identical(other.roundId, roundId) || other.roundId == roundId) &&
            (identical(other.status, status) || other.status == status) &&
            (identical(other.stage, stage) || other.stage == stage) &&
            (identical(other.progress, progress) ||
                other.progress == progress) &&
            (identical(other.error, error) || other.error == error) &&
            (identical(other.doneLabel, doneLabel) ||
                other.doneLabel == doneLabel) &&
            (identical(other.txHash, txHash) || other.txHash == txHash) &&
            (identical(other.confirmHeight, confirmHeight) ||
                other.confirmHeight == confirmHeight) &&
            (identical(other.eligibleWeightZatoshi, eligibleWeightZatoshi) ||
                other.eligibleWeightZatoshi == eligibleWeightZatoshi));
  }

  @override
  int get hashCode => Object.hash(runtimeType, roundId, status, stage, progress,
      error, doneLabel, txHash, confirmHeight, eligibleWeightZatoshi);

  @override
  String toString() {
    return 'VotingWorkflowStatus(roundId: $roundId, status: $status, stage: $stage, progress: $progress, error: $error, doneLabel: $doneLabel, txHash: $txHash, confirmHeight: $confirmHeight, eligibleWeightZatoshi: $eligibleWeightZatoshi)';
  }
}

/// @nodoc
abstract mixin class $VotingWorkflowStatusCopyWith<$Res> {
  factory $VotingWorkflowStatusCopyWith(VotingWorkflowStatus value,
          $Res Function(VotingWorkflowStatus) _then) =
      _$VotingWorkflowStatusCopyWithImpl;
  @useResult
  $Res call(
      {String roundId,
      String status,
      String stage,
      double? progress,
      String? error,
      String? doneLabel,
      String? txHash,
      BigInt? confirmHeight,
      BigInt? eligibleWeightZatoshi});
}

/// @nodoc
class _$VotingWorkflowStatusCopyWithImpl<$Res>
    implements $VotingWorkflowStatusCopyWith<$Res> {
  _$VotingWorkflowStatusCopyWithImpl(this._self, this._then);

  final VotingWorkflowStatus _self;
  final $Res Function(VotingWorkflowStatus) _then;

  /// Create a copy of VotingWorkflowStatus
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? roundId = null,
    Object? status = null,
    Object? stage = null,
    Object? progress = freezed,
    Object? error = freezed,
    Object? doneLabel = freezed,
    Object? txHash = freezed,
    Object? confirmHeight = freezed,
    Object? eligibleWeightZatoshi = freezed,
  }) {
    return _then(_self.copyWith(
      roundId: null == roundId
          ? _self.roundId
          : roundId // ignore: cast_nullable_to_non_nullable
              as String,
      status: null == status
          ? _self.status
          : status // ignore: cast_nullable_to_non_nullable
              as String,
      stage: null == stage
          ? _self.stage
          : stage // ignore: cast_nullable_to_non_nullable
              as String,
      progress: freezed == progress
          ? _self.progress
          : progress // ignore: cast_nullable_to_non_nullable
              as double?,
      error: freezed == error
          ? _self.error
          : error // ignore: cast_nullable_to_non_nullable
              as String?,
      doneLabel: freezed == doneLabel
          ? _self.doneLabel
          : doneLabel // ignore: cast_nullable_to_non_nullable
              as String?,
      txHash: freezed == txHash
          ? _self.txHash
          : txHash // ignore: cast_nullable_to_non_nullable
              as String?,
      confirmHeight: freezed == confirmHeight
          ? _self.confirmHeight
          : confirmHeight // ignore: cast_nullable_to_non_nullable
              as BigInt?,
      eligibleWeightZatoshi: freezed == eligibleWeightZatoshi
          ? _self.eligibleWeightZatoshi
          : eligibleWeightZatoshi // ignore: cast_nullable_to_non_nullable
              as BigInt?,
    ));
  }
}

/// Adds pattern-matching-related methods to [VotingWorkflowStatus].
extension VotingWorkflowStatusPatterns on VotingWorkflowStatus {
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
    TResult Function(_VotingWorkflowStatus value)? $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _VotingWorkflowStatus() when $default != null:
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
    TResult Function(_VotingWorkflowStatus value) $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingWorkflowStatus():
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
    TResult? Function(_VotingWorkflowStatus value)? $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingWorkflowStatus() when $default != null:
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
            String status,
            String stage,
            double? progress,
            String? error,
            String? doneLabel,
            String? txHash,
            BigInt? confirmHeight,
            BigInt? eligibleWeightZatoshi)?
        $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _VotingWorkflowStatus() when $default != null:
        return $default(
            _that.roundId,
            _that.status,
            _that.stage,
            _that.progress,
            _that.error,
            _that.doneLabel,
            _that.txHash,
            _that.confirmHeight,
            _that.eligibleWeightZatoshi);
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
            String status,
            String stage,
            double? progress,
            String? error,
            String? doneLabel,
            String? txHash,
            BigInt? confirmHeight,
            BigInt? eligibleWeightZatoshi)
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingWorkflowStatus():
        return $default(
            _that.roundId,
            _that.status,
            _that.stage,
            _that.progress,
            _that.error,
            _that.doneLabel,
            _that.txHash,
            _that.confirmHeight,
            _that.eligibleWeightZatoshi);
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
            String status,
            String stage,
            double? progress,
            String? error,
            String? doneLabel,
            String? txHash,
            BigInt? confirmHeight,
            BigInt? eligibleWeightZatoshi)?
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingWorkflowStatus() when $default != null:
        return $default(
            _that.roundId,
            _that.status,
            _that.stage,
            _that.progress,
            _that.error,
            _that.doneLabel,
            _that.txHash,
            _that.confirmHeight,
            _that.eligibleWeightZatoshi);
      case _:
        return null;
    }
  }
}

/// @nodoc

class _VotingWorkflowStatus implements VotingWorkflowStatus {
  const _VotingWorkflowStatus(
      {required this.roundId,
      required this.status,
      required this.stage,
      this.progress,
      this.error,
      this.doneLabel,
      this.txHash,
      this.confirmHeight,
      this.eligibleWeightZatoshi});

  @override
  final String roundId;
  @override
  final String status;
  @override
  final String stage;
  @override
  final double? progress;
  @override
  final String? error;
  @override
  final String? doneLabel;
  @override
  final String? txHash;
  @override
  final BigInt? confirmHeight;
  @override
  final BigInt? eligibleWeightZatoshi;

  /// Create a copy of VotingWorkflowStatus
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  _$VotingWorkflowStatusCopyWith<_VotingWorkflowStatus> get copyWith =>
      __$VotingWorkflowStatusCopyWithImpl<_VotingWorkflowStatus>(
          this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _VotingWorkflowStatus &&
            (identical(other.roundId, roundId) || other.roundId == roundId) &&
            (identical(other.status, status) || other.status == status) &&
            (identical(other.stage, stage) || other.stage == stage) &&
            (identical(other.progress, progress) ||
                other.progress == progress) &&
            (identical(other.error, error) || other.error == error) &&
            (identical(other.doneLabel, doneLabel) ||
                other.doneLabel == doneLabel) &&
            (identical(other.txHash, txHash) || other.txHash == txHash) &&
            (identical(other.confirmHeight, confirmHeight) ||
                other.confirmHeight == confirmHeight) &&
            (identical(other.eligibleWeightZatoshi, eligibleWeightZatoshi) ||
                other.eligibleWeightZatoshi == eligibleWeightZatoshi));
  }

  @override
  int get hashCode => Object.hash(runtimeType, roundId, status, stage, progress,
      error, doneLabel, txHash, confirmHeight, eligibleWeightZatoshi);

  @override
  String toString() {
    return 'VotingWorkflowStatus(roundId: $roundId, status: $status, stage: $stage, progress: $progress, error: $error, doneLabel: $doneLabel, txHash: $txHash, confirmHeight: $confirmHeight, eligibleWeightZatoshi: $eligibleWeightZatoshi)';
  }
}

/// @nodoc
abstract mixin class _$VotingWorkflowStatusCopyWith<$Res>
    implements $VotingWorkflowStatusCopyWith<$Res> {
  factory _$VotingWorkflowStatusCopyWith(_VotingWorkflowStatus value,
          $Res Function(_VotingWorkflowStatus) _then) =
      __$VotingWorkflowStatusCopyWithImpl;
  @override
  @useResult
  $Res call(
      {String roundId,
      String status,
      String stage,
      double? progress,
      String? error,
      String? doneLabel,
      String? txHash,
      BigInt? confirmHeight,
      BigInt? eligibleWeightZatoshi});
}

/// @nodoc
class __$VotingWorkflowStatusCopyWithImpl<$Res>
    implements _$VotingWorkflowStatusCopyWith<$Res> {
  __$VotingWorkflowStatusCopyWithImpl(this._self, this._then);

  final _VotingWorkflowStatus _self;
  final $Res Function(_VotingWorkflowStatus) _then;

  /// Create a copy of VotingWorkflowStatus
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $Res call({
    Object? roundId = null,
    Object? status = null,
    Object? stage = null,
    Object? progress = freezed,
    Object? error = freezed,
    Object? doneLabel = freezed,
    Object? txHash = freezed,
    Object? confirmHeight = freezed,
    Object? eligibleWeightZatoshi = freezed,
  }) {
    return _then(_VotingWorkflowStatus(
      roundId: null == roundId
          ? _self.roundId
          : roundId // ignore: cast_nullable_to_non_nullable
              as String,
      status: null == status
          ? _self.status
          : status // ignore: cast_nullable_to_non_nullable
              as String,
      stage: null == stage
          ? _self.stage
          : stage // ignore: cast_nullable_to_non_nullable
              as String,
      progress: freezed == progress
          ? _self.progress
          : progress // ignore: cast_nullable_to_non_nullable
              as double?,
      error: freezed == error
          ? _self.error
          : error // ignore: cast_nullable_to_non_nullable
              as String?,
      doneLabel: freezed == doneLabel
          ? _self.doneLabel
          : doneLabel // ignore: cast_nullable_to_non_nullable
              as String?,
      txHash: freezed == txHash
          ? _self.txHash
          : txHash // ignore: cast_nullable_to_non_nullable
              as String?,
      confirmHeight: freezed == confirmHeight
          ? _self.confirmHeight
          : confirmHeight // ignore: cast_nullable_to_non_nullable
              as BigInt?,
      eligibleWeightZatoshi: freezed == eligibleWeightZatoshi
          ? _self.eligibleWeightZatoshi
          : eligibleWeightZatoshi // ignore: cast_nullable_to_non_nullable
              as BigInt?,
    ));
  }
}

// dart format on
