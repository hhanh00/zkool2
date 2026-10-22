/// Shared scaffolding for the docs screenshot runs
/// (`docs_guide_ui_test.dart`, ...).
///
/// A docs run walks the real app through the flows a section of the docs
/// site describes and takes a named screenshot of each screen the section
/// shows, with the annotations its text refers to: a box around a button,
/// a rounded outline around a region, or a numbered badge next to a field.
/// The frames come from the app itself: the widget tree sits in a
/// [RepaintBoundary] that is rasterized to PNG, so no screen-recording
/// permission is needed and the window may be covered. The annotations are
/// recorded as logical rectangles, taken from the widgets' real bounds, and
/// `tests/tools/docs_shots.py` draws them on the frames and copies the
/// result into the docs site. `<frames_dir>/index.json` in the documents
/// directory lists the shots and their annotations.
library;

import 'dart:convert';
import 'dart:io';
import 'dart:ui' as ui;

import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter_test/flutter_test.dart';

import 'support.dart';

/// The pixel ratio of the frames: a 450x800 window gives 900x1600 PNGs.
const shotRatio = 2;

enum Mark { box, round, badge, blank }

/// One annotation on a shot, in logical pixels of the app window.
class Annot {
  final Mark mark;
  final Rect rect;
  final String color;

  /// The badge number, for [Mark.badge].
  final int? n;

  /// Where a badge sits, for [Mark.badge].
  final Offset? at;

  const Annot(this.mark, this.rect, this.color, {this.n, this.at});

  Map<String, dynamic> toJson() => {
        "mark": mark.name,
        "rect": [rect.left, rect.top, rect.width, rect.height],
        "color": color,
        if (n != null) "n": n,
        if (at != null) "at": [at!.dx, at!.dy],
      };
}

/// Takes the named shots of a docs run and writes their index.
class Shots {
  final WidgetTester tester;
  final GlobalKey boundary = GlobalKey();
  final Directory dir;
  final List<Map<String, dynamic>> shots = [];

  Shots(this.tester, Directory docsDir, Map<String, dynamic> config)
      : dir = Directory("${docsDir.path}/${config["frames_dir"] ?? "docs_frames"}") {
    if (dir.existsSync()) dir.deleteSync(recursive: true);
    dir.createSync(recursive: true);
  }

  // ── Annotations ───────────────────────────────────────────────────────────
  // The boundary covers the whole window, so a widget's global rect is its
  // rect in the frame. Each takes the first match of its finder.

  Rect rectOf(Finder finder) {
    if (finder.evaluate().isEmpty) fail("nothing on screen matches $finder; visible: ${visibleTexts().join(" | ")}");
    return tester.getRect(finder.first);
  }

  Rect unionOf(Iterable<Finder> finders) =>
      finders.map(rectOf).reduce((a, b) => a.expandToInclude(b));

  /// A box around the widget [finder] matches.
  Annot box(Finder finder, {String color = "red"}) => Annot(Mark.box, rectOf(finder), color);

  /// The rectangle of the glyphs of the text [finder] matches: a text laid
  /// out full width in its row has a box far wider than what it shows.
  Rect glyphsOf(Finder finder) {
    final paragraph = tester.renderObject<RenderParagraph>(
      find.descendant(of: finder.first, matching: find.byType(RichText)).first,
    );
    final width = paragraph.getMaxIntrinsicWidth(double.infinity).clamp(0.0, paragraph.size.width);
    return paragraph.localToGlobal(Offset.zero) & Size(width, paragraph.size.height);
  }

  /// A box around the glyphs of the text [finder] matches.
  Annot textBox(Finder finder, {String color = "red"}) => Annot(Mark.box, glyphsOf(finder), color);

  /// A box around everything [finders] match.
  Annot boxAround(Iterable<Finder> finders, {String color = "red"}) =>
      Annot(Mark.box, unionOf(finders), color);

