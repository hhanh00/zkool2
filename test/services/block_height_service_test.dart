import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:zkool/services/block_height_service.dart';

void main() {
  test('polls only while subscribed', () async {
    final requests = StreamController<Completer<int>>();
    final service = BlockHeightService(
      fetchHeight: () {
        final request = Completer<int>();
        requests.add(request);
        return request.future;
      },
      pollInterval: const Duration(hours: 1),
    );

    final heights = <int>[];
    final subscription = service.heights.listen(heights.add);
    final request = await requests.stream.first;

    expect(service.subscriberCount, 1);
    expect(service.isPolling, isTrue);

    request.complete(42);
    await pumpEventQueue();
    expect(heights, [42]);

    await subscription.cancel();
    expect(service.subscriberCount, 0);
    expect(service.isPolling, isFalse);

    await requests.close();
  });

  test('shares one poller and fetches a fresh height for late subscribers', () async {
    var polls = 0;
    final service = BlockHeightService(
      fetchHeight: () async {
        polls++;
        return 100;
      },
      pollInterval: const Duration(hours: 1),
    );

    final firstHeights = <int>[];
    final first = service.heights.listen(firstHeights.add);
    await pumpEventQueue();

    final secondHeights = <int>[];
    final second = service.heights.listen(secondHeights.add);
    await pumpEventQueue();

    expect(polls, 2);
    expect(firstHeights, [100]);
    expect(secondHeights, [100]);
    expect(service.subscriberCount, 2);

    await first.cancel();
    expect(service.isPolling, isTrue);
    await second.cancel();
    expect(service.isPolling, isFalse);
  });

  test('shares an in-flight fetch between new subscribers', () async {
    var polls = 0;
    final request = Completer<int>();
    final service = BlockHeightService(
      fetchHeight: () {
        polls++;
        return request.future;
      },
      pollInterval: const Duration(hours: 1),
    );

    final firstHeights = <int>[];
    final secondHeights = <int>[];
    final first = service.heights.listen(firstHeights.add);
    final second = service.heights.listen(secondHeights.add);
    await pumpEventQueue();

    expect(polls, 1);
    request.complete(100);
    await pumpEventQueue();
    expect(firstHeights, [100]);
    expect(secondHeights, [100]);

    await first.cancel();
    await second.cancel();
  });

  test('does not register after cancellation during the initial poll', () async {
    var polls = 0;
    final request = Completer<int>();
    final service = BlockHeightService(
      fetchHeight: () {
        polls++;
        return request.future;
      },
      pollInterval: const Duration(milliseconds: 1),
    );

    final subscription = service.heights.listen((_) {});
    await pumpEventQueue();
    expect(polls, 1);

    await subscription.cancel();
    request.complete(100);
    await pumpEventQueue();

    expect(service.subscriberCount, 0);
    expect(service.isPolling, isFalse);
    expect(polls, 1);
  });

  test('emits only when the block height changes after the initial value', () async {
    var polls = 0;
    final secondHeight = Completer<void>();
    final service = BlockHeightService(
      fetchHeight: () async {
        polls++;
        if (polls < 3) return 100;
        return 101;
      },
      pollInterval: const Duration(milliseconds: 1),
    );

    final heights = <int>[];
    final subscription = service.heights.listen((height) {
      heights.add(height);
      if (height == 101) secondHeight.complete();
    });

    await secondHeight.future.timeout(const Duration(seconds: 1));
    expect(heights, [100, 101]);

    await subscription.cancel();
  });
}
