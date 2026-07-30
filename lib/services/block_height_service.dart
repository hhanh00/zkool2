import 'dart:async';

typedef FetchBlockHeight = Future<int> Function();

/// Polls the lightwalletd server while at least one component is listening.
///
/// Every new subscriber receives a freshly fetched height. After that,
/// subscribers are notified only when the height changes.
class BlockHeightService {
  BlockHeightService({
    required FetchBlockHeight fetchHeight,
    this.pollInterval = const Duration(seconds: 10),
  }) : _fetchHeight = fetchHeight;

  final FetchBlockHeight _fetchHeight;
  final Duration pollInterval;
  final Set<MultiStreamController<int>> _subscribers = {};

  Timer? _timer;
  Future<int>? _fetchInProgress;
  int? _lastHeight;
  bool _pollInProgress = false;
  int _subscriberCount = 0;

  int? get lastHeight => _lastHeight;
  int get subscriberCount => _subscriberCount;
  bool get isPolling => _subscriberCount > 0;

  /// Returns a stream backed by the shared polling loop.
  ///
  /// Each subscriber receives a freshly polled initial height, followed by
  /// observed tip-height changes.
  Stream<int> get heights => Stream.multi((controller) {
        var cancelled = false;
        var registered = false;
        _subscriberCount++;

        controller.onCancel = () {
          if (cancelled) return;

          cancelled = true;
          _subscriberCount--;
          if (registered) {
            _removeSubscriber(controller);
          }
        };

        unawaited(
          Future<void>(
            () async {
              try {
                final height = await fetchCurrent();
                if (cancelled) return;

                _subscribers.add(controller);
                registered = true;
                controller.add(height);
                _scheduleNextPoll();
              } catch (error, stackTrace) {
                if (cancelled) return;

                // Keep the subscriber active so the shared polling loop can retry.
                _subscribers.add(controller);
                registered = true;
                controller.addError(error, stackTrace);
                _requestPoll();
              }
            },
          ),
        );
      });

  /// Fetches one height without keeping the polling loop alive.
  Future<int> fetchCurrent() async {
    final height = await _fetchShared();
    _recordHeight(height);
    return height;
  }

  void _removeSubscriber(MultiStreamController<int> controller) {
    _subscribers.remove(controller);
    if (_subscribers.isEmpty) {
      _timer?.cancel();
      _timer = null;
    }
  }

  void _requestPoll() {
    if (_subscribers.isEmpty) return;

    if (_pollInProgress) return;

    _timer?.cancel();
    _timer = null;
    unawaited(_poll());
  }

  void _scheduleNextPoll() {
    if (_subscribers.isEmpty || _pollInProgress || _timer != null) return;
    _timer = Timer(pollInterval, _requestPoll);
  }

  Future<void> _poll() async {
    _pollInProgress = true;
    try {
      final height = await _fetchShared();
      _recordHeight(height);
    } catch (error, stackTrace) {
      for (final subscriber in _subscribers.toList()) {
        subscriber.addError(error, stackTrace);
      }
    } finally {
      _pollInProgress = false;
      _scheduleNextPoll();
    }
  }

  Future<int> _fetchShared() {
    final inProgress = _fetchInProgress;
    if (inProgress != null) return inProgress;

    late final Future<int> request;
    request = _fetchHeight().whenComplete(() {
      if (identical(_fetchInProgress, request)) {
        _fetchInProgress = null;
      }
    });
    _fetchInProgress = request;
    return request;
  }

  void _recordHeight(int height) {
    final changed = height != _lastHeight;
    _lastHeight = height;
    if (!changed) return;

    for (final subscriber in _subscribers.toList()) {
      subscriber.add(height);
    }
  }
}
