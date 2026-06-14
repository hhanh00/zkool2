// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'contacts.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;

/// @nodoc
mixin _$Contact {
  int get id;
  String get name;
  List<String> get addresses;
  String get notes;

  /// Create a copy of Contact
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $ContactCopyWith<Contact> get copyWith =>
      _$ContactCopyWithImpl<Contact>(this as Contact, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is Contact &&
            (identical(other.id, id) || other.id == id) &&
            (identical(other.name, name) || other.name == name) &&
            const DeepCollectionEquality().equals(other.addresses, addresses) &&
            (identical(other.notes, notes) || other.notes == notes));
  }

  @override
  int get hashCode => Object.hash(runtimeType, id, name,
      const DeepCollectionEquality().hash(addresses), notes);

  @override
  String toString() {
    return 'Contact(id: $id, name: $name, addresses: $addresses, notes: $notes)';
  }
}

/// @nodoc
abstract mixin class $ContactCopyWith<$Res> {
  factory $ContactCopyWith(Contact value, $Res Function(Contact) _then) =
      _$ContactCopyWithImpl;
  @useResult
  $Res call({int id, String name, List<String> addresses, String notes});
}

/// @nodoc
class _$ContactCopyWithImpl<$Res> implements $ContactCopyWith<$Res> {
  _$ContactCopyWithImpl(this._self, this._then);

  final Contact _self;
  final $Res Function(Contact) _then;

  /// Create a copy of Contact
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? id = null,
    Object? name = null,
    Object? addresses = null,
    Object? notes = null,
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
      addresses: null == addresses
          ? _self.addresses
          : addresses // ignore: cast_nullable_to_non_nullable
              as List<String>,
      notes: null == notes
          ? _self.notes
          : notes // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// Adds pattern-matching-related methods to [Contact].
extension ContactPatterns on Contact {
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
    TResult Function(_Contact value)? $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _Contact() when $default != null:
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
    TResult Function(_Contact value) $default,
  ) {
    final _that = this;
    switch (_that) {
      case _Contact():
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
    TResult? Function(_Contact value)? $default,
  ) {
    final _that = this;
    switch (_that) {
      case _Contact() when $default != null:
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
    TResult Function(int id, String name, List<String> addresses, String notes)?
        $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _Contact() when $default != null:
        return $default(_that.id, _that.name, _that.addresses, _that.notes);
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
    TResult Function(int id, String name, List<String> addresses, String notes)
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _Contact():
        return $default(_that.id, _that.name, _that.addresses, _that.notes);
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
            int id, String name, List<String> addresses, String notes)?
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _Contact() when $default != null:
        return $default(_that.id, _that.name, _that.addresses, _that.notes);
      case _:
        return null;
    }
  }
}

/// @nodoc

class _Contact implements Contact {
  const _Contact(
      {required this.id,
      required this.name,
      required final List<String> addresses,
      required this.notes})
      : _addresses = addresses;

  @override
  final int id;
  @override
  final String name;
  final List<String> _addresses;
  @override
  List<String> get addresses {
    if (_addresses is EqualUnmodifiableListView) return _addresses;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_addresses);
  }

  @override
  final String notes;

  /// Create a copy of Contact
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  _$ContactCopyWith<_Contact> get copyWith =>
      __$ContactCopyWithImpl<_Contact>(this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _Contact &&
            (identical(other.id, id) || other.id == id) &&
            (identical(other.name, name) || other.name == name) &&
            const DeepCollectionEquality()
                .equals(other._addresses, _addresses) &&
            (identical(other.notes, notes) || other.notes == notes));
  }

  @override
  int get hashCode => Object.hash(runtimeType, id, name,
      const DeepCollectionEquality().hash(_addresses), notes);

  @override
  String toString() {
    return 'Contact(id: $id, name: $name, addresses: $addresses, notes: $notes)';
  }
}

/// @nodoc
abstract mixin class _$ContactCopyWith<$Res> implements $ContactCopyWith<$Res> {
  factory _$ContactCopyWith(_Contact value, $Res Function(_Contact) _then) =
      __$ContactCopyWithImpl;
  @override
  @useResult
  $Res call({int id, String name, List<String> addresses, String notes});
}

/// @nodoc
class __$ContactCopyWithImpl<$Res> implements _$ContactCopyWith<$Res> {
  __$ContactCopyWithImpl(this._self, this._then);

  final _Contact _self;
  final $Res Function(_Contact) _then;

