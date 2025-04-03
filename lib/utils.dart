import 'dart:math';

String initials(String name) => name.substring(0, min(2, name.length)).toUpperCase();

