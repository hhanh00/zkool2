import 'package:flutter/material.dart';

/// Which FROST account generations a backup string restores: the legacy
/// (pre-ZIP 2005) account, the ZIP 2005 one, or both. A bit mask like the
/// pool selector's: 1 legacy, 2 ZIP 2005.
enum FrostGeneration { legacy, zip2005 }

extension FrostGenerationBit on FrostGeneration {
  int get bit => 1 << index;
  String get label => switch (this) {
        FrostGeneration.legacy => "Legacy",
        FrostGeneration.zip2005 => "ZIP 2005",
      };
}

class FrostGenerationSelect extends StatefulWidget {
  final int initialValue;
  final void Function(int v)? onChanged;
  const FrostGenerationSelect({
    super.key,
    required this.initialValue,
    required this.onChanged,
  });

  @override
  State<FrostGenerationSelect> createState() => _FrostGenerationSelectState();
}

class _FrostGenerationSelectState extends State<FrostGenerationSelect> {
  late Set<FrostGeneration> selected;

  Set<FrostGeneration> _fromMask(int mask) =>
      FrostGeneration.values.where((g) => mask & g.bit != 0).toSet();

  @override
  void initState() {
    super.initState();
    selected = _fromMask(widget.initialValue);
  }

  @override
  void didUpdateWidget(covariant FrostGenerationSelect oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.initialValue != widget.initialValue) {
      selected = _fromMask(widget.initialValue);
    }
  }

  @override
  Widget build(BuildContext context) {
    return SegmentedButton<FrostGeneration>(
      multiSelectionEnabled: true,
      emptySelectionAllowed: true,
      segments: FrostGeneration.values
          .map((g) => ButtonSegment(value: g, label: Text(g.label)))
          .toList(),
      selected: selected,
      onSelectionChanged: (s) {
        setState(() => selected = s);
        widget.onChanged?.call(s.fold(0, (mask, g) => mask | g.bit));
      },
    );
  }
}
