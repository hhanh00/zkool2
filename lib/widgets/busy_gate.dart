import 'dart:async';

import 'package:flutter/widgets.dart';

/// One asynchronous update at a time for a page.
///
/// A page that writes its state from the result of an await (a DNS lookup,
/// a key derivation) can have that state move on under it: the user edits the
/// field, picks another option, or leaves the page, and the late result then
/// lands on top of the newer value. The gate removes the window instead of
/// detecting it: [busy] is set from the moment [runExclusive] accepts an
/// action until the action settles, the page locks every control that could
/// write the same state while it is set, and a second action asked for while
/// one is pending is dropped.
///
/// [busyShown] follows [busy] after [busyIndicatorDelay], so a page can show
/// a progress indicator for a slow action without flashing one for a fast one.
mixin BusyGate<T extends StatefulWidget> on State<T> {
  /// An action is pending; the page's writers are locked.
  bool busy = false;

  /// [busy] has lasted at least [busyIndicatorDelay]; show an indicator.
  bool busyShown = false;

  Timer? _busyIndicator;

  /// How long an action runs before [busyShown] is set.
  Duration get busyIndicatorDelay => const Duration(seconds: 1);

  /// Runs [action] unless another is pending, in which case [action] is
  /// dropped and the result is false. [busy] is cleared when [action]
  /// settles, whether it returned or threw.
  Future<bool> runExclusive(Future<void> Function() action) async {
    if (busy) return false;
    setState(() => busy = true);
    _busyIndicator = Timer(busyIndicatorDelay, () {
      if (mounted && busy) setState(() => busyShown = true);
    });
    try {
      await action();
    } finally {
      _busyIndicator?.cancel();
      _busyIndicator = null;
      if (mounted) {
        setState(() {
          busy = false;
          busyShown = false;
        });
      } else {
        busy = false;
        busyShown = false;
      }
    }
    return true;
  }

  @override
  void dispose() {
    _busyIndicator?.cancel();
    super.dispose();
  }
}
