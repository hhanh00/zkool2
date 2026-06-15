import 'package:convert/convert.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:gap/gap.dart';
import '../src/rust/api/plugin.dart' as api;
import '../store.dart';
import '../utils.dart';

/// Plugin manager page accessible from Settings.
class PluginManagerPage extends ConsumerStatefulWidget {
  const PluginManagerPage({super.key});

  @override
  ConsumerState<PluginManagerPage> createState() => _PluginManagerPageState();
}

class _PluginManagerPageState extends ConsumerState<PluginManagerPage> {
  final Set<String> _selectedIds = {};

  @override
  Widget build(BuildContext context) {
    final t = Theme.of(context);
    final pluginsAsync = ref.watch(pluginListProvider);

    return Scaffold(
      appBar: AppBar(
        title: Text(
          _selectedIds.isEmpty
              ? 'Plugin Manager'
              : '${_selectedIds.length} selected',
        ),
        actions: [
          if (_selectedIds.isEmpty)
            IconButton(
              onPressed: () => _showInstallDialog(context),
              icon: const Icon(Icons.add),
              tooltip: 'Install Plugin',
            ),
          if (_selectedIds.isNotEmpty)
            IconButton(
              onPressed: () => _confirmRemoveSelected(context),
              icon: const Icon(Icons.delete),
              tooltip: 'Remove selected plugins',
            ),
        ],
      ),
      body: pluginsAsync.when(
        data: (plugins) {
          if (plugins.isEmpty) {
            return Center(
              child: Padding(
                padding: const EdgeInsets.all(32),
                child: Column(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    Icon(Icons.extension_off, size: 64, color: t.disabledColor),
                    const Gap(16),
                    Text('No plugins installed', style: t.textTheme.titleMedium),
                  ],
                ),
              ),
            );
          }
          return ListView.separated(
            itemCount: plugins.length,
            separatorBuilder: (_, __) => const Divider(height: 1),
            itemBuilder: (context, index) {
              final plugin = plugins[index];
              final selected = _selectedIds.contains(plugin.id);
              return _PluginListTile(
                plugin: plugin,
                selected: selected,
                onAvatarTap: () {
                  setState(() {
                    if (selected) {
                      _selectedIds.remove(plugin.id);
                    } else {
                      _selectedIds.add(plugin.id);
                    }
                  });
                },
              );
            },
          );
        },
        loading: () => blank(context),
        error: (error, _) => showError(error),
      ),
    );
  }

  Future<void> _showInstallDialog(BuildContext context) async {
    if (!mounted) return;
    final confirmed = await confirmDialog(
      context,
      title: 'Install Plugin',
      message: 'Installing plugins is at your own risk. Only install plugins from trusted sources.',
    );
    if (!confirmed || !mounted) return;

    final url = await inputText(context, title: 'Plugin URL');
    if (url == null || url.isEmpty || !mounted) return;

    try {
      final c = coinContext.coin;
      await api.installPlugin(url: url, c: c);
      ref.invalidate(pluginListProvider);
      if (mounted) showSnackbar('Plugin installed successfully');
    } catch (e) {
      if (mounted) showSnackbar('Failed to install plugin: $e');
    }
  }

  Future<void> _confirmRemoveSelected(BuildContext context) async {
    final count = _selectedIds.length;
    final confirmed = await confirmDialog(
      context,
      title: 'Remove Plugins',
      message:
          'Remove $count selected plugin${count > 1 ? 's' : ''}? This cannot be undone.',
    );
    if (!confirmed) return;

    final c = coinContext.coin;
    var failed = 0;
    for (final id in _selectedIds.toList()) {
      try {
        await api.removePlugin(id: id, c: c);
      } catch (_) {
        failed++;
      }
    }

    setState(() => _selectedIds.clear());
    ref.invalidate(pluginListProvider);

    if (!mounted) return;
    if (failed == 0) {
      showSnackbar('$count plugin${count > 1 ? 's' : ''} removed');
    } else {
      showSnackbar('${count - failed} removed, $failed failed');
    }
  }
}

class _PluginListTile extends ConsumerWidget {
  final api.PluginInfo plugin;
  final bool selected;
  final VoidCallback onAvatarTap;

  const _PluginListTile({
    required this.plugin,
    required this.selected,
    required this.onAvatarTap,
  });

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final t = Theme.of(context);

    final prefixLabel = _prefixAscii(plugin.memoPrefixes);

    return ListTile(
      selected: selected,
      title: Text(plugin.name),
      subtitle: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          if (plugin.author != null) Text(plugin.author!),
          Text('v${plugin.version}'),
          if (plugin.memoPrefixes.length > 1)
            Text(
              plugin.memoPrefixes
                  .skip(1)
                  .map((p) => _prefixAscii([p]))
                  .join(', '),
              style: TextStyle(
                fontSize: 12,
                fontFamily: 'monospace',
                color: t.colorScheme.outline,
              ),
            ),
        ],
      ),
      leading: GestureDetector(
        onTap: onAvatarTap,
        child: selected
            ? CircleAvatar(
                backgroundColor: t.colorScheme.primary,
                child: Icon(Icons.check, color: t.colorScheme.onPrimary),
              )
            : prefixLabel != null
                ? CircleAvatar(
                    backgroundColor: plugin.enabled
                        ? t.colorScheme.primaryContainer
                        : t.disabledColor.withAlpha(40),
                    child: Text(
                      prefixLabel,
                      style: TextStyle(
                        fontSize: 11,
                        fontWeight: FontWeight.bold,
                        fontFamily: 'monospace',
                        color: plugin.enabled
                            ? t.colorScheme.onPrimaryContainer
                            : t.disabledColor,
                      ),
                    ),
                  )
                : Icon(
                    plugin.enabled ? Icons.extension : Icons.extension_off,
                    color: plugin.enabled
                        ? t.colorScheme.primary
                        : t.disabledColor,
                  ),
      ),
      trailing: Switch(
        value: plugin.enabled,
        onChanged: (enabled) async {
          final c = coinContext.coin;
          await api.setPluginEnabled(id: plugin.id, enabled: enabled, c: c);
          ref.invalidate(pluginListProvider);
        },
      ),
    );
  }
}

/// Decode hex prefixes to ASCII (e.g. "444b3030" → "DK00").
String? _prefixAscii(List<String> prefixes) {
  if (prefixes.isEmpty) return null;
  try {
    final bytes = hex.decode(prefixes.first);
    final ascii = String.fromCharCodes(bytes);
    return ascii.length > 4 ? ascii.substring(0, 4) : ascii;
  } catch (_) {
    return prefixes.first.length > 4
        ? prefixes.first.substring(0, 4)
        : prefixes.first;
  }
}
