#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
iconmask.py — macOS-style adaptive icon background renderer.

Takes any irregular transparent PNG icon and produces a composite tile:
  - panel color derived from the ICON'S DOMINANT color (light/dark version)
  - per-region colors only fine-tune the panel (subtle, smooth, no abrupt jumps)
  - soft vertical gradient + gloss highlight
  - squircle / rounded-rect mask with anti-aliased edge
  - faint soft shadow cast by the icon, blended over the panel (not pure black)

Usage:
    python iconmask.py input.png output.png [options]
"""
import argparse
import os

from PIL import Image, ImageChops, ImageDraw, ImageEnhance, ImageFilter

def dominant_color(im, grid=8):
    """overall accent color of the icon (alpha + saturation weighted)."""
    r, g, b, a = im.split()
    rl = r.resize((grid, grid), Image.Resampling.BOX)
    gl = g.resize((grid, grid), Image.Resampling.BOX)
    bl = b.resize((grid, grid), Image.Resampling.BOX)
    al = a.resize((grid, grid), Image.Resampling.BOX)
    rp, gp, bp, ap = rl.load(), gl.load(), bl.load(), al.load()
    tr = tg = tb = tw = 0.0
    for y in range(grid):
        for x in range(grid):
            av = ap[x, y]
            if av <= 0:
                continue
            cr, cg, cb = rp[x, y], gp[x, y], bp[x, y]
            mx = max(cr, cg, cb)
            mn = min(cr, cg, cb)
            sat = (mx - mn) / 255.0
            w = av * (0.25 + 1.5 * sat)
            tr += cr * w
            tg += cg * w
            tb += cb * w
            tw += w
    if tw <= 0:
        return (255, 255, 255)
    return (int(tr / tw), int(tg / tw), int(tb / tw))

def radial_ray_colors(icon_layer, N=360, ang_smooth=10):
    """For each of N angles, average the icon's color along that radial line,
    weighted toward the icon's OUTER edge (the rim color radiates outward)."""
    import math
    a = icon_layer.split()[3]
    bbox = a.getbbox()
    w, h = icon_layer.size
    if bbox is None:
        return [(255, 255, 255)] * N, w / 2, h / 2, min(w, h) / 2
    icx = (bbox[0] + bbox[2]) / 2.0
    icy = (bbox[1] + bbox[3]) / 2.0
    icw_b = bbox[2] - bbox[0]
    ich_b = bbox[3] - bbox[1]
    max_r = math.hypot(icw_b, ich_b) / 2 + 1
    px = icon_layer.load()
    colors = []
    for i in range(N):
        th = 2 * math.pi * i / N
        dx, dy = math.cos(th), math.sin(th)
        sr = sg = sb = sw = 0.0
        for r in range(1, int(max_r)):
            x = int(icx + dx * r)
            y = int(icy - dy * r)   # y flipped: screen coords grow downward
            if 0 <= x < w and 0 <= y < h:
                p = px[x, y]
                if p[3] > 10:
                    wgt = p[3] * (r / max_r) ** 1.4   # outer edge dominates
                    sw += wgt
                    sr += p[0] * wgt
                    sg += p[1] * wgt
                    sb += p[2] * wgt
            else:
                break
        if sw > 0:
            colors.append((sr / sw, sg / sw, sb / sw))
        else:
            colors.append((255, 255, 255))
    if ang_smooth > 0:
        sm = []
        for i in range(N):
            s = [colors[(i + j) % N] for j in range(-ang_smooth, ang_smooth + 1)]
            sm.append((sum(c[0] for c in s) / len(s),
                       sum(c[1] for c in s) / len(s),
                       sum(c[2] for c in s) / len(s)))
        colors = sm
    return colors, icx, icy, icw_b / 2, ich_b / 2

def edge_radius_at(mask, icx, icy, th):
    """distance from center to the rounded-rect edge along angle th."""
    import math
    dx, dy = math.cos(th), math.sin(th)
    w, h = mask.size
    px = mask.load()
    r = 0
    while True:
        r += 1
        x = int(icx + dx * r)
        y = int(icy + dy * r)
        if 0 <= x < w and 0 <= y < h:
            if px[x, y] < 128:
                return r
        else:
            return r
    return r

