#!/usr/bin/env python3
# Grid-based svgbob layout generator for the soft3 troika scheme.
# Exact column placement -> no jogs, no orphan stubs, clean junctions.
# Three kinds of language wrap the spine:
#   cybermark (write, top)  .  compute languages (right)  .  neural (mean, bottom)

class Grid:
    def __init__(self, w, h):
        self.w, self.h = w, h
        self.g = [[' '] * w for _ in range(h)]

    def put(self, x, y, s):
        for i, ch in enumerate(s):
            self.g[y][x + i] = ch

    def putc(self, x, y, ch):
        self.g[y][x] = ch

    def hline(self, x1, x2, y, ch='-'):
        for x in range(x1, x2 + 1):
            cur = self.g[y][x]
            self.g[y][x] = '+' if cur in '|+' else ch

    def vline(self, y1, y2, x, ch='|'):
        for y in range(y1, y2 + 1):
            cur = self.g[y][x]
            self.g[y][x] = '+' if cur in '-+' else ch

    def box(self, x, y, w, title=None, body=None, ticks=None, center=False):
        body = body or []
        nrows = (2 if title is not None else 0) + len(body)
        h = 2 + nrows
        self.putc(x, y, '+'); self.putc(x + w - 1, y, '+')
        self.putc(x, y + h - 1, '+'); self.putc(x + w - 1, y + h - 1, '+')
        self.hline(x + 1, x + w - 2, y)
        self.hline(x + 1, x + w - 2, y + h - 1)
        self.vline(y + 1, y + h - 2, x)
        self.vline(y + 1, y + h - 2, x + w - 1)

        def place(s, row):
            px = x + (w - len(s)) // 2 if center else x + 2
            self.put(px, row, s)

        row = y + 1
        if title is not None:
            place(title, row); row += 1
            self.putc(x, row, '+'); self.putc(x + w - 1, row, '+')
            self.hline(x + 1, x + w - 2, row); row += 1
        for line in body:
            place(line, row); row += 1
        if ticks:
            for tx in ticks:
                self.putc(tx, y + h - 1, '+')
        return (x, y, w, h)

    def a_down(self, x, y_from, y_target):
        self.vline(y_from, y_target - 2, x)
        self.putc(x, y_target - 1, 'v')

    def a_right(self, x_from, x_target, y):
        self.hline(x_from, x_target - 2, y)
        self.putc(x_target - 1, y, '>')

    def a_left(self, x_from, x_target, y):
        self.hline(x_target + 2, x_from, y)
        self.putc(x_target + 1, y, '<')

    def text(self):
        return '\n'.join(''.join(r).rstrip() for r in self.g)


G = Grid(118, 80)

# ===== columns ==========================================================
CG_X, CG_W = 40, 38
CG_R = CG_X + CG_W - 1
CG_CX = CG_X + CG_W // 2
BBG_X, SYNC_X, TAPE_X = 44, 56, 68
SW = 9
bbg_c, sync_c, tape_c = BBG_X + SW // 2, SYNC_X + SW // 2, TAPE_X + SW // 2

# ===== rows =============================================================
Y_CYB   = 0
Y_CMARK = 7
Y_CG    = 16
Y_SMALL = 29
Y_FOC   = 39
Y_PF    = 48

# ===== cyb : the robot (top band) =======================================
cyb = G.box(3, Y_CYB, 84, title="cyb : the robot",
            body=["soma : mind     pipeline : infer     worlds : face     honeycrisp : silicon"])
cyb_b = cyb[1] + cyb[3] - 1

# ===== cybermark : the WRITE language (cyb -> cybergraph) ================
CM_W = 40
CM_X = CG_CX - CM_W // 2
cmark = G.box(CM_X, Y_CMARK, CM_W, title="cybermark : the markup language",
              body=["address . navigate . compute inline",
                    "the markup is the graph"])
cm_b = cmark[1] + cmark[3] - 1
# cyb -> cybermark
G.a_down(CG_CX, cyb_b + 1, Y_CMARK)
G.put(CG_CX + 2, cyb_b + 2, "neurons write")

# ===== cybergraph (spine) ===============================================
cg = G.box(CG_X, Y_CG, CG_W, title="cybergraph",
           body=[":  link", "the  spine", "", "", "", ""],
           ticks=[bbg_c, sync_c, tape_c])
CG_BOT = cg[1] + cg[3] - 1
# cybermark -> cybergraph
G.a_down(CG_CX, cm_b + 1, Y_CG)
G.put(CG_CX + 2, cm_b + 2, "soma : intend . seal . link . subscribe . query")

# ===== cyberia (left) ===================================================
cyberia = G.box(3, Y_CG, 22, title="cyberia : social",
                body=[". contract", ". service", "mimi  midao  my"])
