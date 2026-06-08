#!/usr/bin/env python3
# Foundations diagram: the components rest on four stack-wide substrates.

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


G = Grid(100, 58)
X, W = 3, 86

# ---- the components rest on top ----------------------------------------
comp = G.box(X, 0, W, title="the 20 components  —  each a specialization of the substrate below",
             body=[
               "strata . hemera . lens . trident . nox . zheng . cybergraph . bbg",
               "tru . glia . mir . mudra . radio . tape . sync . foculus . soma",
               "conformance . rune . fs . plumb",
             ])
comp_b = comp[1] + comp[3] - 1
cx = X + W // 2
G.a_down(cx, comp_b + 1, comp_b + 4)
G.put(cx + 2, comp_b + 2, "every component obeys the same four substrates")

# ---- four flush-stacked foundation layers (bedrock) --------------------
y = comp_b + 4
def layer(y, title, rows):
    b = G.box(X, y, W, title=title, body=rows)
    return b[1] + b[3] - 1   # bottom row (next layer starts here, flush)

y = layer(y, "I  .  ONE REPRESENTATION    —  everything is one polynomial over one field", [
    "one field            every value is one goldilocks field element",
    "one object           state, data and proofs are one committed polynomial",
    "five algebras        the field hosts five regimes of computation",
    "particle identity    a name is its content hash : a polynomial commitment",
])
y = layer(y, "II  .  ONE PROOF            —  execution, programs, and the prover itself", [
    "proof-native         running a program and proving it are one act",
    "recursive closure    proofs verify proofs : all history folds to one",
    "transparent          hash-based, post-quantum, no trusted setup",
    "conformance          every output fingerprinted : drift caught at commit",
    "eidos                a proof assistant in the kernel : programs proven correct",
    "self-hosting         the verifier is a nox program : the system closes on itself",
])
y = layer(y, "III  .  ONE CONVERGENCE     —  the whole graph settles into one mind", [
    "global focus            one phi* distribution over every particle",
    "settle, do not derive   fixed points discovered, not theorems derived",
    "tri-kernel              diffusion + springs + heat : the only local basis",
    "one answer, four ways   phi* is consensus + rank + reward + meaning at once",
    "speed of thought        the spectral gap is how fast the whole graph decides",
    "stake-weighted          security is economic mass, not honest majority",
])
y = layer(y, "IV  .  ONE FABRIC           —  it holds together at planet scale", [
    "five-layer sync      validity . ordering . completeness . availability . merge",
    "bounded locality     every change is local to a log-n neighborhood",
    "privacy trilateral   ZK + FHE + MPC compose over the shared field",
])

# ---- the language trinity : how you interface --------------------------
ty = y + 2
G.put(X, ty, "you interface the whole stack in three languages :")
tw = 24
gap = 4
b1 = G.box(X, ty + 1, tw, title="write", body=["cybermark"], center=True)
b2 = G.box(X + tw + gap, ty + 1, tw, title="compute", body=["the trident family"], center=True)
b3 = G.box(X + 2*(tw + gap), ty + 1, tw, title="mean", body=["neural"], center=True)

print(G.text())
