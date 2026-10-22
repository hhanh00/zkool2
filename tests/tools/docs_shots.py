"""Draws the annotations of a docs screenshot run and copies the images into
the docs site.

    docs_shots.py <frames_dir> --out <docs dir>

`<frames_dir>/index.json` is written by `integration_test/docs_shots_support.dart`:
every shot names its image, its pixel ratio and the annotations the docs text
refers to, as logical rectangles of the widgets they point at. Three kinds:
`box` (a rectangle around a control), `round` (a rounded outline around a
region) and `badge` (a numbered disc at a point). Each has a color, red
unless the page separates regions by color. The annotated image lands at
`<docs dir>/<shot name>.png`, e.g. `guide/images/02.new_account.png`.
"""

import argparse
import json
import os

from PIL import Image, ImageDraw, ImageFont

COLORS = {"red": (229, 57, 53), "green": (67, 160, 71), "blue": (30, 136, 229)}
# In output pixels (the frames are rasterized at 2x).
STROKE = 6
PAD = 6
RADIUS = 10
ROUND_RADIUS = 36
BADGE = 24
FONT = "/System/Library/Fonts/Helvetica.ttc"


def badge_font(size: int) -> ImageFont.ImageFont:
    for index in (1, 0):  # the ttc's bold face, then whatever comes first
        try:
            return ImageFont.truetype(FONT, size, index=index)
        except OSError:
            continue
    return ImageFont.load_default(size)


def draw_annot(draw: ImageDraw.ImageDraw, annot: dict, ratio: float, font: ImageFont.ImageFont):
    color = COLORS[annot.get("color", "red")]
    left, top, width, height = (v * ratio for v in annot["rect"])
    mark = annot["mark"]
    if mark in ("box", "round"):
        radius = RADIUS if mark == "box" else ROUND_RADIUS
        # A full-width widget would put the sides of its box past the edges
        # of the image; keep the whole outline visible.
        edge = STROKE // 2 + 1
        x0 = max(left - PAD, edge)
        y0 = max(top - PAD, edge)
        x1 = min(left + width + PAD, draw.im.size[0] - edge)
        y1 = min(top + height + PAD, draw.im.size[1] - edge)
        draw.rounded_rectangle([x0, y0, x1, y1], radius=radius, outline=color, width=STROKE)
    elif mark == "badge":
        cx, cy = (v * ratio for v in annot["at"])
        draw.ellipse([cx - BADGE, cy - BADGE, cx + BADGE, cy + BADGE], fill=color)
        draw.text((cx, cy), str(annot["n"]), fill="white", font=font, anchor="mm")
    else:
        raise ValueError(f"unknown annotation {mark}")


def main():
    parser = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    parser.add_argument("frames_dir")
    parser.add_argument("--out", required=True, help="the docs directory to copy the images into")
    args = parser.parse_args()

    with open(os.path.join(args.frames_dir, "index.json")) as f:
        index = json.load(f)
    font = badge_font(int(BADGE * 1.2))
    for shot in index["shots"]:
        image = Image.open(os.path.join(args.frames_dir, shot["file"])).convert("RGB")
        draw = ImageDraw.Draw(image)
        for annot in shot["annots"]:
            draw_annot(draw, annot, shot["ratio"], font)
        out = os.path.join(args.out, shot["file"])
        os.makedirs(os.path.dirname(out), exist_ok=True)
        image.save(out)
        print(f"{out} ({len(shot['annots'])} annotations)")


if __name__ == "__main__":
    main()