CYB_R = 3 + 22 - 1
G.a_right(CYB_R + 1, CG_X, Y_CG + 4)
G.put(CYB_R + 3, Y_CG + 3, "cyberlinks")
G.put(CYB_R + 3, Y_CG + 5, "focus . karma")

# ===== fs (right) =======================================================
fs = G.box(84, Y_CG, 16, title="fs", body=[":  mount"])
G.a_left(83, CG_R, Y_CG + 3)
G.put(CG_R + 2, Y_CG + 2, "patch")

# ===== mudra / plumb -> cybergraph (lower-left) =========================
def lbl_arrow(src, label, y):
    G.put(4, y, src)
    a = 4 + len(src) + 1
    lx = a + 5
    G.hline(a, lx - 1, y)
    G.put(lx, y, label)
    G.a_right(lx + len(label) + 1, CG_X, y)

lbl_arrow("mudra", "keys", Y_CG + 7)
lbl_arrow("plumb", "value", Y_CG + 8)

# ===== fan-out: cybergraph -> bbg / sync / tape =========================
for cx, lab in [(bbg_c, "store"), (sync_c, "sync"), (tape_c, "transmit")]:
    G.a_down(cx, CG_BOT + 1, Y_SMALL)
    G.put(cx + 1, CG_BOT + 2, lab)

G.box(BBG_X, Y_SMALL, SW, title="bbg",  body=[":store"], center=True)
G.box(SYNC_X, Y_SMALL, SW, title="sync", body=[":sync"], center=True)
G.box(TAPE_X, Y_SMALL, SW, title="tape", body=[":frame"], center=True)
SMALL_BOT = Y_SMALL + (2 + 2 + 1) - 1

# ===== bbg->proof (verify); sync->foculus(agree); tape->radio(frames) ===
G.a_down(bbg_c, SMALL_BOT + 1, Y_PF)
G.put(bbg_c + 1, SMALL_BOT + 2, "verify")
G.a_down(sync_c, SMALL_BOT + 1, Y_FOC)
G.put(sync_c + 1, SMALL_BOT + 2, "agree")
G.a_down(tape_c, SMALL_BOT + 1, Y_FOC)
G.put(tape_c + 1, SMALL_BOT + 2, "frames")

G.box(SYNC_X, Y_FOC, SW, title="foculus", body=[":agree"], center=True)
G.box(TAPE_X, Y_FOC, SW, title="radio", body=[":trans"], center=True)

# ===== PROOF FLOOR ======================================================
pf = G.box(3, Y_PF, 54, title="PROOF FLOOR",
           body=[
             "trident --.nox--.",
             " " * 16 + "v",
             "rune ---noun--> nox --trace--> zheng",
             " " * 20 + "<--open-- lens",
             "strata : math   +   hemera : hash",
           ])
PF_R = 3 + 54 - 1
PF_BOT = pf[1] + pf[3] - 1

# ===== languages (compute, right of proof floor) ========================
lang = G.box(68, Y_PF, 22, title="languages",
             body=["trident . prove", "rune    . eval",
                   "eidos   . proof", "inf     . query",
                   "nu      . shell", "rs      . jets"])
G.a_left(67, PF_R, Y_PF + 4)
G.put(PF_R + 3, Y_PF + 3, "lower")
G.put(PF_R + 3, Y_PF + 5, ".nox noun")
LANG_BOT = lang[1] + lang[3] - 1
CONTENT_BOT = max(PF_BOT, LANG_BOT)

# ===== conformance note =================================================
G.put(3, CONTENT_BOT + 1, "conformance : snapshot  --  one hemera fingerprint per encoding & mechanism")

# ===== read side ========================================================
RY = CONTENT_BOT + 3
G.put(3, RY, "read side, recomputed every step :")
G.put(3, RY + 1, "cybergraph --.graph--> tru --.model--> glia --features--> mir --> R-1.0")
G.put(27, RY + 2, "'----- phi* . positions . rank ----^")

# ===== neural : the MEANING language (stack -> cyber) ===================
NR_W = 44
NR_X = 45 - NR_W // 2          # centred on cyber centre (cyber x=3,w=84 -> 45)
neural = G.box(NR_X, RY + 4, NR_W, title="neural : the semantic language",
               body=["meaning emerges from cyberlinks",
                     "dialects . sentences . motifs . names"])
nr_b = neural[1] + neural[3] - 1
nr_cx = NR_X + NR_W // 2

# ===== cyber : collective consciousness (bottom band) ===================
CY_TOP = nr_b + 3
cyber = G.box(3, CY_TOP, 84, title="cyber : collective consciousness",
              body=["the whole graph converges to one mind",
                    "tru --> phi* --> foculus  ==>  cyberank . syntropy . CT-0 model"])
# neural -> cyber
G.a_down(nr_cx, nr_b + 1, CY_TOP)
G.put(nr_cx + 3, nr_b + 1, "the egregore thinks in neural")

print(G.text())
