// coverage:ignore-file
// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'account.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

T _$identity<T>(T value) => value;

final _privateConstructorUsedError = UnsupportedError(
    'It seems like you constructed your class using `MyClass._()`. This constructor is only meant to be used by freezed and you are not supposed to need it nor use it.\nPlease check the documentation here for more information: https://github.com/rrousselGit/freezed#adding-getters-and-methods-to-our-models');

/// @nodoc
mixin _$Account {
  int get id => throw _privateConstructorUsedError;
  String get name => throw _privateConstructorUsedError;
  String? get seed => throw _privateConstructorUsedError;
  int get aindex => throw _privateConstructorUsedError;
  Uint8List? get icon => throw _privateConstructorUsedError;
  int get birth => throw _privateConstructorUsedError;
  int get height => throw _privateConstructorUsedError;
  int get position => throw _privateConstructorUsedError;
  bool get hidden => throw _privateConstructorUsedError;
  bool get saved => throw _privateConstructorUsedError;
  bool get enabled => throw _privateConstructorUsedError;

  /// Create a copy of Account
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $AccountCopyWith<Account> get copyWith => throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $AccountCopyWith<$Res> {
  factory $AccountCopyWith(Account value, $Res Function(Account) then) =
      _$AccountCopyWithImpl<$Res, Account>;
  @useResult
  $Res call(
      {int id,
      String name,
      String? seed,
      int aindex,
      Uint8List? icon,
      int birth,
      int height,
      int position,
      bool hidden,
      bool saved,
      bool enabled});
}

/// @nodoc
class _$AccountCopyWithImpl<$Res, $Val extends Account>
    implements $AccountCopyWith<$Res> {
  _$AccountCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of Account
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? id = null,
    Object? name = null,
    Object? seed = freezed,
    Object? aindex = null,
    Object? icon = freezed,
    Object? birth = null,
    Object? height = null,
    Object? position = null,
    Object? hidden = null,
    Object? saved = null,
    Object? enabled = null,
  }) {
    return _then(_value.copyWith(
      id: null == id
          ? _value.id
          : id // ignore: cast_nullable_to_non_nullable
              as int,
      name: null == name
          ? _value.name
          : name // ignore: cast_nullable_to_non_nullable
              as String,
      seed: freezed == seed
          ? _value.seed
          : seed // ignore: cast_nullable_to_non_nullable
              as String?,
      aindex: null == aindex
          ? _value.aindex
          : aindex // ignore: cast_nullable_to_non_nullable
              as int,
      icon: freezed == icon
          ? _value.icon
          : icon // ignore: cast_nullable_to_non_nullable
              as Uint8List?,
      birth: null == birth
          ? _value.birth
          : birth // ignore: cast_nullable_to_non_nullable
              as int,
      height: null == height
          ? _value.height
          : height // ignore: cast_nullable_to_non_nullable
              as int,
      position: null == position
          ? _value.position
          : position // ignore: cast_nullable_to_non_nullable
              as int,
      hidden: null == hidden
          ? _value.hidden
          : hidden // ignore: cast_nullable_to_non_nullable
              as bool,
      saved: null == saved
          ? _value.saved
          : saved // ignore: cast_nullable_to_non_nullable
              as bool,
      enabled: null == enabled
          ? _value.enabled
          : enabled // ignore: cast_nullable_to_non_nullable
              as bool,
    ) as $Val);
  }
}

/// @nodoc
abstract class _$$AccountImplCopyWith<$Res> implements $AccountCopyWith<$Res> {
  factory _$$AccountImplCopyWith(
          _$AccountImpl value, $Res Function(_$AccountImpl) then) =
      __$$AccountImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call(
      {int id,
      String name,
      String? seed,
      int aindex,
      Uint8List? icon,
      int birth,
      int height,
      int position,
      bool hidden,
      bool saved,
      bool enabled});
}

