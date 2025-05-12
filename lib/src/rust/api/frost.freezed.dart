// coverage:ignore-file
// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'frost.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

T _$identity<T>(T value) => value;

final _privateConstructorUsedError = UnsupportedError(
    'It seems like you constructed your class using `MyClass._()`. This constructor is only meant to be used by freezed and you are not supposed to need it nor use it.\nPlease check the documentation here for more information: https://github.com/rrousselGit/freezed#adding-getters-and-methods-to-our-models');

/// @nodoc
mixin _$FrostPackage {
  String get name => throw _privateConstructorUsedError;
  int get id => throw _privateConstructorUsedError;
  int get n => throw _privateConstructorUsedError;
  int get t => throw _privateConstructorUsedError;
  int get fundingAccount => throw _privateConstructorUsedError;
  int get mailboxAccount => throw _privateConstructorUsedError;
  List<String> get addresses => throw _privateConstructorUsedError;

  /// Create a copy of FrostPackage
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $FrostPackageCopyWith<FrostPackage> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $FrostPackageCopyWith<$Res> {
  factory $FrostPackageCopyWith(
          FrostPackage value, $Res Function(FrostPackage) then) =
      _$FrostPackageCopyWithImpl<$Res, FrostPackage>;
  @useResult
  $Res call(
      {String name,
      int id,
      int n,
      int t,
      int fundingAccount,
      int mailboxAccount,
      List<String> addresses});
}

/// @nodoc
class _$FrostPackageCopyWithImpl<$Res, $Val extends FrostPackage>
    implements $FrostPackageCopyWith<$Res> {
  _$FrostPackageCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of FrostPackage
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? name = null,
    Object? id = null,
    Object? n = null,
    Object? t = null,
    Object? fundingAccount = null,
    Object? mailboxAccount = null,
    Object? addresses = null,
  }) {
    return _then(_value.copyWith(
      name: null == name
          ? _value.name
          : name // ignore: cast_nullable_to_non_nullable
              as String,
      id: null == id
          ? _value.id
          : id // ignore: cast_nullable_to_non_nullable
              as int,
      n: null == n
          ? _value.n
          : n // ignore: cast_nullable_to_non_nullable
              as int,
      t: null == t
          ? _value.t
          : t // ignore: cast_nullable_to_non_nullable
              as int,
      fundingAccount: null == fundingAccount
          ? _value.fundingAccount
          : fundingAccount // ignore: cast_nullable_to_non_nullable
              as int,
      mailboxAccount: null == mailboxAccount
          ? _value.mailboxAccount
          : mailboxAccount // ignore: cast_nullable_to_non_nullable
              as int,
      addresses: null == addresses
          ? _value.addresses
          : addresses // ignore: cast_nullable_to_non_nullable
              as List<String>,
    ) as $Val);
  }
}

/// @nodoc
abstract class _$$FrostPackageImplCopyWith<$Res>
    implements $FrostPackageCopyWith<$Res> {
  factory _$$FrostPackageImplCopyWith(
          _$FrostPackageImpl value, $Res Function(_$FrostPackageImpl) then) =
      __$$FrostPackageImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call(
      {String name,
      int id,
      int n,
      int t,
      int fundingAccount,
      int mailboxAccount,
      List<String> addresses});
}

/// @nodoc
class __$$FrostPackageImplCopyWithImpl<$Res>
    extends _$FrostPackageCopyWithImpl<$Res, _$FrostPackageImpl>
    implements _$$FrostPackageImplCopyWith<$Res> {
  __$$FrostPackageImplCopyWithImpl(
      _$FrostPackageImpl _value, $Res Function(_$FrostPackageImpl) _then)
      : super(_value, _then);

  /// Create a copy of FrostPackage
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? name = null,
    Object? id = null,
    Object? n = null,
    Object? t = null,
    Object? fundingAccount = null,
    Object? mailboxAccount = null,
    Object? addresses = null,
  }) {
    return _then(_$FrostPackageImpl(
      name: null == name
          ? _value.name
          : name // ignore: cast_nullable_to_non_nullable
              as String,
      id: null == id
          ? _value.id
          : id // ignore: cast_nullable_to_non_nullable
              as int,
      n: null == n
          ? _value.n
          : n // ignore: cast_nullable_to_non_nullable
              as int,
      t: null == t
          ? _value.t
          : t // ignore: cast_nullable_to_non_nullable
              as int,
      fundingAccount: null == fundingAccount
          ? _value.fundingAccount
          : fundingAccount // ignore: cast_nullable_to_non_nullable
              as int,
      mailboxAccount: null == mailboxAccount
          ? _value.mailboxAccount
          : mailboxAccount // ignore: cast_nullable_to_non_nullable
              as int,
      addresses: null == addresses
          ? _value._addresses
          : addresses // ignore: cast_nullable_to_non_nullable
              as List<String>,
    ));
  }
}

/// @nodoc

class _$FrostPackageImpl extends _FrostPackage {
  const _$FrostPackageImpl(
      {required this.name,
      required this.id,
      required this.n,
      required this.t,
      required this.fundingAccount,
      required this.mailboxAccount,
      required final List<String> addresses})
      : _addresses = addresses,
        super._();

  @override
  final String name;
  @override
  final int id;
  @override
  final int n;
  @override
  final int t;
  @override
  final int fundingAccount;
  @override
  final int mailboxAccount;
  final List<String> _addresses;
  @override
  List<String> get addresses {
    if (_addresses is EqualUnmodifiableListView) return _addresses;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_addresses);
  }

  @override
  String toString() {
    return 'FrostPackage(name: $name, id: $id, n: $n, t: $t, fundingAccount: $fundingAccount, mailboxAccount: $mailboxAccount, addresses: $addresses)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$FrostPackageImpl &&
            (identical(other.name, name) || other.name == name) &&
            (identical(other.id, id) || other.id == id) &&
            (identical(other.n, n) || other.n == n) &&
            (identical(other.t, t) || other.t == t) &&
            (identical(other.fundingAccount, fundingAccount) ||
                other.fundingAccount == fundingAccount) &&
            (identical(other.mailboxAccount, mailboxAccount) ||
                other.mailboxAccount == mailboxAccount) &&
            const DeepCollectionEquality()
                .equals(other._addresses, _addresses));
  }

  @override
  int get hashCode => Object.hash(runtimeType, name, id, n, t, fundingAccount,
      mailboxAccount, const DeepCollectionEquality().hash(_addresses));

  /// Create a copy of FrostPackage
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$FrostPackageImplCopyWith<_$FrostPackageImpl> get copyWith =>
      __$$FrostPackageImplCopyWithImpl<_$FrostPackageImpl>(this, _$identity);
}

abstract class _FrostPackage extends FrostPackage {
  const factory _FrostPackage(
      {required final String name,
      required final int id,
      required final int n,
      required final int t,
      required final int fundingAccount,
      required final int mailboxAccount,
      required final List<String> addresses}) = _$FrostPackageImpl;
  const _FrostPackage._() : super._();

  @override
  String get name;
  @override
  int get id;
  @override
  int get n;
  @override
  int get t;
  @override
  int get fundingAccount;
  @override
  int get mailboxAccount;
  @override
  List<String> get addresses;

  /// Create a copy of FrostPackage
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$FrostPackageImplCopyWith<_$FrostPackageImpl> get copyWith =>
      throw _privateConstructorUsedError;
}
