import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart' show SliverLayoutDimensions;
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

class _KeyTreePageState extends ConsumerState<KeyTreePage> {
  late final c = coinContext.coin;
  KeyNode? root;
  // The viewing face is shown. A viewing-only account has nothing else to
  // show and opens here; the others open on their secrets.
  bool viewing = false;
  final controller = TreeSliverController();
  // Built once: the faces are picked at render time, so flipping the mode
  // changes no node and keeps the expansion state the nodes hold.
  List<TreeSliverNode<KeyNode>> tree = [];

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
          tree = r == null ? [] : [_node(r, 0)];
        });
      } on AnyhowException catch (e) {
        if (mounted) await showException(context, e.message);
      }
    });
  }

  // The root opens one level; lists below it, the addresses, start closed.
  TreeSliverNode<KeyNode> _node(KeyNode n, int depth) =>
      TreeSliverNode(n, children: n.children.map((c) => _node(c, depth + 1)).toList(), expanded: depth == 0);

  @override
  Widget build(BuildContext context) {
    final pinlock = ref.watch(lifecycleProvider);
    if (pinlock.value ?? false) return PinLock();

    final hasSecret = root?.secret.value != null;
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
                  sliver: TreeSliver<KeyNode>(
                    tree: tree,
                    controller: controller,
                    treeNodeBuilder: _row,
                    treeRowExtentBuilder: _extent,
                  ),
                ),
              ],
            ),
    );
  }

  static const _toggleWidth = 32.0;
  static const _indent = 10.0; // TreeSliverIndentationType.standard, per level
  static const _rowPadding = 12.0;

  KeyFace _face(KeyNode n) => viewing ? n.viewing : n.secret;

  Widget _row(BuildContext context, TreeSliverNode<Object?> node, AnimationStyle style) {
    final n = node.content as KeyNode;
    final face = _face(n);
    final t = Theme.of(context).textTheme;
    final toggle = node.children.isNotEmpty
        ? TreeSliver.wrapChildToToggleNode(
            node: node,
            child: Icon(node.isExpanded ? Icons.expand_more : Icons.chevron_right),
          )
        : SizedBox.shrink();
    return Padding(
      padding: EdgeInsets.symmetric(vertical: _rowPadding / 2),
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

  /// A tree sliver lays its rows out at fixed extents, so each row's height
  /// is measured from its texts at the width the row will get.
  double _extent(TreeSliverNode<Object?> node, SliverLayoutDimensions dimensions) {
    final n = node.content as KeyNode;
    final face = _face(n);
    final t = Theme.of(context).textTheme;
    final width = dimensions.crossAxisExtent - _toggleWidth - _indent * (node.depth ?? 0);
    double h = _rowPadding;
    h += _measure(n.name, t.titleMedium, width);
    h += _measure(face.label, t.labelSmall, width);
    h += _measure(face.value ?? "N/A", t.bodyMedium, width);
    if (n.detail != null) h += _measure(n.detail!, t.bodySmall, width);
    if (face.note != null) h += _measure(face.note!, t.bodySmall, width - 20);
    return h;
  }

  double _measure(String text, TextStyle? style, double width) {
    final painter = TextPainter(
      text: TextSpan(text: text, style: style),
      textDirection: TextDirection.ltr,
      textScaler: MediaQuery.textScalerOf(context),
    )..layout(maxWidth: width.clamp(1, double.infinity));
    final h = painter.height;
    painter.dispose();
    return h;
  }
}
