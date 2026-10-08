// soft3 — the world: the living canvas behind every page. the page sets the hue through worldHue(h).
// one seed and one clock, so the world is the same on every page and keeps moving across them:
// the scene is drawn from a fixed seed, and time runs from the first visit of the session.
const SEED = 0x50f73;
const rnd = (() => { let a = SEED >>> 0; return () => { a |= 0; a = (a + 0x6d2b79f5) | 0; let t = Math.imul(a ^ (a >>> 15), 1 | a); t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t; return ((t ^ (t >>> 14)) >>> 0) / 4294967296; }; })();
const EPOCH = (() => { try { const k = "soft3.world.epoch"; let e = +sessionStorage.getItem(k); if (!e) { e = Date.now(); sessionStorage.setItem(k, e); } return e; } catch { return Date.now(); } })();
let organHue = 0;
window.worldHue = (h) => { organHue = h; };
/* ========== WORLD ========== */
const canvas = document.getElementById("world");
// a phone gets a cheaper canvas: no desynchronised buffer (it tears), a lower pixel ratio
const SMALL = matchMedia("(max-width: 900px), (pointer: coarse)").matches;
const screen = canvas.getContext("2d", {
    alpha: false,
    desynchronized: !SMALL,
});
// the glitch draws a burst frame here first, then slips slices of it onto the
// screen — never reading the screen back (a desynchronised canvas returns nothing,
// a phone pays a full readback per frame)
const off = document.createElement("canvas");
const octx = off.getContext("2d", { alpha: false });
let ctx = screen;
let W,
    H,
    dpr = 1,
    t0 = 0;
let mx = 0.5,
    my = 0.5,
    mxt = 0.5,
    myt = 0.5;

const trees = [],
    mushrooms = [],
    flowers = [],
    ferns = [];
const birds = [],
    fish = [],
    butterflies = [],
    spores = [],
    planets = [];

const hsl = (h, s, l, a = 1) =>
    `hsla(${((h % 360) + 360) % 360},${s}%,${l}%,${a})`;

function resize() {
    dpr = Math.min(devicePixelRatio || 1, SMALL ? 1.25 : 2.5);
    W = innerWidth;
    H = innerHeight;
    canvas.width = (W * dpr) | 0;
    canvas.height = (H * dpr) | 0;
    canvas.style.width = W + "px";
    canvas.style.height = H + "px";
    off.width = canvas.width;
    off.height = canvas.height;
    for (const c of [screen, octx]) {
        c.setTransform(dpr, 0, 0, dpr, 0, 0);
        c.imageSmoothingEnabled = true;
        c.imageSmoothingQuality = "high";
    }
    seed();
}

function seed() {
    trees.length =
        mushrooms.length =
        flowers.length =
        ferns.length =
            0;
    birds.length =
        fish.length =
        butterflies.length =
        spores.length =
        planets.length =
            0;

    // planets in the sky
    planets.push(
        {
            x: 0.12,
            y: 0.14,
            r: 0.035,
            hue: 25,
            rings: true,
            phase: 0.2,
        },
        {
            x: 0.28,
            y: 0.1,
            r: 0.018,
            hue: 200,
            rings: false,
            phase: 1.1,
        },
        {
            x: 0.88,
            y: 0.12,
            r: 0.028,
            hue: 320,
            rings: false,
            phase: 2.4,
        },
        {
            x: 0.72,
            y: 0.08,
            r: 0.012,
            hue: 140,
            rings: false,
            phase: 3.5,
        },
        {
            x: 0.42,
            y: 0.16,
            r: 0.01,
            hue: 50,
            rings: false,
            phase: 4.2,
        },
    );

    const nT = Math.floor(W / 55) + 8;
    for (let i = 0; i < nT; i++) {
        trees.push({
            x: (i + rnd() * 0.85) / nT,
            h: 0.28 + rnd() * 0.42,
            w: 0.03 + rnd() * 0.04,
            phase: rnd() * 6.28,
            lean: (rnd() - 0.5) * 0.12,
            hue: 110 + rnd() * 60,
            layers: 5 + ((rnd() * 4) | 0),
            kind: rnd() < 0.35 ? "pine" : "broad",
        });
    }

    for (let i = 0; i < Math.floor(22 + W / 50); i++) {
        mushrooms.push({
            x: rnd(),
            y: 0.55 + rnd() * 0.38,
            s: 0.4 + rnd() * 1.3,
            hue:
                [0, 8, 340, 280, 200, 45, 160][i % 7] +
                rnd() * 20,
            phase: rnd() * 6.28,
            spots: 5 + ((rnd() * 9) | 0),
            kind: ["fly", "bolete", "morel", "ink"][i % 4],
            tilt: (rnd() - 0.5) * 0.2,
        });
    }

    for (let i = 0; i < Math.floor(32 + (W * H) / 12000); i++) {
        flowers.push({
            x: rnd(),
            y: 0.5 + rnd() * 0.42,
            s: 0.35 + rnd() * 1,
            petals: 5 + ((rnd() * 6) | 0),
            hue: rnd() * 360,
            phase: rnd() * 6.28,
            type: ["daisy", "poppy", "sun", "orchid", "tulip"][
                i % 5
            ],
        });
    }

    for (let i = 0; i < 12; i++) {
        ferns.push({
            x: rnd(),
            y: 0.58 + rnd() * 0.35,
            s: 0.5 + rnd() * 1.1,
            phase: rnd() * 6.28,
            side: rnd() < 0.5 ? -1 : 1,
            hue: 130 + rnd() * 35,
        });
    }

    for (let i = 0; i < 14; i++) {
        butterflies.push({
            x: rnd(),
            y: 0.15 + rnd() * 0.45,
            s: 0.5 + rnd() * 0.9,
            hue: rnd() * 360,
            phase: rnd() * 6.28,
            speed: 0.3 + rnd() * 0.5,
        });
    }

    for (let i = 0; i < 10; i++) {
        birds.push({
            x: rnd(),
            y: 0.08 + rnd() * 0.28,
            s: 0.6 + rnd() * 0.8,
            phase: rnd() * 6.28,
            speed: 0.15 + rnd() * 0.25,
            hue: 20 + rnd() * 40,
        });
    }

    for (let i = 0; i < 18; i++) {
        fish.push({
            x: rnd(),
            y: 0.82 + rnd() * 0.14,
            s: 0.4 + rnd() * 0.8,
            phase: rnd() * 6.28,
            speed: 0.2 + rnd() * 0.35,
            hue: 180 + rnd() * 80,
        });
    }

    for (let i = 0; i < 70; i++) {
        spores.push({
            x: rnd(),
            y: rnd(),
            z: 0.3 + rnd() * 0.7,
            r: 0.5 + rnd() * 2,
            hue: rnd() * 360,
            phase: rnd() * 6.28,
            sp: 0.06 + rnd() * 0.18,
        });
    }
}