/// @nodoc
class __$$AccountImplCopyWithImpl<$Res>
    extends _$AccountCopyWithImpl<$Res, _$AccountImpl>
    implements _$$AccountImplCopyWith<$Res> {
  __$$AccountImplCopyWithImpl(
      _$AccountImpl _value, $Res Function(_$AccountImpl) _then)
      : super(_value, _then);

  /// Create a copy of Account
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? id = null,
    Object? name = null,
    Object? seed = freezed,
    Object? aindex = null,
    Object? icon = freezed,
    Object? birth = null,
    Object? height = null,
    Object? position = null,
    Object? hidden = null,
    Object? saved = null,
    Object? enabled = null,
  }) {
    return _then(_$AccountImpl(
      id: null == id
          ? _value.id
          : id // ignore: cast_nullable_to_non_nullable
              as int,
      name: null == name
          ? _value.name
          : name // ignore: cast_nullable_to_non_nullable
              as String,
      seed: freezed == seed
          ? _value.seed
          : seed // ignore: cast_nullable_to_non_nullable
              as String?,
      aindex: null == aindex
          ? _value.aindex
          : aindex // ignore: cast_nullable_to_non_nullable
              as int,
      icon: freezed == icon
          ? _value.icon
          : icon // ignore: cast_nullable_to_non_nullable
              as Uint8List?,
      birth: null == birth
          ? _value.birth
          : birth // ignore: cast_nullable_to_non_nullable
              as int,
      height: null == height
          ? _value.height
          : height // ignore: cast_nullable_to_non_nullable
              as int,
      position: null == position
          ? _value.position
          : position // ignore: cast_nullable_to_non_nullable
              as int,
      hidden: null == hidden
          ? _value.hidden
          : hidden // ignore: cast_nullable_to_non_nullable
              as bool,
      saved: null == saved
          ? _value.saved
          : saved // ignore: cast_nullable_to_non_nullable
              as bool,
      enabled: null == enabled
          ? _value.enabled
          : enabled // ignore: cast_nullable_to_non_nullable
              as bool,
    ));
  }
}

/// @nodoc

class _$AccountImpl implements _Account {
  const _$AccountImpl(
      {required this.id,
      required this.name,
      this.seed,
      required this.aindex,
      this.icon,
      required this.birth,
      required this.height,
      required this.position,
      required this.hidden,
      required this.saved,
      required this.enabled});

  @override
  final int id;
  @override
  final String name;
  @override
  final String? seed;
  @override
  final int aindex;
  @override
  final Uint8List? icon;
  @override
  final int birth;
  @override
  final int height;
  @override
  final int position;
  @override
  final bool hidden;
  @override
  final bool saved;
  @override
  final bool enabled;

  @override
  String toString() {
    return 'Account(id: $id, name: $name, seed: $seed, aindex: $aindex, icon: $icon, birth: $birth, height: $height, position: $position, hidden: $hidden, saved: $saved, enabled: $enabled)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$AccountImpl &&
            (identical(other.id, id) || other.id == id) &&
            (identical(other.name, name) || other.name == name) &&
            (identical(other.seed, seed) || other.seed == seed) &&
            (identical(other.aindex, aindex) || other.aindex == aindex) &&
            const DeepCollectionEquality().equals(other.icon, icon) &&
            (identical(other.birth, birth) || other.birth == birth) &&
            (identical(other.height, height) || other.height == height) &&
            (identical(other.position, position) ||
                other.position == position) &&
            (identical(other.hidden, hidden) || other.hidden == hidden) &&
            (identical(other.saved, saved) || other.saved == saved) &&
            (identical(other.enabled, enabled) || other.enabled == enabled));
  }

  @override
  int get hashCode => Object.hash(
      runtimeType,
      id,
      name,
      seed,
      aindex,
      const DeepCollectionEquality().hash(icon),
      birth,
      height,
      position,
      hidden,
      saved,
      enabled);

  /// Create a copy of Account
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$AccountImplCopyWith<_$AccountImpl> get copyWith =>
      __$$AccountImplCopyWithImpl<_$AccountImpl>(this, _$identity);
}

abstract class _Account implements Account {
  const factory _Account(
      {required final int id,
      required final String name,
      final String? seed,
      required final int aindex,
      final Uint8List? icon,
      required final int birth,
      required final int height,
      required final int position,
      required final bool hidden,
      required final bool saved,
      required final bool enabled}) = _$AccountImpl;

  @override
  int get id;
  @override
  String get name;
  @override
  String? get seed;
  @override
  int get aindex;
  @override
  Uint8List? get icon;
  @override
  int get birth;
  @override
  int get height;
  @override
  int get position;
  @override
  bool get hidden;
  @override
  bool get saved;
  @override
  bool get enabled;

  /// Create a copy of Account
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$AccountImplCopyWith<_$AccountImpl> get copyWith =>
      throw _privateConstructorUsedError;
}
