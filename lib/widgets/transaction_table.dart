import 'dart:typed_data';

import 'package:flutter/material.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated.dart';
import 'package:zkool/src/rust/api/account.dart';
import 'package:zkool/utils.dart';

class TransactionTable extends StatefulWidget {
  final List<Tx> transactions;
  final List<Memo> memos;
  final String currency;
  final void Function(int id) onTap;

  const TransactionTable({
    super.key,
    required this.transactions,
    required this.memos,
    required this.currency,
    required this.onTap,
  });

  @override
  State<TransactionTable> createState() => _TransactionTableState();
}

class _TransactionTableState extends State<TransactionTable> {
  int _sortColumnIndex = 1; // default sort by date
  bool _sortAscending = false;

  // Sortable column indices
  static const _colType = 0;
  static const _colDate = 1;
  static const _colAmount = 2;
  static const _colHeight = 6;

  @override
  Widget build(BuildContext context) {
    final t = Theme.of(context).textTheme;
    final cellStyle = t.bodySmall?.copyWith(fontSize: 12);

    // Group memos by transaction id for quick lookup
    final memosByTx = <int, String?>{};
    for (final memo in widget.memos) {
      if (memo.memo != null && memo.memo!.isNotEmpty) {
        memosByTx.putIfAbsent(memo.idTx, () => memo.memo);
      }
    }

    final sorted = List<Tx>.from(widget.transactions);
    _sortTransactions(sorted);

    return LayoutBuilder(
      builder: (context, constraints) {
        return SingleChildScrollView(
          scrollDirection: Axis.horizontal,
          child: ConstrainedBox(
            constraints: BoxConstraints(minWidth: constraints.maxWidth),
            child: DataTable(
              sortColumnIndex: _sortColumnIndex,
              sortAscending: _sortAscending,
              headingTextStyle:
                  t.labelSmall?.copyWith(fontWeight: FontWeight.bold),
              dataTextStyle: cellStyle,
              columnSpacing: 16,
              horizontalMargin: 12,
              columns: [
                DataColumn(
                  label: const Text("Type"),
                  onSort: (_, asc) => _onSort(_colType, asc),
                ),
                DataColumn(
                  label: const Text("Date"),
                  onSort: (_, asc) => _onSort(_colDate, asc),
                ),
                DataColumn(
                  label: const Text("Amount"),
                  onSort: (_, asc) => _onSort(_colAmount, asc),
                ),
                const DataColumn(label: Text("Fiat Amount")),
                const DataColumn(label: Text("Price")),
                const DataColumn(label: Text("TxID")),
                DataColumn(
                  label: const Text("Height"),
                  onSort: (_, asc) => _onSort(_colHeight, asc),
                ),
                const DataColumn(label: Text("Category")),
                const DataColumn(label: Text("Memo")),
                const DataColumn(label: Text("Asset")),
              ],
              rows: sorted.map((tx) {
                final (color, icon, label) = getTransactionType(tx.tpe);
                final isZsa = tx.zsaValue != 0;

                return DataRow(
                  onSelectChanged: (_) => widget.onTap(tx.id),
                  cells: [
                    // Type
                    DataCell(SizedBox(
                      width: 101,
                      child: Row(
                        mainAxisSize: MainAxisSize.min,
                        children: [
                          Icon(icon, color: color, size: 16),
                          const SizedBox(width: 6),
                          Text(label),
                        ],
                      ),
                    )),
                    // Date
                    DataCell(SizedBox(
                      width: 120,
                      child: Text(timeToString(tx.time)),
                    )),
                    // Amount (ZEC)
                    DataCell(SizedBox(
                      width: 88,
                      child: Text(
                        zatToShortString(BigInt.from(tx.value)),
                        style: TextStyle(
                          color: _amountColor(tx.value),
                          fontWeight: FontWeight.w500,
                        ),
                      ),
                    )),
                    // Fiat Amount
                    DataCell(SizedBox(
                      width: 96,
                      child: Text(_formatFiatAmount(tx)),
                    )),
                    // Price
                    DataCell(SizedBox(
                      width: 72,
                      child: Text(_formatPrice(tx)),
                    )),
                    // TxID
                    DataCell(SizedBox(
                      width: 130,
                      child: Text(
                        _shortTxid(tx.txid),
                        style:
                            const TextStyle(fontFamily: 'monospace', fontSize: 11),
                      ),
                    )),
                    // Height
                    DataCell(SizedBox(
                      width: 60,
                      child: Text(tx.height.toString()),
                    )),
                    // Category
                    DataCell(SizedBox(
                      width: 76,
                      child: Text(tx.category ?? "—"),
                    )),
                    // Memo — no width constraint, fills remaining space
                    DataCell(Text(
                      _truncateMemo(memosByTx[tx.id]),
                      maxLines: 1,
                      overflow: TextOverflow.ellipsis,
                    )),
                    // Asset
                    DataCell(SizedBox(
                      width: 56,
                      child: Text(
                        isZsa ? tx.assetDisplay : "ZEC",
                        style: isZsa
                            ? const TextStyle(
                                color: Colors.purple,
                                fontWeight: FontWeight.w500,
                              )
                            : null,
                      ),
                    )),
                  ],
                );
              }).toList(),
            ),
          ),
        );
      },
    );
  }

  Color _amountColor(PlatformInt64 value) {
    if (value > 0) return Colors.green;
    if (value < 0) return Colors.red;
    return Colors.grey;
  }

  String _formatFiatAmount(Tx tx) {
    if (tx.zsaValue != 0) return "N/A";
    if (tx.price == null) return "—";
    final zecAmount = BigInt.from(tx.value).toDouble() / 100000000.0;
    final fiat = zecAmount * tx.price!;
    return formatFiat(fiat, widget.currency);
  }

  String _formatPrice(Tx tx) {
    if (tx.zsaValue != 0) return "N/A";
    if (tx.price == null) return "—";
    return formatFiat(tx.price!, widget.currency);
  }

  String _shortTxid(Uint8List txid) {
    final full = txIdToString(txid);
    if (full.length <= 16) return full;
    return "${full.substring(0, 8)}…${full.substring(full.length - 8)}";
  }

  String _truncateMemo(String? memo) {
    if (memo == null || memo.isEmpty) return "—";
    if (memo.length > 60) return "${memo.substring(0, 60)}…";
    return memo;
  }

  void _onSort(int columnIndex, bool ascending) {
    setState(() {
      _sortColumnIndex = columnIndex;
      _sortAscending = ascending;
    });
  }

  void _sortTransactions(List<Tx> txs) {
    txs.sort((a, b) {
      final cmp = switch (_sortColumnIndex) {
        _colType => (a.tpe ?? -1).compareTo(b.tpe ?? -1),
        _colDate => a.time.compareTo(b.time),
        _colAmount => a.value.compareTo(b.value),
        _colHeight => a.height.compareTo(b.height),
        _ => 0,
      };
      return _sortAscending ? cmp : -cmp;
    });
  }
}
