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
  @override
  Widget build(BuildContext context) {
    final t = Theme.of(context);
    final pluginsAsync = ref.watch(pluginListProvider);

    return Scaffold(
      appBar: AppBar(
        title: const Text('Plugin Manager'),
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
              return _PluginListTile(plugin: plugin);
            },
          );
        },
        loading: () => blank(context),
        error: (error, _) => showError(error),
      ),
      floatingActionButton: FloatingActionButton(
        onPressed: () => _showInstallDialog(context),
        tooltip: 'Install Plugin',
        child: const Icon(Icons.add),
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
}

class _PluginListTile extends ConsumerWidget {
  final api.PluginInfo plugin;
  const _PluginListTile({required this.plugin});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final t = Theme.of(context);

    final prefixLabel = _prefixAscii(plugin.memoPrefixes);

    return ListTile(
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
      leading: prefixLabel != null
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
      trailing: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          Switch(
            value: plugin.enabled,
            onChanged: (enabled) async {
              final c = coinContext.coin;
              api.setPluginEnabled(id: plugin.id, enabled: enabled, c: c);
              ref.invalidate(pluginListProvider);
            },
          ),
          IconButton(
            icon: const Icon(Icons.delete_outline),
            tooltip: 'Remove plugin',
            onPressed: () => _confirmRemove(context, ref),
          ),
        ],
      ),
    );
  }

  Future<void> _confirmRemove(BuildContext context, WidgetRef ref) async {
    final confirmed = await confirmDialog(
      context,
      title: 'Remove Plugin',
      message: 'Remove "${plugin.name}"? This cannot be undone.',
    );
    if (!confirmed) return;

    try {
      final c = coinContext.coin;
      await api.removePlugin(id: plugin.id, c: c);
      ref.invalidate(pluginListProvider);
    } catch (e) {
      showSnackbar('Failed to remove plugin: $e');
    }
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
