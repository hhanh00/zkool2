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
mixin _$VotingDelegationConfirmation {
  String get txHash;
  int get vanLeafPosition;

  /// Create a copy of VotingDelegationConfirmation
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $VotingDelegationConfirmationCopyWith<VotingDelegationConfirmation>
      get copyWith => _$VotingDelegationConfirmationCopyWithImpl<
              VotingDelegationConfirmation>(
          this as VotingDelegationConfirmation, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is VotingDelegationConfirmation &&
            (identical(other.txHash, txHash) || other.txHash == txHash) &&
            (identical(other.vanLeafPosition, vanLeafPosition) ||
                other.vanLeafPosition == vanLeafPosition));
  }

  @override
  int get hashCode => Object.hash(runtimeType, txHash, vanLeafPosition);

  @override
  String toString() {
    return 'VotingDelegationConfirmation(txHash: $txHash, vanLeafPosition: $vanLeafPosition)';
  }
}

/// @nodoc
abstract mixin class $VotingDelegationConfirmationCopyWith<$Res> {
  factory $VotingDelegationConfirmationCopyWith(
          VotingDelegationConfirmation value,
          $Res Function(VotingDelegationConfirmation) _then) =
      _$VotingDelegationConfirmationCopyWithImpl;
  @useResult
  $Res call({String txHash, int vanLeafPosition});
}

/// @nodoc
class _$VotingDelegationConfirmationCopyWithImpl<$Res>
    implements $VotingDelegationConfirmationCopyWith<$Res> {
  _$VotingDelegationConfirmationCopyWithImpl(this._self, this._then);

  final VotingDelegationConfirmation _self;
  final $Res Function(VotingDelegationConfirmation) _then;

  /// Create a copy of VotingDelegationConfirmation
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? txHash = null,
    Object? vanLeafPosition = null,
  }) {
    return _then(_self.copyWith(
      txHash: null == txHash
          ? _self.txHash
          : txHash // ignore: cast_nullable_to_non_nullable
              as String,
      vanLeafPosition: null == vanLeafPosition
          ? _self.vanLeafPosition
          : vanLeafPosition // ignore: cast_nullable_to_non_nullable
              as int,
    ));
  }
}

/// Adds pattern-matching-related methods to [VotingDelegationConfirmation].
extension VotingDelegationConfirmationPatterns on VotingDelegationConfirmation {
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
    TResult Function(_VotingDelegationConfirmation value)? $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _VotingDelegationConfirmation() when $default != null:
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
    TResult Function(_VotingDelegationConfirmation value) $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingDelegationConfirmation():
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
    TResult? Function(_VotingDelegationConfirmation value)? $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingDelegationConfirmation() when $default != null:
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
    TResult Function(String txHash, int vanLeafPosition)? $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _VotingDelegationConfirmation() when $default != null:
        return $default(_that.txHash, _that.vanLeafPosition);
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
    TResult Function(String txHash, int vanLeafPosition) $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingDelegationConfirmation():
        return $default(_that.txHash, _that.vanLeafPosition);
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
    TResult? Function(String txHash, int vanLeafPosition)? $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingDelegationConfirmation() when $default != null:
        return $default(_that.txHash, _that.vanLeafPosition);
      case _:
        return null;
    }
  }
}

/// @nodoc

class _VotingDelegationConfirmation implements VotingDelegationConfirmation {
  const _VotingDelegationConfirmation(
      {required this.txHash, required this.vanLeafPosition});

  @override
  final String txHash;
  @override
  final int vanLeafPosition;

  /// Create a copy of VotingDelegationConfirmation
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  _$VotingDelegationConfirmationCopyWith<_VotingDelegationConfirmation>
      get copyWith => __$VotingDelegationConfirmationCopyWithImpl<
          _VotingDelegationConfirmation>(this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _VotingDelegationConfirmation &&
            (identical(other.txHash, txHash) || other.txHash == txHash) &&
            (identical(other.vanLeafPosition, vanLeafPosition) ||
                other.vanLeafPosition == vanLeafPosition));
  }

  @override
  int get hashCode => Object.hash(runtimeType, txHash, vanLeafPosition);

  @override
  String toString() {
    return 'VotingDelegationConfirmation(txHash: $txHash, vanLeafPosition: $vanLeafPosition)';
  }
}

/// @nodoc
abstract mixin class _$VotingDelegationConfirmationCopyWith<$Res>
    implements $VotingDelegationConfirmationCopyWith<$Res> {
  factory _$VotingDelegationConfirmationCopyWith(
          _VotingDelegationConfirmation value,
          $Res Function(_VotingDelegationConfirmation) _then) =
      __$VotingDelegationConfirmationCopyWithImpl;
  @override
  @useResult
  $Res call({String txHash, int vanLeafPosition});
}

/// @nodoc
class __$VotingDelegationConfirmationCopyWithImpl<$Res>
    implements _$VotingDelegationConfirmationCopyWith<$Res> {
  __$VotingDelegationConfirmationCopyWithImpl(this._self, this._then);

  final _VotingDelegationConfirmation _self;
  final $Res Function(_VotingDelegationConfirmation) _then;

  /// Create a copy of VotingDelegationConfirmation
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $Res call({
    Object? txHash = null,
    Object? vanLeafPosition = null,
  }) {
    return _then(_VotingDelegationConfirmation(
      txHash: null == txHash
          ? _self.txHash
          : txHash // ignore: cast_nullable_to_non_nullable
              as String,
      vanLeafPosition: null == vanLeafPosition
          ? _self.vanLeafPosition
          : vanLeafPosition // ignore: cast_nullable_to_non_nullable
              as int,
    ));
  }
}

/// @nodoc
mixin _$VotingDelegationSetup {
  Uint8List get pcztBytes;
  Uint8List get pcztSighash;
  Uint8List get rk;
  int get actionIndex;
  Uint8List get actionBytes;
  Uint8List get tx1Effects;

  /// Create a copy of VotingDelegationSetup
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $VotingDelegationSetupCopyWith<VotingDelegationSetup> get copyWith =>
      _$VotingDelegationSetupCopyWithImpl<VotingDelegationSetup>(
          this as VotingDelegationSetup, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is VotingDelegationSetup &&
            const DeepCollectionEquality().equals(other.pcztBytes, pcztBytes) &&
            const DeepCollectionEquality()
                .equals(other.pcztSighash, pcztSighash) &&
            const DeepCollectionEquality().equals(other.rk, rk) &&
            (identical(other.actionIndex, actionIndex) ||
                other.actionIndex == actionIndex) &&
            const DeepCollectionEquality()
                .equals(other.actionBytes, actionBytes) &&
            const DeepCollectionEquality()
                .equals(other.tx1Effects, tx1Effects));
  }

  @override
  int get hashCode => Object.hash(
      runtimeType,
      const DeepCollectionEquality().hash(pcztBytes),
      const DeepCollectionEquality().hash(pcztSighash),
      const DeepCollectionEquality().hash(rk),
      actionIndex,
      const DeepCollectionEquality().hash(actionBytes),
      const DeepCollectionEquality().hash(tx1Effects));

  @override
  String toString() {
    return 'VotingDelegationSetup(pcztBytes: $pcztBytes, pcztSighash: $pcztSighash, rk: $rk, actionIndex: $actionIndex, actionBytes: $actionBytes, tx1Effects: $tx1Effects)';
  }
}

/// @nodoc
abstract mixin class $VotingDelegationSetupCopyWith<$Res> {
  factory $VotingDelegationSetupCopyWith(VotingDelegationSetup value,
          $Res Function(VotingDelegationSetup) _then) =
      _$VotingDelegationSetupCopyWithImpl;
  @useResult
  $Res call(
      {Uint8List pcztBytes,
      Uint8List pcztSighash,
      Uint8List rk,
      int actionIndex,
      Uint8List actionBytes,
      Uint8List tx1Effects});
}

/// @nodoc
class _$VotingDelegationSetupCopyWithImpl<$Res>
    implements $VotingDelegationSetupCopyWith<$Res> {
  _$VotingDelegationSetupCopyWithImpl(this._self, this._then);

  final VotingDelegationSetup _self;
  final $Res Function(VotingDelegationSetup) _then;

  /// Create a copy of VotingDelegationSetup
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? pcztBytes = null,
    Object? pcztSighash = null,
    Object? rk = null,
    Object? actionIndex = null,
    Object? actionBytes = null,
    Object? tx1Effects = null,
  }) {
    return _then(_self.copyWith(
      pcztBytes: null == pcztBytes
          ? _self.pcztBytes
          : pcztBytes // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      pcztSighash: null == pcztSighash
          ? _self.pcztSighash
          : pcztSighash // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      rk: null == rk
          ? _self.rk
          : rk // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      actionIndex: null == actionIndex
          ? _self.actionIndex
          : actionIndex // ignore: cast_nullable_to_non_nullable
              as int,
      actionBytes: null == actionBytes
          ? _self.actionBytes
          : actionBytes // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      tx1Effects: null == tx1Effects
          ? _self.tx1Effects
          : tx1Effects // ignore: cast_nullable_to_non_nullable
              as Uint8List,
    ));
  }
}

/// Adds pattern-matching-related methods to [VotingDelegationSetup].
extension VotingDelegationSetupPatterns on VotingDelegationSetup {
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
    TResult Function(_VotingDelegationSetup value)? $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _VotingDelegationSetup() when $default != null:
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
    TResult Function(_VotingDelegationSetup value) $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingDelegationSetup():
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
    TResult? Function(_VotingDelegationSetup value)? $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingDelegationSetup() when $default != null:
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
    TResult Function(Uint8List pcztBytes, Uint8List pcztSighash, Uint8List rk,
            int actionIndex, Uint8List actionBytes, Uint8List tx1Effects)?
        $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _VotingDelegationSetup() when $default != null:
        return $default(_that.pcztBytes, _that.pcztSighash, _that.rk,
            _that.actionIndex, _that.actionBytes, _that.tx1Effects);
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
    TResult Function(Uint8List pcztBytes, Uint8List pcztSighash, Uint8List rk,
            int actionIndex, Uint8List actionBytes, Uint8List tx1Effects)
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingDelegationSetup():
        return $default(_that.pcztBytes, _that.pcztSighash, _that.rk,
            _that.actionIndex, _that.actionBytes, _that.tx1Effects);
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
    TResult? Function(Uint8List pcztBytes, Uint8List pcztSighash, Uint8List rk,
            int actionIndex, Uint8List actionBytes, Uint8List tx1Effects)?
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingDelegationSetup() when $default != null:
        return $default(_that.pcztBytes, _that.pcztSighash, _that.rk,
            _that.actionIndex, _that.actionBytes, _that.tx1Effects);
      case _:
        return null;
    }
  }
}

/// @nodoc

class _VotingDelegationSetup implements VotingDelegationSetup {
  const _VotingDelegationSetup(
      {required this.pcztBytes,
      required this.pcztSighash,
      required this.rk,
      required this.actionIndex,
      required this.actionBytes,
      required this.tx1Effects});

  @override
  final Uint8List pcztBytes;
  @override
  final Uint8List pcztSighash;
  @override
  final Uint8List rk;
  @override
  final int actionIndex;
  @override
  final Uint8List actionBytes;
  @override
  final Uint8List tx1Effects;

  /// Create a copy of VotingDelegationSetup
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  _$VotingDelegationSetupCopyWith<_VotingDelegationSetup> get copyWith =>
      __$VotingDelegationSetupCopyWithImpl<_VotingDelegationSetup>(
          this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _VotingDelegationSetup &&
            const DeepCollectionEquality().equals(other.pcztBytes, pcztBytes) &&
            const DeepCollectionEquality()
                .equals(other.pcztSighash, pcztSighash) &&
            const DeepCollectionEquality().equals(other.rk, rk) &&
            (identical(other.actionIndex, actionIndex) ||
                other.actionIndex == actionIndex) &&
            const DeepCollectionEquality()
                .equals(other.actionBytes, actionBytes) &&
            const DeepCollectionEquality()
                .equals(other.tx1Effects, tx1Effects));
  }

  @override
  int get hashCode => Object.hash(
      runtimeType,
      const DeepCollectionEquality().hash(pcztBytes),
      const DeepCollectionEquality().hash(pcztSighash),
      const DeepCollectionEquality().hash(rk),
      actionIndex,
      const DeepCollectionEquality().hash(actionBytes),
      const DeepCollectionEquality().hash(tx1Effects));

  @override
  String toString() {
    return 'VotingDelegationSetup(pcztBytes: $pcztBytes, pcztSighash: $pcztSighash, rk: $rk, actionIndex: $actionIndex, actionBytes: $actionBytes, tx1Effects: $tx1Effects)';
  }
}

/// @nodoc
abstract mixin class _$VotingDelegationSetupCopyWith<$Res>
    implements $VotingDelegationSetupCopyWith<$Res> {
  factory _$VotingDelegationSetupCopyWith(_VotingDelegationSetup value,
          $Res Function(_VotingDelegationSetup) _then) =
      __$VotingDelegationSetupCopyWithImpl;
  @override
  @useResult
  $Res call(
      {Uint8List pcztBytes,
      Uint8List pcztSighash,
      Uint8List rk,
      int actionIndex,
      Uint8List actionBytes,
      Uint8List tx1Effects});
}

/// @nodoc
class __$VotingDelegationSetupCopyWithImpl<$Res>
    implements _$VotingDelegationSetupCopyWith<$Res> {
  __$VotingDelegationSetupCopyWithImpl(this._self, this._then);

  final _VotingDelegationSetup _self;
  final $Res Function(_VotingDelegationSetup) _then;

  /// Create a copy of VotingDelegationSetup
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $Res call({
    Object? pcztBytes = null,
    Object? pcztSighash = null,
    Object? rk = null,
    Object? actionIndex = null,
    Object? actionBytes = null,
    Object? tx1Effects = null,
  }) {
    return _then(_VotingDelegationSetup(
      pcztBytes: null == pcztBytes
          ? _self.pcztBytes
          : pcztBytes // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      pcztSighash: null == pcztSighash
          ? _self.pcztSighash
          : pcztSighash // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      rk: null == rk
          ? _self.rk
          : rk // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      actionIndex: null == actionIndex
          ? _self.actionIndex
          : actionIndex // ignore: cast_nullable_to_non_nullable
              as int,
      actionBytes: null == actionBytes
          ? _self.actionBytes
          : actionBytes // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      tx1Effects: null == tx1Effects
          ? _self.tx1Effects
          : tx1Effects // ignore: cast_nullable_to_non_nullable
              as Uint8List,
    ));
  }
}