addEventListener("resize", resize);
addEventListener("pointermove", (e) => {
    mxt = e.clientX / W;
    myt = e.clientY / H;
});

function drawSky(t, breath) {
    const shift = organHue + t * 8;
    const g = ctx.createLinearGradient(0, 0, 0, H * 0.7);
    g.addColorStop(0, hsl(250 + shift * 0.2, 70, 12 + breath * 3));
    g.addColorStop(0.25, hsl(300 + shift * 0.3, 65, 22));
    g.addColorStop(0.45, hsl(35 + shift * 0.15, 80, 48));
    g.addColorStop(0.65, hsl(55, 75, 50));
    g.addColorStop(1, hsl(160, 45, 30));
    ctx.fillStyle = g;
    ctx.fillRect(0, 0, W, H);

    // aurora blobs
    ctx.save();
    ctx.globalCompositeOperation = "screen";
    for (let i = 0; i < 5; i++) {
        const cx =
            W * (0.15 + i * 0.18 + 0.03 * Math.sin(t * 0.25 + i));
        const cy = H * (0.12 + 0.06 * Math.cos(t * 0.2 + i));
        const r =
            Math.min(W, H) * (0.16 + 0.06 * Math.sin(t * 0.4 + i));
        const rg = ctx.createRadialGradient(cx, cy, 0, cx, cy, r);
        const hue = i * 55 + shift;
        rg.addColorStop(0, hsl(hue, 100, 65, 0.28));
        rg.addColorStop(1, hsl(hue, 80, 40, 0));
        ctx.fillStyle = rg;
        ctx.beginPath();
        ctx.arc(cx, cy, r, 0, Math.PI * 2);
        ctx.fill();
    }
    ctx.restore();

    // stars
    for (let i = 0; i < 60; i++) {
        const x = (((i * 97.3) % 100) / 100) * W;
        const y = (((i * 53.1) % 100) / 100) * H * 0.35;
        const tw = 0.3 + 0.7 * Math.sin(t * 2 + i);
        ctx.fillStyle = hsl(50, 30, 95, 0.15 + tw * 0.45);
        ctx.beginPath();
        ctx.arc(x, y, 0.6 + (i % 3) * 0.4 * tw, 0, Math.PI * 2);
        ctx.fill();
    }
}

