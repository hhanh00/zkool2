// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'plugin.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;

/// @nodoc
mixin _$MemoCell {
  String get cellType;
  String get value;

  /// Create a copy of MemoCell
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $MemoCellCopyWith<MemoCell> get copyWith =>
      _$MemoCellCopyWithImpl<MemoCell>(this as MemoCell, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is MemoCell &&
            (identical(other.cellType, cellType) ||
                other.cellType == cellType) &&
            (identical(other.value, value) || other.value == value));
  }

  @override
  int get hashCode => Object.hash(runtimeType, cellType, value);

  @override
  String toString() {
    return 'MemoCell(cellType: $cellType, value: $value)';
  }
}

/// @nodoc
abstract mixin class $MemoCellCopyWith<$Res> {
  factory $MemoCellCopyWith(MemoCell value, $Res Function(MemoCell) _then) =
      _$MemoCellCopyWithImpl;
  @useResult
  $Res call({String cellType, String value});
}

/// @nodoc
class _$MemoCellCopyWithImpl<$Res> implements $MemoCellCopyWith<$Res> {
  _$MemoCellCopyWithImpl(this._self, this._then);

  final MemoCell _self;
  final $Res Function(MemoCell) _then;

  /// Create a copy of MemoCell
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? cellType = null,
    Object? value = null,
  }) {
    return _then(_self.copyWith(
      cellType: null == cellType
          ? _self.cellType
          : cellType // ignore: cast_nullable_to_non_nullable
              as String,
      value: null == value
          ? _self.value
          : value // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// Adds pattern-matching-related methods to [MemoCell].
extension MemoCellPatterns on MemoCell {
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
    TResult Function(_MemoCell value)? $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _MemoCell() when $default != null:
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
    TResult Function(_MemoCell value) $default,
  ) {
    final _that = this;
    switch (_that) {
      case _MemoCell():
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
    TResult? Function(_MemoCell value)? $default,
  ) {
    final _that = this;
    switch (_that) {
      case _MemoCell() when $default != null:
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
    TResult Function(String cellType, String value)? $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _MemoCell() when $default != null:
        return $default(_that.cellType, _that.value);
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
    TResult Function(String cellType, String value) $default,
  ) {
    final _that = this;
    switch (_that) {
      case _MemoCell():
        return $default(_that.cellType, _that.value);
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
    TResult? Function(String cellType, String value)? $default,
  ) {
    final _that = this;
    switch (_that) {
      case _MemoCell() when $default != null:
        return $default(_that.cellType, _that.value);
      case _:
        return null;
    }
  }
}

/// @nodoc

class _MemoCell implements MemoCell {
  const _MemoCell({required this.cellType, required this.value});

  @override
  final String cellType;
  @override
  final String value;

  /// Create a copy of MemoCell
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  _$MemoCellCopyWith<_MemoCell> get copyWith =>
      __$MemoCellCopyWithImpl<_MemoCell>(this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _MemoCell &&
            (identical(other.cellType, cellType) ||
                other.cellType == cellType) &&
            (identical(other.value, value) || other.value == value));
  }

  @override
  int get hashCode => Object.hash(runtimeType, cellType, value);

  @override
  String toString() {
    return 'MemoCell(cellType: $cellType, value: $value)';
  }
}

/// @nodoc
abstract mixin class _$MemoCellCopyWith<$Res>
    implements $MemoCellCopyWith<$Res> {
  factory _$MemoCellCopyWith(_MemoCell value, $Res Function(_MemoCell) _then) =
      __$MemoCellCopyWithImpl;
  @override
  @useResult
  $Res call({String cellType, String value});
}

/// @nodoc
class __$MemoCellCopyWithImpl<$Res> implements _$MemoCellCopyWith<$Res> {
  __$MemoCellCopyWithImpl(this._self, this._then);

  final _MemoCell _self;
  final $Res Function(_MemoCell) _then;

  /// Create a copy of MemoCell
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $Res call({
    Object? cellType = null,
    Object? value = null,
  }) {
    return _then(_MemoCell(
      cellType: null == cellType
          ? _self.cellType
          : cellType // ignore: cast_nullable_to_non_nullable
              as String,
      value: null == value
          ? _self.value
          : value // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// @nodoc
mixin _$MemoRow {
  List<MemoCell> get cells;

  /// Create a copy of MemoRow
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $MemoRowCopyWith<MemoRow> get copyWith =>
      _$MemoRowCopyWithImpl<MemoRow>(this as MemoRow, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is MemoRow &&
            const DeepCollectionEquality().equals(other.cells, cells));
  }

  @override
  int get hashCode =>
      Object.hash(runtimeType, const DeepCollectionEquality().hash(cells));

  @override
  String toString() {
    return 'MemoRow(cells: $cells)';
  }
}

/// @nodoc
abstract mixin class $MemoRowCopyWith<$Res> {
  factory $MemoRowCopyWith(MemoRow value, $Res Function(MemoRow) _then) =
      _$MemoRowCopyWithImpl;
  @useResult
  $Res call({List<MemoCell> cells});
}

/// @nodoc
class _$MemoRowCopyWithImpl<$Res> implements $MemoRowCopyWith<$Res> {
  _$MemoRowCopyWithImpl(this._self, this._then);

  final MemoRow _self;
  final $Res Function(MemoRow) _then;

  /// Create a copy of MemoRow
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? cells = null,
  }) {
    return _then(_self.copyWith(
      cells: null == cells
          ? _self.cells
          : cells // ignore: cast_nullable_to_non_nullable
              as List<MemoCell>,
    ));
  }
}

/// Adds pattern-matching-related methods to [MemoRow].
extension MemoRowPatterns on MemoRow {
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
    TResult Function(_MemoRow value)? $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _MemoRow() when $default != null:
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
    TResult Function(_MemoRow value) $default,
  ) {
    final _that = this;
    switch (_that) {
      case _MemoRow():
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
    TResult? Function(_MemoRow value)? $default,
  ) {
    final _that = this;
    switch (_that) {
      case _MemoRow() when $default != null:
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
    TResult Function(List<MemoCell> cells)? $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _MemoRow() when $default != null:
        return $default(_that.cells);
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
    TResult Function(List<MemoCell> cells) $default,
  ) {
    final _that = this;
    switch (_that) {
      case _MemoRow():
        return $default(_that.cells);
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
    TResult? Function(List<MemoCell> cells)? $default,
  ) {
    final _that = this;
    switch (_that) {
      case _MemoRow() when $default != null:
        return $default(_that.cells);
      case _:
        return null;
    }
  }
}

/// @nodoc

class _MemoRow implements MemoRow {
  const _MemoRow({required final List<MemoCell> cells}) : _cells = cells;

  final List<MemoCell> _cells;
  @override
  List<MemoCell> get cells {
    if (_cells is EqualUnmodifiableListView) return _cells;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_cells);
  }

  /// Create a copy of MemoRow
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  _$MemoRowCopyWith<_MemoRow> get copyWith =>
      __$MemoRowCopyWithImpl<_MemoRow>(this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _MemoRow &&
            const DeepCollectionEquality().equals(other._cells, _cells));
  }

  @override
  int get hashCode =>
      Object.hash(runtimeType, const DeepCollectionEquality().hash(_cells));

  @override
  String toString() {
    return 'MemoRow(cells: $cells)';
  }
}

/// @nodoc
abstract mixin class _$MemoRowCopyWith<$Res> implements $MemoRowCopyWith<$Res> {
  factory _$MemoRowCopyWith(_MemoRow value, $Res Function(_MemoRow) _then) =
      __$MemoRowCopyWithImpl;
  @override
  @useResult
  $Res call({List<MemoCell> cells});
}

/// @nodoc
class __$MemoRowCopyWithImpl<$Res> implements _$MemoRowCopyWith<$Res> {
  __$MemoRowCopyWithImpl(this._self, this._then);

  final _MemoRow _self;
  final $Res Function(_MemoRow) _then;

  /// Create a copy of MemoRow
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $Res call({
    Object? cells = null,
  }) {
    return _then(_MemoRow(
      cells: null == cells
          ? _self._cells
          : cells // ignore: cast_nullable_to_non_nullable
              as List<MemoCell>,
    ));
  }
}

/// @nodoc
mixin _$MemoSection {
  String get title;
  List<String> get headers;
  List<MemoRow> get rows;

  /// Create a copy of MemoSection
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $MemoSectionCopyWith<MemoSection> get copyWith =>
      _$MemoSectionCopyWithImpl<MemoSection>(this as MemoSection, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is MemoSection &&
            (identical(other.title, title) || other.title == title) &&
            const DeepCollectionEquality().equals(other.headers, headers) &&
            const DeepCollectionEquality().equals(other.rows, rows));
  }

  @override
  int get hashCode => Object.hash(
      runtimeType,
      title,
      const DeepCollectionEquality().hash(headers),
      const DeepCollectionEquality().hash(rows));

  @override
  String toString() {
    return 'MemoSection(title: $title, headers: $headers, rows: $rows)';
  }
}

/// @nodoc
abstract mixin class $MemoSectionCopyWith<$Res> {
  factory $MemoSectionCopyWith(
          MemoSection value, $Res Function(MemoSection) _then) =
      _$MemoSectionCopyWithImpl;
  @useResult
  $Res call({String title, List<String> headers, List<MemoRow> rows});
}

/// @nodoc
class _$MemoSectionCopyWithImpl<$Res> implements $MemoSectionCopyWith<$Res> {
  _$MemoSectionCopyWithImpl(this._self, this._then);

  final MemoSection _self;
  final $Res Function(MemoSection) _then;

  /// Create a copy of MemoSection
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? title = null,
    Object? headers = null,
    Object? rows = null,
  }) {
    return _then(_self.copyWith(
      title: null == title
          ? _self.title
          : title // ignore: cast_nullable_to_non_nullable
              as String,
      headers: null == headers
          ? _self.headers
          : headers // ignore: cast_nullable_to_non_nullable
              as List<String>,
      rows: null == rows
          ? _self.rows
          : rows // ignore: cast_nullable_to_non_nullable
              as List<MemoRow>,
    ));
  }
}

/// Adds pattern-matching-related methods to [MemoSection].
extension MemoSectionPatterns on MemoSection {
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
    TResult Function(_MemoSection value)? $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _MemoSection() when $default != null:
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
    TResult Function(_MemoSection value) $default,
  ) {
    final _that = this;
    switch (_that) {
      case _MemoSection():
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
    TResult? Function(_MemoSection value)? $default,
  ) {
    final _that = this;
    switch (_that) {
      case _MemoSection() when $default != null:
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
    TResult Function(String title, List<String> headers, List<MemoRow> rows)?
        $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _MemoSection() when $default != null:
        return $default(_that.title, _that.headers, _that.rows);
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
    TResult Function(String title, List<String> headers, List<MemoRow> rows)
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _MemoSection():
        return $default(_that.title, _that.headers, _that.rows);
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
    TResult? Function(String title, List<String> headers, List<MemoRow> rows)?
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _MemoSection() when $default != null:
        return $default(_that.title, _that.headers, _that.rows);
      case _:
        return null;
    }
  }
}

/// @nodoc

class _MemoSection implements MemoSection {
  const _MemoSection(
      {required this.title,
      required final List<String> headers,
      required final List<MemoRow> rows})
      : _headers = headers,
        _rows = rows;

  @override
  final String title;
  final List<String> _headers;
  @override
  List<String> get headers {
    if (_headers is EqualUnmodifiableListView) return _headers;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_headers);
  }

  final List<MemoRow> _rows;
  @override
  List<MemoRow> get rows {
    if (_rows is EqualUnmodifiableListView) return _rows;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_rows);
  }

  /// Create a copy of MemoSection
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  _$MemoSectionCopyWith<_MemoSection> get copyWith =>
      __$MemoSectionCopyWithImpl<_MemoSection>(this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _MemoSection &&
            (identical(other.title, title) || other.title == title) &&
            const DeepCollectionEquality().equals(other._headers, _headers) &&
            const DeepCollectionEquality().equals(other._rows, _rows));
  }

  @override
  int get hashCode => Object.hash(
      runtimeType,
      title,
      const DeepCollectionEquality().hash(_headers),
      const DeepCollectionEquality().hash(_rows));

  @override
  String toString() {
    return 'MemoSection(title: $title, headers: $headers, rows: $rows)';
  }
}