/// @nodoc
mixin _$VotingDelegationSubmission {
  Uint8List get proof;
  Uint8List get rk;
  Uint8List get nfSigned;
  Uint8List get cmxNew;
  Uint8List get govComm;
  List<Uint8List> get govNullifiers;
  Uint8List get alpha;
  String get voteRoundId;
  Uint8List get spendAuthSig;
  Uint8List get sighash;
  Uint8List get tx1Effects;

  /// Create a copy of VotingDelegationSubmission
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $VotingDelegationSubmissionCopyWith<VotingDelegationSubmission>
      get copyWith =>
          _$VotingDelegationSubmissionCopyWithImpl<VotingDelegationSubmission>(
              this as VotingDelegationSubmission, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is VotingDelegationSubmission &&
            const DeepCollectionEquality().equals(other.proof, proof) &&
            const DeepCollectionEquality().equals(other.rk, rk) &&
            const DeepCollectionEquality().equals(other.nfSigned, nfSigned) &&
            const DeepCollectionEquality().equals(other.cmxNew, cmxNew) &&
            const DeepCollectionEquality().equals(other.govComm, govComm) &&
            const DeepCollectionEquality()
                .equals(other.govNullifiers, govNullifiers) &&
            const DeepCollectionEquality().equals(other.alpha, alpha) &&
            (identical(other.voteRoundId, voteRoundId) ||
                other.voteRoundId == voteRoundId) &&
            const DeepCollectionEquality()
                .equals(other.spendAuthSig, spendAuthSig) &&
            const DeepCollectionEquality().equals(other.sighash, sighash) &&
            const DeepCollectionEquality()
                .equals(other.tx1Effects, tx1Effects));
  }

  @override
  int get hashCode => Object.hash(
      runtimeType,
      const DeepCollectionEquality().hash(proof),
      const DeepCollectionEquality().hash(rk),
      const DeepCollectionEquality().hash(nfSigned),
      const DeepCollectionEquality().hash(cmxNew),
      const DeepCollectionEquality().hash(govComm),
      const DeepCollectionEquality().hash(govNullifiers),
      const DeepCollectionEquality().hash(alpha),
      voteRoundId,
      const DeepCollectionEquality().hash(spendAuthSig),
      const DeepCollectionEquality().hash(sighash),
      const DeepCollectionEquality().hash(tx1Effects));

  @override
  String toString() {
    return 'VotingDelegationSubmission(proof: $proof, rk: $rk, nfSigned: $nfSigned, cmxNew: $cmxNew, govComm: $govComm, govNullifiers: $govNullifiers, alpha: $alpha, voteRoundId: $voteRoundId, spendAuthSig: $spendAuthSig, sighash: $sighash, tx1Effects: $tx1Effects)';
  }
}

/// @nodoc
abstract mixin class $VotingDelegationSubmissionCopyWith<$Res> {
  factory $VotingDelegationSubmissionCopyWith(VotingDelegationSubmission value,
          $Res Function(VotingDelegationSubmission) _then) =
      _$VotingDelegationSubmissionCopyWithImpl;
  @useResult
  $Res call(
      {Uint8List proof,
      Uint8List rk,
      Uint8List nfSigned,
      Uint8List cmxNew,
      Uint8List govComm,
      List<Uint8List> govNullifiers,
      Uint8List alpha,
      String voteRoundId,
      Uint8List spendAuthSig,
      Uint8List sighash,
      Uint8List tx1Effects});
}

/// @nodoc
class _$VotingDelegationSubmissionCopyWithImpl<$Res>
    implements $VotingDelegationSubmissionCopyWith<$Res> {
  _$VotingDelegationSubmissionCopyWithImpl(this._self, this._then);

  final VotingDelegationSubmission _self;
  final $Res Function(VotingDelegationSubmission) _then;

  /// Create a copy of VotingDelegationSubmission
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? proof = null,
    Object? rk = null,
    Object? nfSigned = null,
    Object? cmxNew = null,
    Object? govComm = null,
    Object? govNullifiers = null,
    Object? alpha = null,
    Object? voteRoundId = null,
    Object? spendAuthSig = null,
    Object? sighash = null,
    Object? tx1Effects = null,
  }) {
    return _then(_self.copyWith(
      proof: null == proof
          ? _self.proof
          : proof // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      rk: null == rk
          ? _self.rk
          : rk // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      nfSigned: null == nfSigned
          ? _self.nfSigned
          : nfSigned // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      cmxNew: null == cmxNew
          ? _self.cmxNew
          : cmxNew // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      govComm: null == govComm
          ? _self.govComm
          : govComm // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      govNullifiers: null == govNullifiers
          ? _self.govNullifiers
          : govNullifiers // ignore: cast_nullable_to_non_nullable
              as List<Uint8List>,
      alpha: null == alpha
          ? _self.alpha
          : alpha // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      voteRoundId: null == voteRoundId
          ? _self.voteRoundId
          : voteRoundId // ignore: cast_nullable_to_non_nullable
              as String,
      spendAuthSig: null == spendAuthSig
          ? _self.spendAuthSig
          : spendAuthSig // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      sighash: null == sighash
          ? _self.sighash
          : sighash // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      tx1Effects: null == tx1Effects
          ? _self.tx1Effects
          : tx1Effects // ignore: cast_nullable_to_non_nullable
              as Uint8List,
    ));
  }
}

/// Adds pattern-matching-related methods to [VotingDelegationSubmission].
extension VotingDelegationSubmissionPatterns on VotingDelegationSubmission {
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
    TResult Function(_VotingDelegationSubmission value)? $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _VotingDelegationSubmission() when $default != null:
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
    TResult Function(_VotingDelegationSubmission value) $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingDelegationSubmission():
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
    TResult? Function(_VotingDelegationSubmission value)? $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingDelegationSubmission() when $default != null:
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
            Uint8List proof,
            Uint8List rk,
            Uint8List nfSigned,
            Uint8List cmxNew,
            Uint8List govComm,
            List<Uint8List> govNullifiers,
            Uint8List alpha,
            String voteRoundId,
            Uint8List spendAuthSig,
            Uint8List sighash,
            Uint8List tx1Effects)?
        $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _VotingDelegationSubmission() when $default != null:
        return $default(
            _that.proof,
            _that.rk,
            _that.nfSigned,
            _that.cmxNew,
            _that.govComm,
            _that.govNullifiers,
            _that.alpha,
            _that.voteRoundId,
            _that.spendAuthSig,
            _that.sighash,
            _that.tx1Effects);
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
            Uint8List proof,
            Uint8List rk,
            Uint8List nfSigned,
            Uint8List cmxNew,
            Uint8List govComm,
            List<Uint8List> govNullifiers,
            Uint8List alpha,
            String voteRoundId,
            Uint8List spendAuthSig,
            Uint8List sighash,
            Uint8List tx1Effects)
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingDelegationSubmission():
        return $default(
            _that.proof,
            _that.rk,
            _that.nfSigned,
            _that.cmxNew,
            _that.govComm,
            _that.govNullifiers,
            _that.alpha,
            _that.voteRoundId,
            _that.spendAuthSig,
            _that.sighash,
            _that.tx1Effects);
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
            Uint8List proof,
            Uint8List rk,
            Uint8List nfSigned,
            Uint8List cmxNew,
            Uint8List govComm,
            List<Uint8List> govNullifiers,
            Uint8List alpha,
            String voteRoundId,
            Uint8List spendAuthSig,
            Uint8List sighash,
            Uint8List tx1Effects)?
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingDelegationSubmission() when $default != null:
        return $default(
            _that.proof,
            _that.rk,
            _that.nfSigned,
            _that.cmxNew,
            _that.govComm,
            _that.govNullifiers,
            _that.alpha,
            _that.voteRoundId,
            _that.spendAuthSig,
            _that.sighash,
            _that.tx1Effects);
      case _:
        return null;
    }
  }
}

/// @nodoc

class _VotingDelegationSubmission implements VotingDelegationSubmission {
  const _VotingDelegationSubmission(
      {required this.proof,
      required this.rk,
      required this.nfSigned,
      required this.cmxNew,
      required this.govComm,
      required final List<Uint8List> govNullifiers,
      required this.alpha,
      required this.voteRoundId,
      required this.spendAuthSig,
      required this.sighash,
      required this.tx1Effects})
      : _govNullifiers = govNullifiers;

  @override
  final Uint8List proof;
  @override
  final Uint8List rk;
  @override
  final Uint8List nfSigned;
  @override
  final Uint8List cmxNew;
  @override
  final Uint8List govComm;
  final List<Uint8List> _govNullifiers;
  @override
  List<Uint8List> get govNullifiers {
    if (_govNullifiers is EqualUnmodifiableListView) return _govNullifiers;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_govNullifiers);
  }

  @override
  final Uint8List alpha;
  @override
  final String voteRoundId;
  @override
  final Uint8List spendAuthSig;
  @override
  final Uint8List sighash;
  @override
  final Uint8List tx1Effects;

  /// Create a copy of VotingDelegationSubmission
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  _$VotingDelegationSubmissionCopyWith<_VotingDelegationSubmission>
      get copyWith => __$VotingDelegationSubmissionCopyWithImpl<
          _VotingDelegationSubmission>(this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _VotingDelegationSubmission &&
            const DeepCollectionEquality().equals(other.proof, proof) &&
            const DeepCollectionEquality().equals(other.rk, rk) &&
            const DeepCollectionEquality().equals(other.nfSigned, nfSigned) &&
            const DeepCollectionEquality().equals(other.cmxNew, cmxNew) &&
            const DeepCollectionEquality().equals(other.govComm, govComm) &&
            const DeepCollectionEquality()
                .equals(other._govNullifiers, _govNullifiers) &&
            const DeepCollectionEquality().equals(other.alpha, alpha) &&
            (identical(other.voteRoundId, voteRoundId) ||
                other.voteRoundId == voteRoundId) &&
            const DeepCollectionEquality()
                .equals(other.spendAuthSig, spendAuthSig) &&
            const DeepCollectionEquality().equals(other.sighash, sighash) &&
            const DeepCollectionEquality()
                .equals(other.tx1Effects, tx1Effects));
  }

  @override
  int get hashCode => Object.hash(
      runtimeType,
      const DeepCollectionEquality().hash(proof),
      const DeepCollectionEquality().hash(rk),
      const DeepCollectionEquality().hash(nfSigned),
      const DeepCollectionEquality().hash(cmxNew),
      const DeepCollectionEquality().hash(govComm),
      const DeepCollectionEquality().hash(_govNullifiers),
      const DeepCollectionEquality().hash(alpha),
      voteRoundId,
      const DeepCollectionEquality().hash(spendAuthSig),
      const DeepCollectionEquality().hash(sighash),
      const DeepCollectionEquality().hash(tx1Effects));

  @override
  String toString() {
    return 'VotingDelegationSubmission(proof: $proof, rk: $rk, nfSigned: $nfSigned, cmxNew: $cmxNew, govComm: $govComm, govNullifiers: $govNullifiers, alpha: $alpha, voteRoundId: $voteRoundId, spendAuthSig: $spendAuthSig, sighash: $sighash, tx1Effects: $tx1Effects)';
  }
}

/// @nodoc
abstract mixin class _$VotingDelegationSubmissionCopyWith<$Res>
    implements $VotingDelegationSubmissionCopyWith<$Res> {
  factory _$VotingDelegationSubmissionCopyWith(
          _VotingDelegationSubmission value,
          $Res Function(_VotingDelegationSubmission) _then) =
      __$VotingDelegationSubmissionCopyWithImpl;
  @override
  @useResult
  $Res call(
      {Uint8List proof,
      Uint8List rk,
      Uint8List nfSigned,
      Uint8List cmxNew,
      Uint8List govComm,
      List<Uint8List> govNullifiers,
      Uint8List alpha,
      String voteRoundId,
      Uint8List spendAuthSig,
      Uint8List sighash,
      Uint8List tx1Effects});
}

/// @nodoc
class __$VotingDelegationSubmissionCopyWithImpl<$Res>
    implements _$VotingDelegationSubmissionCopyWith<$Res> {
  __$VotingDelegationSubmissionCopyWithImpl(this._self, this._then);

  final _VotingDelegationSubmission _self;
  final $Res Function(_VotingDelegationSubmission) _then;

  /// Create a copy of VotingDelegationSubmission
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $Res call({
    Object? proof = null,
    Object? rk = null,
    Object? nfSigned = null,
    Object? cmxNew = null,
    Object? govComm = null,
    Object? govNullifiers = null,
    Object? alpha = null,
    Object? voteRoundId = null,
    Object? spendAuthSig = null,
    Object? sighash = null,
    Object? tx1Effects = null,
  }) {
    return _then(_VotingDelegationSubmission(
      proof: null == proof
          ? _self.proof
          : proof // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      rk: null == rk
          ? _self.rk
          : rk // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      nfSigned: null == nfSigned
          ? _self.nfSigned
          : nfSigned // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      cmxNew: null == cmxNew
          ? _self.cmxNew
          : cmxNew // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      govComm: null == govComm
          ? _self.govComm
          : govComm // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      govNullifiers: null == govNullifiers
          ? _self._govNullifiers
          : govNullifiers // ignore: cast_nullable_to_non_nullable
              as List<Uint8List>,
      alpha: null == alpha
          ? _self.alpha
          : alpha // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      voteRoundId: null == voteRoundId
          ? _self.voteRoundId
          : voteRoundId // ignore: cast_nullable_to_non_nullable
              as String,
      spendAuthSig: null == spendAuthSig
          ? _self.spendAuthSig
          : spendAuthSig // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      sighash: null == sighash
          ? _self.sighash
          : sighash // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      tx1Effects: null == tx1Effects
          ? _self.tx1Effects
          : tx1Effects // ignore: cast_nullable_to_non_nullable
              as Uint8List,
    ));
  }
}

