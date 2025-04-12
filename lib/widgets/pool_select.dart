import 'package:flutter/material.dart';

class PoolSelect extends StatefulWidget {
  final void Function(int v)? onChanged;
  const PoolSelect({super.key, required this.onChanged});

  @override
  State<PoolSelect> createState() => _PoolSelectState();
}

enum Pool { transparent, sapling, orchard }

class _PoolSelectState extends State<PoolSelect> {
  Set<Pool> pools = {Pool.transparent, Pool.sapling, Pool.orchard};

  @override
  Widget build(BuildContext context) {
    return Row(mainAxisSize: MainAxisSize.min,
      children: [SegmentedButton<Pool>(
      style: SegmentedButton.styleFrom(
        backgroundColor: Colors.grey[200],
        foregroundColor: Colors.red,
        selectedForegroundColor: Colors.white,
        selectedBackgroundColor: Colors.green,
      ),
      multiSelectionEnabled: true,
      showSelectedIcon: false,
      segments: const <ButtonSegment<Pool>>[
        ButtonSegment<Pool>(
          value: Pool.transparent,
          label: Text('Transparent'),
        ),
        ButtonSegment<Pool>(
          value: Pool.sapling,
          label: Text('Sapling'),
        ),
        ButtonSegment<Pool>(
          value: Pool.orchard,
          label: Text('Orchard'),
        ),
      ],
      selected: pools,
      onSelectionChanged: (Set<Pool> newSelection) {
        setState(() {
          pools = newSelection;
          widget.onChanged?.call(
            newSelection.fold(0, (previousValue, element) {
              switch (element) {
                case Pool.transparent:
                  return previousValue | 1;
                case Pool.sapling:
                  return previousValue | 2;
                case Pool.orchard:
                  return previousValue | 4;
              }
            }),
          );
        });
      },
    )]);
  }
}
