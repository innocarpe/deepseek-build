"""Colorimetry for theme candidates: WCAG 2.1 contrast, CIE L*, APCA, LCh<->sRGB.

Part of the theme preview toolchain (see
`docs/product/THEME_CLASSIC_READABILITY_2026-09-27.md` for the workflow and the
values this produced). Pure stdlib — no numpy, no Pillow.

    from metrics import cr, Lstar, apca, hx, lch2rgb
    cr((215, 217, 231), (14, 20, 37))   # body text on bg_base
"""

import math


def lin(c):
    c /= 255
    return c / 12.92 if c <= 0.04045 else ((c + 0.055) / 1.055) ** 2.4


def Y(rgb):
    r, g, b = [lin(x) for x in rgb]
    return 0.2126 * r + 0.7152 * g + 0.0722 * b


def cr(a, b):
    ya, yb = Y(a), Y(b)
    hi, lo = max(ya, yb), min(ya, yb)
    return (hi + 0.05) / (lo + 0.05)


def Lstar(rgb):
    y = Y(rgb)
    return 116 * (y ** (1 / 3)) - 16 if y > 0.008856 else 903.3 * y


def lab(rgb):
    r, g, b = [lin(x) for x in rgb]
    X = 0.4124 * r + 0.3576 * g + 0.1805 * b
    Yv = 0.2126 * r + 0.7152 * g + 0.0722 * b
    Z = 0.0193 * r + 0.1192 * g + 0.9505 * b
    Xn, Yn, Zn = 0.95047, 1, 1.08883
    f = lambda t: t ** (1 / 3) if t > 0.008856 else 7.787 * t + 16 / 116
    fx, fy, fz = f(X / Xn), f(Yv / Yn), f(Z / Zn)
    L = 116 * fy - 16
    a = 500 * (fx - fy)
    bb = 200 * (fy - fz)
    return L, math.hypot(a, bb), (math.degrees(math.atan2(bb, a)) % 360)


# APCA 0.0.98G-4g. A supporting indicator only — not a standard.
def apca(txt, bg):
    def sY(rgb):
        r, g, b = [(x / 255) ** 2.4 for x in rgb]
        return 0.2126729 * r + 0.7151522 * g + 0.0721750 * b

    Yt, Yb = sY(txt), sY(bg)
    bt = 0.022
    Yt = Yt + (bt - Yt) ** 1.414 if Yt < bt else Yt
    Yb = Yb + (bt - Yb) ** 1.414 if Yb < bt else Yb
    if Yb > Yt:
        S = (Yb ** 0.56 - Yt ** 0.57) * 1.14
        return 0 if S < 0.1 else (S - 0.027) * 100
    else:
        S = (Yb ** 0.65 - Yt ** 0.62) * 1.14
        return 0 if S > -0.1 else (S + 0.027) * 100


def hx(c):
    return "#%02X%02X%02X" % tuple(c)


def lch2rgb(L, C, h):
    a = C * math.cos(math.radians(h))
    b = C * math.sin(math.radians(h))
    fy = (L + 16) / 116
    fx = fy + a / 500
    fz = fy - b / 200
    finv = lambda t: t**3 if t**3 > 0.008856 else (t - 16 / 116) / 7.787
    X = 0.95047 * finv(fx)
    Yv = finv(fy)
    Z = 1.08883 * finv(fz)
    r = 3.2406 * X - 1.5372 * Yv - 0.4986 * Z
    g = -0.9689 * X + 1.8758 * Yv + 0.0415 * Z
    bb = 0.0557 * X - 0.2040 * Yv + 1.0570 * Z

    def enc(v):
        v = max(0, min(1, v))
        v = 12.92 * v if v <= 0.0031308 else 1.055 * v ** (1 / 2.4) - 0.055
        return round(v * 255)

    return (enc(r), enc(g), enc(bb))