/// @nodoc
mixin _$VotingEncryptedShare {
  Uint8List get c1;
  Uint8List get c2;
  int get shareIndex;

  /// Create a copy of VotingEncryptedShare
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $VotingEncryptedShareCopyWith<VotingEncryptedShare> get copyWith =>
      _$VotingEncryptedShareCopyWithImpl<VotingEncryptedShare>(
          this as VotingEncryptedShare, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is VotingEncryptedShare &&
            const DeepCollectionEquality().equals(other.c1, c1) &&
            const DeepCollectionEquality().equals(other.c2, c2) &&
            (identical(other.shareIndex, shareIndex) ||
                other.shareIndex == shareIndex));
  }

  @override
  int get hashCode => Object.hash(
      runtimeType,
      const DeepCollectionEquality().hash(c1),
      const DeepCollectionEquality().hash(c2),
      shareIndex);

  @override
  String toString() {
    return 'VotingEncryptedShare(c1: $c1, c2: $c2, shareIndex: $shareIndex)';
  }
}

/// @nodoc
abstract mixin class $VotingEncryptedShareCopyWith<$Res> {
  factory $VotingEncryptedShareCopyWith(VotingEncryptedShare value,
          $Res Function(VotingEncryptedShare) _then) =
      _$VotingEncryptedShareCopyWithImpl;
  @useResult
  $Res call({Uint8List c1, Uint8List c2, int shareIndex});
}

/// @nodoc
class _$VotingEncryptedShareCopyWithImpl<$Res>
    implements $VotingEncryptedShareCopyWith<$Res> {
  _$VotingEncryptedShareCopyWithImpl(this._self, this._then);

  final VotingEncryptedShare _self;
  final $Res Function(VotingEncryptedShare) _then;

  /// Create a copy of VotingEncryptedShare
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? c1 = null,
    Object? c2 = null,
    Object? shareIndex = null,
  }) {
    return _then(_self.copyWith(
      c1: null == c1
          ? _self.c1
          : c1 // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      c2: null == c2
          ? _self.c2
          : c2 // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      shareIndex: null == shareIndex
          ? _self.shareIndex
          : shareIndex // ignore: cast_nullable_to_non_nullable
              as int,
    ));
  }
}

/// Adds pattern-matching-related methods to [VotingEncryptedShare].
extension VotingEncryptedSharePatterns on VotingEncryptedShare {
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
    TResult Function(_VotingEncryptedShare value)? $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _VotingEncryptedShare() when $default != null:
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
    TResult Function(_VotingEncryptedShare value) $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingEncryptedShare():
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
    TResult? Function(_VotingEncryptedShare value)? $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingEncryptedShare() when $default != null:
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
    TResult Function(Uint8List c1, Uint8List c2, int shareIndex)? $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _VotingEncryptedShare() when $default != null:
        return $default(_that.c1, _that.c2, _that.shareIndex);
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
    TResult Function(Uint8List c1, Uint8List c2, int shareIndex) $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingEncryptedShare():
        return $default(_that.c1, _that.c2, _that.shareIndex);
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
    TResult? Function(Uint8List c1, Uint8List c2, int shareIndex)? $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingEncryptedShare() when $default != null:
        return $default(_that.c1, _that.c2, _that.shareIndex);
      case _:
        return null;
    }
  }
}

/// @nodoc

class _VotingEncryptedShare implements VotingEncryptedShare {
  const _VotingEncryptedShare(
      {required this.c1, required this.c2, required this.shareIndex});

  @override
  final Uint8List c1;
  @override
  final Uint8List c2;
  @override
  final int shareIndex;

  /// Create a copy of VotingEncryptedShare
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  _$VotingEncryptedShareCopyWith<_VotingEncryptedShare> get copyWith =>
      __$VotingEncryptedShareCopyWithImpl<_VotingEncryptedShare>(
          this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _VotingEncryptedShare &&
            const DeepCollectionEquality().equals(other.c1, c1) &&
            const DeepCollectionEquality().equals(other.c2, c2) &&
            (identical(other.shareIndex, shareIndex) ||
                other.shareIndex == shareIndex));
  }

  @override
  int get hashCode => Object.hash(
      runtimeType,
      const DeepCollectionEquality().hash(c1),
      const DeepCollectionEquality().hash(c2),
      shareIndex);

  @override
  String toString() {
    return 'VotingEncryptedShare(c1: $c1, c2: $c2, shareIndex: $shareIndex)';
  }
}

/// @nodoc
abstract mixin class _$VotingEncryptedShareCopyWith<$Res>
    implements $VotingEncryptedShareCopyWith<$Res> {
  factory _$VotingEncryptedShareCopyWith(_VotingEncryptedShare value,
          $Res Function(_VotingEncryptedShare) _then) =
      __$VotingEncryptedShareCopyWithImpl;
  @override
  @useResult
  $Res call({Uint8List c1, Uint8List c2, int shareIndex});
}

/// @nodoc
class __$VotingEncryptedShareCopyWithImpl<$Res>
    implements _$VotingEncryptedShareCopyWith<$Res> {
  __$VotingEncryptedShareCopyWithImpl(this._self, this._then);

  final _VotingEncryptedShare _self;
  final $Res Function(_VotingEncryptedShare) _then;

  /// Create a copy of VotingEncryptedShare
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $Res call({
    Object? c1 = null,
    Object? c2 = null,
    Object? shareIndex = null,
  }) {
    return _then(_VotingEncryptedShare(
      c1: null == c1
          ? _self.c1
          : c1 // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      c2: null == c2
          ? _self.c2
          : c2 // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      shareIndex: null == shareIndex
          ? _self.shareIndex
          : shareIndex // ignore: cast_nullable_to_non_nullable
              as int,
    ));
  }
}

/// @nodoc
mixin _$VotingPirLayout {
  int get pirDepth;
  int get tier0Layers;
  int get tier1Layers;
  int get polyLen;

  /// Create a copy of VotingPirLayout
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $VotingPirLayoutCopyWith<VotingPirLayout> get copyWith =>
      _$VotingPirLayoutCopyWithImpl<VotingPirLayout>(
          this as VotingPirLayout, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is VotingPirLayout &&
            (identical(other.pirDepth, pirDepth) ||
                other.pirDepth == pirDepth) &&
            (identical(other.tier0Layers, tier0Layers) ||
                other.tier0Layers == tier0Layers) &&
            (identical(other.tier1Layers, tier1Layers) ||
                other.tier1Layers == tier1Layers) &&
            (identical(other.polyLen, polyLen) || other.polyLen == polyLen));
  }

  @override
  int get hashCode =>
      Object.hash(runtimeType, pirDepth, tier0Layers, tier1Layers, polyLen);

  @override
  String toString() {
    return 'VotingPirLayout(pirDepth: $pirDepth, tier0Layers: $tier0Layers, tier1Layers: $tier1Layers, polyLen: $polyLen)';
  }
}

/// @nodoc
abstract mixin class $VotingPirLayoutCopyWith<$Res> {
  factory $VotingPirLayoutCopyWith(
          VotingPirLayout value, $Res Function(VotingPirLayout) _then) =
      _$VotingPirLayoutCopyWithImpl;
  @useResult
  $Res call({int pirDepth, int tier0Layers, int tier1Layers, int polyLen});
}

/// @nodoc
class _$VotingPirLayoutCopyWithImpl<$Res>
    implements $VotingPirLayoutCopyWith<$Res> {
  _$VotingPirLayoutCopyWithImpl(this._self, this._then);

  final VotingPirLayout _self;
  final $Res Function(VotingPirLayout) _then;

  /// Create a copy of VotingPirLayout
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? pirDepth = null,
    Object? tier0Layers = null,
    Object? tier1Layers = null,
    Object? polyLen = null,
  }) {
    return _then(_self.copyWith(
      pirDepth: null == pirDepth
          ? _self.pirDepth
          : pirDepth // ignore: cast_nullable_to_non_nullable
              as int,
      tier0Layers: null == tier0Layers
          ? _self.tier0Layers
          : tier0Layers // ignore: cast_nullable_to_non_nullable
              as int,
      tier1Layers: null == tier1Layers
          ? _self.tier1Layers
          : tier1Layers // ignore: cast_nullable_to_non_nullable
              as int,
      polyLen: null == polyLen
          ? _self.polyLen
          : polyLen // ignore: cast_nullable_to_non_nullable
              as int,
    ));
  }
}

/// Adds pattern-matching-related methods to [VotingPirLayout].
extension VotingPirLayoutPatterns on VotingPirLayout {
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
    TResult Function(_VotingPirLayout value)? $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _VotingPirLayout() when $default != null:
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
    TResult Function(_VotingPirLayout value) $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingPirLayout():
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
    TResult? Function(_VotingPirLayout value)? $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingPirLayout() when $default != null:
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
            int pirDepth, int tier0Layers, int tier1Layers, int polyLen)?
        $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _VotingPirLayout() when $default != null:
        return $default(_that.pirDepth, _that.tier0Layers, _that.tier1Layers,
            _that.polyLen);
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
            int pirDepth, int tier0Layers, int tier1Layers, int polyLen)
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingPirLayout():
        return $default(_that.pirDepth, _that.tier0Layers, _that.tier1Layers,
            _that.polyLen);
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
            int pirDepth, int tier0Layers, int tier1Layers, int polyLen)?
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingPirLayout() when $default != null:
        return $default(_that.pirDepth, _that.tier0Layers, _that.tier1Layers,
            _that.polyLen);
      case _:
        return null;
    }
  }
}

/// @nodoc

class _VotingPirLayout implements VotingPirLayout {
  const _VotingPirLayout(
      {required this.pirDepth,
      required this.tier0Layers,
      required this.tier1Layers,
      required this.polyLen});

  @override
  final int pirDepth;
  @override
  final int tier0Layers;
  @override
  final int tier1Layers;
  @override
  final int polyLen;

  /// Create a copy of VotingPirLayout
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  _$VotingPirLayoutCopyWith<_VotingPirLayout> get copyWith =>
      __$VotingPirLayoutCopyWithImpl<_VotingPirLayout>(this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _VotingPirLayout &&
            (identical(other.pirDepth, pirDepth) ||
                other.pirDepth == pirDepth) &&
            (identical(other.tier0Layers, tier0Layers) ||
                other.tier0Layers == tier0Layers) &&
            (identical(other.tier1Layers, tier1Layers) ||
                other.tier1Layers == tier1Layers) &&
            (identical(other.polyLen, polyLen) || other.polyLen == polyLen));
  }

  @override
  int get hashCode =>
      Object.hash(runtimeType, pirDepth, tier0Layers, tier1Layers, polyLen);

  @override
  String toString() {
    return 'VotingPirLayout(pirDepth: $pirDepth, tier0Layers: $tier0Layers, tier1Layers: $tier1Layers, polyLen: $polyLen)';
  }
}

/// @nodoc
abstract mixin class _$VotingPirLayoutCopyWith<$Res>
    implements $VotingPirLayoutCopyWith<$Res> {
  factory _$VotingPirLayoutCopyWith(
          _VotingPirLayout value, $Res Function(_VotingPirLayout) _then) =
      __$VotingPirLayoutCopyWithImpl;
  @override
  @useResult
  $Res call({int pirDepth, int tier0Layers, int tier1Layers, int polyLen});
}

/// @nodoc
class __$VotingPirLayoutCopyWithImpl<$Res>
    implements _$VotingPirLayoutCopyWith<$Res> {
  __$VotingPirLayoutCopyWithImpl(this._self, this._then);

  final _VotingPirLayout _self;
  final $Res Function(_VotingPirLayout) _then;

  /// Create a copy of VotingPirLayout
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $Res call({
    Object? pirDepth = null,
    Object? tier0Layers = null,
    Object? tier1Layers = null,
    Object? polyLen = null,
  }) {
    return _then(_VotingPirLayout(
      pirDepth: null == pirDepth
          ? _self.pirDepth
          : pirDepth // ignore: cast_nullable_to_non_nullable
              as int,
      tier0Layers: null == tier0Layers
          ? _self.tier0Layers
          : tier0Layers // ignore: cast_nullable_to_non_nullable
              as int,
      tier1Layers: null == tier1Layers
          ? _self.tier1Layers
          : tier1Layers // ignore: cast_nullable_to_non_nullable
              as int,
      polyLen: null == polyLen
          ? _self.polyLen
          : polyLen // ignore: cast_nullable_to_non_nullable
              as int,
    ));
  }
}

/// @nodoc
mixin _$VotingPreparedInfo {
  String get roundId;
  int get bundleIndex;
  BigInt get eligibleWeightZatoshi;
  BigInt get delegatedWeightZatoshi;
  String get roundName;

  /// Create a copy of VotingPreparedInfo
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $VotingPreparedInfoCopyWith<VotingPreparedInfo> get copyWith =>
      _$VotingPreparedInfoCopyWithImpl<VotingPreparedInfo>(
          this as VotingPreparedInfo, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is VotingPreparedInfo &&
            (identical(other.roundId, roundId) || other.roundId == roundId) &&
            (identical(other.bundleIndex, bundleIndex) ||
                other.bundleIndex == bundleIndex) &&
            (identical(other.eligibleWeightZatoshi, eligibleWeightZatoshi) ||
                other.eligibleWeightZatoshi == eligibleWeightZatoshi) &&
            (identical(other.delegatedWeightZatoshi, delegatedWeightZatoshi) ||
                other.delegatedWeightZatoshi == delegatedWeightZatoshi) &&
            (identical(other.roundName, roundName) ||
                other.roundName == roundName));
  }

  @override
  int get hashCode => Object.hash(runtimeType, roundId, bundleIndex,
      eligibleWeightZatoshi, delegatedWeightZatoshi, roundName);

  @override
  String toString() {
    return 'VotingPreparedInfo(roundId: $roundId, bundleIndex: $bundleIndex, eligibleWeightZatoshi: $eligibleWeightZatoshi, delegatedWeightZatoshi: $delegatedWeightZatoshi, roundName: $roundName)';
  }
}

