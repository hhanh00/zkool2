import 'package:flutter/material.dart';

class PoolSelect extends StatefulWidget {
  final int enabled;
  final int initialValue;
  final void Function(int v)? onChanged;
  const PoolSelect({super.key,
    required this.enabled,
    required this.initialValue,
    required this.onChanged,});

  @override
  State<PoolSelect> createState() => _PoolSelectState();
}

enum Pool { transparent, sapling, orchard }

class _PoolSelectState extends State<PoolSelect> {
  late Set<Pool> pools;

  @override
  void initState() {
    super.initState();
    final initialValue = widget.initialValue;
    pools = {
      if (initialValue & 1 != 0) Pool.transparent,
      if (initialValue & 2 != 0) Pool.sapling,
      if (initialValue & 4 != 0) Pool.orchard,
    };
  }

  @override
  Widget build(BuildContext context) {
    final onChanged = widget.onChanged;

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
      segments: <ButtonSegment<Pool>>[
        ButtonSegment<Pool>(
          value: Pool.transparent,
          label: Text('Transparent'),
          enabled: widget.enabled & 1 != 0,
        ),
        ButtonSegment<Pool>(
          value: Pool.sapling,
          label: Text('Sapling'),
          enabled: widget.enabled & 2 != 0,
        ),
        ButtonSegment<Pool>(
          value: Pool.orchard,
          label: Text('Orchard'),
          enabled: widget.enabled & 4 != 0,
        ),
      ],
      selected: pools,
      onSelectionChanged: onChanged != null ? (Set<Pool> newSelection) {
        setState(() {
          pools = newSelection;
          onChanged(
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
      } : null,
    ),],);
  }
}
