"""Drawing primitives for the mascot sprites.

Shapes are composed from ellipses and rectangles, each layer outlined before the next goes in
front of it, and shaded from one light source in the upper left. Nothing here knows about any
particular character.
"""

W, H = 25, 20
PORTRAIT_TOP = 4  # portrait designs draw in the lower 16 rows and stand on the panel floor
T = '.'
CX = 12.0

def blank(rows=None): return [[T]*W for _ in range(rows or H)]


def portrait(draw):
    """Draw a 16-row bust, then sit it on the floor of the 20-row canvas."""
    global H
    H, full = 16, H
    try:
        drawn = draw()
    finally:
        H = full
    return [T * W] * PORTRAIT_TOP + drawn

def ell(c, cx, cy, rx, ry, ch, only=None):
    for y in range(H):
        for x in range(W):
            if ((x-cx)/rx)**2 + ((y-cy)/ry)**2 <= 1.0 and (only is None or c[y][x] in only):
                c[y][x] = ch

def rect(c, x0, y0, x1, y1, ch, only=None):
    for y in range(max(0,y0), min(H-1,y1)+1):
        for x in range(max(0,x0), min(W-1,x1)+1):
            if only is None or c[y][x] in only: c[y][x] = ch

def wedge(c, cx, top, h, ch):
    for i in range(h):
        y = top + i
        if 0 <= y < H:
            for x in range(int(cx-i*0.8), int(cx+i*0.8)+1):
                if 0 <= x < W: c[y][x] = ch

def outline(c, ink='k', edges=True):
    """Ring the silhouette. With `edges` off, the canvas border is not air: a shape running off
    the bottom keeps its colour instead of gaining a black line where it is simply cut."""
    sil = {(x,y) for y in range(H) for x in range(W) if c[y][x] != T}
    def air(p):
        x, y = p
        if not (0 <= x < W and 0 <= y < H):
            return edges
        return p not in sil
    for x,y in [(x,y) for (x,y) in sil
                if any(air((x+dx,y+dy)) for dx,dy in ((1,0),(-1,0),(0,1),(0,-1)))]:
        c[y][x] = ink

def over(base, top):
    """Composite one outlined layer in front of another."""
    for y in range(H):
        for x in range(W):
            if top[y][x] != T: base[y][x] = top[y][x]
    return base

def shade(c, mats, light, dark):
    src = [row[:] for row in c]
    def gone(x, y):
        # Off-canvas is a crop, not air: shading against it would draw a dark rim along the
        # border of any shape that runs off the edge.
        if not (0 <= x < W and 0 <= y < H):
            return False
        return src[y][x] in ('k', T)
    for y in range(H):
        for x in range(W):
            if src[y][x] not in mats: continue
            if sum(gone(x+dx, y+dy) for dx,dy in ((-1,0),(0,-1),(-1,-1))) >= 2: c[y][x] = light
            elif sum(gone(x+dx, y+dy) for dx,dy in ((1,0),(0,1),(1,1))) >= 2: c[y][x] = dark

def under_shadow(c, mat, shadow, above):
    for y in range(1, H):
        for x in range(W):
            if c[y][x] == mat and c[y-1][x] in above: c[y][x] = shadow

def rows(c):
    out = [''.join(r) for r in c]
    for r in out: assert len(r) == W, (len(r), r)
    assert len(out) == H
    return out