/// @nodoc
abstract mixin class $VotingPreparedInfoCopyWith<$Res> {
  factory $VotingPreparedInfoCopyWith(
          VotingPreparedInfo value, $Res Function(VotingPreparedInfo) _then) =
      _$VotingPreparedInfoCopyWithImpl;
  @useResult
  $Res call(
      {String roundId,
      int bundleIndex,
      BigInt eligibleWeightZatoshi,
      BigInt delegatedWeightZatoshi,
      String roundName});
}

/// @nodoc
class _$VotingPreparedInfoCopyWithImpl<$Res>
    implements $VotingPreparedInfoCopyWith<$Res> {
  _$VotingPreparedInfoCopyWithImpl(this._self, this._then);

  final VotingPreparedInfo _self;
  final $Res Function(VotingPreparedInfo) _then;

  /// Create a copy of VotingPreparedInfo
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? roundId = null,
    Object? bundleIndex = null,
    Object? eligibleWeightZatoshi = null,
    Object? delegatedWeightZatoshi = null,
    Object? roundName = null,
  }) {
    return _then(_self.copyWith(
      roundId: null == roundId
          ? _self.roundId
          : roundId // ignore: cast_nullable_to_non_nullable
              as String,
      bundleIndex: null == bundleIndex
          ? _self.bundleIndex
          : bundleIndex // ignore: cast_nullable_to_non_nullable
              as int,
      eligibleWeightZatoshi: null == eligibleWeightZatoshi
          ? _self.eligibleWeightZatoshi
          : eligibleWeightZatoshi // ignore: cast_nullable_to_non_nullable
              as BigInt,
      delegatedWeightZatoshi: null == delegatedWeightZatoshi
          ? _self.delegatedWeightZatoshi
          : delegatedWeightZatoshi // ignore: cast_nullable_to_non_nullable
              as BigInt,
      roundName: null == roundName
          ? _self.roundName
          : roundName // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// Adds pattern-matching-related methods to [VotingPreparedInfo].
extension VotingPreparedInfoPatterns on VotingPreparedInfo {
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
    TResult Function(_VotingPreparedInfo value)? $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _VotingPreparedInfo() when $default != null:
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
    TResult Function(_VotingPreparedInfo value) $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingPreparedInfo():
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
    TResult? Function(_VotingPreparedInfo value)? $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingPreparedInfo() when $default != null:
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
            int bundleIndex,
            BigInt eligibleWeightZatoshi,
            BigInt delegatedWeightZatoshi,
            String roundName)?
        $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _VotingPreparedInfo() when $default != null:
        return $default(
            _that.roundId,
            _that.bundleIndex,
            _that.eligibleWeightZatoshi,
            _that.delegatedWeightZatoshi,
            _that.roundName);
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
            int bundleIndex,
            BigInt eligibleWeightZatoshi,
            BigInt delegatedWeightZatoshi,
            String roundName)
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingPreparedInfo():
        return $default(
            _that.roundId,
            _that.bundleIndex,
            _that.eligibleWeightZatoshi,
            _that.delegatedWeightZatoshi,
            _that.roundName);
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
            int bundleIndex,
            BigInt eligibleWeightZatoshi,
            BigInt delegatedWeightZatoshi,
            String roundName)?
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingPreparedInfo() when $default != null:
        return $default(
            _that.roundId,
            _that.bundleIndex,
            _that.eligibleWeightZatoshi,
            _that.delegatedWeightZatoshi,
            _that.roundName);
      case _:
        return null;
    }
  }
}

/// @nodoc

class _VotingPreparedInfo implements VotingPreparedInfo {
  const _VotingPreparedInfo(
      {required this.roundId,
      required this.bundleIndex,
      required this.eligibleWeightZatoshi,
      required this.delegatedWeightZatoshi,
      required this.roundName});

  @override
  final String roundId;
  @override
  final int bundleIndex;
  @override
  final BigInt eligibleWeightZatoshi;
  @override
  final BigInt delegatedWeightZatoshi;
  @override
  final String roundName;

  /// Create a copy of VotingPreparedInfo
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  _$VotingPreparedInfoCopyWith<_VotingPreparedInfo> get copyWith =>
      __$VotingPreparedInfoCopyWithImpl<_VotingPreparedInfo>(this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _VotingPreparedInfo &&
            (identical(other.roundId, roundId) || other.roundId == roundId) &&
            (identical(other.bundleIndex, bundleIndex) ||
                other.bundleIndex == bundleIndex) &&
            (identical(other.eligibleWeightZatoshi, eligibleWeightZatoshi) ||
                other.eligibleWeightZatoshi == eligibleWeightZatoshi) &&
            (identical(other.delegatedWeightZatoshi, delegatedWeightZatoshi) ||
                other.delegatedWeightZatoshi == delegatedWeightZatoshi) &&
            (identical(other.roundName, roundName) ||
                other.roundName == roundName));
  }

  @override
  int get hashCode => Object.hash(runtimeType, roundId, bundleIndex,
      eligibleWeightZatoshi, delegatedWeightZatoshi, roundName);

  @override
  String toString() {
    return 'VotingPreparedInfo(roundId: $roundId, bundleIndex: $bundleIndex, eligibleWeightZatoshi: $eligibleWeightZatoshi, delegatedWeightZatoshi: $delegatedWeightZatoshi, roundName: $roundName)';
  }
}

/// @nodoc
abstract mixin class _$VotingPreparedInfoCopyWith<$Res>
    implements $VotingPreparedInfoCopyWith<$Res> {
  factory _$VotingPreparedInfoCopyWith(
          _VotingPreparedInfo value, $Res Function(_VotingPreparedInfo) _then) =
      __$VotingPreparedInfoCopyWithImpl;
  @override
  @useResult
  $Res call(
      {String roundId,
      int bundleIndex,
      BigInt eligibleWeightZatoshi,
      BigInt delegatedWeightZatoshi,
      String roundName});
}

/// @nodoc
class __$VotingPreparedInfoCopyWithImpl<$Res>
    implements _$VotingPreparedInfoCopyWith<$Res> {
  __$VotingPreparedInfoCopyWithImpl(this._self, this._then);

  final _VotingPreparedInfo _self;
  final $Res Function(_VotingPreparedInfo) _then;

  /// Create a copy of VotingPreparedInfo
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $Res call({
    Object? roundId = null,
    Object? bundleIndex = null,
    Object? eligibleWeightZatoshi = null,
    Object? delegatedWeightZatoshi = null,
    Object? roundName = null,
  }) {
    return _then(_VotingPreparedInfo(
      roundId: null == roundId
          ? _self.roundId
          : roundId // ignore: cast_nullable_to_non_nullable
              as String,
      bundleIndex: null == bundleIndex
          ? _self.bundleIndex
          : bundleIndex // ignore: cast_nullable_to_non_nullable
              as int,
      eligibleWeightZatoshi: null == eligibleWeightZatoshi
          ? _self.eligibleWeightZatoshi
          : eligibleWeightZatoshi // ignore: cast_nullable_to_non_nullable
              as BigInt,
      delegatedWeightZatoshi: null == delegatedWeightZatoshi
          ? _self.delegatedWeightZatoshi
          : delegatedWeightZatoshi // ignore: cast_nullable_to_non_nullable
              as BigInt,
      roundName: null == roundName
          ? _self.roundName
          : roundName // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// @nodoc
mixin _$VotingSharePayload {
  Uint8List get sharesHash;
  int get proposalId;
  int get voteDecision;
  VotingEncryptedShare get encShare;
  BigInt get treePosition;
  List<VotingEncryptedShare> get allEncShares;
  List<Uint8List> get shareComms;
  Uint8List get primaryBlind;

  /// Create a copy of VotingSharePayload
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $VotingSharePayloadCopyWith<VotingSharePayload> get copyWith =>
      _$VotingSharePayloadCopyWithImpl<VotingSharePayload>(
          this as VotingSharePayload, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is VotingSharePayload &&
            const DeepCollectionEquality()
                .equals(other.sharesHash, sharesHash) &&
            (identical(other.proposalId, proposalId) ||
                other.proposalId == proposalId) &&
            (identical(other.voteDecision, voteDecision) ||
                other.voteDecision == voteDecision) &&
            (identical(other.encShare, encShare) ||
                other.encShare == encShare) &&
            (identical(other.treePosition, treePosition) ||
                other.treePosition == treePosition) &&
            const DeepCollectionEquality()
                .equals(other.allEncShares, allEncShares) &&
            const DeepCollectionEquality()
                .equals(other.shareComms, shareComms) &&
            const DeepCollectionEquality()
                .equals(other.primaryBlind, primaryBlind));
  }

  @override
  int get hashCode => Object.hash(
      runtimeType,
      const DeepCollectionEquality().hash(sharesHash),
      proposalId,
      voteDecision,
      encShare,
      treePosition,
      const DeepCollectionEquality().hash(allEncShares),
      const DeepCollectionEquality().hash(shareComms),
      const DeepCollectionEquality().hash(primaryBlind));

  @override
  String toString() {
    return 'VotingSharePayload(sharesHash: $sharesHash, proposalId: $proposalId, voteDecision: $voteDecision, encShare: $encShare, treePosition: $treePosition, allEncShares: $allEncShares, shareComms: $shareComms, primaryBlind: $primaryBlind)';
  }
}

/// @nodoc
abstract mixin class $VotingSharePayloadCopyWith<$Res> {
  factory $VotingSharePayloadCopyWith(
          VotingSharePayload value, $Res Function(VotingSharePayload) _then) =
      _$VotingSharePayloadCopyWithImpl;
  @useResult
  $Res call(
      {Uint8List sharesHash,
      int proposalId,
      int voteDecision,
      VotingEncryptedShare encShare,
      BigInt treePosition,
      List<VotingEncryptedShare> allEncShares,
      List<Uint8List> shareComms,
      Uint8List primaryBlind});

  $VotingEncryptedShareCopyWith<$Res> get encShare;
}

/// @nodoc
class _$VotingSharePayloadCopyWithImpl<$Res>
    implements $VotingSharePayloadCopyWith<$Res> {
  _$VotingSharePayloadCopyWithImpl(this._self, this._then);

  final VotingSharePayload _self;
  final $Res Function(VotingSharePayload) _then;

  /// Create a copy of VotingSharePayload
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? sharesHash = null,
    Object? proposalId = null,
    Object? voteDecision = null,
    Object? encShare = null,
    Object? treePosition = null,
    Object? allEncShares = null,
    Object? shareComms = null,
    Object? primaryBlind = null,
  }) {
    return _then(_self.copyWith(
      sharesHash: null == sharesHash
          ? _self.sharesHash
          : sharesHash // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      proposalId: null == proposalId
          ? _self.proposalId
          : proposalId // ignore: cast_nullable_to_non_nullable
              as int,
      voteDecision: null == voteDecision
          ? _self.voteDecision
          : voteDecision // ignore: cast_nullable_to_non_nullable
              as int,
      encShare: null == encShare
          ? _self.encShare
          : encShare // ignore: cast_nullable_to_non_nullable
              as VotingEncryptedShare,
      treePosition: null == treePosition
          ? _self.treePosition
          : treePosition // ignore: cast_nullable_to_non_nullable
              as BigInt,
      allEncShares: null == allEncShares
          ? _self.allEncShares
          : allEncShares // ignore: cast_nullable_to_non_nullable
              as List<VotingEncryptedShare>,
      shareComms: null == shareComms
          ? _self.shareComms
          : shareComms // ignore: cast_nullable_to_non_nullable
              as List<Uint8List>,
      primaryBlind: null == primaryBlind
          ? _self.primaryBlind
          : primaryBlind // ignore: cast_nullable_to_non_nullable
              as Uint8List,
    ));
  }

  /// Create a copy of VotingSharePayload
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $VotingEncryptedShareCopyWith<$Res> get encShare {
    return $VotingEncryptedShareCopyWith<$Res>(_self.encShare, (value) {
      return _then(_self.copyWith(encShare: value));
    });
  }
}

/// Adds pattern-matching-related methods to [VotingSharePayload].
extension VotingSharePayloadPatterns on VotingSharePayload {
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
    TResult Function(_VotingSharePayload value)? $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _VotingSharePayload() when $default != null:
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
    TResult Function(_VotingSharePayload value) $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingSharePayload():
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
    TResult? Function(_VotingSharePayload value)? $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingSharePayload() when $default != null:
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
            Uint8List sharesHash,
            int proposalId,
            int voteDecision,
            VotingEncryptedShare encShare,
            BigInt treePosition,
            List<VotingEncryptedShare> allEncShares,
            List<Uint8List> shareComms,
            Uint8List primaryBlind)?
        $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _VotingSharePayload() when $default != null:
        return $default(
            _that.sharesHash,
            _that.proposalId,
            _that.voteDecision,
            _that.encShare,
            _that.treePosition,
            _that.allEncShares,
            _that.shareComms,
            _that.primaryBlind);
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
            Uint8List sharesHash,
            int proposalId,
            int voteDecision,
            VotingEncryptedShare encShare,
            BigInt treePosition,
            List<VotingEncryptedShare> allEncShares,
            List<Uint8List> shareComms,
            Uint8List primaryBlind)
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingSharePayload():
        return $default(
            _that.sharesHash,
            _that.proposalId,
            _that.voteDecision,
            _that.encShare,
            _that.treePosition,
            _that.allEncShares,
            _that.shareComms,
            _that.primaryBlind);
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
            Uint8List sharesHash,
            int proposalId,
            int voteDecision,
            VotingEncryptedShare encShare,
            BigInt treePosition,
            List<VotingEncryptedShare> allEncShares,
            List<Uint8List> shareComms,
            Uint8List primaryBlind)?
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingSharePayload() when $default != null:
        return $default(
            _that.sharesHash,
            _that.proposalId,
            _that.voteDecision,
            _that.encShare,
            _that.treePosition,
            _that.allEncShares,
            _that.shareComms,
            _that.primaryBlind);
      case _:
        return null;
    }
  }
}

/// @nodoc

class _VotingSharePayload implements VotingSharePayload {
  const _VotingSharePayload(
      {required this.sharesHash,
      required this.proposalId,
      required this.voteDecision,
      required this.encShare,
      required this.treePosition,
      required final List<VotingEncryptedShare> allEncShares,
      required final List<Uint8List> shareComms,
      required this.primaryBlind})
      : _allEncShares = allEncShares,
        _shareComms = shareComms;

