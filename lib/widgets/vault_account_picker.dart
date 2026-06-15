import 'package:flutter/material.dart';
import 'package:gap/gap.dart';
import 'package:zkool/src/rust/api/vault.dart' show RestoredAccount;
import 'package:zkool/utils.dart';

Future<List<RestoredAccount>?> showVaultAccountPicker(
  BuildContext context, {
  required List<RestoredAccount> accounts,
}) async {
  if (accounts.isEmpty) {
    if (context.mounted) {
      await showMessage(context, "No accounts found in the vault backup.",
          title: "Recovery");
    }
    return null;
  }

  final result = await Navigator.of(context).push<List<RestoredAccount>>(
    MaterialPageRoute(
      builder: (_) => _VaultAccountPickerDialog(accounts: accounts),
    ),
  );
  return result;
}

class _VaultAccountPickerDialog extends StatefulWidget {
  final List<RestoredAccount> accounts;

  const _VaultAccountPickerDialog({required this.accounts});

  @override
  State<_VaultAccountPickerDialog> createState() =>
      _VaultAccountPickerDialogState();
}

class _VaultAccountPickerDialogState extends State<_VaultAccountPickerDialog> {
  final _selected = <RestoredAccount>{};
  final _searchController = TextEditingController();
  String _query = '';

  bool get _allSelected => _selected.length == widget.accounts.length;

  @override
  void initState() {
    super.initState();
    // All accounts selected by default
    _selected.addAll(widget.accounts);
  }

  @override
  void dispose() {
    _searchController.dispose();
    super.dispose();
  }

  List<RestoredAccount> get _filtered {
    if (_query.isEmpty) return widget.accounts;
    final q = _query.toLowerCase();
    return widget.accounts
        .where((a) => a.name.toLowerCase().contains(q))
        .toList();
  }

  void _toggleAll() {
    setState(() {
      if (_allSelected) {
        _selected.clear();
      } else {
        _selected.addAll(widget.accounts);
      }
    });
  }

  @override
  Widget build(BuildContext context) {
    final tt = Theme.of(context).textTheme;
    final cs = Theme.of(context).colorScheme;
    final filtered = _filtered;

    return Scaffold(
      appBar: AppBar(
        title: const Text("Recover Accounts"),
        leading: IconButton(
          icon: const Icon(Icons.close),
          onPressed: () => Navigator.of(context).pop(null),
        ),
        actions: [
          TextButton(
            onPressed: _toggleAll,
            child: Text(_allSelected ? "Deselect All" : "Select All"),
          ),
        ],
      ),
      body: Column(
        children: [
          // Search
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 4),
            child: TextField(
              controller: _searchController,
              decoration: const InputDecoration(
                hintText: "Search accounts...",
                prefixIcon: Icon(Icons.search),
                border: OutlineInputBorder(),
                isDense: true,
              ),
              onChanged: (v) => setState(() => _query = v),
            ),
          ),
          const Gap(8),
          // Account list
          Expanded(
            child: filtered.isEmpty
                ? Center(
                    child: Column(
                      mainAxisSize: MainAxisSize.min,
                      children: [
                        Icon(
                          _query.isEmpty
                              ? Icons.account_balance_wallet_outlined
                              : Icons.search_off,
                          size: 64,
                          color: cs.onSurface.withAlpha(100),
                        ),
                        const Gap(16),
                        Text(
                          _query.isEmpty
                              ? "No accounts available"
                              : "No accounts matching \"$_query\"",
                          style: tt.bodyLarge,
                        ),
                      ],
                    ),
                  )
                : ListView.builder(
                    itemCount: filtered.length,
                    itemBuilder: (context, index) {
                      final ra = filtered[index];
                      final isSelected = _selected.contains(ra);
                      final initialsText = ra.name.length > 1
                          ? ra.name.substring(0, 2).toUpperCase()
                          : ra.name.toUpperCase();

                      return ListTile(
                        leading: CircleAvatar(
                          backgroundColor: isSelected
                              ? Colors.blue.shade700
                              : cs.primaryContainer,
                          child: isSelected
                              ? const Icon(Icons.check, color: Colors.white)
                              : Text(
                                  initialsText,
                                  style: TextStyle(
                                      color: cs.onPrimaryContainer),
                                ),
                        ),
                        title: Text(ra.name,
                            overflow: TextOverflow.ellipsis),
                        subtitle: Text(
                          "Birth height: ${ra.birthHeight}",
                          style: tt.bodySmall,
                        ),
                        onTap: () {
                          setState(() {
                            if (isSelected) {
                              _selected.remove(ra);
                            } else {
                              _selected.add(ra);
                            }
                          });
                        },
                      );
                    },
                  ),
          ),
        ],
      ),
      bottomNavigationBar: _selected.isNotEmpty
          ? SafeArea(
              child: Padding(
                padding: const EdgeInsets.all(16),
                child: FilledButton.icon(
                  icon: const Icon(Icons.download),
                  label: Text("Restore ${_selected.length} account(s)"),
                  onPressed: () {
                    Navigator.of(context).pop(_selected.toList());
                  },
                ),
              ),
            )
          : null,
    );
  }
}