function drawPlanets(t) {
    for (const p of planets) {
        const x =
            (p.x + Math.sin(t * 0.08 + p.phase) * 0.008) * W +
            (mx - 0.5) * 12;
        const y = (p.y + Math.cos(t * 0.06 + p.phase) * 0.006) * H;
        const r =
            p.r *
            Math.min(W, H) *
            (1 + 0.03 * Math.sin(t * 0.5 + p.phase));
        const hue = p.hue + organHue + t * 3;

        // glow
        const glow = ctx.createRadialGradient(
            x,
            y,
            0,
            x,
            y,
            r * 2.5,
        );
        glow.addColorStop(0, hsl(hue, 80, 60, 0.35));
        glow.addColorStop(1, hsl(hue, 60, 40, 0));
        ctx.fillStyle = glow;
        ctx.beginPath();
        ctx.arc(x, y, r * 2.5, 0, Math.PI * 2);
        ctx.fill();

        // body
        const body = ctx.createRadialGradient(
            x - r * 0.3,
            y - r * 0.3,
            0,
            x,
            y,
            r,
        );
        body.addColorStop(0, hsl(hue + 20, 70, 72));
        body.addColorStop(0.5, hsl(hue, 65, 45));
        body.addColorStop(1, hsl(hue - 20, 55, 22));
        ctx.fillStyle = body;
        ctx.beginPath();
        ctx.arc(x, y, r, 0, Math.PI * 2);
        ctx.fill();

        // bands
        ctx.save();
        ctx.beginPath();
        ctx.arc(x, y, r, 0, Math.PI * 2);
        ctx.clip();
        ctx.strokeStyle = hsl(hue + 30, 40, 60, 0.25);
        ctx.lineWidth = r * 0.12;
        for (let b = -2; b <= 2; b++) {
            ctx.beginPath();
            ctx.ellipse(
                x,
                y + b * r * 0.25,
                r * 0.95,
                r * 0.18,
                0.1,
                0,
                Math.PI * 2,
            );
            ctx.stroke();
        }
        ctx.restore();

        if (p.rings) {
            ctx.save();
            ctx.strokeStyle = hsl(hue + 40, 50, 70, 0.45);
            ctx.lineWidth = r * 0.15;
            ctx.beginPath();
            ctx.ellipse(
                x,
                y,
                r * 1.7,
                r * 0.45,
                -0.25,
                0,
                Math.PI * 2,
            );
            ctx.stroke();
            ctx.strokeStyle = hsl(hue + 60, 40, 80, 0.25);
            ctx.lineWidth = r * 0.08;
            ctx.beginPath();
            ctx.ellipse(
                x,
                y,
                r * 1.95,
                r * 0.52,
                -0.25,
                0,
                Math.PI * 2,
            );
            ctx.stroke();
            ctx.restore();
        }
    }
}

function drawSun(t, breath) {
    const sx = W * (0.78 + (mx - 0.5) * 0.03);
    const sy = H * (0.18 + (my - 0.5) * 0.02);
    const sr = Math.min(W, H) * (0.12 + breath * 0.015);

    // big corona
    const corona = ctx.createRadialGradient(
        sx,
        sy,
        0,
        sx,
        sy,
        sr * 3.5,
    );
    corona.addColorStop(0, hsl(50 + organHue, 100, 96, 1));
    corona.addColorStop(0.12, hsl(45, 100, 78, 0.95));
    corona.addColorStop(0.28, hsl(35, 100, 60, 0.55));
    corona.addColorStop(0.5, hsl(20, 100, 50, 0.2));
    corona.addColorStop(
        0.75,
        hsl(320 + organHue, 90, 50, 0.08),
    );
    corona.addColorStop(1, hsl(280, 70, 40, 0));
    ctx.fillStyle = corona;
    ctx.beginPath();
    ctx.arc(sx, sy, sr * 3.5, 0, Math.PI * 2);
    ctx.fill();

    // core
    const core = ctx.createRadialGradient(
        sx - sr * 0.15,
        sy - sr * 0.15,
        0,
        sx,
        sy,
        sr,
    );
    core.addColorStop(0, "#fffef5");
    core.addColorStop(0.4, hsl(48, 100, 75));
    core.addColorStop(1, hsl(30, 100, 55));
    ctx.fillStyle = core;
    ctx.beginPath();
    ctx.arc(sx, sy, sr, 0, Math.PI * 2);
    ctx.fill();

    // rays
    ctx.save();
    ctx.translate(sx, sy);
    ctx.rotate(t * 0.1);
    ctx.globalCompositeOperation = "screen";
    for (let i = 0; i < 36; i++) {
        const a = (i / 36) * Math.PI * 2;
        const len =
            sr * (2.4 + Math.sin(t * 1.2 + i) * 0.8 + breath * 0.4);
        ctx.strokeStyle = hsl(
            42 + i * 4,
            100,
            70,
            0.16 + 0.08 * Math.sin(t + i),
        );
        ctx.lineWidth = 1.5 + Math.sin(t + i) * 0.8;
        ctx.beginPath();
        ctx.moveTo(
            Math.cos(a) * sr * 1.05,
            Math.sin(a) * sr * 1.05,
        );
        ctx.lineTo(Math.cos(a) * len, Math.sin(a) * len);
        ctx.stroke();
    }
    ctx.restore();
}