  @override
  final Uint8List sharesHash;
  @override
  final int proposalId;
  @override
  final int voteDecision;
  @override
  final VotingEncryptedShare encShare;
  @override
  final BigInt treePosition;
  final List<VotingEncryptedShare> _allEncShares;
  @override
  List<VotingEncryptedShare> get allEncShares {
    if (_allEncShares is EqualUnmodifiableListView) return _allEncShares;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_allEncShares);
  }

  final List<Uint8List> _shareComms;
  @override
  List<Uint8List> get shareComms {
    if (_shareComms is EqualUnmodifiableListView) return _shareComms;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_shareComms);
  }

  @override
  final Uint8List primaryBlind;

  /// Create a copy of VotingSharePayload
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  _$VotingSharePayloadCopyWith<_VotingSharePayload> get copyWith =>
      __$VotingSharePayloadCopyWithImpl<_VotingSharePayload>(this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _VotingSharePayload &&
            const DeepCollectionEquality()
                .equals(other.sharesHash, sharesHash) &&
            (identical(other.proposalId, proposalId) ||
                other.proposalId == proposalId) &&
            (identical(other.voteDecision, voteDecision) ||
                other.voteDecision == voteDecision) &&
            (identical(other.encShare, encShare) ||
                other.encShare == encShare) &&
            (identical(other.treePosition, treePosition) ||
                other.treePosition == treePosition) &&
            const DeepCollectionEquality()
                .equals(other._allEncShares, _allEncShares) &&
            const DeepCollectionEquality()
                .equals(other._shareComms, _shareComms) &&
            const DeepCollectionEquality()
                .equals(other.primaryBlind, primaryBlind));
  }

  @override
  int get hashCode => Object.hash(
      runtimeType,
      const DeepCollectionEquality().hash(sharesHash),
      proposalId,
      voteDecision,
      encShare,
      treePosition,
      const DeepCollectionEquality().hash(_allEncShares),
      const DeepCollectionEquality().hash(_shareComms),
      const DeepCollectionEquality().hash(primaryBlind));

  @override
  String toString() {
    return 'VotingSharePayload(sharesHash: $sharesHash, proposalId: $proposalId, voteDecision: $voteDecision, encShare: $encShare, treePosition: $treePosition, allEncShares: $allEncShares, shareComms: $shareComms, primaryBlind: $primaryBlind)';
  }
}

/// @nodoc
abstract mixin class _$VotingSharePayloadCopyWith<$Res>
    implements $VotingSharePayloadCopyWith<$Res> {
  factory _$VotingSharePayloadCopyWith(
          _VotingSharePayload value, $Res Function(_VotingSharePayload) _then) =
      __$VotingSharePayloadCopyWithImpl;
  @override
  @useResult
  $Res call(
      {Uint8List sharesHash,
      int proposalId,
      int voteDecision,
      VotingEncryptedShare encShare,
      BigInt treePosition,
      List<VotingEncryptedShare> allEncShares,
      List<Uint8List> shareComms,
      Uint8List primaryBlind});

  @override
  $VotingEncryptedShareCopyWith<$Res> get encShare;
}

/// @nodoc
class __$VotingSharePayloadCopyWithImpl<$Res>
    implements _$VotingSharePayloadCopyWith<$Res> {
  __$VotingSharePayloadCopyWithImpl(this._self, this._then);

  final _VotingSharePayload _self;
  final $Res Function(_VotingSharePayload) _then;

  /// Create a copy of VotingSharePayload
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $Res call({
    Object? sharesHash = null,
    Object? proposalId = null,
    Object? voteDecision = null,
    Object? encShare = null,
    Object? treePosition = null,
    Object? allEncShares = null,
    Object? shareComms = null,
    Object? primaryBlind = null,
  }) {
    return _then(_VotingSharePayload(
      sharesHash: null == sharesHash
          ? _self.sharesHash
          : sharesHash // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      proposalId: null == proposalId
          ? _self.proposalId
          : proposalId // ignore: cast_nullable_to_non_nullable
              as int,
      voteDecision: null == voteDecision
          ? _self.voteDecision
          : voteDecision // ignore: cast_nullable_to_non_nullable
              as int,
      encShare: null == encShare
          ? _self.encShare
          : encShare // ignore: cast_nullable_to_non_nullable
              as VotingEncryptedShare,
      treePosition: null == treePosition
          ? _self.treePosition
          : treePosition // ignore: cast_nullable_to_non_nullable
              as BigInt,
      allEncShares: null == allEncShares
          ? _self._allEncShares
          : allEncShares // ignore: cast_nullable_to_non_nullable
              as List<VotingEncryptedShare>,
      shareComms: null == shareComms
          ? _self._shareComms
          : shareComms // ignore: cast_nullable_to_non_nullable
              as List<Uint8List>,
      primaryBlind: null == primaryBlind
          ? _self.primaryBlind
          : primaryBlind // ignore: cast_nullable_to_non_nullable
              as Uint8List,
    ));
  }

  /// Create a copy of VotingSharePayload
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $VotingEncryptedShareCopyWith<$Res> get encShare {
    return $VotingEncryptedShareCopyWith<$Res>(_self.encShare, (value) {
      return _then(_self.copyWith(encShare: value));
    });
  }
}

/// @nodoc
mixin _$VotingSignedVoteCommitment {
  int get proposalId;
  int get choice;
  String get voteRoundId;
  Uint8List get vanNullifier;
  Uint8List get voteAuthorityNoteNew;
  Uint8List get voteCommitment;
  Uint8List get proof;
  int get anchorHeight;
  Uint8List get rVpk;
  Uint8List get voteAuthSig;
  String get commitmentBundleJson;

  /// Create a copy of VotingSignedVoteCommitment
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $VotingSignedVoteCommitmentCopyWith<VotingSignedVoteCommitment>
      get copyWith =>
          _$VotingSignedVoteCommitmentCopyWithImpl<VotingSignedVoteCommitment>(
              this as VotingSignedVoteCommitment, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is VotingSignedVoteCommitment &&
            (identical(other.proposalId, proposalId) ||
                other.proposalId == proposalId) &&
            (identical(other.choice, choice) || other.choice == choice) &&
            (identical(other.voteRoundId, voteRoundId) ||
                other.voteRoundId == voteRoundId) &&
            const DeepCollectionEquality()
                .equals(other.vanNullifier, vanNullifier) &&
            const DeepCollectionEquality()
                .equals(other.voteAuthorityNoteNew, voteAuthorityNoteNew) &&
            const DeepCollectionEquality()
                .equals(other.voteCommitment, voteCommitment) &&
            const DeepCollectionEquality().equals(other.proof, proof) &&
            (identical(other.anchorHeight, anchorHeight) ||
                other.anchorHeight == anchorHeight) &&
            const DeepCollectionEquality().equals(other.rVpk, rVpk) &&
            const DeepCollectionEquality()
                .equals(other.voteAuthSig, voteAuthSig) &&
            (identical(other.commitmentBundleJson, commitmentBundleJson) ||
                other.commitmentBundleJson == commitmentBundleJson));
  }

  @override
  int get hashCode => Object.hash(
      runtimeType,
      proposalId,
      choice,
      voteRoundId,
      const DeepCollectionEquality().hash(vanNullifier),
      const DeepCollectionEquality().hash(voteAuthorityNoteNew),
      const DeepCollectionEquality().hash(voteCommitment),
      const DeepCollectionEquality().hash(proof),
      anchorHeight,
      const DeepCollectionEquality().hash(rVpk),
      const DeepCollectionEquality().hash(voteAuthSig),
      commitmentBundleJson);

  @override
  String toString() {
    return 'VotingSignedVoteCommitment(proposalId: $proposalId, choice: $choice, voteRoundId: $voteRoundId, vanNullifier: $vanNullifier, voteAuthorityNoteNew: $voteAuthorityNoteNew, voteCommitment: $voteCommitment, proof: $proof, anchorHeight: $anchorHeight, rVpk: $rVpk, voteAuthSig: $voteAuthSig, commitmentBundleJson: $commitmentBundleJson)';
  }
}

/// @nodoc
abstract mixin class $VotingSignedVoteCommitmentCopyWith<$Res> {
  factory $VotingSignedVoteCommitmentCopyWith(VotingSignedVoteCommitment value,
          $Res Function(VotingSignedVoteCommitment) _then) =
      _$VotingSignedVoteCommitmentCopyWithImpl;
  @useResult
  $Res call(
      {int proposalId,
      int choice,
      String voteRoundId,
      Uint8List vanNullifier,
      Uint8List voteAuthorityNoteNew,
      Uint8List voteCommitment,
      Uint8List proof,
      int anchorHeight,
      Uint8List rVpk,
      Uint8List voteAuthSig,
      String commitmentBundleJson});
}

/// @nodoc
class _$VotingSignedVoteCommitmentCopyWithImpl<$Res>
    implements $VotingSignedVoteCommitmentCopyWith<$Res> {
  _$VotingSignedVoteCommitmentCopyWithImpl(this._self, this._then);

  final VotingSignedVoteCommitment _self;
  final $Res Function(VotingSignedVoteCommitment) _then;

  /// Create a copy of VotingSignedVoteCommitment
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? proposalId = null,
    Object? choice = null,
    Object? voteRoundId = null,
    Object? vanNullifier = null,
    Object? voteAuthorityNoteNew = null,
    Object? voteCommitment = null,
    Object? proof = null,
    Object? anchorHeight = null,
    Object? rVpk = null,
    Object? voteAuthSig = null,
    Object? commitmentBundleJson = null,
  }) {
    return _then(_self.copyWith(
      proposalId: null == proposalId
          ? _self.proposalId
          : proposalId // ignore: cast_nullable_to_non_nullable
              as int,
      choice: null == choice
          ? _self.choice
          : choice // ignore: cast_nullable_to_non_nullable
              as int,
      voteRoundId: null == voteRoundId
          ? _self.voteRoundId
          : voteRoundId // ignore: cast_nullable_to_non_nullable
              as String,
      vanNullifier: null == vanNullifier
          ? _self.vanNullifier
          : vanNullifier // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      voteAuthorityNoteNew: null == voteAuthorityNoteNew
          ? _self.voteAuthorityNoteNew
          : voteAuthorityNoteNew // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      voteCommitment: null == voteCommitment
          ? _self.voteCommitment
          : voteCommitment // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      proof: null == proof
          ? _self.proof
          : proof // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      anchorHeight: null == anchorHeight
          ? _self.anchorHeight
          : anchorHeight // ignore: cast_nullable_to_non_nullable
              as int,
      rVpk: null == rVpk
          ? _self.rVpk
          : rVpk // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      voteAuthSig: null == voteAuthSig
          ? _self.voteAuthSig
          : voteAuthSig // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      commitmentBundleJson: null == commitmentBundleJson
          ? _self.commitmentBundleJson
          : commitmentBundleJson // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// Adds pattern-matching-related methods to [VotingSignedVoteCommitment].
extension VotingSignedVoteCommitmentPatterns on VotingSignedVoteCommitment {
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
    TResult Function(_VotingSignedVoteCommitment value)? $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _VotingSignedVoteCommitment() when $default != null:
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
    TResult Function(_VotingSignedVoteCommitment value) $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingSignedVoteCommitment():
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
    TResult? Function(_VotingSignedVoteCommitment value)? $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingSignedVoteCommitment() when $default != null:
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
            int proposalId,
            int choice,
            String voteRoundId,
            Uint8List vanNullifier,
            Uint8List voteAuthorityNoteNew,
            Uint8List voteCommitment,
            Uint8List proof,
            int anchorHeight,
            Uint8List rVpk,
            Uint8List voteAuthSig,
            String commitmentBundleJson)?
        $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _VotingSignedVoteCommitment() when $default != null:
        return $default(
            _that.proposalId,
            _that.choice,
            _that.voteRoundId,
            _that.vanNullifier,
            _that.voteAuthorityNoteNew,
            _that.voteCommitment,
            _that.proof,
            _that.anchorHeight,
            _that.rVpk,
            _that.voteAuthSig,
            _that.commitmentBundleJson);
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
            int proposalId,
            int choice,
            String voteRoundId,
            Uint8List vanNullifier,
            Uint8List voteAuthorityNoteNew,
            Uint8List voteCommitment,
            Uint8List proof,
            int anchorHeight,
            Uint8List rVpk,
            Uint8List voteAuthSig,
            String commitmentBundleJson)
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingSignedVoteCommitment():
        return $default(
            _that.proposalId,
            _that.choice,
            _that.voteRoundId,
            _that.vanNullifier,
            _that.voteAuthorityNoteNew,
            _that.voteCommitment,
            _that.proof,
            _that.anchorHeight,
            _that.rVpk,
            _that.voteAuthSig,
            _that.commitmentBundleJson);
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
            int proposalId,
            int choice,
            String voteRoundId,
            Uint8List vanNullifier,
            Uint8List voteAuthorityNoteNew,
            Uint8List voteCommitment,
            Uint8List proof,
            int anchorHeight,
            Uint8List rVpk,
            Uint8List voteAuthSig,
            String commitmentBundleJson)?
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingSignedVoteCommitment() when $default != null:
        return $default(
            _that.proposalId,
            _that.choice,
            _that.voteRoundId,
            _that.vanNullifier,
            _that.voteAuthorityNoteNew,
            _that.voteCommitment,
            _that.proof,
            _that.anchorHeight,
            _that.rVpk,
            _that.voteAuthSig,
            _that.commitmentBundleJson);
      case _:
        return null;
    }
  }
}

/// @nodoc

class _VotingSignedVoteCommitment implements VotingSignedVoteCommitment {
  const _VotingSignedVoteCommitment(
      {required this.proposalId,
      required this.choice,
      required this.voteRoundId,
      required this.vanNullifier,
      required this.voteAuthorityNoteNew,
      required this.voteCommitment,
      required this.proof,
      required this.anchorHeight,
      required this.rVpk,
      required this.voteAuthSig,
      required this.commitmentBundleJson});

