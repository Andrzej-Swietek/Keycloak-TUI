#!/usr/bin/env python3
"""Draw the mascot sprites and write them into the Rust sources.

The bodies in `crates/kct-tui/src/view/mascot/sprite/*.rs` are generated, not hand-typed: at
this size a row that is one pixel wide shears the whole sprite, and shading a curve by hand is
guesswork. Shapes are composed from primitives, each layer is outlined before the next goes in
front of it, and every row is width-checked before it reaches Rust.

    python3 tools/sprites.py            # rewrite the Rust bodies
    python3 tools/sprites.py --print    # dump the palette-key grids instead

Colour lives in `palette.rs`; this only decides which key each pixel carries.
"""

import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))

from pixels import *  # noqa: F403  — the drawing vocabulary
from pixels import CX, H, T, W

HAIR = ('h','w','W','H')
FACE_COL, FACE_W = 7, 11

def face_oval(c, cy=8.0, rx=5.5, ry=5.0, mat='s'):
    ell(c, CX, cy, rx, ry, mat)

def finish_face(c, blush_row=11, cheeks=(8, 9, 15, 16)):
    under_shadow(c, 's', 'S', HAIR + ('k',))
    for x in cheeks:
        if c[blush_row][x] == 's': c[blush_row][x] = 'o'

def twintails():
    return portrait(_twintails)


def _twintails():
    # Miku is a silhouette before she is a face: the head rides high so the tails can fall past
    # the chin, which is the shape nothing else here has. Head and tails share one outline —
    # outlining them apart puts a black seam between hair off the same head — so the head's
    # shadow separates them instead.
    c = blank()
    for cx in (3.0, 21.0):
        ell(c, cx, 5.0, 3.0, 2.6, 'h')               # gathered at the tie
        ell(c, cx, 11.0, 3.0, 5.4, 'h')              # falling past the chin
    ell(c, CX, 6.0, 8.0, 5.8, 'h')
    ell(c, CX, 6.2, 5.5, 4.6, 's')
    ell(c, CX, 0.5, 8.0, 3.8, 'h', only=('s',))      # fringe, cut straight across the brow
    outline(c); shade(c, ('h',), 'w', 'H')
    ell(c, CX - 3, 1.5, 4.0, 1.2, 'W', only=('h','w'))

    for x in (3, 4, 20, 21):                          # the head shades the tails behind it
        rect(c, x, 3, x, 12, 'H', only=HAIR)
    for x0, x1 in ((0, 3), (21, 24)):                 # ties, clipped to the hair they bind
        rect(c, x0, 3, x1, 4, 'c', only=HAIR + ('k',))
    finish_face(c, blush_row=9)
    return rows(c)


def miku():
    # The same character full length, at chibi proportions: the head stays a touch larger than
    # the body, because the face is the only part that carries an expression and it has to stay
    # legible. She wears the narrow face — an eleven-pixel window forces a head so broad there
    # is no hair left around it and the body looks stuck on underneath.
    c = blank()
    for cx in (2.5, 21.5):
        ell(c, cx, 3.0, 2.8, 2.8, 'h')               # gathered at the tie
        rect(c, int(cx - 2), 3, int(cx + 2), 16, 'h')  # falling straight, not pinched
        ell(c, cx, 16.5, 2.6, 2.8, 'h')              # tapered tip
    ell(c, CX, 5.5, 7.0, 5.8, 'h')
    ell(c, CX, 5.8, 4.6, 4.8, 's')
    ell(c, CX, 0.5, 7.0, 3.4, 'h', only=('s',))      # fringe
    outline(c); shade(c, ('h',), 'w', 'H')
    ell(c, CX - 2, 1.0, 3.2, 1.0, 'W', only=('h','w'))
    for x in (6, 18):                                 # the head shades the tails behind it
        rect(c, x, 2, x, 11, 'H', only=HAIR)
    for x0, x1 in ((1, 3), (21, 23)):                 # ties, clipped to the hair they bind
        rect(c, x0, 3, x1, 4, 'c', only=HAIR + ('k',))

    body = blank()
    rect(body, 11, 11, 13, 12, 'S')                  # neck
    rect(body, 9, 13, 15, 17, 'C')                   # sleeveless top
    rect(body, 5, 13, 8, 16, 'l')                    # detached sleeves
    rect(body, 16, 13, 19, 16, 'l')
    rect(body, 6, 17, 7, 17, 'S')                    # hands
    rect(body, 17, 17, 18, 17, 'S')
    ell(body, CX, 18.0, 8.0, 3.0, 'C')               # skirt, flaring to the floor
    outline(body, edges=False); shade(body, ('C',), 'l', 'c')
    rect(body, 11, 13, 13, 17, 'w')                  # the tie
    rect(body, 12, 14, 12, 17, 'W')

    c = over(c, body)
    finish_face(c, blush_row=7, cheeks=(9, 10, 14, 15))
    return rows(c)