function drawTree(tr, t, breath) {
    const x = tr.x * W + (mx - 0.5) * 16;
    const ground = H * 0.52;
    const th = tr.h * H * (1 + breath * 0.02);
    const tw = tr.w * W;
    ctx.save();
    ctx.translate(x, ground);
    ctx.rotate(tr.lean + Math.sin(t * 0.3 + tr.phase) * 0.03);

    const trunk = ctx.createLinearGradient(-tw, 0, tw, -th);
    trunk.addColorStop(0, hsl(25, 45, 16));
    trunk.addColorStop(1, hsl(30, 40, 32));
    ctx.fillStyle = trunk;
    ctx.beginPath();
    ctx.moveTo(-tw * 0.3, 0);
    ctx.bezierCurveTo(
        -tw * 0.18,
        -th * 0.45,
        -tw * 0.1,
        -th * 0.7,
        -tw * 0.06,
        -th * 0.85,
    );
    ctx.lineTo(tw * 0.06, -th * 0.85);
    ctx.bezierCurveTo(
        tw * 0.1,
        -th * 0.7,
        tw * 0.18,
        -th * 0.45,
        tw * 0.3,
        0,
    );
    ctx.closePath();
    ctx.fill();

    if (tr.kind === "pine") {
        for (let k = 0; k < tr.layers; k++) {
            const u = k / tr.layers;
            const cy = -th * (0.35 + u * 0.55);
            const cw =
                tw * (4.5 - u * 2.5) * (1 + 0.04 * Math.sin(t + k));
            const ch = tw * (2.2 - u * 0.5);
            const hue = tr.hue + organHue + k * 6;
            ctx.beginPath();
            ctx.moveTo(0, cy - ch);
            ctx.lineTo(-cw, cy + ch * 0.4);
            ctx.lineTo(cw, cy + ch * 0.4);
            ctx.closePath();
            const g = ctx.createLinearGradient(
                0,
                cy - ch,
                0,
                cy + ch,
            );
            g.addColorStop(0, hsl(hue + 20, 60, 42));
            g.addColorStop(1, hsl(hue - 10, 55, 22));
            ctx.fillStyle = g;
            ctx.fill();
        }
    } else {
        for (let k = 0; k < tr.layers; k++) {
            const ang = tr.phase + k * 0.85 + t * 0.1;
            const cx = Math.sin(ang) * tw * (1.4 + k * 0.3);
            const cy = -th * 0.7 - k * tw * 0.5;
            const r =
                tw * (2.5 + k * 0.5) * (1 + 0.04 * Math.sin(t + k));
            const hue =
                tr.hue +
                organHue +
                k * 8 +
                Math.sin(t * 0.3 + k) * 15;
            const g = ctx.createRadialGradient(
                cx - r * 0.2,
                cy - r * 0.2,
                0,
                cx,
                cy,
                r,
            );
            g.addColorStop(0, hsl(hue + 25, 70, 50, 0.9));
            g.addColorStop(0.5, hsl(hue, 60, 34, 0.85));
            g.addColorStop(1, hsl(hue - 20, 50, 16, 0));
            ctx.fillStyle = g;
            ctx.beginPath();
            ctx.ellipse(
                cx,
                cy,
                r * 1.15,
                r * 0.9,
                ang * 0.1,
                0,
                Math.PI * 2,
            );
            ctx.fill();
        }
    }
    ctx.restore();
}

function drawHills(t, breath) {
    const layers = [
        { y: 0.42, amp: 0.05, hue: 150, lit: 22, sp: 0.12, z: 0.3 },
        { y: 0.5, amp: 0.06, hue: 140, lit: 26, sp: 0.2, z: 0.5 },
        {
            y: 0.58,
            amp: 0.055,
            hue: 130,
            lit: 28,
            sp: 0.28,
            z: 0.7,
        },
    ];
    for (const L of layers) {
        ctx.beginPath();
        ctx.moveTo(0, H);
        for (let i = 0; i <= 56; i++) {
            const u = i / 56;
            const wave =
                Math.sin(u * 4.5 + t * L.sp) * L.amp +
                Math.sin(u * 10 - t * L.sp * 1.3) * L.amp * 0.35 +
                breath * 0.01 * L.z +
                (mx - 0.5) * 0.015 * L.z;
            ctx.lineTo(u * W, H * (L.y + wave));
        }
        ctx.lineTo(W, H);
        ctx.closePath();
        const hue =
            L.hue + organHue + Math.sin(t * 0.2) * 12;
        const g = ctx.createLinearGradient(0, H * L.y - 40, 0, H);
        g.addColorStop(0, hsl(hue + 25, 50, L.lit + 14));
        g.addColorStop(1, hsl(hue - 15, 45, L.lit - 6));
        ctx.fillStyle = g;
        ctx.fill();
    }
}