  @override
  final int proposalId;
  @override
  final int choice;
  @override
  final String voteRoundId;
  @override
  final Uint8List vanNullifier;
  @override
  final Uint8List voteAuthorityNoteNew;
  @override
  final Uint8List voteCommitment;
  @override
  final Uint8List proof;
  @override
  final int anchorHeight;
  @override
  final Uint8List rVpk;
  @override
  final Uint8List voteAuthSig;
  @override
  final String commitmentBundleJson;

  /// Create a copy of VotingSignedVoteCommitment
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  _$VotingSignedVoteCommitmentCopyWith<_VotingSignedVoteCommitment>
      get copyWith => __$VotingSignedVoteCommitmentCopyWithImpl<
          _VotingSignedVoteCommitment>(this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _VotingSignedVoteCommitment &&
            (identical(other.proposalId, proposalId) ||
                other.proposalId == proposalId) &&
            (identical(other.choice, choice) || other.choice == choice) &&
            (identical(other.voteRoundId, voteRoundId) ||
                other.voteRoundId == voteRoundId) &&
            const DeepCollectionEquality()
                .equals(other.vanNullifier, vanNullifier) &&
            const DeepCollectionEquality()
                .equals(other.voteAuthorityNoteNew, voteAuthorityNoteNew) &&
            const DeepCollectionEquality()
                .equals(other.voteCommitment, voteCommitment) &&
            const DeepCollectionEquality().equals(other.proof, proof) &&
            (identical(other.anchorHeight, anchorHeight) ||
                other.anchorHeight == anchorHeight) &&
            const DeepCollectionEquality().equals(other.rVpk, rVpk) &&
            const DeepCollectionEquality()
                .equals(other.voteAuthSig, voteAuthSig) &&
            (identical(other.commitmentBundleJson, commitmentBundleJson) ||
                other.commitmentBundleJson == commitmentBundleJson));
  }

  @override
  int get hashCode => Object.hash(
      runtimeType,
      proposalId,
      choice,
      voteRoundId,
      const DeepCollectionEquality().hash(vanNullifier),
      const DeepCollectionEquality().hash(voteAuthorityNoteNew),
      const DeepCollectionEquality().hash(voteCommitment),
      const DeepCollectionEquality().hash(proof),
      anchorHeight,
      const DeepCollectionEquality().hash(rVpk),
      const DeepCollectionEquality().hash(voteAuthSig),
      commitmentBundleJson);

  @override
  String toString() {
    return 'VotingSignedVoteCommitment(proposalId: $proposalId, choice: $choice, voteRoundId: $voteRoundId, vanNullifier: $vanNullifier, voteAuthorityNoteNew: $voteAuthorityNoteNew, voteCommitment: $voteCommitment, proof: $proof, anchorHeight: $anchorHeight, rVpk: $rVpk, voteAuthSig: $voteAuthSig, commitmentBundleJson: $commitmentBundleJson)';
  }
}

/// @nodoc
abstract mixin class _$VotingSignedVoteCommitmentCopyWith<$Res>
    implements $VotingSignedVoteCommitmentCopyWith<$Res> {
  factory _$VotingSignedVoteCommitmentCopyWith(
          _VotingSignedVoteCommitment value,
          $Res Function(_VotingSignedVoteCommitment) _then) =
      __$VotingSignedVoteCommitmentCopyWithImpl;
  @override
  @useResult
  $Res call(
      {int proposalId,
      int choice,
      String voteRoundId,
      Uint8List vanNullifier,
      Uint8List voteAuthorityNoteNew,
      Uint8List voteCommitment,
      Uint8List proof,
      int anchorHeight,
      Uint8List rVpk,
      Uint8List voteAuthSig,
      String commitmentBundleJson});
}

/// @nodoc
class __$VotingSignedVoteCommitmentCopyWithImpl<$Res>
    implements _$VotingSignedVoteCommitmentCopyWith<$Res> {
  __$VotingSignedVoteCommitmentCopyWithImpl(this._self, this._then);

  final _VotingSignedVoteCommitment _self;
  final $Res Function(_VotingSignedVoteCommitment) _then;

  /// Create a copy of VotingSignedVoteCommitment
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $Res call({
    Object? proposalId = null,
    Object? choice = null,
    Object? voteRoundId = null,
    Object? vanNullifier = null,
    Object? voteAuthorityNoteNew = null,
    Object? voteCommitment = null,
    Object? proof = null,
    Object? anchorHeight = null,
    Object? rVpk = null,
    Object? voteAuthSig = null,
    Object? commitmentBundleJson = null,
  }) {
    return _then(_VotingSignedVoteCommitment(
      proposalId: null == proposalId
          ? _self.proposalId
          : proposalId // ignore: cast_nullable_to_non_nullable
              as int,
      choice: null == choice
          ? _self.choice
          : choice // ignore: cast_nullable_to_non_nullable
              as int,
      voteRoundId: null == voteRoundId
          ? _self.voteRoundId
          : voteRoundId // ignore: cast_nullable_to_non_nullable
              as String,
      vanNullifier: null == vanNullifier
          ? _self.vanNullifier
          : vanNullifier // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      voteAuthorityNoteNew: null == voteAuthorityNoteNew
          ? _self.voteAuthorityNoteNew
          : voteAuthorityNoteNew // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      voteCommitment: null == voteCommitment
          ? _self.voteCommitment
          : voteCommitment // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      proof: null == proof
          ? _self.proof
          : proof // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      anchorHeight: null == anchorHeight
          ? _self.anchorHeight
          : anchorHeight // ignore: cast_nullable_to_non_nullable
              as int,
      rVpk: null == rVpk
          ? _self.rVpk
          : rVpk // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      voteAuthSig: null == voteAuthSig
          ? _self.voteAuthSig
          : voteAuthSig // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      commitmentBundleJson: null == commitmentBundleJson
          ? _self.commitmentBundleJson
          : commitmentBundleJson // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// @nodoc
mixin _$VotingVanWitness {
  List<Uint8List> get authPath;
  int get position;
  int get anchorHeight;

  /// Create a copy of VotingVanWitness
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $VotingVanWitnessCopyWith<VotingVanWitness> get copyWith =>
      _$VotingVanWitnessCopyWithImpl<VotingVanWitness>(
          this as VotingVanWitness, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is VotingVanWitness &&
            const DeepCollectionEquality().equals(other.authPath, authPath) &&
            (identical(other.position, position) ||
                other.position == position) &&
            (identical(other.anchorHeight, anchorHeight) ||
                other.anchorHeight == anchorHeight));
  }

  @override
  int get hashCode => Object.hash(runtimeType,
      const DeepCollectionEquality().hash(authPath), position, anchorHeight);

  @override
  String toString() {
    return 'VotingVanWitness(authPath: $authPath, position: $position, anchorHeight: $anchorHeight)';
  }
}

/// @nodoc
abstract mixin class $VotingVanWitnessCopyWith<$Res> {
  factory $VotingVanWitnessCopyWith(
          VotingVanWitness value, $Res Function(VotingVanWitness) _then) =
      _$VotingVanWitnessCopyWithImpl;
  @useResult
  $Res call({List<Uint8List> authPath, int position, int anchorHeight});
}

/// @nodoc
class _$VotingVanWitnessCopyWithImpl<$Res>
    implements $VotingVanWitnessCopyWith<$Res> {
  _$VotingVanWitnessCopyWithImpl(this._self, this._then);

  final VotingVanWitness _self;
  final $Res Function(VotingVanWitness) _then;

  /// Create a copy of VotingVanWitness
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? authPath = null,
    Object? position = null,
    Object? anchorHeight = null,
  }) {
    return _then(_self.copyWith(
      authPath: null == authPath
          ? _self.authPath
          : authPath // ignore: cast_nullable_to_non_nullable
              as List<Uint8List>,
      position: null == position
          ? _self.position
          : position // ignore: cast_nullable_to_non_nullable
              as int,
      anchorHeight: null == anchorHeight
          ? _self.anchorHeight
          : anchorHeight // ignore: cast_nullable_to_non_nullable
              as int,
    ));
  }
}

/// Adds pattern-matching-related methods to [VotingVanWitness].
extension VotingVanWitnessPatterns on VotingVanWitness {
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
    TResult Function(_VotingVanWitness value)? $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _VotingVanWitness() when $default != null:
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
    TResult Function(_VotingVanWitness value) $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingVanWitness():
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
    TResult? Function(_VotingVanWitness value)? $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingVanWitness() when $default != null:
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
    TResult Function(List<Uint8List> authPath, int position, int anchorHeight)?
        $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _VotingVanWitness() when $default != null:
        return $default(_that.authPath, _that.position, _that.anchorHeight);
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
    TResult Function(List<Uint8List> authPath, int position, int anchorHeight)
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingVanWitness():
        return $default(_that.authPath, _that.position, _that.anchorHeight);
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
    TResult? Function(List<Uint8List> authPath, int position, int anchorHeight)?
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingVanWitness() when $default != null:
        return $default(_that.authPath, _that.position, _that.anchorHeight);
      case _:
        return null;
    }
  }
}

/// @nodoc

class _VotingVanWitness implements VotingVanWitness {
  const _VotingVanWitness(
      {required final List<Uint8List> authPath,
      required this.position,
      required this.anchorHeight})
      : _authPath = authPath;

  final List<Uint8List> _authPath;
  @override
  List<Uint8List> get authPath {
    if (_authPath is EqualUnmodifiableListView) return _authPath;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_authPath);
  }

  @override
  final int position;
  @override
  final int anchorHeight;

  /// Create a copy of VotingVanWitness
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  _$VotingVanWitnessCopyWith<_VotingVanWitness> get copyWith =>
      __$VotingVanWitnessCopyWithImpl<_VotingVanWitness>(this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _VotingVanWitness &&
            const DeepCollectionEquality().equals(other._authPath, _authPath) &&
            (identical(other.position, position) ||
                other.position == position) &&
            (identical(other.anchorHeight, anchorHeight) ||
                other.anchorHeight == anchorHeight));
  }

  @override
  int get hashCode => Object.hash(runtimeType,
      const DeepCollectionEquality().hash(_authPath), position, anchorHeight);

  @override
  String toString() {
    return 'VotingVanWitness(authPath: $authPath, position: $position, anchorHeight: $anchorHeight)';
  }
}

/// @nodoc
abstract mixin class _$VotingVanWitnessCopyWith<$Res>
    implements $VotingVanWitnessCopyWith<$Res> {
  factory _$VotingVanWitnessCopyWith(
          _VotingVanWitness value, $Res Function(_VotingVanWitness) _then) =
      __$VotingVanWitnessCopyWithImpl;
  @override
  @useResult
  $Res call({List<Uint8List> authPath, int position, int anchorHeight});
}

/// @nodoc
class __$VotingVanWitnessCopyWithImpl<$Res>
    implements _$VotingVanWitnessCopyWith<$Res> {
  __$VotingVanWitnessCopyWithImpl(this._self, this._then);

  final _VotingVanWitness _self;
  final $Res Function(_VotingVanWitness) _then;

  /// Create a copy of VotingVanWitness
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $Res call({
    Object? authPath = null,
    Object? position = null,
    Object? anchorHeight = null,
  }) {
    return _then(_VotingVanWitness(
      authPath: null == authPath
          ? _self._authPath
          : authPath // ignore: cast_nullable_to_non_nullable
              as List<Uint8List>,
      position: null == position
          ? _self.position
          : position // ignore: cast_nullable_to_non_nullable
              as int,
      anchorHeight: null == anchorHeight
          ? _self.anchorHeight
          : anchorHeight // ignore: cast_nullable_to_non_nullable
              as int,
    ));
  }
}

/// @nodoc
mixin _$VotingVoteCommitments {
  int get bundleIndex;
  List<VotingSignedVoteCommitment> get commitments;

  /// Create a copy of VotingVoteCommitments
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $VotingVoteCommitmentsCopyWith<VotingVoteCommitments> get copyWith =>
      _$VotingVoteCommitmentsCopyWithImpl<VotingVoteCommitments>(
          this as VotingVoteCommitments, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is VotingVoteCommitments &&
            (identical(other.bundleIndex, bundleIndex) ||
                other.bundleIndex == bundleIndex) &&
            const DeepCollectionEquality()
                .equals(other.commitments, commitments));
  }

  @override
  int get hashCode => Object.hash(runtimeType, bundleIndex,
      const DeepCollectionEquality().hash(commitments));

  @override
  String toString() {
    return 'VotingVoteCommitments(bundleIndex: $bundleIndex, commitments: $commitments)';
  }
}

/// @nodoc
abstract mixin class $VotingVoteCommitmentsCopyWith<$Res> {
  factory $VotingVoteCommitmentsCopyWith(VotingVoteCommitments value,
          $Res Function(VotingVoteCommitments) _then) =
      _$VotingVoteCommitmentsCopyWithImpl;
  @useResult
  $Res call({int bundleIndex, List<VotingSignedVoteCommitment> commitments});
}

/// @nodoc
class _$VotingVoteCommitmentsCopyWithImpl<$Res>
    implements $VotingVoteCommitmentsCopyWith<$Res> {
  _$VotingVoteCommitmentsCopyWithImpl(this._self, this._then);

  final VotingVoteCommitments _self;
  final $Res Function(VotingVoteCommitments) _then;

  /// Create a copy of VotingVoteCommitments
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? bundleIndex = null,
    Object? commitments = null,
  }) {
    return _then(_self.copyWith(
      bundleIndex: null == bundleIndex
          ? _self.bundleIndex
          : bundleIndex // ignore: cast_nullable_to_non_nullable
              as int,
      commitments: null == commitments
          ? _self.commitments
          : commitments // ignore: cast_nullable_to_non_nullable
              as List<VotingSignedVoteCommitment>,
    ));
  }
}

/// Adds pattern-matching-related methods to [VotingVoteCommitments].
extension VotingVoteCommitmentsPatterns on VotingVoteCommitments {
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
    TResult Function(_VotingVoteCommitments value)? $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _VotingVoteCommitments() when $default != null:
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
    TResult Function(_VotingVoteCommitments value) $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingVoteCommitments():
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
    TResult? Function(_VotingVoteCommitments value)? $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingVoteCommitments() when $default != null:
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
            int bundleIndex, List<VotingSignedVoteCommitment> commitments)?
        $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _VotingVoteCommitments() when $default != null:
        return $default(_that.bundleIndex, _that.commitments);
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
            int bundleIndex, List<VotingSignedVoteCommitment> commitments)
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingVoteCommitments():
        return $default(_that.bundleIndex, _that.commitments);
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
            int bundleIndex, List<VotingSignedVoteCommitment> commitments)?
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingVoteCommitments() when $default != null:
        return $default(_that.bundleIndex, _that.commitments);
      case _:
        return null;
    }
  }
}

