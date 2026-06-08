import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:data_table_2/data_table_2.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:zkool/src/rust/api/network.dart';
import 'package:zkool/store.dart';

class LWDSelectPage extends ConsumerStatefulWidget {
  const LWDSelectPage({super.key});

  @override
  ConsumerState<LWDSelectPage> createState() => _LWDSelectPageState();
}

class _LWDSelectPageState extends ConsumerState<LWDSelectPage> {
  int _sortColumnIndex = 0;
  bool _sortAscending = true;
  bool? _onlineFilter; // null = all, true = online, false = offline

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
      body: Builder(builder: (context) {
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
                padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 4),
                child: Row(
                  children: [
                    _FilterChip(
                      label: "All",
                      selected: _onlineFilter == null,
                      onTap: () => setState(() => _onlineFilter = null),
                    ),
                    const SizedBox(width: 8),
                    _FilterChip(
                      label: "Online",
                      selected: _onlineFilter == true,
                      color: Colors.green,
                      onTap: () => setState(() => _onlineFilter = true),
                    ),
                    const SizedBox(width: 8),
                    _FilterChip(
                      label: "Offline",
                      selected: _onlineFilter == false,
                      color: Colors.red,
                      onTap: () => setState(() => _onlineFilter = false),
                    ),
                    const Spacer(),
                    Text("${filtered.length} servers", style: tt.bodySmall),
                  ],
                ),
              ),
              // Paginated DataTable
              Expanded(
                child: PaginatedDataTable2(
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
                  const SizedBox(width: 4),
                  const Icon(Icons.shield, size: 14, color: Colors.purple),
                ],
                const SizedBox(width: 8),
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(server.url, overflow: TextOverflow.ellipsis),
                      if (isOnline && server.version.isNotEmpty)
                        Text(server.version, style: tt.bodySmall),
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

class _FilterChip extends StatelessWidget {
  final String label;
  final bool selected;
  final VoidCallback onTap;
  final Color? color;

  const _FilterChip({
    required this.label,
    required this.selected,
    required this.onTap,
    this.color,
  });

  @override
  Widget build(BuildContext context) {
    return FilterChip(
      label: Text(label),
      selected: selected,
      onSelected: (_) => onTap(),
      selectedColor: color?.withValues(alpha: 0.3),
      checkmarkColor: color,
      side: selected && color != null ? BorderSide(color: color!) : null,
    );
  }
}