function drawWater(t, breath) {
    // wide lake + river tongue
    const y0 = H * 0.68;
    ctx.beginPath();
    ctx.moveTo(0, H);
    for (let x = 0; x <= W; x += 3) {
        const shore =
            y0 +
            Math.sin(x * 0.008 + t * 0.5) * 22 +
            Math.sin(x * 0.02 - t * 0.8) * 12 +
            Math.sin(x * 0.002 + t * 0.2) * 30 +
            breath * 8 +
            (my - 0.5) * 18;
        // deeper bay in middle
        const bay =
            Math.exp(-Math.pow((x / W - 0.45) / 0.35, 2)) *
            H *
            0.08;
        ctx.lineTo(x, shore - bay);
    }
    ctx.lineTo(W, H);
    ctx.closePath();

    const g = ctx.createLinearGradient(0, y0 - 80, 0, H);
    g.addColorStop(
        0,
        hsl(185 + organHue + t * 4, 70, 58, 0.7),
    );
    g.addColorStop(0.25, hsl(200, 75, 45, 0.8));
    g.addColorStop(0.55, hsl(215, 65, 32, 0.88));
    g.addColorStop(1, hsl(230, 55, 14, 0.95));
    ctx.fillStyle = g;
    ctx.fill();

    // reflections of sun
    ctx.save();
    ctx.globalCompositeOperation = "screen";
    const sx = W * 0.78;
    for (let i = 0; i < 12; i++) {
        const yy = y0 + 20 + i * 12 + Math.sin(t + i) * 4;
        const ww = 40 + i * 18 + Math.sin(t * 2 + i) * 10;
        ctx.fillStyle = hsl(
            45,
            100,
            80,
            0.08 + 0.04 * Math.sin(t * 2 + i),
        );
        ctx.beginPath();
        ctx.ellipse(
            sx + Math.sin(t + i) * 20,
            yy,
            ww,
            3 + i * 0.3,
            0,
            0,
            Math.PI * 2,
        );
        ctx.fill();
    }
    // wave sparkles
    for (let i = 0; i < 40; i++) {
        const x =
            ((t * 35 * (0.4 + (i % 4) * 0.15) + i * 61) %
                (W + 30)) -
            15;
        const y =
            y0 + 25 + Math.sin(t * 1.8 + i) * 18 + (i % 7) * 14;
        if (y > H) continue;
        ctx.fillStyle = hsl(
            50 + i * 5,
            100,
            92,
            0.12 + 0.1 * Math.sin(t * 3 + i),
        );
        ctx.beginPath();
        ctx.ellipse(x, y, 8 + (i % 3) * 3, 1.8, 0, 0, Math.PI * 2);
        ctx.fill();
    }
    ctx.restore();

    // foam line
    ctx.strokeStyle = hsl(0, 0, 100, 0.15);
    ctx.lineWidth = 1.5;
    ctx.beginPath();
    for (let x = 0; x <= W; x += 6) {
        const shore =
            y0 +
            Math.sin(x * 0.008 + t * 0.5) * 22 +
            Math.sin(x * 0.02 - t * 0.8) * 12 +
            Math.sin(x * 0.002 + t * 0.2) * 30 +
            breath * 8 -
            Math.exp(-Math.pow((x / W - 0.45) / 0.35, 2)) *
                H *
                0.08;
        if (x === 0) ctx.moveTo(x, shore);
        else ctx.lineTo(x, shore);
    }
    ctx.stroke();
}

function drawFish(f, t) {
    const x =
        (((f.x +
            t * f.speed * 0.04 +
            Math.sin(t * 0.5 + f.phase) * 0.02) %
            1.15) -
            0.05) *
        W;
    const y = f.y * H + Math.sin(t * 1.5 + f.phase) * 6;
    const s = f.s * 7;
    const dir = Math.sin(t * 0.2 + f.phase) >= 0 ? 1 : -1;
    ctx.save();
    ctx.translate(x, y);
    ctx.scale(dir, 1);
    ctx.globalAlpha = 0.55;
    const hue = f.hue + organHue + t * 10;
    ctx.fillStyle = hsl(hue, 70, 55);
    ctx.beginPath();
    ctx.ellipse(0, 0, s, s * 0.4, 0, 0, Math.PI * 2);
    ctx.fill();
    // tail
    ctx.beginPath();
    ctx.moveTo(-s * 0.8, 0);
    ctx.lineTo(
        -s * 1.4,
        -s * 0.45 + Math.sin(t * 8 + f.phase) * s * 0.15,
    );
    ctx.lineTo(
        -s * 1.4,
        s * 0.45 + Math.sin(t * 8 + f.phase) * s * 0.15,
    );
    ctx.closePath();
    ctx.fill();
    ctx.fillStyle = hsl(0, 0, 10, 0.5);
    ctx.beginPath();
    ctx.arc(s * 0.45, -s * 0.08, s * 0.08, 0, Math.PI * 2);
    ctx.fill();
    ctx.restore();
}

function drawBird(b, t) {
    const x = (((b.x + t * b.speed * 0.03) % 1.2) - 0.1) * W;
    const y = b.y * H + Math.sin(t * 1.2 + b.phase) * 10;
    const s = b.s * 6;
    const flap = Math.sin(t * 6 + b.phase);
    ctx.save();
    ctx.translate(x, y);
    ctx.strokeStyle = hsl(b.hue, 30, 15, 0.7);
    ctx.lineWidth = 1.4;
    ctx.lineCap = "round";
    ctx.beginPath();
    ctx.moveTo(-s, flap * s * 0.5);
    ctx.quadraticCurveTo(-s * 0.2, -s * 0.3 - flap * s * 0.2, 0, 0);
    ctx.quadraticCurveTo(
        s * 0.2,
        -s * 0.3 - flap * s * 0.2,
        s,
        flap * s * 0.5,
    );
    ctx.stroke();
    ctx.restore();
}

function drawFern(f, t, breath) {
    const x = f.x * W,
        y = f.y * H;
    const s = f.s * Math.min(W, H) * 0.035 * (1 + breath * 0.04);
    ctx.save();
    ctx.translate(x, y);
    ctx.rotate(f.side * 0.12 + Math.sin(t * 0.35 + f.phase) * 0.06);
    const hue = f.hue + organHue;
    ctx.strokeStyle = hsl(hue, 50, 30, 0.85);
    ctx.lineWidth = 1.3;
    ctx.beginPath();
    for (let i = 0; i <= 18; i++) {
        const u = i / 18;
        const px =
            Math.sin(u * 2.5 + t * 0.25 + f.phase) *
            s *
            0.25 *
            f.side;
        const py = -u * s * 4.5;
        if (i === 0) ctx.moveTo(px, py);
        else ctx.lineTo(px, py);
    }
    ctx.stroke();
    for (let i = 2; i < 16; i++) {
        const u = i / 18;
        const px =
            Math.sin(u * 2.5 + t * 0.25 + f.phase) *
            s *
            0.25 *
            f.side;
        const py = -u * s * 4.5;
        const len = s * (1.2 - u);
        for (const side of [-1, 1]) {
            ctx.beginPath();
            ctx.moveTo(px, py);
            ctx.quadraticCurveTo(
                px + side * len * 0.5,
                py - len * 0.2,
                px + side * len,
                py - len * 0.35,
            );
            ctx.strokeStyle = hsl(hue + i * 2, 55, 38, 0.65);
            ctx.lineWidth = 1;
            ctx.stroke();
        }
    }
    ctx.restore();
}