/// @nodoc
abstract mixin class _$MemoSectionCopyWith<$Res>
    implements $MemoSectionCopyWith<$Res> {
  factory _$MemoSectionCopyWith(
          _MemoSection value, $Res Function(_MemoSection) _then) =
      __$MemoSectionCopyWithImpl;
  @override
  @useResult
  $Res call({String title, List<String> headers, List<MemoRow> rows});
}

/// @nodoc
class __$MemoSectionCopyWithImpl<$Res> implements _$MemoSectionCopyWith<$Res> {
  __$MemoSectionCopyWithImpl(this._self, this._then);

  final _MemoSection _self;
  final $Res Function(_MemoSection) _then;

  /// Create a copy of MemoSection
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $Res call({
    Object? title = null,
    Object? headers = null,
    Object? rows = null,
  }) {
    return _then(_MemoSection(
      title: null == title
          ? _self.title
          : title // ignore: cast_nullable_to_non_nullable
              as String,
      headers: null == headers
          ? _self._headers
          : headers // ignore: cast_nullable_to_non_nullable
              as List<String>,
      rows: null == rows
          ? _self._rows
          : rows // ignore: cast_nullable_to_non_nullable
              as List<MemoRow>,
    ));
  }
}

/// @nodoc
mixin _$PluginInfo {
  String get id;
  String get name;
  String get version;
  String? get author;
  String? get description;
  bool get enabled;
  List<String> get types;
  List<String> get memoPrefixes;

  /// Create a copy of PluginInfo
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $PluginInfoCopyWith<PluginInfo> get copyWith =>
      _$PluginInfoCopyWithImpl<PluginInfo>(this as PluginInfo, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is PluginInfo &&
            (identical(other.id, id) || other.id == id) &&
            (identical(other.name, name) || other.name == name) &&
            (identical(other.version, version) || other.version == version) &&
            (identical(other.author, author) || other.author == author) &&
            (identical(other.description, description) ||
                other.description == description) &&
            (identical(other.enabled, enabled) || other.enabled == enabled) &&
            const DeepCollectionEquality().equals(other.types, types) &&
            const DeepCollectionEquality()
                .equals(other.memoPrefixes, memoPrefixes));
  }

  @override
  int get hashCode => Object.hash(
      runtimeType,
      id,
      name,
      version,
      author,
      description,
      enabled,
      const DeepCollectionEquality().hash(types),
      const DeepCollectionEquality().hash(memoPrefixes));

  @override
  String toString() {
    return 'PluginInfo(id: $id, name: $name, version: $version, author: $author, description: $description, enabled: $enabled, types: $types, memoPrefixes: $memoPrefixes)';
  }
}