def luminance_of(im, grid=16):
    """alpha-weighted average luminance 0..1 of an RGBA image."""
    r, g, b, a = im.split()
    rl = r.resize((grid, grid), Image.Resampling.BOX)
    gl = g.resize((grid, grid), Image.Resampling.BOX)
    bl = b.resize((grid, grid), Image.Resampling.BOX)
    al = a.resize((grid, grid), Image.Resampling.BOX)
    rp, gp, bp, ap = rl.load(), gl.load(), bl.load(), al.load()
    tot_l, tot_w = 0.0, 0.0
    for y in range(grid):
        for x in range(grid):
            av = ap[x, y]
            if av <= 0:
                continue
            lum = (0.2126 * rp[x, y] + 0.7152 * gp[x, y] + 0.0722 * bp[x, y]) / 255.0
            tot_l += lum * av
            tot_w += av
    return tot_l / tot_w if tot_w > 0 else 0.5

def make_mask(w, h, shape, corner_frac=0.22, ss=4):
    """anti-aliased mask: 'rect' | 'squircle' | 'circle'."""
    hs = h * ss
    ws = w * ss
    mask = Image.new("L", (ws, hs), 0)
    dd = ImageDraw.Draw(mask)
    mw, mh = ws - 1, hs - 1
    if shape == "circle":
        dd.ellipse((0, 0, mw, mh), fill=255)
    elif shape == "rect":
        dd.rounded_rectangle((0, 0, mw, mh), radius=corner_frac * ws, fill=255)
    else:  # squircle (superellipse, n=5)
        cx, cy = ws / 2.0, hs / 2.0
        Rx = ws / 2.0 - 0.5
        Ry = hs / 2.0 - 0.5
        n = 5.0
        p = mask.load()
        for yy in range(hs):
            for xx in range(ws):
                dx = (xx - cx) / Rx
                dy = (yy - cy) / Ry
                if abs(dx) ** n + abs(dy) ** n <= 1.0:
                    p[xx, yy] = 255
    return mask.resize((w, h), Image.Resampling.BILINEAR)

def vertical_gradient(w, h, top=1.02, bottom=0.98):
    """near-flat brightness: the panel is a solid color, gradient kept negligible."""
    grad = Image.new("L", (1, h))
    gd = grad.load()
    for y in range(h):
        t = y / (h - 1)
        gd[0, y] = int(255 * (top - (top - bottom) * t))
    return grad.resize((w, h))

