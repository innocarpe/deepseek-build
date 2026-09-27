"""Blend-aware recolor of a screenshot's terminal region with a candidate palette.

Same toolchain as `remap.py` (see
`docs/product/THEME_CLASSIC_READABILITY_2026-09-27.md`); this pass additionally
resolves background colors that are a `t`-blend of two ramp colors — animated or
staged surfaces — and reprojects them at the same `t` on the new ramp. Host
chrome colors (Orca's own edges inside the screenshot) stay untouched.
Dependencies: numpy, Pillow.

Usage:
    python3 remap2.py --src phone.png \
        --old palettes/current-classic.json --new palettes/E1.json \
        --out preview.webp [--crop 432:2532]
"""

import argparse
from collections import Counter

import numpy as np
from PIL import Image

from remap import (
    BG_KEYS,
    UNEXPLAINED_RESIDUAL,
    decompose,
    fill_backgrounds,
    glyph_candidates,
    load_palettes,
    parse_crop,
    save_image,
)

# A blended background qualifies when it owns at least this many pixels among
# the 200 most common colors, is not an exact ramp color, and is dark.
BLEND_PIXELS = 600
BLEND_MAX_SUM = 200
BLEND_RESIDUAL = 1.0

# Orca paints its own chrome inside the captured screen (window edges and the
# strip around the terminal); those pixels are the host's, not the theme's.
HOST_CHROME = [(26, 26, 26), (42, 42, 42), (40, 44, 52)]


def detect_blend_bgs(reg, old):
    """Most common colors that are `A + t * (B - A)` for two ramp colors A, B."""
    counts = Counter(map(tuple, reg.reshape(-1, 3)))
    known = set(old.values())
    out = []
    for p, n in counts.most_common(200):
        p = tuple(int(x) for x in p)
        if n < BLEND_PIXELS or p in known or sum(p) > BLEND_MAX_SUM:
            continue
        best = None
        for i, a in enumerate(BG_KEYS):
            for b in BG_KEYS[i + 1 :]:
                A = np.array(old[a], float)
                B = np.array(old[b], float)
                P = np.array(p, float)
                d = B - A
                t = np.clip(np.dot(P - A, d) / max(np.dot(d, d), 1), 0, 1)
                res = np.linalg.norm(P - (A + t * d))
                if best is None or res < best[0]:
                    best = (res, a, b, t)
        if best[0] <= BLEND_RESIDUAL:
            out.append((p, best[1], best[2], best[3]))
    return out


def remap_image(src_path, old, new, out_path, crop=None):
    im = np.array(Image.open(src_path).convert("RGB")).astype(np.float32)
    y0, y1 = parse_crop(crop, im.shape[0])
    reg = im[y0:y1].copy()
    h, w, _ = reg.shape
    blends = detect_blend_bgs(reg.astype(np.uint8), old)
    bgcols = np.array([old[k] for k in BG_KEYS] + [p for p, _, _, _ in blends], np.float32)
    newbg = np.array(
        [new[k] for k in BG_KEYS]
        + [
            tuple(np.array(new[a]) + t * (np.array(new[b]) - np.array(new[a])))
            for _, a, b, t in blends
        ],
        np.float32,
    )
    B, B2, isbg = fill_backgrounds(reg, bgcols, newbg)
    F, F2 = glyph_candidates(old, new)
    best, ba, bi = decompose(reg, B, F)
    out = B2 + ba[..., None] * (F2[bi] - B2)
    unexplained = best > UNEXPLAINED_RESIDUAL
    out = np.where(unexplained[..., None], reg + (B2 - B), out)
    out = np.where(isbg[..., None], B2, out)
    for chrome in HOST_CHROME:
        m = np.all(reg == np.array(chrome, np.float32), axis=2)
        out[m] = reg[m]
    im[y0:y1] = out
    save_image(np.clip(np.rint(im), 0, 255).astype(np.uint8), out_path)
    print(
        f"{out_path}: {len(blends)} blend backgrounds, "
        f"unexplained {int(unexplained.sum())} of {h * w} region pixels"
    )


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