/// @nodoc
abstract mixin class $PluginInfoCopyWith<$Res> {
  factory $PluginInfoCopyWith(
          PluginInfo value, $Res Function(PluginInfo) _then) =
      _$PluginInfoCopyWithImpl;
  @useResult
  $Res call(
      {String id,
      String name,
      String version,
      String? author,
      String? description,
      bool enabled,
      List<String> types,
      List<String> memoPrefixes});
}

/// @nodoc
class _$PluginInfoCopyWithImpl<$Res> implements $PluginInfoCopyWith<$Res> {
  _$PluginInfoCopyWithImpl(this._self, this._then);

  final PluginInfo _self;
  final $Res Function(PluginInfo) _then;

  /// Create a copy of PluginInfo
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? id = null,
    Object? name = null,
    Object? version = null,
    Object? author = freezed,
    Object? description = freezed,
    Object? enabled = null,
    Object? types = null,
    Object? memoPrefixes = null,
  }) {
    return _then(_self.copyWith(
      id: null == id
          ? _self.id
          : id // ignore: cast_nullable_to_non_nullable
              as String,
      name: null == name
          ? _self.name
          : name // ignore: cast_nullable_to_non_nullable
              as String,
      version: null == version
          ? _self.version
          : version // ignore: cast_nullable_to_non_nullable
              as String,
      author: freezed == author
          ? _self.author
          : author // ignore: cast_nullable_to_non_nullable
              as String?,
      description: freezed == description
          ? _self.description
          : description // ignore: cast_nullable_to_non_nullable
              as String?,
      enabled: null == enabled
          ? _self.enabled
          : enabled // ignore: cast_nullable_to_non_nullable
              as bool,
      types: null == types
          ? _self.types
          : types // ignore: cast_nullable_to_non_nullable
              as List<String>,
      memoPrefixes: null == memoPrefixes
          ? _self.memoPrefixes
          : memoPrefixes // ignore: cast_nullable_to_non_nullable
              as List<String>,
    ));
  }
}