  /// Create a copy of Contact
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $Res call({
    Object? id = null,
    Object? name = null,
    Object? addresses = null,
    Object? notes = null,
  }) {
    return _then(_Contact(
      id: null == id
          ? _self.id
          : id // ignore: cast_nullable_to_non_nullable
              as int,
      name: null == name
          ? _self.name
          : name // ignore: cast_nullable_to_non_nullable
              as String,
      addresses: null == addresses
          ? _self._addresses
          : addresses // ignore: cast_nullable_to_non_nullable
              as List<String>,
      notes: null == notes
          ? _self.notes
          : notes // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// @nodoc
mixin _$ContactMatch {
  Contact get contact;
  String get matchedAddress;

  /// Create a copy of ContactMatch
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $ContactMatchCopyWith<ContactMatch> get copyWith =>
      _$ContactMatchCopyWithImpl<ContactMatch>(
          this as ContactMatch, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is ContactMatch &&
            (identical(other.contact, contact) || other.contact == contact) &&
            (identical(other.matchedAddress, matchedAddress) ||
                other.matchedAddress == matchedAddress));
  }

  @override
  int get hashCode => Object.hash(runtimeType, contact, matchedAddress);

  @override
  String toString() {
    return 'ContactMatch(contact: $contact, matchedAddress: $matchedAddress)';
  }
}

/// @nodoc
abstract mixin class $ContactMatchCopyWith<$Res> {
  factory $ContactMatchCopyWith(
          ContactMatch value, $Res Function(ContactMatch) _then) =
      _$ContactMatchCopyWithImpl;
  @useResult
  $Res call({Contact contact, String matchedAddress});

  $ContactCopyWith<$Res> get contact;
}

/// @nodoc
class _$ContactMatchCopyWithImpl<$Res> implements $ContactMatchCopyWith<$Res> {
  _$ContactMatchCopyWithImpl(this._self, this._then);

  final ContactMatch _self;
  final $Res Function(ContactMatch) _then;

  /// Create a copy of ContactMatch
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? contact = null,
    Object? matchedAddress = null,
  }) {
    return _then(_self.copyWith(
      contact: null == contact
          ? _self.contact
          : contact // ignore: cast_nullable_to_non_nullable
              as Contact,
      matchedAddress: null == matchedAddress
          ? _self.matchedAddress
          : matchedAddress // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }

  /// Create a copy of ContactMatch
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $ContactCopyWith<$Res> get contact {
    return $ContactCopyWith<$Res>(_self.contact, (value) {
      return _then(_self.copyWith(contact: value));
    });
  }
}

/// Adds pattern-matching-related methods to [ContactMatch].
extension ContactMatchPatterns on ContactMatch {
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
    TResult Function(_ContactMatch value)? $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _ContactMatch() when $default != null:
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
    TResult Function(_ContactMatch value) $default,
  ) {
    final _that = this;
    switch (_that) {
      case _ContactMatch():
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
    TResult? Function(_ContactMatch value)? $default,
  ) {
    final _that = this;
    switch (_that) {
      case _ContactMatch() when $default != null:
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
    TResult Function(Contact contact, String matchedAddress)? $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _ContactMatch() when $default != null:
        return $default(_that.contact, _that.matchedAddress);
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
    TResult Function(Contact contact, String matchedAddress) $default,
  ) {
    final _that = this;
    switch (_that) {
      case _ContactMatch():
        return $default(_that.contact, _that.matchedAddress);
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
    TResult? Function(Contact contact, String matchedAddress)? $default,
  ) {
    final _that = this;
    switch (_that) {
      case _ContactMatch() when $default != null:
        return $default(_that.contact, _that.matchedAddress);
      case _:
        return null;
    }
  }
}

/// @nodoc

class _ContactMatch implements ContactMatch {
  const _ContactMatch({required this.contact, required this.matchedAddress});

  @override
  final Contact contact;
  @override
  final String matchedAddress;

  /// Create a copy of ContactMatch
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  _$ContactMatchCopyWith<_ContactMatch> get copyWith =>
      __$ContactMatchCopyWithImpl<_ContactMatch>(this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _ContactMatch &&
            (identical(other.contact, contact) || other.contact == contact) &&
            (identical(other.matchedAddress, matchedAddress) ||
                other.matchedAddress == matchedAddress));
  }

  @override
  int get hashCode => Object.hash(runtimeType, contact, matchedAddress);

  @override
  String toString() {
    return 'ContactMatch(contact: $contact, matchedAddress: $matchedAddress)';
  }
}

/// @nodoc
abstract mixin class _$ContactMatchCopyWith<$Res>
    implements $ContactMatchCopyWith<$Res> {
  factory _$ContactMatchCopyWith(
          _ContactMatch value, $Res Function(_ContactMatch) _then) =
      __$ContactMatchCopyWithImpl;
  @override
  @useResult
  $Res call({Contact contact, String matchedAddress});

  @override
  $ContactCopyWith<$Res> get contact;
}

/// @nodoc
class __$ContactMatchCopyWithImpl<$Res>
    implements _$ContactMatchCopyWith<$Res> {
  __$ContactMatchCopyWithImpl(this._self, this._then);

  final _ContactMatch _self;
  final $Res Function(_ContactMatch) _then;

  /// Create a copy of ContactMatch
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $Res call({
    Object? contact = null,
    Object? matchedAddress = null,
  }) {
    return _then(_ContactMatch(
      contact: null == contact
          ? _self.contact
          : contact // ignore: cast_nullable_to_non_nullable
              as Contact,
      matchedAddress: null == matchedAddress
          ? _self.matchedAddress
          : matchedAddress // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }

  /// Create a copy of ContactMatch
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $ContactCopyWith<$Res> get contact {
    return $ContactCopyWith<$Res>(_self.contact, (value) {
      return _then(_self.copyWith(contact: value));
    });
  }
}

// dart format on