def render_icon_layer(icon, cw, ch, ratio=0.66, icolight=0.0, offset=(0, 0)):
    r, g, b, a = icon.split()
    bbox = a.getbbox()
    if bbox is None:
        return Image.new("RGBA", (cw, ch), (0, 0, 0, 0))
    icon = icon.crop(bbox)
    iw, ih = icon.size
    s = min((cw * ratio) / iw, (ch * ratio) / ih)  # icon fills `ratio` of the canvas
    nw, nh = max(1, int(iw * s + 0.5)), max(1, int(ih * s + 0.5))
    icon = icon.resize((nw, nh), Image.Resampling.LANCZOS)
    if icolight > 0:
        icon = ImageEnhance.Brightness(icon).enhance(1 + icolight)
    canvas = Image.new("RGBA", (cw, ch), (0, 0, 0, 0))
    canvas.paste(icon, (cw // 2 - nw // 2 + offset[0], ch // 2 - nh // 2 + offset[1]), icon)
    return canvas

def gloss_layer(w, h, strength):
    """soft top sheen, only in the upper part of the tile, very subtle."""
    gl = Image.new("L", (1, h))
    gd = gl.load()
    for y in range(h):
        t = y / (h - 1)
        f = max(0.0, 1.0 - t / 0.30)   # zone: top 30%
        f = f * f * strength
        gd[0, y] = int(255 * f)
    gloss = gl.resize((w, h))
    return Image.merge("RGBA", (Image.new("L", (w, h), 255),
                                Image.new("L", (w, h), 255),
                                Image.new("L", (w, h), 255),
                                gloss))

def cast_shadow(src, offset, blur_r, opacity, fade):
    """faint shadow layer blended over the background (soft, not black)."""
    w, h = src.size
    sh = Image.new("L", (w, h), 0)
    y_off = offset if isinstance(offset, int) else offset[1]
    sh.paste(src, (0, y_off))
    sh = sh.filter(ImageFilter.GaussianBlur(blur_r))
    px = sh.load()
    for y in range(h):
        t = y / (h - 1)
        factor = 1.0 - fade * max(0.0, (t - 0.5) / 0.5)
        for x in range(w):
            v = px[x, y]
            if v:
                px[x, y] = int(v * factor)
    alpha_im = sh.point(lambda v: min(255, int(v * opacity)))
    return Image.merge("RGBA", (Image.new("L", (w, h), 0),
                                Image.new("L", (w, h), 0),
                                Image.new("L", (w, h), 0),
                                alpha_im))

def main():
    ap = argparse.ArgumentParser(description="adaptive icon mask renderer")
    ap.add_argument("input")
    ap.add_argument("output")
    ap.add_argument("--size", type=int, default=None)
    ap.add_argument("--shape", choices=["rect", "squircle", "circle"], default="rect")
    ap.add_argument("--corner", type=float, default=0.22)
    ap.add_argument("--bg", type=float, default=0.10,
                    help="径向添头强度 0..1: 扇区色在单色面板上的微调")
    ap.add_argument("--thresh", type=float, default=0.5, help="明暗分界 (0..1 亮度)")
    ap.add_argument("--light-mix", type=float, default=0.90,
                    help="亮图标面板调白比例 (1=纯白)")
    ap.add_argument("--dark-mix", type=float, default=0.72,
                    help="暗图标面板加深系数 (0=保持原色, 1=纯黑)")
    ap.add_argument("--gloss", type=float, default=0.16)
    ap.add_argument("--icolight", type=float, default=0.08)
    ap.add_argument("--iconratio", type=float, default=0.66)
    ap.add_argument("--shadow-offset", type=float, default=None)
    ap.add_argument("--shadow-blur", type=float, default=None)
    ap.add_argument("--shadow-opacity", type=float, default=0.22)
    ap.add_argument("--shadow-fade", type=float, default=0.25)
    ap.add_argument("--shadow-mode", choices=["icon", "badge"], default="icon")
    ap.add_argument("--debug-points", default="",
                    help="e.g. '8,48;8,128;248,128' prints per-layer values at those pixels")
    args = ap.parse_args()

    dbg = [(int(p.split(",")[0]), int(p.split(",")[1]))
           for p in args.debug_points.split(";") if p.strip()]

    icon_raw = Image.open(args.input).convert("RGBA")
    # trim transparent margin first so analysis never touches void/black areas
    bbox = icon_raw.split()[3].getbbox()
    icon = icon_raw.crop(bbox) if bbox else icon_raw
    # always pad into a SQUARE canvas (dock tiles are square); the rounded-rect
    # mask then exactly matches the expectation
    _s = args.size or max(icon.size)
    _pw, _ph = icon.size
    _f = min(_s / _pw, _s / _ph)
    _nw, _nh = max(1, int(_pw * _f + 0.5)), max(1, int(_ph * _f + 0.5))
    _sq = Image.new("RGBA", (_s, _s), (0, 0, 0, 0))
    _sq.paste(icon.resize((_nw, _nh), Image.Resampling.LANCZOS),
              ((_s - _nw) // 2, (_s - _nh) // 2), icon)
    icon = _sq
    w, h = icon.size
    size = w  # square

    # 1. panel base color = dominant accent color, lightened/darkened
    dom = dominant_color(icon)
    lum = luminance_of(icon)
    bright = lum > args.thresh
    if bright:
        panel = tuple(int(c + (255 - c) * args.light_mix) for c in dom)
    else:
        panel = tuple(int(c * (1 - args.dark_mix)) for c in dom)
    print(f"luminance={lum:.2f} {'LIGHT' if bright else 'DARK'} dom={dom} "
          f"panel={panel} region-adjust={args.bg}")

    # 2. mask shape
    mask = make_mask(w, h, args.shape, args.corner)

    # 3. icon layer (needed both for drawing and for radial color sampling)
    icon_layer = render_icon_layer(icon, w, h, args.iconratio, args.icolight,
                                   (0, max(1, int(h * 0.012))))

    # 4. radial background: split the rounded rect into 360 sectors from center,
    #    each sector tinted by the icon color along that radial line
    import math
    ray_colors, icx, icy, ihw, ihh = radial_ray_colors(icon_layer)
    N = len(ray_colors)
    edges = [edge_radius_at(mask, icx, icy, 2 * math.pi * i / N) for i in range(N)]
    bg = Image.new("RGB", (w, h))
    bgpx = bg.load()
    vgl = vertical_gradient(w, h).load()
    k = args.bg
    for y in range(h):
        for x in range(w):
            th = math.atan2(icy - y, x - icx)        # right=0, up=pi/2, ccw
            if th < 0:
                th += 2 * math.pi                    # wrap to 0..2pi to match ray bins
            fi = th / (2 * math.pi) * N
            i0 = int(fi) % N
            i1 = (i0 + 1) % N
            f = fi - int(fi)
            c0, c1 = ray_colors[i0], ray_colors[i1]
            cr = c0[0] + (c1[0] - c0[0]) * f
            cg_ = c0[1] + (c1[1] - c0[1]) * f
            cb = c0[2] + (c1[2] - c0[2]) * f
            # radial falloff across the margin band: full tint right at the icon
            # edge, fading to the panel tone at the rounded-rect edge
            er = max(1.0, edges[i0] + (edges[i1] - edges[i0]) * f)
            dist = math.hypot(x - icx, y - icy)
            ac, as_ = abs(math.cos(th)), abs(math.sin(th))
            ri = min(ihw / max(ac, 1e-9), ihh / max(as_, 1e-9))
            if dist <= ri:
                rf = 1.0
            else:
                span = max(1.0, er - ri)
                s = min(1.0, (dist - ri) / span)
                rf = (1.0 - s) ** 0.5   # slow decay: halo travels far into the panel
            r_ = int(panel[0] + (cr - dom[0]) * k * rf)
            g_ = int(panel[1] + (cg_ - dom[1]) * k * rf)
            b_ = int(panel[2] + (cb - dom[2]) * k * rf)
            fg = vgl[x, y] / 255.0
            bgpx[x, y] = (min(255, max(0, int(r_ * fg))),
                          min(255, max(0, int(g_ * fg))),
                          min(255, max(0, int(b_ * fg))))
    if dbg:
        print("  [dbg] bg@pts:", [bg.getpixel(p) for p in dbg])
        print("  [dbg] ray0..3:", ray_colors[:4], "...N=", N)

    bg_layer = bg.convert("RGBA")
    bg_layer.putalpha(mask)

    # 5. faint shadow from icon silhouette, overlaid on top of bg
    offset = int(args.shadow_offset) if args.shadow_offset is not None else max(2, int(size * 0.012))
    blur = args.shadow_blur if args.shadow_blur is not None else max(3.0, size * 0.022)
    shadow_src = icon_layer.split()[3] if args.shadow_mode == "icon" else mask
    shadow = cast_shadow(shadow_src, offset, blur,
                         args.shadow_opacity, args.shadow_fade)

    # 5. composite bg -> gloss -> shadow -> icon, clip to mask (nothing overflows corners)
    canvas = Image.new("RGBA", (w, h), (0, 0, 0, 0))
    canvas = Image.alpha_composite(canvas, bg_layer)
    canvas = Image.alpha_composite(canvas, gloss_layer(w, h, args.gloss))
    canvas = Image.alpha_composite(canvas, shadow)
    canvas = Image.alpha_composite(canvas, icon_layer)
    r, g_, b_, a = canvas.split()
    a = ImageChops.multiply(a, mask)
    canvas = Image.merge("RGBA", (r, g_, b_, a))
    if dbg:
        print("  [dbg] final@pts:", [canvas.getpixel(p) for p in dbg])

    os.makedirs(os.path.dirname(os.path.abspath(args.output)), exist_ok=True)
    canvas.save(args.output)
    print(f"saved -> {args.output}")

if __name__ == "__main__":
    main()