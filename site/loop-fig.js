// soft3 — loop-fig: the seven-verb ring as one SVG string. a port of the inline render in /loop/index.html.
// data: VERBS and ARCS from /data.js (never copied here). no DOM, no iframes — call it, insert the string.
//
//   window.loopFig(opts) → string
//     opts.id     marker/gradient id prefix, unique per instance on a page (default "loop")
//     opts.href   (verb) => url | null — what a verb circle links to (default "/chart/?sel=" + verb.sel; null = no links)
//     opts.focus  verb id to keep bright while the other verbs and arcs dim (default none)
//     opts.heart  false to drop the centre text (default true)
//     opts.title  centre title (default "the loop"), opts.sub centre line (default "one graph · one proof · one mind")
//     opts.class  class on the <svg> (default "loopfig")
//     opts.verbs / opts.arcs  override the data (defaults window.VERBS / window.ARCS)
//   the svg carries viewBox 0 0 1400 1120, preserveAspectRatio xMidYMid meet; size it with CSS.
//   every verb group has class="verb" data-id data-h data-c so a host can wire hover → worldHue(h - 150).
(() => {
    // /data.js declares its tables with top-level const: global lexical bindings, not window properties — resolve them by name
    const G = (name) => { try { return (0, eval)(name); } catch { return window[name]; } };
    const esc = (s) => String(s).replace(/[&<>"]/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" }[c]));
    const attrs = (o) => Object.entries(o).filter(([, v]) => v !== null && v !== undefined && v !== "").map(([k, v]) => ` ${k}="${esc(v)}"`).join("");
    const el = (tag, at, inner) => `<${tag}${attrs(at)}>${inner || ""}</${tag}>`;
    const SANS = "Play, system-ui, sans-serif", MONO = "'JetBrains Mono', ui-monospace, monospace";
    // fill and font go in style= so a host stylesheet (fig.css paints every `.fig svg text` cream) cannot repaint them
    const text = (x, y, str, o) => el("text", { x: r1(x), y: r1(y), "text-anchor": o.anchor || "middle", "font-size": o.fs, "font-weight": o.w, "letter-spacing": o.ls, "paint-order": o.stroke ? "stroke" : null, stroke: o.stroke || null, "stroke-width": o.stroke ? o.sw || 8 : null, style: `fill:${o.fill};font-family:${o.mono ? MONO : SANS}` }, o.raw ? str : esc(str));
    const r1 = (v) => Math.round(v * 10) / 10;

    function loopFig(opts = {}) {
        const VERBS = opts.verbs || G("VERBS"), ARCS = opts.arcs || G("ARCS");
        if (!VERBS || !ARCS) throw new Error("loopFig: VERBS and ARCS are missing — load /data.js first");
        const id = opts.id || "loop";
        const href = opts.href === undefined ? (v) => "/chart/?sel=" + v.sel : opts.href;
        const focus = opts.focus || null;
        const W = 1400, H = 1120, CX = 700, CY = 552, R = 340, VR = 88, N = VERBS.length;
        const ang = {}; VERBS.forEach((v, i) => { ang[v.id] = -Math.PI / 2 + (i * 2 * Math.PI) / N; });
        const P = (a, r = R) => ({ x: CX + r * Math.cos(a), y: CY + r * Math.sin(a) });
        const dim = (vid) => (focus && vid !== focus ? 0.3 : 1);

        // defs: one arrow head per verb in the verb's colour, and the heart glow
        let defs = VERBS.map((v) => el("marker", { id: `${id}-arr-${v.id}`, viewBox: "0 0 10 10", refX: 8, refY: 5, markerWidth: 26, markerHeight: 26, markerUnits: "userSpaceOnUse", orient: "auto" }, el("path", { d: "M 0 0 L 10 5 L 0 10 z", fill: v.c }))).join("");
        defs += el("radialGradient", { id: `${id}-heart`, cx: "50%", cy: "50%", r: "50%" }, el("stop", { offset: "0%", "stop-color": "#7dffc4", "stop-opacity": 0.16 }) + el("stop", { offset: "100%", "stop-color": "#7dffc4", "stop-opacity": 0 }));

        // the ring: seven coloured arcs, each the verb's colour, arrowed into the next; labels ride outside the ring
        let arcs = el("circle", { cx: CX, cy: CY, r: 240, fill: `url(#${id}-heart)` });
        const gap = Math.asin((VR + 10) / R);
        ARCS.forEach(([a, b, label]) => {
            const va = VERBS.find((v) => v.id === a); if (!va || ang[b] === undefined) return;
            let a0 = ang[a] + gap, a1 = ang[b] - gap; if (a1 < a0) a1 += 2 * Math.PI; // leave the verb circles clear, always go clockwise
            const p0 = P(a0), p1 = P(a1);
            // near the top and bottom verbs the label steps further out, past the verb's sentence
            const am = (a0 + a1) / 2, sn = Math.sin(am), mid = P(am, sn > 0.85 ? R - 150 : Math.abs(sn) > 0.8 ? R + 118 : sn > 0.4 ? R + 110 : R + 72);
            arcs += el("g", { class: "arc", "data-from": a, "data-to": b, opacity: dim(a) === 1 ? null : dim(a) },
                el("path", { d: `M ${r1(p0.x)} ${r1(p0.y)} A ${R} ${R} 0 0 1 ${r1(p1.x)} ${r1(p1.y)}`, fill: "none", stroke: va.c, "stroke-opacity": 0.85, "stroke-width": 9, "stroke-linecap": "round", "marker-end": `url(#${id}-arr-${a})` }) +
                text(mid.x, mid.y + 6, label, { fs: 18, fill: "rgba(255,248,240,0.85)", stroke: "#06100a", sw: 8 }));
        });

        // the heart
        let heart = "";
        if (opts.heart !== false) {
            heart += text(CX, CY - 6, opts.title === undefined ? "the loop" : opts.title, { fs: 50, w: 700, ls: 6, fill: "#7dffc4" });
            heart += text(CX, CY + 34, opts.sub === undefined ? "one graph · one proof · one mind" : opts.sub, { fs: 19, fill: "rgba(255,248,240,0.7)", mono: true });
        }

        // the verbs: the circle names the verb and who does it; the sentence and its parts sit beside the circle
        let verbs = "";
        VERBS.forEach((v) => {
            const a = ang[v.id], p = P(a), ink = v.ink || v.c;
            let g = el("circle", { cx: r1(p.x), cy: r1(p.y), r: VR, fill: "#06100a", "fill-opacity": 0.85 });
            g += el("circle", { cx: r1(p.x), cy: r1(p.y), r: VR, fill: v.c, "fill-opacity": 0.22, stroke: v.c, "stroke-width": 3 });
            const fs = v.id.length > 6 ? 23 : v.id.length > 5 ? 32 : 38;
            g += text(p.x, p.y - 6, v.id.toUpperCase(), { fs, w: 700, ls: v.id.length > 6 ? 1 : 3, fill: ink });
            g += text(p.x, p.y + 26, v.who, { fs: 17, fill: "rgba(255,248,240,0.9)" });
            // the block: sentence lines, then the parts line — above the top verb, beside the side verbs, below the bottom one
            // two lines, always: what it becomes, then the one property after the dash
            const c = Math.cos(a), sn = Math.sin(a), lines = v.does.split(" — ").map((t, i) => (i ? "— " : "") + t), LH = 25, block = lines.length * LH + 24;
            let x = p.x, y0, anchor = "middle";
            if (sn < -0.9) { y0 = p.y - VR - 22 - block + LH; }
            else if (sn > 0.85) { y0 = p.y + VR + 36; x = p.x + (c > 0 ? 1 : -1) * 16; }
            else { anchor = c > 0 ? "start" : "end"; x = p.x + (c > 0 ? 1 : -1) * (VR + 24); y0 = p.y - block / 2 + LH; }
            const spans = lines.map((ln, i) => el("tspan", { x: r1(x), dy: i ? LH : 0 }, esc(ln))).join("");
            g += text(x, y0, spans, { fs: 19, anchor, fill: "rgba(255,248,240,0.92)", stroke: "#06100a", sw: 8, raw: true });
            g += text(x, y0 + (lines.length - 1) * LH + 24, v.parts.join(" · "), { fs: 15, anchor, fill: ink, mono: true, stroke: "#06100a", sw: 6 });
            const link = href ? href(v) : null;
            if (link) g = el("a", { href: link, "aria-label": `${v.id} — ${v.sel}` }, g);
            verbs += el("g", { class: "verb", "data-id": v.id, "data-h": v.h, "data-c": v.c, "data-sel": v.sel, opacity: dim(v.id) === 1 ? null : dim(v.id), style: link ? "cursor:pointer" : null }, g);
        });

        return el("svg", { xmlns: "http://www.w3.org/2000/svg", class: opts.class || "loopfig", viewBox: `0 0 ${W} ${H}`, preserveAspectRatio: "xMidYMid meet", role: "img", "aria-label": "the loop: " + VERBS.map((v) => v.id).join(", ") },
            el("defs", {}, defs) + el("g", { class: "arcs" }, arcs) + el("g", { class: "heart" }, heart) + el("g", { class: "verbs" }, verbs));
    }
    loopFig.SIZE = { w: 1400, h: 1120 };
    window.loopFig = loopFig;
})();