function drawMushroom(m, t, breath) {
    const x = m.x * W + (mx - 0.5) * 10;
    const y = m.y * H;
    const s =
        m.s *
        Math.min(W, H) *
        0.03 *
        (1 + breath * 0.05 + 0.03 * Math.sin(t * 0.6 + m.phase));
    const hue =
        m.hue + organHue + Math.sin(t * 0.25 + m.phase) * 12;
    ctx.save();
    ctx.translate(x, y);
    ctx.rotate(m.tilt);

    ctx.fillStyle = hsl(140, 30, 8, 0.2);
    ctx.beginPath();
    ctx.ellipse(0, s * 0.08, s * 0.85, s * 0.18, 0, 0, Math.PI * 2);
    ctx.fill();

    const stemH = s * (m.kind === "morel" ? 2 : 1.45);
    const stem = ctx.createLinearGradient(
        -s * 0.3,
        0,
        s * 0.3,
        -stemH,
    );
    stem.addColorStop(0, hsl(40, 30, 55));
    stem.addColorStop(1, hsl(35, 25, 90));
    ctx.fillStyle = stem;
    ctx.beginPath();
    ctx.moveTo(-s * 0.22, 0);
    ctx.bezierCurveTo(
        -s * 0.18,
        -stemH * 0.5,
        -s * 0.12,
        -stemH * 0.8,
        -s * 0.12,
        -stemH,
    );
    ctx.lineTo(s * 0.12, -stemH);
    ctx.bezierCurveTo(
        s * 0.12,
        -stemH * 0.8,
        s * 0.2,
        -stemH * 0.5,
        s * 0.28,
        0,
    );
    ctx.closePath();
    ctx.fill();

    const capY = -stemH;
    if (m.kind === "morel") {
        for (let k = 0; k < 5; k++) {
            const cy = capY - k * s * 0.26;
            const cr = s * (0.5 + k * 0.08);
            ctx.fillStyle = hsl(hue + k * 5, 50, 40 - k * 3);
            ctx.beginPath();
            ctx.ellipse(0, cy, cr, cr * 0.65, 0, 0, Math.PI * 2);
            ctx.fill();
        }
    } else {
        const capR =
            s *
            (m.kind === "fly" ? 1.5 : 1.3) *
            (1 + 0.05 * Math.sin(t * 0.5 + m.phase));
        const g = ctx.createRadialGradient(
            -capR * 0.25,
            capY - capR * 0.3,
            0,
            0,
            capY,
            capR,
        );
        g.addColorStop(0, hsl(hue + 15, 88, 70));
        g.addColorStop(0.45, hsl(hue, 82, 48));
        g.addColorStop(1, hsl(hue - 25, 70, 25));
        ctx.fillStyle = g;
        ctx.beginPath();
        ctx.ellipse(0, capY, capR, capR * 0.7, 0, Math.PI, 0, true);
        ctx.ellipse(
            0,
            capY,
            capR,
            capR * 0.22,
            0,
            0,
            Math.PI,
            false,
        );
        ctx.fill();

        if (m.kind === "fly" || m.kind === "bolete") {
            for (let i = 0; i < m.spots; i++) {
                const a = m.phase + i * 1.6;
                const pr = 0.22 + (i % 4) * 0.12;
                const px = Math.cos(a) * capR * pr * 0.8;
                const py =
                    capY -
                    Math.abs(Math.sin(a)) * capR * 0.4 * pr -
                    capR * 0.1;
                ctx.fillStyle = "rgba(255,255,255,0.88)";
                ctx.beginPath();
                ctx.ellipse(
                    px,
                    py,
                    s * 0.1,
                    s * 0.08,
                    a,
                    0,
                    Math.PI * 2,
                );
                ctx.fill();
            }
        }
        ctx.fillStyle = "rgba(255,255,255,0.35)";
        ctx.beginPath();
        ctx.ellipse(
            -capR * 0.28,
            capY - capR * 0.25,
            capR * 0.2,
            capR * 0.1,
            -0.4,
            0,
            Math.PI * 2,
        );
        ctx.fill();
    }

    ctx.globalCompositeOperation = "screen";
    const aura = ctx.createRadialGradient(
        0,
        capY,
        0,
        0,
        capY,
        s * 3,
    );
    aura.addColorStop(
        0,
        hsl(hue, 100, 65, 0.22 + 0.08 * Math.sin(t * 2 + m.phase)),
    );
    aura.addColorStop(1, hsl(hue, 80, 40, 0));
    ctx.fillStyle = aura;
    ctx.beginPath();
    ctx.arc(0, capY, s * 3, 0, Math.PI * 2);
    ctx.fill();
    ctx.restore();
}