/// Adds pattern-matching-related methods to [PluginInfo].
extension PluginInfoPatterns on PluginInfo {
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
    TResult Function(_PluginInfo value)? $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _PluginInfo() when $default != null:
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
    TResult Function(_PluginInfo value) $default,
  ) {
    final _that = this;
    switch (_that) {
      case _PluginInfo():
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
    TResult? Function(_PluginInfo value)? $default,
  ) {
    final _that = this;
    switch (_that) {
      case _PluginInfo() when $default != null:
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
            String id,
            String name,
            String version,
            String? author,
            String? description,
            bool enabled,
            List<String> types,
            List<String> memoPrefixes)?
        $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _PluginInfo() when $default != null:
        return $default(_that.id, _that.name, _that.version, _that.author,
            _that.description, _that.enabled, _that.types, _that.memoPrefixes);
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
            String id,
            String name,
            String version,
            String? author,
            String? description,
            bool enabled,
            List<String> types,
            List<String> memoPrefixes)
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _PluginInfo():
        return $default(_that.id, _that.name, _that.version, _that.author,
            _that.description, _that.enabled, _that.types, _that.memoPrefixes);
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
            String id,
            String name,
            String version,
            String? author,
            String? description,
            bool enabled,
            List<String> types,
            List<String> memoPrefixes)?
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _PluginInfo() when $default != null:
        return $default(_that.id, _that.name, _that.version, _that.author,
            _that.description, _that.enabled, _that.types, _that.memoPrefixes);
      case _:
        return null;
    }
  }
}

