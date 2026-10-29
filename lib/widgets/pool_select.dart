import 'package:flutter/material.dart';

/// A pool multi-selector. Controlled: it shows [value] and nothing else,
/// so a page that sets its state only once an async fetch completes keeps
/// the selector in step with what it displays. Pass [enabled] as 0 to
/// refuse input while such a fetch is in flight.
class PoolSelect extends StatelessWidget {
  final int enabled;
  final int value;
  final void Function(int v)? onChanged;
  const PoolSelect({
    super.key,
    required this.enabled,
    required this.value,
    required this.onChanged,
  });

  @override
  Widget build(BuildContext context) {
    final onChanged = this.onChanged;

    return Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        SegmentedButton<Pool>(
          style: SegmentedButton.styleFrom(
            backgroundColor: Colors.grey[200],
            foregroundColor: Colors.red,
            selectedForegroundColor: Colors.white,
            selectedBackgroundColor: Colors.green,
          ),
          multiSelectionEnabled: true,
          showSelectedIcon: false,
          segments: <ButtonSegment<Pool>>[
            ButtonSegment<Pool>(
              value: Pool.transparent,
              label: Text('Trp'),
              enabled: enabled & Pool.transparent.bit != 0,
            ),
            ButtonSegment<Pool>(
              value: Pool.sapling,
              label: Text('Sap'),
              enabled: enabled & Pool.sapling.bit != 0,
            ),
            ButtonSegment<Pool>(
              value: Pool.orchard,
              label: Text('Orc'),
              enabled: enabled & Pool.orchard.bit != 0,
            ),
            ButtonSegment<Pool>(
              value: Pool.ironwood,
              label: Text('Iwd'),
              enabled: enabled & Pool.ironwood.bit != 0,
            ),
          ],
          selected: poolsOf(value),
          onSelectionChanged: onChanged != null
              ? (Set<Pool> newSelection) => onChanged(poolMask(newSelection))
              : null,
        ),
      ],
    );
  }
}

enum Pool { transparent, sapling, orchard, ironwood }

extension PoolBit on Pool {
  int get bit => 1 << index;
}

int poolMask(Iterable<Pool> pools) =>
    pools.fold(0, (mask, pool) => mask | pool.bit);

Set<Pool> poolsOf(int mask) =>
    Pool.values.where((p) => mask & p.bit != 0).toSet();