function drawFlower(f, t, breath) {
    const x = f.x * W + Math.sin(t * 0.5 + f.phase) * 5;
    const y =
        f.y * H + Math.sin(t * 0.7 + f.phase) * 4 + breath * 4;
    const s = f.s * Math.min(W, H) * 0.024 * (1 + breath * 0.05);
    const hue = f.hue + organHue + t * 12;
    const open = 0.85 + 0.15 * Math.sin(t * 0.8 + f.phase);
    ctx.save();
    ctx.translate(x, y);
    ctx.strokeStyle = hsl(135, 50, 32, 0.85);
    ctx.lineWidth = 1.2 * f.s;
    ctx.beginPath();
    ctx.moveTo(0, 0);
    ctx.quadraticCurveTo(
        Math.sin(t + f.phase) * s,
        s * 2,
        0,
        s * 4.2,
    );
    ctx.stroke();

    const n = f.petals;
    if (f.type === "sun") {
        for (let i = 0; i < 16; i++) {
            const a = (i / 16) * Math.PI * 2 + t * 0.1;
            ctx.beginPath();
            ctx.ellipse(
                Math.cos(a) * s * open,
                Math.sin(a) * s * open,
                s * 0.45,
                s * 0.2,
                a,
                0,
                Math.PI * 2,
            );
            ctx.fillStyle = hsl(hue + i * 2, 90, 58);
            ctx.fill();
        }
        ctx.fillStyle = hsl(35, 80, 28);
        ctx.beginPath();
        ctx.arc(0, 0, s * 0.5, 0, Math.PI * 2);
        ctx.fill();
    } else if (f.type === "daisy") {
        for (let i = 0; i < n; i++) {
            const a = (i / n) * Math.PI * 2;
            ctx.beginPath();
            ctx.ellipse(
                Math.cos(a) * s * 0.7 * open,
                Math.sin(a) * s * 0.7 * open,
                s * 0.42,
                s * 0.18,
                a,
                0,
                Math.PI * 2,
            );
            ctx.fillStyle = hsl(0, 0, 96, 0.92);
            ctx.fill();
        }
        ctx.fillStyle = hsl(48, 95, 55);
        ctx.beginPath();
        ctx.arc(0, 0, s * 0.32, 0, Math.PI * 2);
        ctx.fill();
    } else {
        for (let i = 0; i < n; i++) {
            const a = (i / n) * Math.PI * 2;
            ctx.beginPath();
            ctx.ellipse(
                Math.cos(a) * s * 0.5 * open,
                Math.sin(a) * s * 0.45 * open,
                s * 0.6,
                s * 0.38,
                a,
                0,
                Math.PI * 2,
            );
            const g = ctx.createRadialGradient(0, 0, 0, 0, 0, s);
            g.addColorStop(0, hsl(hue + 20, 90, 75, 0.95));
            g.addColorStop(1, hsl(hue - 15, 75, 42, 0.7));
            ctx.fillStyle = g;
            ctx.fill();
        }
        ctx.fillStyle = hsl(hue + 40, 40, 12, 0.9);
        ctx.beginPath();
        ctx.arc(0, 0, s * 0.18, 0, Math.PI * 2);
        ctx.fill();
    }
    ctx.globalCompositeOperation = "screen";
    const glow = ctx.createRadialGradient(0, 0, 0, 0, 0, s * 2.5);
    glow.addColorStop(0, hsl(hue, 100, 70, 0.25));
    glow.addColorStop(1, hsl(hue, 80, 50, 0));
    ctx.fillStyle = glow;
    ctx.beginPath();
    ctx.arc(0, 0, s * 2.5, 0, Math.PI * 2);
    ctx.fill();
    ctx.restore();
}

function drawButterfly(b, t) {
    const x = (b.x + Math.sin(t * b.speed + b.phase) * 0.12) * W;
    const y =
        (b.y + Math.cos(t * b.speed * 0.8 + b.phase) * 0.08) * H;
    const s = b.s * 8;
    const flap = Math.abs(0.45 + 0.55 * Math.sin(t * 8 * b.speed + b.phase));
    const hue = b.hue + organHue + t * 20;
    ctx.save();
    ctx.translate(x, y);
    ctx.rotate(Math.sin(t * b.speed + b.phase) * 0.4);
    for (const side of [-1, 1]) {
        ctx.beginPath();
        ctx.ellipse(
            side * s * 0.5 * flap,
            0,
            s * 0.7 * flap,
            s * 0.9,
            side * 0.3,
            0,
            Math.PI * 2,
        );
        const g = ctx.createRadialGradient(
            side * s * 0.2,
            0,
            0,
            0,
            0,
            s,
        );
        g.addColorStop(0, hsl(hue, 90, 75, 0.9));
        g.addColorStop(1, hsl(hue + 60, 70, 40, 0.15));
        ctx.fillStyle = g;
        ctx.fill();
    }
    ctx.fillStyle = hsl(hue, 40, 15);
    ctx.beginPath();
    ctx.ellipse(0, 0, s * 0.1, s * 0.5, 0, 0, Math.PI * 2);
    ctx.fill();
    ctx.restore();
}

