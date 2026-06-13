import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:data_table_2/data_table_2.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:zkool/src/rust/api/network.dart';
import 'package:gap/gap.dart';
import 'package:zkool/store.dart';

import '../main.dart';

class LWDSelectPage extends ConsumerStatefulWidget {
  const LWDSelectPage({super.key});

  @override
  ConsumerState<LWDSelectPage> createState() => _LWDSelectPageState();
}

class _LWDSelectPageState extends ConsumerState<LWDSelectPage> {
  int _sortColumnIndex = 3; // Uptime
  bool _sortAscending = false; // descending
  bool? _onlineFilter = true; // null = all, true = online, false = offline

  List<LWDInfo>? _servers;
  bool _loading = true;
  String? _error;

  Future<void> _onSelectServer(LWDInfo server) async {
    var url = server.url;
    // Ensure URL has a scheme (defense in depth)
    if (!url.startsWith('https://') && !url.startsWith('http://')) {
      final scheme = server.isTor ? 'http' : 'https';
      url = '$scheme://$url';
    }
    // Enable Tor for onion addresses
    if (server.isTor) {
      final prefs = SharedPreferencesAsync();
      await prefs.setBool("use_tor", true);
      final c = coinContext.coin;
      await c.setUseTor(useTor: true);
      ref.invalidate(appSettingsProvider);
    }
    if (mounted) {
      Navigator.of(context).pop(url);
    }
  }

  @override
  void initState() {
    super.initState();
    _loadServers();
  }

  Future<void> _loadServers() async {
    try {
      final servers = await queryLwdList();
      if (!mounted) return;
      setState(() {
        _servers = servers;
        _loading = false;
        _error = null;
      });
    } catch (e) {
      if (!mounted) return;
      setState(() {
        _loading = false;
        _error = e.toString();
      });
    }
  }

  @override
  Widget build(BuildContext context) {
    final tt = Theme.of(context).textTheme;

    return Scaffold(
      appBar: AppBar(
        title: const Text("Select Lightwalletd Server"),
      ),
      body: Builder(
        builder: (context) {
          if (_loading) {
            return const Center(child: CircularProgressIndicator());
          }
          if (_error != null) {
            return Center(
              child: Padding(
                padding: const EdgeInsets.all(16),
                child: Text(
                  "Failed to load server list: $_error",
                  style: tt.bodyLarge,
                  textAlign: TextAlign.center,
                ),
              ),
            );
          }

          final servers = _servers!;
          if (servers.isEmpty) {
            return Center(
              child: Text("No servers available", style: tt.bodyLarge),
            );
          }

          // Filter
          final filtered = servers.where((s) {
            if (_onlineFilter == null) return true;
            final isOnline = s.status == "online";
            return _onlineFilter! ? isOnline : !isOnline;
          }).toList();

          // Sort
          filtered.sort((a, b) {
            int cmp;
            switch (_sortColumnIndex) {
              case 0:
                cmp = a.url.compareTo(b.url);
              case 1:
                cmp = a.ping.compareTo(b.ping);
              case 2:
                cmp = a.height.compareTo(b.height);
              case 3:
                cmp = a.uptime.compareTo(b.uptime);
              default:
                cmp = 0;
            }
            return _sortAscending ? cmp : -cmp;
          });

          return Column(
            children: [
              // Online/offline filter
              Padding(
                padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
                child: Row(
                  children: [
                    SegmentedButton<bool?>(
                      segments: const [
                        ButtonSegment<bool?>(
                          value: null,
                          label: Text("All"),
                        ),
                        ButtonSegment<bool?>(
                          value: true,
                          label: Text("Online"),
                          icon: Icon(Icons.circle, color: Colors.green, size: 12),
                        ),
                        ButtonSegment<bool?>(
                          value: false,
                          label: Text("Offline"),
                          icon: Icon(Icons.circle, color: Colors.red, size: 12),
                        ),
                      ],
                      selected: {_onlineFilter},
                      onSelectionChanged: (selected) {
                        _applyFilter(selected.first);
                      },
                      style: const ButtonStyle(
                        visualDensity: VisualDensity.compact,
                        tapTargetSize: MaterialTapTargetSize.shrinkWrap,
                      ),
                    ),
                    const Spacer(),
                    Text("${filtered.length} servers", style: tt.bodySmall),
                  ],
                ),
              ),
              // Paginated DataTable
              Expanded(
                child: PaginatedDataTable2(
                  key: ValueKey(_onlineFilter),
                  sortColumnIndex: _sortColumnIndex,
                  sortAscending: _sortAscending,
                  headingRowColor: WidgetStateProperty.all(
                    Theme.of(context).colorScheme.surfaceContainerHighest,
                  ),
                  columnSpacing: 12,
                  horizontalMargin: 12,
                  minWidth: 600,
                  fixedLeftColumns: 1,
                  rowsPerPage: 10,
                  availableRowsPerPage: const [10, 20, 50],
                  onRowsPerPageChanged: (_) {},
                  columns: [
                    DataColumn2(
                      label: const Text("Server"),
                      onSort: (i, asc) => _onSort(i, asc),
                      size: ColumnSize.L,
                    ),
                    DataColumn2(
                      label: const Text("Ping (ms)"),
                      numeric: true,
                      onSort: (i, asc) => _onSort(i, asc),
                      size: ColumnSize.S,
                    ),
                    DataColumn2(
                      label: const Text("Height"),
                      numeric: true,
                      onSort: (i, asc) => _onSort(i, asc),
                      size: ColumnSize.S,
                    ),
                    DataColumn2(
                      label: const Text("Uptime"),
                      numeric: true,
                      onSort: (i, asc) => _onSort(i, asc),
                      size: ColumnSize.S,
                    ),
                  ],
                  source: _LwdDataSource(filtered, context, _onSelectServer),
                ),
              ),
            ],
          );
        },
      ),
    );
  }

  void _applyFilter(bool? filter) {
    logger.i("_applyFilter $filter");
    setState(() => _onlineFilter = filter);
  }

  void _onSort(int columnIndex, bool ascending) {
    setState(() {
      _sortColumnIndex = columnIndex;
      _sortAscending = ascending;
    });
  }
}

class _LwdDataSource extends DataTableSource {
  final List<LWDInfo> servers;
  final BuildContext context;
  final void Function(LWDInfo) onSelect;

  _LwdDataSource(this.servers, this.context, this.onSelect);

  @override
  DataRow? getRow(int index) {
    if (index >= servers.length) return null;
    final server = servers[index];
    final isOnline = server.status == "online";
    final tt = Theme.of(context).textTheme;
    return DataRow2(
      cells: [
        DataCell(
          GestureDetector(
            behavior: HitTestBehavior.opaque,
            onTap: () => onSelect(server),
            child: Row(
              children: [
                Icon(
                  Icons.circle,
                  size: 16,
                  color: isOnline ? Colors.green : Colors.red.shade300,
                ),
                if (server.isTor) ...[
                  const Gap(4),
                  const Icon(Icons.shield, size: 14, color: Colors.purple),
                ],
                const Gap(8),
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(server.url, overflow: TextOverflow.ellipsis),
                      if (isOnline && server.version.isNotEmpty) Text(server.version, style: tt.bodySmall),
                    ],
                  ),
                ),
              ],
            ),
          ),
        ),
        DataCell(Text("${server.ping}")),
        DataCell(Text(server.height > 0 ? "${server.height}" : "-")),
        DataCell(Text("${server.uptime}%")),
      ],
    );
  }

  @override
  bool get isRowCountApproximate => false;

  @override
  int get rowCount => servers.length;

  @override
  int get selectedRowCount => 0;
}
