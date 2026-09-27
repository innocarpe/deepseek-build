"""Recolor a screenshot's terminal region with a candidate theme palette.

Part of the theme preview toolchain (see
`docs/product/THEME_CLASSIC_READABILITY_2026-09-27.md` for the workflow, the
limits, and the invariant checks). Dependencies: numpy, Pillow.

Usage:
    python3 remap.py --src phone.png \
        --old palettes/current-classic.json --new palettes/E1.json \
        --out preview.webp [--crop 432:2532]

Each pixel inside `--crop` is decomposed as `background + alpha * (glyph -
background)` against the old palette and recomposed with the new one, so
antialiased glyph edges follow. Backgrounds are the exact ramp colors, forward/
backward-filled along a row; a pixel whose residue stays above the threshold is
shifted by the background delta instead. `remap2.py` additionally resolves
backgrounds that are a blend of two ramp colors.

Palette JSON maps the classic ramp constant names (`BG_STORM`, `FG_DARK`, ...)
to `[r, g, b]`, as in `docs/product/evidence/theme-classic-2026-09-27/palettes/`.
"""

import argparse
import json

import numpy as np
from PIL import Image

# Background roles; the row-wise fill picks the nearest registered background.
BG_KEYS = [
    "BG_STORM_DARK",
    "BG_STORM",
    "BG_SURFACE",
    "MD_CODE_BG",
    "BG_HIGHLIGHT",
    "BG_HOVER",
    "BG_VISUAL",
]

# Shared accents — identical in every DeepSeek Night skin, so a candidate
# palette never moves them; they only need to be resolvable as glyph colors.
ACCENTS = [
    (77, 107, 254),
    (110, 140, 255),
    (50, 72, 190),
    (90, 160, 220),
    (120, 210, 160),
    (100, 200, 180),
    (250, 120, 140),
    (230, 190, 110),
    (255, 170, 110),
    (120, 210, 240),
    (180, 160, 250),
    (60, 190, 180),
    (160, 140, 230),
    (255, 219, 141),
    (139, 195, 74),
    (255, 255, 255),
]

UNEXPLAINED_RESIDUAL = 14.0


def load_palettes(old_path, new_path):
    with open(old_path) as f:
        old = {k: tuple(v) for k, v in json.load(f).items()}
    with open(new_path) as f:
        new = {k: tuple(v) for k, v in json.load(f).items()}
    missing = sorted(k for k in old if k not in new)
    if missing:
        raise SystemExit(f"{new_path} is missing ramp keys: {', '.join(missing)}")
    return old, new


def parse_crop(spec, height):
    """`y0:y1` rows of the source, or the whole image when omitted."""
    if not spec:
        return 0, height
    y0, y1 = (int(v) for v in spec.split(":"))
    return y0, y1


def save_image(array, path):
    """Write a uint8 RGB array as PNG, or a lossless WebP for a `.webp` path."""
    im = Image.fromarray(array)
    if path.lower().endswith(".webp"):
        im.save(path, lossless=True, method=6)
    else:
        im.save(path)


def fill_backgrounds(reg, bgcols, newbg):
    """Nearest registered background per pixel (row-wise fill), and its new color."""
    h, w, _ = reg.shape
    idx = np.full((h, w), -1, np.int32)
    for i, c in enumerate(bgcols):
        idx[np.all(reg == c, axis=2)] = i
    isbg = idx >= 0
    pos = np.where(isbg, np.arange(w)[None, :], -1)
    left = np.maximum.accumulate(pos, axis=1)
    pos2 = np.where(isbg, np.arange(w)[None, :], w)
    right = np.minimum.accumulate(pos2[:, ::-1], axis=1)[:, ::-1]
    rows = np.arange(h)[:, None]
    li = np.where(left >= 0, idx[rows, np.clip(left, 0, w - 1)], -1)
    ri = np.where(right < w, idx[rows, np.clip(right, 0, w - 1)], -1)
    bidx = np.where(li >= 0, li, ri)
    bidx = np.where(bidx < 0, 1, bidx)
    return bgcols[bidx], newbg[bidx], isbg


def region_backgrounds(reg, old, new):
    """Per-pixel old/new background colors, from the exact ramp colors."""
    bgcols = np.array([old[k] for k in BG_KEYS], np.float32)
    newbg = np.array([new[k] for k in BG_KEYS], np.float32)
    return fill_backgrounds(reg, bgcols, newbg)


def glyph_candidates(old, new):
    """Glyph colors to trial per pixel: every ramp slot plus the shared accents."""
    F = np.array([old[k] for k in old] + ACCENTS, np.float32)
    F2 = np.array([new[k] for k in old] + ACCENTS, np.float32)
    return F, F2


def decompose(reg, B, F):
    """Best (color slot, alpha) per pixel; the caller recomposes on the new ramp."""
    best = np.full(reg.shape[:2], 1e9, np.float32)
    ba = np.zeros(reg.shape[:2], np.float32)
    bi = np.zeros(reg.shape[:2], np.int32)
    for j in range(len(F)):
        d = F[j][None, None, :] - B
        dd = (d * d).sum(2)
        dd = np.where(dd < 1, 1, dd)
        a = np.clip(((reg - B) * d).sum(2) / dd, 0, 1)
        res = np.linalg.norm(reg - (B + a[..., None] * d), axis=2)
        # A candidate equal to the background explains nothing.
        res = np.where(np.all(np.abs(d) < 1, axis=2), 1e9, res)
        m = res < best
        best[m] = res[m]
        ba[m] = a[m]
        bi[m] = j
    return best, ba, bi


def remap_image(src_path, old, new, out_path, crop=None):
    im = np.array(Image.open(src_path).convert("RGB")).astype(np.float32)
    y0, y1 = parse_crop(crop, im.shape[0])
    reg = im[y0:y1].copy()
    h, w, _ = reg.shape
    B, B2, isbg = region_backgrounds(reg, old, new)
    F, F2 = glyph_candidates(old, new)
    best, ba, bi = decompose(reg, B, F)
    out = B2 + ba[..., None] * (F2[bi] - B2)
    unexplained = best > UNEXPLAINED_RESIDUAL
    out = np.where(unexplained[..., None], reg + (B2 - B), out)
    out = np.where(isbg[..., None], B2, out)
    im[y0:y1] = out
    save_image(np.clip(im, 0, 255).astype(np.uint8), out_path)
    print(f"{out_path}: unexplained {int(unexplained.sum())} of {h * w} region pixels")


def main():
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--src", required=True, help="source screenshot (PNG/WebP)")
    ap.add_argument("--old", required=True, help="current palette JSON")
    ap.add_argument("--new", required=True, help="candidate palette JSON")
    ap.add_argument("--out", required=True, help="output path (.png or lossless .webp)")
    ap.add_argument("--crop", help="terminal region rows, `y0:y1` (default: whole image)")
    args = ap.parse_args()
    old, new = load_palettes(args.old, args.new)
    remap_image(args.src, old, new, args.out, args.crop)


if __name__ == "__main__":
    main()