function drawSpores(t) {
    ctx.save();
    ctx.globalCompositeOperation = "screen";
    for (const s of spores) {
        const x =
            (s.x + Math.sin(t * s.sp + s.phase) * 0.03 * s.z) * W;
        const y = ((s.y + t * s.sp * 0.035 * s.z) % 1.1) * H;
        const hue = s.hue + organHue + t * 25;
        const r = s.r * s.z;
        const g = ctx.createRadialGradient(x, y, 0, x, y, r * 4);
        g.addColorStop(0, hsl(hue, 100, 75, 0.45 * s.z));
        g.addColorStop(1, hsl(hue, 80, 50, 0));
        ctx.fillStyle = g;
        ctx.beginPath();
        ctx.arc(x, y, r * 4, 0, Math.PI * 2);
        ctx.fill();
    }
    ctx.restore();
}

// the glitch — the stack has not crystallised yet, and the world says so: every
// ~30 s a burst of a few frames where slices of the picture slip sideways. rare,
// short, the same on every device; a phone gets a shorter burst with fewer
// slices. it goes away when the stack stabilises.
const GLITCH_EVERY = 30, GLITCH_LEN = SMALL ? 0.12 : 0.22;
let jolted = false;
function glitchPhase(t) {
    const phase = (t + 7) % GLITCH_EVERY;
    return phase < GLITCH_LEN ? phase / GLITCH_LEN : -1;
}
// blit the burst frame from `off` to the screen, slices displaced
function glitchBlit(t, k) {
    const n = SMALL ? 2 : 3 + ((k * 9) | 0) % 4;
    const seedT = Math.floor(t / GLITCH_EVERY) * 977;
    screen.drawImage(off, 0, 0, off.width, off.height, 0, 0, W, H);
    for (let i = 0; i < n; i++) {
        const r1 = ((Math.sin(seedT + i * 12.9898 + k * 3) * 43758.5453) % 1 + 1) % 1;
        const r2 = ((Math.sin(seedT + i * 78.233 + k * 5) * 43758.5453) % 1 + 1) % 1;
        const y = r1 * H, h = 6 + r2 * 40, dx = (r2 - 0.5) * 36 * (1 - k);
        screen.drawImage(off, 0, y * dpr, off.width, h * dpr, dx, y, W, h);
    }
    // on a desktop the glass hides most of the world, so the page itself jolts
    // too: three stepped offsets, transform only, back in place before the end
    if (!SMALL) {
        const shell = document.querySelector(".shell");
        if (shell) {
            const step = (k * 3) | 0;
            const jx = k < 0.7 ? [1, -1, 0.5][step] * (5 + 5 * (seedT % 3)) : 0;
            shell.style.transform = jx ? `translate3d(${jx}px,0,0)` : "";
            jolted = !!jx;
        }
    }
    if (!SMALL && k < 0.35) {
        screen.globalCompositeOperation = "lighter";
        screen.globalAlpha = 0.07;
        screen.drawImage(off, 0, 0, off.width, off.height, 4, 0, W, H);
        screen.globalAlpha = 1;
        screen.globalCompositeOperation = "source-over";
    }
}
function frame() {
    const t = (Date.now() - EPOCH) / 1000;
    const gk = glitchPhase(t);
    ctx = gk < 0 ? screen : octx;
    mx += (mxt - mx) * 0.05;
    my += (myt - my) * 0.05;
    const breath = 0.5 + 0.5 * Math.sin(t * 0.48);

    drawSky(t, breath);
    drawPlanets(t);
    drawSun(t, breath);
    for (const b of birds) drawBird(b, t);
    for (const tr of trees) drawTree(tr, t, breath);
    drawHills(t, breath);
    for (const f of ferns) drawFern(f, t, breath);
    drawWater(t, breath);
    for (const f of fish) drawFish(f, t);

    const life = [
        ...mushrooms.map((m) => ({
            y: m.y,
            d: () => drawMushroom(m, t, breath),
        })),
        ...flowers.map((f) => ({
            y: f.y,
            d: () => drawFlower(f, t, breath),
        })),
    ].sort((a, b) => a.y - b.y);
    for (const L of life) L.d();

    for (const b of butterflies) drawButterfly(b, t);
    drawSpores(t);

    // mild grade
    ctx.save();
    ctx.globalCompositeOperation = "soft-light";
    ctx.fillStyle = hsl(
        300 + organHue + t * 10,
        70,
        50,
        0.1 + breath * 0.05,
    );
    ctx.fillRect(0, 0, W, H);
    ctx.restore();

    // edge vignette only (center stays vivid; glass handles text)
    const vig = ctx.createRadialGradient(
        W / 2,
        H * 0.45,
        H * 0.25,
        W / 2,
        H / 2,
        H * 0.95,
    );
    vig.addColorStop(0, "rgba(0,0,0,0)");
    vig.addColorStop(1, `rgba(4,10,8,${0.22 + breath * 0.08})`);
    ctx.fillStyle = vig;
    ctx.fillRect(0, 0, W, H);

    if (gk >= 0) glitchBlit(t, gk);
    else if (jolted) { const shell = document.querySelector(".shell"); if (shell) shell.style.transform = ""; jolted = false; }
    requestAnimationFrame(frame);
}

resize();
requestAnimationFrame(frame);
