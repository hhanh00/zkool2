import 'dart:typed_data';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:url_launcher/url_launcher.dart';
import '../src/rust/api/plugin.dart' as plugin_api;
import '../store.dart';
import '../utils.dart';

/// A widget that takes memo bytes, runs them through installed plugins,
/// and renders the parsed sections as DataTables.
class PluginMemoView extends ConsumerWidget {
  final Uint8List memoBytes;

  const PluginMemoView(this.memoBytes, {super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    // Only attempt plugin parsing for 0xFF (Arbitrary) memos
    if (memoBytes.isEmpty || memoBytes[0] != 0xFF) {
      return const SizedBox.shrink();
    }

    final c = coinContext.coin;
    final sectionsAsync =
        ref.watch(pluginMemoSectionsProvider(memoBytes, c));

    return sectionsAsync.when(
      data: (sections) {
        if (sections.isEmpty) return const SizedBox.shrink();
        return Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            for (final section in sections) _buildSection(context, section),
          ],
        );
      },
      loading: () => const SizedBox.shrink(),
      error: (e, _) {
        showSnackbar('Plugin error: $e');
        return const SizedBox.shrink();
      },
    );
  }

  Widget _buildSection(BuildContext context, plugin_api.MemoSection section) {
    final t = Theme.of(context);

    return Card(
      margin: const EdgeInsets.symmetric(vertical: 4),
      child: Padding(
        padding: const EdgeInsets.all(8.0),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(section.title, style: t.textTheme.titleSmall),
            const SizedBox(height: 4),
            SingleChildScrollView(
              scrollDirection: Axis.horizontal,
              child: DataTable(
                columnSpacing: 16,
                headingTextStyle: t.textTheme.labelMedium,
                dataTextStyle: t.textTheme.bodySmall,
                columns: [
                  for (final header in section.headers)
                    DataColumn(label: Text(header)),
                ],
                rows: [
                  for (final row in section.rows)
                    DataRow(
                      cells: [
                        for (final cell in row.cells)
                          DataCell(_renderCell(cell, t)),
                      ],
                    ),
                ],
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _renderCell(plugin_api.MemoCell cell, ThemeData t) {
    switch (cell.cellType) {
      case 'url':
        return GestureDetector(
          onTap: () => launchUrl(Uri.parse(cell.value)),
          child: Text(
            cell.value,
            style: TextStyle(
              color: t.colorScheme.primary,
              decoration: TextDecoration.underline,
            ),
          ),
        );
      case 'number':
        return Text(
          cell.value,
          textAlign: TextAlign.right,
          style: const TextStyle(fontFeatures: [FontFeature.tabularFigures()]),
        );
      case 'date':
        final ts = int.tryParse(cell.value);
        if (ts != null) {
          final dt =
              DateTime.fromMillisecondsSinceEpoch(ts * 1000).toLocal();
          return Text(
            '${dt.year}-${dt.month.toString().padLeft(2, '0')}-${dt.day.toString().padLeft(2, '0')} '
            '${dt.hour.toString().padLeft(2, '0')}:${dt.minute.toString().padLeft(2, '0')}',
          );
        }
        return Text(cell.value);
      default:
        return Text(cell.value);
    }
  }
}