  /// A rounded outline around the widget [finder] matches.
  Annot round(Finder finder, {String color = "red"}) => Annot(Mark.round, rectOf(finder), color);

  /// A rounded outline around everything [finders] match.
  Annot roundAround(Iterable<Finder> finders, {String color = "red"}) =>
      Annot(Mark.round, unionOf(finders), color);

  /// Badge [n] next to the widget [finder] matches: inside its right edge by
  /// default, or at [align] within its rect.
  Annot badge(Finder finder, int n, {String color = "red", Alignment align = const Alignment(0.9, 0)}) {
    final r = rectOf(finder);
    return Annot(Mark.badge, r, color, n: n, at: align.withinRect(r));
  }

  /// Badge [n] at [at].
  Annot badgeAt(Offset at, int n, {String color = "red"}) =>
      Annot(Mark.badge, Rect.fromCenter(center: at, width: 0, height: 0), color, n: n, at: at);

  /// Badge [n] just after the glyphs of the text [finder] matches: a text
  /// that fills its row has its box far wider than what it shows.
  Annot badgeAfterText(Finder finder, int n, {String color = "red"}) =>
      badgeAt(glyphsOf(finder).centerRight + const Offset(26, 0), n, color: color);

  /// Blanks out the widgets [finder] matches, for text the docs must not
  /// show, such as a path on this machine. A widget scrolled out of the
  /// window is left alone, and the blanks are clipped to [within] when
  /// given: a list's rows half under its app bar would blank the bar too.
  List<Annot> blank(Finder finder, {Finder? within}) {
    final window = Offset.zero & (boundary.currentContext!.findRenderObject() as RenderBox).size;
    final clip = within == null ? window : rectOf(within).intersect(window);
    return [
      for (final e in finder.evaluate())
        if (tester.getRect(find.byWidget(e.widget)).overlaps(clip))
          Annot(Mark.blank, tester.getRect(find.byWidget(e.widget)).intersect(clip), "none"),
    ];
  }

  // ── Shots ─────────────────────────────────────────────────────────────────

  /// Rasterizes the window into `<name>.png` under the frames directory,
  /// after [settle] of pumping, and records the annotations [annots] builds
  /// for it. They are built after the settle, so they measure the frame as
  /// it is shot, not the layout of a moment before. [name] is the path of
  /// the image in the docs site, without the extension.
  Future<void> take(
    String name, {
    List<Annot> Function()? annots,
    Duration settle = const Duration(milliseconds: 500),
  }) async {
    await pumpFor(tester, settle);
    final marks = annots?.call() ?? const <Annot>[];
    await tester.runAsync(() async {
      final render = boundary.currentContext?.findRenderObject();
      if (render is! RenderRepaintBoundary) fail("no repaint boundary to shoot $name");
      final image = await render.toImage(pixelRatio: shotRatio.toDouble());
      final bytes = await image.toByteData(format: ui.ImageByteFormat.png);
      image.dispose();
      if (bytes == null) fail("could not encode the shot $name");
      final file = File("${dir.path}/$name.png");
      file.parent.createSync(recursive: true);
      file.writeAsBytesSync(bytes.buffer.asUint8List());
    });
    shots.add({
      "name": name,
      "file": "$name.png",
      "ratio": shotRatio,
      "annots": marks.map((a) => a.toJson()).toList(),
    });
    debugPrint("[docs-shots] $name (${marks.length} annotations)");
    // After every shot, so a run that fails midway still leaves a usable
    // index for the shots it took.
    writeIndex();
  }

  void writeIndex() {
    final size = (boundary.currentContext?.findRenderObject() as RenderBox?)?.size;
    File("${dir.path}/index.json").writeAsStringSync(jsonEncode({
      "shots": shots,
      if (size != null) "size": [size.width, size.height],
    }));
  }

  /// Ends the run: logs the count, for the orchestrator's log.
  void finish() {
    writeIndex();
    debugPrint("[docs-shots] ${shots.length} shots in ${dir.path}");
  }
}