/// @nodoc

class _PluginInfo implements PluginInfo {
  const _PluginInfo(
      {required this.id,
      required this.name,
      required this.version,
      this.author,
      this.description,
      required this.enabled,
      required final List<String> types,
      required final List<String> memoPrefixes})
      : _types = types,
        _memoPrefixes = memoPrefixes;

  @override
  final String id;
  @override
  final String name;
  @override
  final String version;
  @override
  final String? author;
  @override
  final String? description;
  @override
  final bool enabled;
  final List<String> _types;
  @override
  List<String> get types {
    if (_types is EqualUnmodifiableListView) return _types;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_types);
  }

  final List<String> _memoPrefixes;
  @override
  List<String> get memoPrefixes {
    if (_memoPrefixes is EqualUnmodifiableListView) return _memoPrefixes;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_memoPrefixes);
  }

  /// Create a copy of PluginInfo
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  _$PluginInfoCopyWith<_PluginInfo> get copyWith =>
      __$PluginInfoCopyWithImpl<_PluginInfo>(this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _PluginInfo &&
            (identical(other.id, id) || other.id == id) &&
            (identical(other.name, name) || other.name == name) &&
            (identical(other.version, version) || other.version == version) &&
            (identical(other.author, author) || other.author == author) &&
            (identical(other.description, description) ||
                other.description == description) &&
            (identical(other.enabled, enabled) || other.enabled == enabled) &&
            const DeepCollectionEquality().equals(other._types, _types) &&
            const DeepCollectionEquality()
                .equals(other._memoPrefixes, _memoPrefixes));
  }

  @override
  int get hashCode => Object.hash(
      runtimeType,
      id,
      name,
      version,
      author,
      description,
      enabled,
      const DeepCollectionEquality().hash(_types),
      const DeepCollectionEquality().hash(_memoPrefixes));

  @override
  String toString() {
    return 'PluginInfo(id: $id, name: $name, version: $version, author: $author, description: $description, enabled: $enabled, types: $types, memoPrefixes: $memoPrefixes)';
  }
}

/// @nodoc
abstract mixin class _$PluginInfoCopyWith<$Res>
    implements $PluginInfoCopyWith<$Res> {
  factory _$PluginInfoCopyWith(
          _PluginInfo value, $Res Function(_PluginInfo) _then) =
      __$PluginInfoCopyWithImpl;
  @override
  @useResult
  $Res call(
      {String id,
      String name,
      String version,
      String? author,
      String? description,
      bool enabled,
      List<String> types,
      List<String> memoPrefixes});
}

/// @nodoc
class __$PluginInfoCopyWithImpl<$Res> implements _$PluginInfoCopyWith<$Res> {
  __$PluginInfoCopyWithImpl(this._self, this._then);

  final _PluginInfo _self;
  final $Res Function(_PluginInfo) _then;

  /// Create a copy of PluginInfo
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $Res call({
    Object? id = null,
    Object? name = null,
    Object? version = null,
    Object? author = freezed,
    Object? description = freezed,
    Object? enabled = null,
    Object? types = null,
    Object? memoPrefixes = null,
  }) {
    return _then(_PluginInfo(
      id: null == id
          ? _self.id
          : id // ignore: cast_nullable_to_non_nullable
              as String,
      name: null == name
          ? _self.name
          : name // ignore: cast_nullable_to_non_nullable
              as String,
      version: null == version
          ? _self.version
          : version // ignore: cast_nullable_to_non_nullable
              as String,
      author: freezed == author
          ? _self.author
          : author // ignore: cast_nullable_to_non_nullable
              as String?,
      description: freezed == description
          ? _self.description
          : description // ignore: cast_nullable_to_non_nullable
              as String?,
      enabled: null == enabled
          ? _self.enabled
          : enabled // ignore: cast_nullable_to_non_nullable
              as bool,
      types: null == types
          ? _self._types
          : types // ignore: cast_nullable_to_non_nullable
              as List<String>,
      memoPrefixes: null == memoPrefixes
          ? _self._memoPrefixes
          : memoPrefixes // ignore: cast_nullable_to_non_nullable
              as List<String>,
    ));
  }
}

// dart format on
