#!/usr/bin/env python3
# Foundations diagram: the twenty components rest on one substrate —
# one mind, reachable in many languages, growing into an open world.
# Kept narrow (box width 74) so it fits the reading column without clipping.

class Grid:
    def __init__(self, w, h):
        self.w, self.h = w, h
        self.g = [[' '] * w for _ in range(h)]
    def put(self, x, y, s):
        for i, ch in enumerate(s):
            self.g[y][x + i] = ch
    def putc(self, x, y, ch): self.g[y][x] = ch
    def hline(self, x1, x2, y, ch='-'):
        for x in range(x1, x2 + 1):
            self.g[y][x] = '+' if self.g[y][x] in '|+' else ch
    def vline(self, y1, y2, x, ch='|'):
        for y in range(y1, y2 + 1):
            self.g[y][x] = '+' if self.g[y][x] in '-+' else ch
    def box(self, x, y, w, title=None, body=None, center=False):
        body = body or []
        h = 2 + (2 if title is not None else 0) + len(body)
        self.putc(x, y, '+'); self.putc(x+w-1, y, '+')
        self.putc(x, y+h-1, '+'); self.putc(x+w-1, y+h-1, '+')
        self.hline(x+1, x+w-2, y); self.hline(x+1, x+w-2, y+h-1)
        self.vline(y+1, y+h-2, x); self.vline(y+1, y+h-2, x+w-1)
        def place(s, row):
            px = x + (w - len(s)) // 2 if center else x + 2
            self.put(px, row, s)
        row = y + 1
        if title is not None:
            place(title, row); row += 1
            self.putc(x, row, '+'); self.putc(x+w-1, row, '+'); self.hline(x+1, x+w-2, row); row += 1
        for line in body:
            place(line, row); row += 1
        return (x, y, w, h)
    def a_down(self, x, y1, y2):
        self.vline(y1, y2-2, x); self.putc(x, y2-1, 'v')
    def text(self):
        return '\n'.join(''.join(r).rstrip() for r in self.g)


G = Grid(80, 42)
X, W = 3, 74

# ---- the twenty components rest on top ---------------------------------
comp = G.box(X, 0, W, title="the 20 components  —  each a specialization of the substrate below",
             body=[
               "strata . hemera . lens . trident . nox . zheng . cybergraph . bbg",
               "tru . glia . mir . mudra . radio . tape . sync . foculus . soma",
               "conformance . rune . fs . plumb",
             ])
comp_b = comp[1] + comp[3] - 1
cx = X + W // 2
G.a_down(cx, comp_b + 1, comp_b + 4)
G.put(cx + 2, comp_b + 2, "each component, the same mind")

# ---- three principles, flush-stacked (the bedrock) ---------------------
y = comp_b + 4
def layer(y, title, rows):
    b = G.box(X, y, W, title=title, body=rows)
    return b[1] + b[3] - 1   # bottom row (next layer starts here, flush)

y = layer(y, "ONE MIND   —  one polynomial, proven, converged", [
    "one form    one polynomial over one field . five algebras",
    "one proof   proof-native . recursive . the prover proves the prover",
    "one focus   one phi* over all particles . tri-kernel . spectral gap",
])
y = layer(y, "MANY LANGUAGES   —  one substrate, three ways to touch it", [
    "write       cybermark . the markup is the graph",
    "compute     the trident family . all lower to nox",
    "mean        neural . meaning is an eigenvector of the graph",
])
y = layer(y, "OPEN WORLD   —  it holds, composes, admits", [
    "it holds    five-layer sync . bounded locality . planet scale",
    "it composes add a cyberlink, not an API . nothing to translate",
    "it admits   no schema . no gatekeeper . ZK+FHE+MPC over one field",
])

print(G.text())