def cat():
    return portrait(_cat)


def _cat():
    ears = blank()                              # big enough to break the silhouette
    wedge(ears, 5, 0, 6, 'h'); wedge(ears, 19, 0, 6, 'h')
    outline(ears)
    ell(ears, 5.0, 4.0, 1.4, 1.8, 'O', only=('h','w','H'))
    ell(ears, 19.0, 4.0, 1.4, 1.8, 'O', only=('h','w','H'))

    head = blank()
    ell(head, CX, 8.5, 8.5, 7.0, 'h')
    face_oval(head, cy=8.5)
    ell(head, CX, 2.5, 8.0, 3.0, 'h', only=('s',))
    outline(head); shade(head, ('h',), 'w', 'H')
    ell(head, CX - 3, 3.5, 4.0, 1.2, 'W', only=('h','w'))

    tail = blank()
    rect(tail, 21, 14, 24, 15, 'h')
    ell(tail, 23.5, 12.5, 2.2, 3.0, 'h')
    ell(tail, 23.5, 12.5, 0.9, 1.4, T)
    outline(tail)

    c = over(over(tail, ears), head)
    finish_face(c)
    return rows(c)

def robot():
    return portrait(_robot)


def _robot():
    ant = blank()
    rect(ant, 11, 0, 13, 2, 'O'); rect(ant, 11, 2, 13, 2, 'o')
    rect(ant, 12, 3, 12, 6, 'W')                # a lit stalk reads where an outlined one would not

    shell = blank()
    rect(shell, 3, 4, 21, 15, 'C')
    for x, y in ((3,4),(21,4)): shell[y][x] = T
    rect(shell, 0, 8, 3, 12, 'C'); rect(shell, 21, 8, 24, 12, 'C')   # pods touch the shell
    outline(shell); shade(shell, ('C',), 'W', 'c')

    visor = blank()
    rect(visor, 5, 6, 19, 13, 'c')
    for x, y in ((5,6),(19,6),(5,13),(19,13)): visor[y][x] = T
    outline(visor)
    rect(visor, 6, 7, 18, 12, 'c')

    c = over(over(shell, ant), visor)
    for x in range(7, 18, 3): c[15][x] = 'c'
    return rows(c)

def ghost():
    return portrait(_ghost)


def _ghost():
    c = blank()
    ell(c, CX, 7.5, 8.5, 7.5, 'g')
    rect(c, 4, 7, 20, 15, 'g')
    for x in range(W):
        if (x + 3) % 6 in (0, 1, 2):
            for y in (14, 15):
                if c[y][x] == 'g': c[y][x] = T
    outline(c); shade(c, ('g',), 'p', 'G')
    ell(c, CX - 3, 2.5, 4.5, 2.2, 'p', only=('g','G'))
    for x in (8, 9, 15, 16):
        if c[11][x] == 'g': c[11][x] = 'o'
    return rows(c)



ALL = {
    "twintails": twintails(),
    "miku": miku(),
    "cat": cat(),
    "robot": robot(),
    "ghost": ghost(),
}

ROOT = pathlib.Path(__file__).resolve().parent.parent
TARGET = ROOT / "crates/kct-tui/src/view/mascot/sprite"


def write():
    for name, body in ALL.items():
        path = TARGET / f"{name}.rs"
        source = path.read_text()
        start = source.index("    body: [")
        end = source.index("    ],", start)
        drawn = "\n".join(f'        "{row}",' for row in body)
        path.write_text(source[:start] + "    body: [\n" + drawn + "\n" + source[end:])
        print(f"{path.relative_to(ROOT)}: {len(body)} rows x {W}")


if __name__ == "__main__":
    if "--print" in sys.argv:
        for name, body in ALL.items():
            print(name, *body, sep="\n")
    else:
        write()
