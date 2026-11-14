import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated.dart';
import 'package:gap/gap.dart';
import 'package:zkool/main.dart';
import 'package:zkool/src/rust/api/account.dart';
import 'package:zkool/store.dart';
import 'package:zkool/utils.dart';
import 'package:zkool/widgets/error_display.dart';

/// Every key of an account as a tree: the key that created the account at
/// the root, what derives from it below. A switch flips every node between
/// its secret face and its viewing face; a face a node does not have reads
/// N/A, so the tree keeps its shape and what the user expanded.
///
/// Reached from the backup block of the Viewing Keys page, behind its
/// authentication: this page asks for none of its own.
class KeyTreePage extends ConsumerStatefulWidget {
  final int account;
  const KeyTreePage(this.account, {super.key});

  @override
  ConsumerState<KeyTreePage> createState() => _KeyTreePageState();
}

/// A node on screen: its place in the tree and how deep it sits.
class _Row {
  final KeyNode node;
  final String id;
  final int depth;
  const _Row(this.node, this.id, this.depth);
}

class _KeyTreePageState extends ConsumerState<KeyTreePage> {
  late final c = coinContext.coin;
  KeyNode? root;
  // The viewing face is shown. A viewing-only account has nothing else to
  // show and opens here; the others open on their secrets.
  bool viewing = false;
  // The open nodes, by their path of child indices from the root ("0",
  // "0/1", ...). The faces are picked at render time, so flipping the mode
  // keeps this set and what the user opened.
  final expanded = <String>{_rootId};
  static const _rootId = "0";

  @override
  void initState() {
    super.initState();
    Future(() async {
      try {
        final r = await getAccountKeyTree(account: widget.account, c: c);
        if (!mounted) return;
        setState(() {
          root = r;
          viewing = r?.secret.value == null;
        });
      } on AnyhowException catch (e) {
        if (mounted) await showException(context, e.message);
      }
    });
  }

  /// The visible nodes in order: a node, then its children while it is
  /// open. The root opens one level; the lists below it, the addresses,
  /// start closed.
  List<_Row> _visible() {
    final rows = <_Row>[];
    void walk(KeyNode n, String id, int depth) {
      rows.add(_Row(n, id, depth));
      if (!expanded.contains(id)) return;
      for (final (i, child) in n.children.indexed) {
        walk(child, "$id/$i", depth + 1);
      }
    }

    if (root != null) walk(root!, _rootId, 0);
    return rows;
  }

  @override
  Widget build(BuildContext context) {
    final pinlock = ref.watch(lifecycleProvider);
    if (pinlock.value ?? false) return PinLock();

    final hasSecret = root?.secret.value != null;
    final rows = _visible();
    return Scaffold(
      appBar: AppBar(
        title: Text("Key Tree"),
        actions: [
          if (hasSecret)
            Padding(
              padding: EdgeInsets.only(right: 8),
              child: Row(children: [
                Text("Viewing keys"),
                Switch(value: viewing, onChanged: (v) => setState(() => viewing = v)),
              ]),
            ),
        ],
      ),
      body: root == null
          ? SizedBox.shrink()
          : CustomScrollView(
              slivers: [
                SliverPadding(
                  padding: EdgeInsets.fromLTRB(16, 8, 16, 32),
                  // Each row is as tall as its texts: a key wraps to as many
                  // lines as it needs.
                  sliver: SliverList.builder(
                    itemCount: rows.length,
                    itemBuilder: (context, i) => _row(rows[i]),
                  ),
                ),
              ],
            ),
    );
  }

  static const _toggleWidth = 32.0;
  static const _indent = 10.0; // per level
  static const _rowPadding = 12.0;

  KeyFace _face(KeyNode n) => viewing ? n.viewing : n.secret;

  void _toggle(String id) => setState(() => expanded.contains(id) ? expanded.remove(id) : expanded.add(id));

  Widget _row(_Row row) {
    final n = row.node;
    final face = _face(n);
    final t = Theme.of(context).textTheme;
    final isOpen = expanded.contains(row.id);
    final toggle = n.children.isNotEmpty
        ? InkWell(
            key: ValueKey("toggle ${row.id}"),
            onTap: () => _toggle(row.id),
            child: Icon(isOpen ? Icons.expand_more : Icons.chevron_right),
          )
        : SizedBox.shrink();
    // The indent is part of the row, so the texts get the width that is
    // left and stay inside the list.
    return Padding(
      key: ValueKey(row.id),
      padding: EdgeInsets.only(left: _indent * row.depth, top: _rowPadding / 2, bottom: _rowPadding / 2),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          SizedBox(width: _toggleWidth, child: toggle),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(n.name, style: t.titleMedium),
                Text(face.label, style: t.labelSmall),
                if (face.value != null)
                  CopyableText(face.value!, style: t.bodyMedium)
                else
                  Text("N/A", style: t.bodyMedium?.copyWith(fontStyle: FontStyle.italic)),
                if (n.detail != null) Text(n.detail!, style: t.bodySmall),
                if (face.note != null)
                  Row(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Icon(viewing ? Icons.info_outline : Icons.warning_amber, size: 16),
                      Gap(4),
                      Expanded(child: Text(face.note!, style: t.bodySmall)),
                    ],
                  ),
              ],
            ),
          ),
        ],
      ),
    );
  }
}
