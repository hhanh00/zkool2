import 'package:flutter/material.dart';

class PoolSelect extends StatefulWidget {
  final int enabled;
  final int initialValue;
  final void Function(int v)? onChanged;
  const PoolSelect({
    super.key,
    required this.enabled,
    required this.initialValue,
    required this.onChanged,
  });

  @override
  State<PoolSelect> createState() => _PoolSelectState();
}

enum Pool { transparent, sapling, orchard, ironwood }

extension PoolBit on Pool {
  int get bit => 1 << index;
}

int poolMask(Iterable<Pool> pools) =>
    pools.fold(0, (mask, pool) => mask | pool.bit);

class _PoolSelectState extends State<PoolSelect> {
  late Set<Pool> pools;

  Set<Pool> _valueToPools(int value) {
    return Pool.values.where((p) => value & p.bit != 0).toSet();
  }

  @override
  void initState() {
    super.initState();
    pools = _valueToPools(widget.initialValue);
  }

  @override
  void didUpdateWidget(covariant PoolSelect oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.initialValue != widget.initialValue) {
      pools = _valueToPools(widget.initialValue);
    }
  }

  @override
  Widget build(BuildContext context) {
    final onChanged = widget.onChanged;

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
              enabled: widget.enabled & Pool.transparent.bit != 0,
            ),
            ButtonSegment<Pool>(
              value: Pool.sapling,
              label: Text('Sap'),
              enabled: widget.enabled & Pool.sapling.bit != 0,
            ),
            ButtonSegment<Pool>(
              value: Pool.orchard,
              label: Text('Orc'),
              enabled: widget.enabled & Pool.orchard.bit != 0,
            ),
            ButtonSegment<Pool>(
              value: Pool.ironwood,
              label: Text('Iwd'),
              enabled: widget.enabled & Pool.ironwood.bit != 0,
            ),
          ],
          selected: pools,
          onSelectionChanged: onChanged != null
              ? (Set<Pool> newSelection) {
                  setState(() {
                    pools = newSelection;
                    onChanged(poolMask(newSelection));
                  });
                }
              : null,
        ),
      ],
    );
  }
}