/// @nodoc

class _VotingVoteCommitments implements VotingVoteCommitments {
  const _VotingVoteCommitments(
      {required this.bundleIndex,
      required final List<VotingSignedVoteCommitment> commitments})
      : _commitments = commitments;

  @override
  final int bundleIndex;
  final List<VotingSignedVoteCommitment> _commitments;
  @override
  List<VotingSignedVoteCommitment> get commitments {
    if (_commitments is EqualUnmodifiableListView) return _commitments;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_commitments);
  }

  /// Create a copy of VotingVoteCommitments
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  _$VotingVoteCommitmentsCopyWith<_VotingVoteCommitments> get copyWith =>
      __$VotingVoteCommitmentsCopyWithImpl<_VotingVoteCommitments>(
          this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _VotingVoteCommitments &&
            (identical(other.bundleIndex, bundleIndex) ||
                other.bundleIndex == bundleIndex) &&
            const DeepCollectionEquality()
                .equals(other._commitments, _commitments));
  }

  @override
  int get hashCode => Object.hash(runtimeType, bundleIndex,
      const DeepCollectionEquality().hash(_commitments));

  @override
  String toString() {
    return 'VotingVoteCommitments(bundleIndex: $bundleIndex, commitments: $commitments)';
  }
}

/// @nodoc
abstract mixin class _$VotingVoteCommitmentsCopyWith<$Res>
    implements $VotingVoteCommitmentsCopyWith<$Res> {
  factory _$VotingVoteCommitmentsCopyWith(_VotingVoteCommitments value,
          $Res Function(_VotingVoteCommitments) _then) =
      __$VotingVoteCommitmentsCopyWithImpl;
  @override
  @useResult
  $Res call({int bundleIndex, List<VotingSignedVoteCommitment> commitments});
}

/// @nodoc
class __$VotingVoteCommitmentsCopyWithImpl<$Res>
    implements _$VotingVoteCommitmentsCopyWith<$Res> {
  __$VotingVoteCommitmentsCopyWithImpl(this._self, this._then);

  final _VotingVoteCommitments _self;
  final $Res Function(_VotingVoteCommitments) _then;

  /// Create a copy of VotingVoteCommitments
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $Res call({
    Object? bundleIndex = null,
    Object? commitments = null,
  }) {
    return _then(_VotingVoteCommitments(
      bundleIndex: null == bundleIndex
          ? _self.bundleIndex
          : bundleIndex // ignore: cast_nullable_to_non_nullable
              as int,
      commitments: null == commitments
          ? _self._commitments
          : commitments // ignore: cast_nullable_to_non_nullable
              as List<VotingSignedVoteCommitment>,
    ));
  }
}

/// @nodoc
mixin _$VotingVoteConfirmation {
  String get txHash;
  int get vanLeafPosition;
  BigInt get vcTreePosition;

  /// Create a copy of VotingVoteConfirmation
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $VotingVoteConfirmationCopyWith<VotingVoteConfirmation> get copyWith =>
      _$VotingVoteConfirmationCopyWithImpl<VotingVoteConfirmation>(
          this as VotingVoteConfirmation, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is VotingVoteConfirmation &&
            (identical(other.txHash, txHash) || other.txHash == txHash) &&
            (identical(other.vanLeafPosition, vanLeafPosition) ||
                other.vanLeafPosition == vanLeafPosition) &&
            (identical(other.vcTreePosition, vcTreePosition) ||
                other.vcTreePosition == vcTreePosition));
  }

  @override
  int get hashCode =>
      Object.hash(runtimeType, txHash, vanLeafPosition, vcTreePosition);

  @override
  String toString() {
    return 'VotingVoteConfirmation(txHash: $txHash, vanLeafPosition: $vanLeafPosition, vcTreePosition: $vcTreePosition)';
  }
}

/// @nodoc
abstract mixin class $VotingVoteConfirmationCopyWith<$Res> {
  factory $VotingVoteConfirmationCopyWith(VotingVoteConfirmation value,
          $Res Function(VotingVoteConfirmation) _then) =
      _$VotingVoteConfirmationCopyWithImpl;
  @useResult
  $Res call({String txHash, int vanLeafPosition, BigInt vcTreePosition});
}

/// @nodoc
class _$VotingVoteConfirmationCopyWithImpl<$Res>
    implements $VotingVoteConfirmationCopyWith<$Res> {
  _$VotingVoteConfirmationCopyWithImpl(this._self, this._then);

  final VotingVoteConfirmation _self;
  final $Res Function(VotingVoteConfirmation) _then;

  /// Create a copy of VotingVoteConfirmation
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? txHash = null,
    Object? vanLeafPosition = null,
    Object? vcTreePosition = null,
  }) {
    return _then(_self.copyWith(
      txHash: null == txHash
          ? _self.txHash
          : txHash // ignore: cast_nullable_to_non_nullable
              as String,
      vanLeafPosition: null == vanLeafPosition
          ? _self.vanLeafPosition
          : vanLeafPosition // ignore: cast_nullable_to_non_nullable
              as int,
      vcTreePosition: null == vcTreePosition
          ? _self.vcTreePosition
          : vcTreePosition // ignore: cast_nullable_to_non_nullable
              as BigInt,
    ));
  }
}

/// Adds pattern-matching-related methods to [VotingVoteConfirmation].
extension VotingVoteConfirmationPatterns on VotingVoteConfirmation {
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
    TResult Function(_VotingVoteConfirmation value)? $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _VotingVoteConfirmation() when $default != null:
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
    TResult Function(_VotingVoteConfirmation value) $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingVoteConfirmation():
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
    TResult? Function(_VotingVoteConfirmation value)? $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingVoteConfirmation() when $default != null:
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
    TResult Function(String txHash, int vanLeafPosition, BigInt vcTreePosition)?
        $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _VotingVoteConfirmation() when $default != null:
        return $default(
            _that.txHash, _that.vanLeafPosition, _that.vcTreePosition);
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
    TResult Function(String txHash, int vanLeafPosition, BigInt vcTreePosition)
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingVoteConfirmation():
        return $default(
            _that.txHash, _that.vanLeafPosition, _that.vcTreePosition);
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
            String txHash, int vanLeafPosition, BigInt vcTreePosition)?
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingVoteConfirmation() when $default != null:
        return $default(
            _that.txHash, _that.vanLeafPosition, _that.vcTreePosition);
      case _:
        return null;
    }
  }
}

/// @nodoc

class _VotingVoteConfirmation implements VotingVoteConfirmation {
  const _VotingVoteConfirmation(
      {required this.txHash,
      required this.vanLeafPosition,
      required this.vcTreePosition});

  @override
  final String txHash;
  @override
  final int vanLeafPosition;
  @override
  final BigInt vcTreePosition;

  /// Create a copy of VotingVoteConfirmation
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  _$VotingVoteConfirmationCopyWith<_VotingVoteConfirmation> get copyWith =>
      __$VotingVoteConfirmationCopyWithImpl<_VotingVoteConfirmation>(
          this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _VotingVoteConfirmation &&
            (identical(other.txHash, txHash) || other.txHash == txHash) &&
            (identical(other.vanLeafPosition, vanLeafPosition) ||
                other.vanLeafPosition == vanLeafPosition) &&
            (identical(other.vcTreePosition, vcTreePosition) ||
                other.vcTreePosition == vcTreePosition));
  }

  @override
  int get hashCode =>
      Object.hash(runtimeType, txHash, vanLeafPosition, vcTreePosition);

  @override
  String toString() {
    return 'VotingVoteConfirmation(txHash: $txHash, vanLeafPosition: $vanLeafPosition, vcTreePosition: $vcTreePosition)';
  }
}

/// @nodoc
abstract mixin class _$VotingVoteConfirmationCopyWith<$Res>
    implements $VotingVoteConfirmationCopyWith<$Res> {
  factory _$VotingVoteConfirmationCopyWith(_VotingVoteConfirmation value,
          $Res Function(_VotingVoteConfirmation) _then) =
      __$VotingVoteConfirmationCopyWithImpl;
  @override
  @useResult
  $Res call({String txHash, int vanLeafPosition, BigInt vcTreePosition});
}

/// @nodoc
class __$VotingVoteConfirmationCopyWithImpl<$Res>
    implements _$VotingVoteConfirmationCopyWith<$Res> {
  __$VotingVoteConfirmationCopyWithImpl(this._self, this._then);

  final _VotingVoteConfirmation _self;
  final $Res Function(_VotingVoteConfirmation) _then;

  /// Create a copy of VotingVoteConfirmation
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $Res call({
    Object? txHash = null,
    Object? vanLeafPosition = null,
    Object? vcTreePosition = null,
  }) {
    return _then(_VotingVoteConfirmation(
      txHash: null == txHash
          ? _self.txHash
          : txHash // ignore: cast_nullable_to_non_nullable
              as String,
      vanLeafPosition: null == vanLeafPosition
          ? _self.vanLeafPosition
          : vanLeafPosition // ignore: cast_nullable_to_non_nullable
              as int,
      vcTreePosition: null == vcTreePosition
          ? _self.vcTreePosition
          : vcTreePosition // ignore: cast_nullable_to_non_nullable
              as BigInt,
    ));
  }
}

/// @nodoc
mixin _$VotingVotePayloads {
  VotingVoteSubmission get submission;
  List<VotingSharePayload> get sharePayloads;

  /// Create a copy of VotingVotePayloads
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $VotingVotePayloadsCopyWith<VotingVotePayloads> get copyWith =>
      _$VotingVotePayloadsCopyWithImpl<VotingVotePayloads>(
          this as VotingVotePayloads, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is VotingVotePayloads &&
            (identical(other.submission, submission) ||
                other.submission == submission) &&
            const DeepCollectionEquality()
                .equals(other.sharePayloads, sharePayloads));
  }

  @override
  int get hashCode => Object.hash(runtimeType, submission,
      const DeepCollectionEquality().hash(sharePayloads));

  @override
  String toString() {
    return 'VotingVotePayloads(submission: $submission, sharePayloads: $sharePayloads)';
  }
}

/// @nodoc
abstract mixin class $VotingVotePayloadsCopyWith<$Res> {
  factory $VotingVotePayloadsCopyWith(
          VotingVotePayloads value, $Res Function(VotingVotePayloads) _then) =
      _$VotingVotePayloadsCopyWithImpl;
  @useResult
  $Res call(
      {VotingVoteSubmission submission,
      List<VotingSharePayload> sharePayloads});

  $VotingVoteSubmissionCopyWith<$Res> get submission;
}

/// @nodoc
class _$VotingVotePayloadsCopyWithImpl<$Res>
    implements $VotingVotePayloadsCopyWith<$Res> {
  _$VotingVotePayloadsCopyWithImpl(this._self, this._then);

  final VotingVotePayloads _self;
  final $Res Function(VotingVotePayloads) _then;

  /// Create a copy of VotingVotePayloads
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? submission = null,
    Object? sharePayloads = null,
  }) {
    return _then(_self.copyWith(
      submission: null == submission
          ? _self.submission
          : submission // ignore: cast_nullable_to_non_nullable
              as VotingVoteSubmission,
      sharePayloads: null == sharePayloads
          ? _self.sharePayloads
          : sharePayloads // ignore: cast_nullable_to_non_nullable
              as List<VotingSharePayload>,
    ));
  }

  /// Create a copy of VotingVotePayloads
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $VotingVoteSubmissionCopyWith<$Res> get submission {
    return $VotingVoteSubmissionCopyWith<$Res>(_self.submission, (value) {
      return _then(_self.copyWith(submission: value));
    });
  }
}

/// Adds pattern-matching-related methods to [VotingVotePayloads].
extension VotingVotePayloadsPatterns on VotingVotePayloads {
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
    TResult Function(_VotingVotePayloads value)? $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _VotingVotePayloads() when $default != null:
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
    TResult Function(_VotingVotePayloads value) $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingVotePayloads():
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
    TResult? Function(_VotingVotePayloads value)? $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingVotePayloads() when $default != null:
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
    TResult Function(VotingVoteSubmission submission,
            List<VotingSharePayload> sharePayloads)?
        $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _VotingVotePayloads() when $default != null:
        return $default(_that.submission, _that.sharePayloads);
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
    TResult Function(VotingVoteSubmission submission,
            List<VotingSharePayload> sharePayloads)
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingVotePayloads():
        return $default(_that.submission, _that.sharePayloads);
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
    TResult? Function(VotingVoteSubmission submission,
            List<VotingSharePayload> sharePayloads)?
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingVotePayloads() when $default != null:
        return $default(_that.submission, _that.sharePayloads);
      case _:
        return null;
    }
  }
}

/// @nodoc

class _VotingVotePayloads implements VotingVotePayloads {
  const _VotingVotePayloads(
      {required this.submission,
      required final List<VotingSharePayload> sharePayloads})
      : _sharePayloads = sharePayloads;

  @override
  final VotingVoteSubmission submission;
  final List<VotingSharePayload> _sharePayloads;
  @override
  List<VotingSharePayload> get sharePayloads {
    if (_sharePayloads is EqualUnmodifiableListView) return _sharePayloads;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_sharePayloads);
  }

  /// Create a copy of VotingVotePayloads
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  _$VotingVotePayloadsCopyWith<_VotingVotePayloads> get copyWith =>
      __$VotingVotePayloadsCopyWithImpl<_VotingVotePayloads>(this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _VotingVotePayloads &&
            (identical(other.submission, submission) ||
                other.submission == submission) &&
            const DeepCollectionEquality()
                .equals(other._sharePayloads, _sharePayloads));
  }

  @override
  int get hashCode => Object.hash(runtimeType, submission,
      const DeepCollectionEquality().hash(_sharePayloads));

  @override
  String toString() {
    return 'VotingVotePayloads(submission: $submission, sharePayloads: $sharePayloads)';
  }
}

