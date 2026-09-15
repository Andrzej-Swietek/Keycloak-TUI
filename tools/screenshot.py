#!/usr/bin/env python3
"""Drive the TUI in a pty and save the screen, colours intact, as a PNG.

Screenshotting a TUI by hand loses exactly what matters — the mascot is half-block pixel art
and the status colours carry meaning, so a plain text capture proves nothing. This runs kct
against a real server, feeds it keystrokes on a timeline, replays the ANSI into a cell grid
and draws it.

    python3 tools/screenshot.py users out.png

Add a scene to SHOTS: the extra CLI arguments, a list of (seconds, bytes) keystrokes, and how
long to record.
"""
import os, pty, select, time, re, sys, fcntl, termios, struct
from PIL import Image, ImageDraw, ImageFont

W, H = 104, 30
CW, CH = 9, 19
FONT = ImageFont.truetype("/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf", 16)

def xterm(i):
    if i < 16:
        return [(0,0,0),(170,0,0),(0,170,0),(170,85,0),(0,0,170),(170,0,170),(0,170,170),(192,192,192),
                (85,85,85),(255,85,85),(85,255,85),(255,255,85),(85,85,255),(255,85,255),(85,255,255),(255,255,255)][i]
    if i < 232:
        i -= 16; s = [0,95,135,175,215,255]
        return (s[i//36], s[(i//6)%6], s[i%6])
    v = 8 + (i-232)*10; return (v,v,v)

DEF_FG, DEF_BG = (200,205,215), (16,17,22)

def run(args, keys, seconds):
    env = dict(os.environ, TERM="xterm-256color")
    pid, fd = pty.fork()
    if pid == 0: os.execvpe(args[0], args, env)
    fcntl.ioctl(fd, termios.TIOCSWINSZ, struct.pack("HHHH", H, W, 0, 0))
    out = bytearray(); start = time.time(); sent = 0
    while time.time() - start < seconds:
        r, _, _ = select.select([fd], [], [], 0.1)
        if r:
            try: chunk = os.read(fd, 1 << 18)
            except OSError: break
            if not chunk: break
            out += chunk
        while sent < len(keys) and time.time() - start >= keys[sent][0]:
            os.write(fd, keys[sent][1]); sent += 1
    os.write(fd, b"\x03"); time.sleep(0.3)
    for close in (lambda: os.close(fd), lambda: os.waitpid(pid, 0)):
        try: close()
        except OSError: pass
        except ChildProcessError: pass
    return bytes(out)

def screen(raw):
    text = raw.decode("utf-8", "replace")
    cell = [[(" ", DEF_FG, DEF_BG) for _ in range(W)] for _ in range(H)]
    cy = cx = 0; fg, bg = DEF_FG, DEF_BG; i = 0
    while i < len(text):
        ch = text[i]
        if ch == "\x1b":
            m = re.match(r'\x1b\[([0-9;?]*)([A-Za-z])', text[i:])
            if m:
                raw_nums = m.group(1); cmd = m.group(2)
                nums = [int(p) for p in raw_nums.split(";") if p.isdigit()]
                if cmd == "H":
                    cy = (nums[0]-1) if nums else 0; cx = (nums[1]-1) if len(nums) > 1 else 0
                elif cmd == "J":
                    cell = [[(" ", DEF_FG, DEF_BG) for _ in range(W)] for _ in range(H)]; cy = cx = 0
                elif cmd == "K":
                    for x in range(cx, W): cell[cy][x] = (" ", fg, bg)
                elif cmd == "m":
                    j = 0
                    if not nums: fg, bg = DEF_FG, DEF_BG
                    while j < len(nums):
                        n = nums[j]
                        if n == 0: fg, bg = DEF_FG, DEF_BG
                        elif n == 39: fg = DEF_FG
                        elif n == 49: bg = DEF_BG
                        elif n in (38, 48) and j+2 < len(nums) and nums[j+1] == 5:
                            c = xterm(nums[j+2])
                            if n == 38: fg = c
                            else: bg = c
                            j += 2
                        elif n in (38, 48) and j+4 < len(nums) and nums[j+1] == 2:
                            c = (nums[j+2], nums[j+3], nums[j+4])
                            if n == 38: fg = c
                            else: bg = c
                            j += 4
                        elif 30 <= n <= 37: fg = xterm(n-30)
                        elif 90 <= n <= 97: fg = xterm(n-90+8)
                        elif 40 <= n <= 47: bg = xterm(n-40)
                        elif 100 <= n <= 107: bg = xterm(n-100+8)
                        j += 1
                i += m.end(); continue
            m2 = re.match(r'\x1b\][^\x07]*\x07', text[i:])
            if m2: i += m2.end(); continue
            m3 = re.match(r'\x1b.', text[i:]); i += m3.end() if m3 else 1; continue
        if ch == "\n": cy = min(cy+1, H-1); i += 1; continue
        if ch == "\r": cx = 0; i += 1; continue
        if 0 <= cy < H and 0 <= cx < W: cell[cy][cx] = (ch, fg, bg)
        cx += 1
        if cx >= W: cx = 0; cy = min(cy+1, H-1)
        i += 1
    return cell

def draw(cell, path):
    img = Image.new("RGB", (W*CW, H*CH), DEF_BG)
    d = ImageDraw.Draw(img)
    for y, row in enumerate(cell):
        for x, (ch, fg, bg) in enumerate(row):
            if bg != DEF_BG:
                d.rectangle([x*CW, y*CH, x*CW+CW-1, y*CH+CH-1], fill=bg)
            if ch not in (" ", " "):
                d.text((x*CW, y*CH), ch, font=FONT, fill=fg)
    img.save(path); print(img.size, "->", path)

BASE = ["./target/release/kct", "-s", "http://localhost:18080", "-r", "avsystem",
        "--auth-realm", "master", "-u", "admin", "-p", "admin"]
ENTER = b"\x0d"

def cycle(n, base=2.8):
    """The mascot switch lives in the command palette, not on a key."""
    out = []
    for i in range(n):
        t = base + i * 1.3
        out += [(t, b":"), (t + 0.4, b"mascot"), (t + 1.0, ENTER)]
    return out

SHOTS = {
 "users":   (["--screen", "users"], [], 4.5),
 "custom":  (["--screen", "users", "--mascot", "pirate"], [], 4.5),
 "custom-alarm": (["--screen", "clients", "--mascot", "pirate"], [(3.0, b"jjx")], 5.2),
 "theme":   (["--screen", "realm-settings"], [(2.8, b"lllllll"), (3.8, b"\x0d")], 6.0),
 "algos":   (["--screen", "realm-settings"], [(2.8, b"lllll"), (3.6, b"jj"), (4.2, b"\x0d")], 6.4),
 "happy":   (["--screen", "users"], [(2.8, ENTER), (3.8, b":"), (4.2, b"w"), (4.6, ENTER)], 6.5),
 "alarmed": (["--screen", "clients"], [(3.0, b"jjx")], 5.0),
 "cat":     (["--screen", "users"], cycle(1), 6.0),
 "robot":   (["--screen", "users"], cycle(2), 7.4),
 "ghost":   (["--screen", "groups"], cycle(3), 8.8),
}
name = sys.argv[1]
extra, keys, secs = SHOTS[name]
draw(screen(run(BASE + extra, keys, secs)), sys.argv[2])