/// @nodoc
abstract mixin class _$VotingVotePayloadsCopyWith<$Res>
    implements $VotingVotePayloadsCopyWith<$Res> {
  factory _$VotingVotePayloadsCopyWith(
          _VotingVotePayloads value, $Res Function(_VotingVotePayloads) _then) =
      __$VotingVotePayloadsCopyWithImpl;
  @override
  @useResult
  $Res call(
      {VotingVoteSubmission submission,
      List<VotingSharePayload> sharePayloads});

  @override
  $VotingVoteSubmissionCopyWith<$Res> get submission;
}

/// @nodoc
class __$VotingVotePayloadsCopyWithImpl<$Res>
    implements _$VotingVotePayloadsCopyWith<$Res> {
  __$VotingVotePayloadsCopyWithImpl(this._self, this._then);

  final _VotingVotePayloads _self;
  final $Res Function(_VotingVotePayloads) _then;

  /// Create a copy of VotingVotePayloads
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $Res call({
    Object? submission = null,
    Object? sharePayloads = null,
  }) {
    return _then(_VotingVotePayloads(
      submission: null == submission
          ? _self.submission
          : submission // ignore: cast_nullable_to_non_nullable
              as VotingVoteSubmission,
      sharePayloads: null == sharePayloads
          ? _self._sharePayloads
          : sharePayloads // ignore: cast_nullable_to_non_nullable
              as List<VotingSharePayload>,
    ));
  }

  /// Create a copy of VotingVotePayloads
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $VotingVoteSubmissionCopyWith<$Res> get submission {
    return $VotingVoteSubmissionCopyWith<$Res>(_self.submission, (value) {
      return _then(_self.copyWith(submission: value));
    });
  }
}

/// @nodoc
mixin _$VotingVoteSubmission {
  String get voteRoundId;
  int get proposalId;
  Uint8List get vanNullifier;
  Uint8List get voteAuthorityNoteNew;
  Uint8List get voteCommitment;
  Uint8List get proof;
  Uint8List get rVpk;
  Uint8List get voteAuthSig;
  int get anchorHeight;

  /// Create a copy of VotingVoteSubmission
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $VotingVoteSubmissionCopyWith<VotingVoteSubmission> get copyWith =>
      _$VotingVoteSubmissionCopyWithImpl<VotingVoteSubmission>(
          this as VotingVoteSubmission, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is VotingVoteSubmission &&
            (identical(other.voteRoundId, voteRoundId) ||
                other.voteRoundId == voteRoundId) &&
            (identical(other.proposalId, proposalId) ||
                other.proposalId == proposalId) &&
            const DeepCollectionEquality()
                .equals(other.vanNullifier, vanNullifier) &&
            const DeepCollectionEquality()
                .equals(other.voteAuthorityNoteNew, voteAuthorityNoteNew) &&
            const DeepCollectionEquality()
                .equals(other.voteCommitment, voteCommitment) &&
            const DeepCollectionEquality().equals(other.proof, proof) &&
            const DeepCollectionEquality().equals(other.rVpk, rVpk) &&
            const DeepCollectionEquality()
                .equals(other.voteAuthSig, voteAuthSig) &&
            (identical(other.anchorHeight, anchorHeight) ||
                other.anchorHeight == anchorHeight));
  }

  @override
  int get hashCode => Object.hash(
      runtimeType,
      voteRoundId,
      proposalId,
      const DeepCollectionEquality().hash(vanNullifier),
      const DeepCollectionEquality().hash(voteAuthorityNoteNew),
      const DeepCollectionEquality().hash(voteCommitment),
      const DeepCollectionEquality().hash(proof),
      const DeepCollectionEquality().hash(rVpk),
      const DeepCollectionEquality().hash(voteAuthSig),
      anchorHeight);

  @override
  String toString() {
    return 'VotingVoteSubmission(voteRoundId: $voteRoundId, proposalId: $proposalId, vanNullifier: $vanNullifier, voteAuthorityNoteNew: $voteAuthorityNoteNew, voteCommitment: $voteCommitment, proof: $proof, rVpk: $rVpk, voteAuthSig: $voteAuthSig, anchorHeight: $anchorHeight)';
  }
}

/// @nodoc
abstract mixin class $VotingVoteSubmissionCopyWith<$Res> {
  factory $VotingVoteSubmissionCopyWith(VotingVoteSubmission value,
          $Res Function(VotingVoteSubmission) _then) =
      _$VotingVoteSubmissionCopyWithImpl;
  @useResult
  $Res call(
      {String voteRoundId,
      int proposalId,
      Uint8List vanNullifier,
      Uint8List voteAuthorityNoteNew,
      Uint8List voteCommitment,
      Uint8List proof,
      Uint8List rVpk,
      Uint8List voteAuthSig,
      int anchorHeight});
}

/// @nodoc
class _$VotingVoteSubmissionCopyWithImpl<$Res>
    implements $VotingVoteSubmissionCopyWith<$Res> {
  _$VotingVoteSubmissionCopyWithImpl(this._self, this._then);

  final VotingVoteSubmission _self;
  final $Res Function(VotingVoteSubmission) _then;

  /// Create a copy of VotingVoteSubmission
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? voteRoundId = null,
    Object? proposalId = null,
    Object? vanNullifier = null,
    Object? voteAuthorityNoteNew = null,
    Object? voteCommitment = null,
    Object? proof = null,
    Object? rVpk = null,
    Object? voteAuthSig = null,
    Object? anchorHeight = null,
  }) {
    return _then(_self.copyWith(
      voteRoundId: null == voteRoundId
          ? _self.voteRoundId
          : voteRoundId // ignore: cast_nullable_to_non_nullable
              as String,
      proposalId: null == proposalId
          ? _self.proposalId
          : proposalId // ignore: cast_nullable_to_non_nullable
              as int,
      vanNullifier: null == vanNullifier
          ? _self.vanNullifier
          : vanNullifier // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      voteAuthorityNoteNew: null == voteAuthorityNoteNew
          ? _self.voteAuthorityNoteNew
          : voteAuthorityNoteNew // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      voteCommitment: null == voteCommitment
          ? _self.voteCommitment
          : voteCommitment // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      proof: null == proof
          ? _self.proof
          : proof // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      rVpk: null == rVpk
          ? _self.rVpk
          : rVpk // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      voteAuthSig: null == voteAuthSig
          ? _self.voteAuthSig
          : voteAuthSig // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      anchorHeight: null == anchorHeight
          ? _self.anchorHeight
          : anchorHeight // ignore: cast_nullable_to_non_nullable
              as int,
    ));
  }
}

/// Adds pattern-matching-related methods to [VotingVoteSubmission].
extension VotingVoteSubmissionPatterns on VotingVoteSubmission {
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
    TResult Function(_VotingVoteSubmission value)? $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _VotingVoteSubmission() when $default != null:
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
    TResult Function(_VotingVoteSubmission value) $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingVoteSubmission():
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
    TResult? Function(_VotingVoteSubmission value)? $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingVoteSubmission() when $default != null:
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
            String voteRoundId,
            int proposalId,
            Uint8List vanNullifier,
            Uint8List voteAuthorityNoteNew,
            Uint8List voteCommitment,
            Uint8List proof,
            Uint8List rVpk,
            Uint8List voteAuthSig,
            int anchorHeight)?
        $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _VotingVoteSubmission() when $default != null:
        return $default(
            _that.voteRoundId,
            _that.proposalId,
            _that.vanNullifier,
            _that.voteAuthorityNoteNew,
            _that.voteCommitment,
            _that.proof,
            _that.rVpk,
            _that.voteAuthSig,
            _that.anchorHeight);
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
            String voteRoundId,
            int proposalId,
            Uint8List vanNullifier,
            Uint8List voteAuthorityNoteNew,
            Uint8List voteCommitment,
            Uint8List proof,
            Uint8List rVpk,
            Uint8List voteAuthSig,
            int anchorHeight)
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingVoteSubmission():
        return $default(
            _that.voteRoundId,
            _that.proposalId,
            _that.vanNullifier,
            _that.voteAuthorityNoteNew,
            _that.voteCommitment,
            _that.proof,
            _that.rVpk,
            _that.voteAuthSig,
            _that.anchorHeight);
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
            String voteRoundId,
            int proposalId,
            Uint8List vanNullifier,
            Uint8List voteAuthorityNoteNew,
            Uint8List voteCommitment,
            Uint8List proof,
            Uint8List rVpk,
            Uint8List voteAuthSig,
            int anchorHeight)?
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _VotingVoteSubmission() when $default != null:
        return $default(
            _that.voteRoundId,
            _that.proposalId,
            _that.vanNullifier,
            _that.voteAuthorityNoteNew,
            _that.voteCommitment,
            _that.proof,
            _that.rVpk,
            _that.voteAuthSig,
            _that.anchorHeight);
      case _:
        return null;
    }
  }
}

/// @nodoc

class _VotingVoteSubmission implements VotingVoteSubmission {
  const _VotingVoteSubmission(
      {required this.voteRoundId,
      required this.proposalId,
      required this.vanNullifier,
      required this.voteAuthorityNoteNew,
      required this.voteCommitment,
      required this.proof,
      required this.rVpk,
      required this.voteAuthSig,
      required this.anchorHeight});

  @override
  final String voteRoundId;
  @override
  final int proposalId;
  @override
  final Uint8List vanNullifier;
  @override
  final Uint8List voteAuthorityNoteNew;
  @override
  final Uint8List voteCommitment;
  @override
  final Uint8List proof;
  @override
  final Uint8List rVpk;
  @override
  final Uint8List voteAuthSig;
  @override
  final int anchorHeight;

  /// Create a copy of VotingVoteSubmission
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  _$VotingVoteSubmissionCopyWith<_VotingVoteSubmission> get copyWith =>
      __$VotingVoteSubmissionCopyWithImpl<_VotingVoteSubmission>(
          this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _VotingVoteSubmission &&
            (identical(other.voteRoundId, voteRoundId) ||
                other.voteRoundId == voteRoundId) &&
            (identical(other.proposalId, proposalId) ||
                other.proposalId == proposalId) &&
            const DeepCollectionEquality()
                .equals(other.vanNullifier, vanNullifier) &&
            const DeepCollectionEquality()
                .equals(other.voteAuthorityNoteNew, voteAuthorityNoteNew) &&
            const DeepCollectionEquality()
                .equals(other.voteCommitment, voteCommitment) &&
            const DeepCollectionEquality().equals(other.proof, proof) &&
            const DeepCollectionEquality().equals(other.rVpk, rVpk) &&
            const DeepCollectionEquality()
                .equals(other.voteAuthSig, voteAuthSig) &&
            (identical(other.anchorHeight, anchorHeight) ||
                other.anchorHeight == anchorHeight));
  }

  @override
  int get hashCode => Object.hash(
      runtimeType,
      voteRoundId,
      proposalId,
      const DeepCollectionEquality().hash(vanNullifier),
      const DeepCollectionEquality().hash(voteAuthorityNoteNew),
      const DeepCollectionEquality().hash(voteCommitment),
      const DeepCollectionEquality().hash(proof),
      const DeepCollectionEquality().hash(rVpk),
      const DeepCollectionEquality().hash(voteAuthSig),
      anchorHeight);

  @override
  String toString() {
    return 'VotingVoteSubmission(voteRoundId: $voteRoundId, proposalId: $proposalId, vanNullifier: $vanNullifier, voteAuthorityNoteNew: $voteAuthorityNoteNew, voteCommitment: $voteCommitment, proof: $proof, rVpk: $rVpk, voteAuthSig: $voteAuthSig, anchorHeight: $anchorHeight)';
  }
}

/// @nodoc
abstract mixin class _$VotingVoteSubmissionCopyWith<$Res>
    implements $VotingVoteSubmissionCopyWith<$Res> {
  factory _$VotingVoteSubmissionCopyWith(_VotingVoteSubmission value,
          $Res Function(_VotingVoteSubmission) _then) =
      __$VotingVoteSubmissionCopyWithImpl;
  @override
  @useResult
  $Res call(
      {String voteRoundId,
      int proposalId,
      Uint8List vanNullifier,
      Uint8List voteAuthorityNoteNew,
      Uint8List voteCommitment,
      Uint8List proof,
      Uint8List rVpk,
      Uint8List voteAuthSig,
      int anchorHeight});
}

/// @nodoc
class __$VotingVoteSubmissionCopyWithImpl<$Res>
    implements _$VotingVoteSubmissionCopyWith<$Res> {
  __$VotingVoteSubmissionCopyWithImpl(this._self, this._then);

  final _VotingVoteSubmission _self;
  final $Res Function(_VotingVoteSubmission) _then;

  /// Create a copy of VotingVoteSubmission
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $Res call({
    Object? voteRoundId = null,
    Object? proposalId = null,
    Object? vanNullifier = null,
    Object? voteAuthorityNoteNew = null,
    Object? voteCommitment = null,
    Object? proof = null,
    Object? rVpk = null,
    Object? voteAuthSig = null,
    Object? anchorHeight = null,
  }) {
    return _then(_VotingVoteSubmission(
      voteRoundId: null == voteRoundId
          ? _self.voteRoundId
          : voteRoundId // ignore: cast_nullable_to_non_nullable
              as String,
      proposalId: null == proposalId
          ? _self.proposalId
          : proposalId // ignore: cast_nullable_to_non_nullable
              as int,
      vanNullifier: null == vanNullifier
          ? _self.vanNullifier
          : vanNullifier // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      voteAuthorityNoteNew: null == voteAuthorityNoteNew
          ? _self.voteAuthorityNoteNew
          : voteAuthorityNoteNew // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      voteCommitment: null == voteCommitment
          ? _self.voteCommitment
          : voteCommitment // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      proof: null == proof
          ? _self.proof
          : proof // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      rVpk: null == rVpk
          ? _self.rVpk
          : rVpk // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      voteAuthSig: null == voteAuthSig
          ? _self.voteAuthSig
          : voteAuthSig // ignore: cast_nullable_to_non_nullable
              as Uint8List,
      anchorHeight: null == anchorHeight
          ? _self.anchorHeight
          : anchorHeight // ignore: cast_nullable_to_non_nullable
              as int,
    ));
  }
}

// dart format on
