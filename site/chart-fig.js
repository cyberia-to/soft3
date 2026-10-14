// soft3 — chart-fig: the dependency chart as one SVG string. a port of the inline render in /chart/index.html.
// data: NODES and EDGES from /data.js (never copied, never mutated — the chart page rewrites n.f in place; this one does not).
//
//   window.chartFig(opts) → string
//     opts.id      marker/gradient id prefix, unique per instance on a page (default "chart")
//     opts.focus   a node id (hub or component): its wires go bright, the rest fade, incoming wires get their verb
//                  labels and the node gets its "verb → n" label — the chart page's select() without the side panels
//     opts.href    (node) => url | null — wraps hubs and pills in <a> (default none; the page selects on click instead)
//     opts.class   class on the <svg> (default "chartfig")
//     opts.nodes / opts.edges  override the data (defaults window.NODES / window.EDGES)
//   the svg carries viewBox 0 0 1700 1700, preserveAspectRatio xMidYMid meet; size it with CSS.
//   hubs are <g class="hub" data-id data-f data-h>, components <g class="node" data-id data-f data-h>; wires carry a <title>.
//
//   window.chartFig.neighbours(id, nodes?, edges?) → { node, facet, colour, ins: [...], outs: [...] }
//     one row per wire: { id, verb, s, k, product } — the side-panel data of the chart page, for a host to render as text.
//   window.chartFig.FACETS — the seven facets + apps: { c, t, h, q } (colour, text colour on it, hue, the question)
(() => {
    // /data.js declares its tables with top-level const: global lexical bindings, not window properties — resolve them by name
    const G = (name) => { try { return (0, eval)(name); } catch { return window[name]; } };
    const esc = (s) => String(s).replace(/[&<>"]/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" }[c]));
    const attrs = (o) => Object.entries(o).filter(([, v]) => v !== null && v !== undefined && v !== "").map(([k, v]) => ` ${k}="${esc(v)}"`).join("");
    const el = (tag, at, inner) => `<${tag}${attrs(at)}>${inner || ""}</${tag}>`;
    const r1 = (v) => Math.round(v * 10) / 10;
    const MONO = "'JetBrains Mono', ui-monospace, monospace";
    const text = (x, y, str, o) => el("text", { x: r1(x), y: r1(y), "text-anchor": o.anchor || "middle", "font-size": r1(o.fs), "font-weight": o.w, style: `fill:${o.fill};font-family:${MONO}` }, esc(str));

    const FACETS = {
        form: { c: "#00ff00", t: "#0a120e", h: 120, q: "what are the rules?" },
        mass: { c: "#0000ff", t: "#fff8f0", h: 240, q: "what is it made of?" },
        space: { c: "#4000ff", t: "#fff8f0", h: 255, q: "where does it happen?" },
        life: { c: "#8000ff", t: "#fff8f0", h: 270, q: "what lives?" },
        word: { c: "#ff0000", t: "#fff8f0", h: 0, q: "what does it mean?" },
        work: { c: "#ff8000", t: "#0a120e", h: 30, q: "who does the work?" },
        play: { c: "#ffff00", t: "#0a120e", h: 60, q: "how do we coordinate?" },
        apps: { c: "#fff8f0", t: "#0a120e", h: 150, q: "" },
    };
    const LADDER = ["nox", "trident", "joy", "cybergraph", "neural", "cy", "soft3", "cyb", "cyber", "cyberia"];
    // the chart paints by rung, not by a node's own facet: rung k wears the k-th colour of the flag
    const PF = { 1: "form", 2: "mass", 3: "space", 4: "life", 5: "word", 6: "work", 7: "play" };
    const facetOf = (n) => (n.f === "apps" ? "apps" : PF[n.layer] || "apps");
    const colourOf = (n) => FACETS[facetOf(n)].c;
    const productOf = (n) => (n.f === "apps" ? null : LADDER[n.layer - 1]);

    // ── layout: hubs at even angles, components on a wobbled outer ring, apps in the centre ──
    const CX = 850, CY = 850, HUB_R = 392, RING_R = 672, WOBBLE = 84;
    const PILL = (n) => 34 + n.id.length * 19, GAP = 22, PH = 52, PF_SIZE = 32;
    function layout(NODES) {
        const byId = Object.fromEntries(NODES.map((n) => [n.id, n]));
        const byLayer = {}; NODES.forEach((n) => (byLayer[n.layer] ||= []).push(n));
        const ringList = [];
        for (let k = 1; k <= 7; k++) (byLayer[k] || []).filter((n) => n.id !== LADDER[k - 1]).forEach((n) => ringList.push(n));
        let K = 1, angles = [];
        for (let tries = 0; tries < 6; tries++) {
            const rowR = [RING_R - WOBBLE, RING_R + WOBBLE]; const lastA = [null, null], lastW = [0, 0]; let prev = -Infinity; angles = [];
            ringList.forEach((n, i) => { const row = i % 2, w = PILL(n) * K, r = rowR[row]; let a = prev === -Infinity ? 0 : prev + 0.02; if (lastA[row] !== null) a = Math.max(a, lastA[row] + ((lastW[row] + w) / 2 + GAP) / r); angles.push(a); lastA[row] = a; lastW[row] = w; prev = a; });
            const n0 = ringList[0], nl = ringList[ringList.length - 1];
            const close = angles[angles.length - 1] + ((PILL(nl) + PILL(n0)) * K / 2 + GAP) / (RING_R - WOBBLE);
            const s = (2 * Math.PI) / close;
            if (s >= 0.98 && s <= 1.06) { angles = angles.map((x) => x * s); break; }
            K *= Math.min(Math.sqrt(s), 1.12);
        }
        const pos = {}, hubPos = {};
        ringList.forEach((n, i) => { const a = -Math.PI / 2 + angles[i] + 0.06; const r = RING_R + (i % 2 ? WOBBLE : -WOBBLE); pos[n.id] = { x: CX + r * Math.cos(a), y: CY + r * Math.sin(a), a }; });
        for (let k = 1; k <= 7; k++) { const a0 = -Math.PI / 2 + (k - 0.5) * (2 * Math.PI / 7); hubPos[LADDER[k - 1]] = { x: CX + HUB_R * Math.cos(a0), y: CY + HUB_R * Math.sin(a0), a: a0 }; }
        Object.assign(pos, hubPos);
        pos["cyberia"] = { x: CX, y: CY - 84 }; pos["cyb"] = { x: CX - 98, y: CY + 56 }; pos["cyber"] = { x: CX + 104, y: CY + 56 };
        // one font for everything named: pills, hubs and the products share PF_SIZE * K
        const NAME_FS = PF_SIZE * K, HUB_RAD = Math.max(122, Math.ceil((10 * 19.2 * K) / 2) + 18);
        const nodeBox = {};
        NODES.filter((n) => !n.hub).forEach((n) => { const p = pos[n.id]; if (!p) return; const apps = n.f === "apps"; const w = PILL(n) * K + (apps ? 22 : 0), h = PH * K + (apps ? 10 : 0); nodeBox[n.id] = { x: p.x - w / 2, y: p.y - h / 2, w, h }; });
        return { byId, pos, hubPos, K, NAME_FS, HUB_RAD, nodeBox };
    }

    // ── labels that do not collide: walk the curve, keep out of boxes, hubs and each other ──
    const bez = (e, t) => ({ x: (1 - t) * (1 - t) * e.A.x + 2 * (1 - t) * t * e.cx + t * t * e.B.x, y: (1 - t) * (1 - t) * e.A.y + 2 * (1 - t) * t * e.cy + t * t * e.B.y });
    function placeLabels(list, L) {
        const placed = Object.values(L.nodeBox).map((b) => ({ ...b }));
        const FS = 20; let out = "";
        const overlap = (b, o) => Math.max(0, Math.min(b.x + b.w, o.x + o.w) - Math.max(b.x, o.x)) * Math.max(0, Math.min(b.y + b.h, o.y + o.h) - Math.max(b.y, o.y));
        const cost = (b) => placed.reduce((s, o) => s + overlap(b, o), 0) + Object.values(L.hubPos).reduce((s, h) => { const d = Math.hypot(h.x - (b.x + b.w / 2), h.y - (b.y + b.h / 2)); return s + (d < L.HUB_RAD + 16 ? (L.HUB_RAD + 16 - d) * 40 : 0); }, 0);
        list.forEach((e) => {
            const str = e.v; const w = str.length * FS * 0.62 + 14, h = FS + 8; let best = null, bestCost = Infinity;
            for (const t of [0.5, 0.42, 0.58, 0.34, 0.66, 0.26, 0.74, 0.18, 0.82]) for (const side of [0, 1, -1]) for (const off of [22, 40]) {
                const p = bez(e, t), q = bez(e, t + 0.01); const nx = -(q.y - p.y), ny = q.x - p.x, nl = Math.hypot(nx, ny) || 1;
                const x = p.x + (nx / nl) * side * off, y = p.y + (ny / nl) * side * off; const bx = { x: x - w / 2, y: y - h / 2, w, h };
                const cst = cost(bx) + Math.abs(t - 0.5) * 30 + (side ? off : 0);
                if (cst < bestCost) { bestCost = cst; best = bx; }
                if (cst === 0) break;
            }
            placed.push(best);
            out += el("rect", { x: r1(best.x), y: r1(best.y), width: r1(best.w), height: r1(best.h), rx: 6, fill: "#06100a", "fill-opacity": 0.85 });
            out += text(best.x + best.w / 2, best.y + best.h / 2 + 7, str, { fs: FS, fill: "#fff8f0" });
        });
        return out;
    }

    function chartFig(opts = {}) {
        const NODES = opts.nodes || G("NODES"), EDGES = opts.edges || G("EDGES");
        if (!NODES || !EDGES) throw new Error("chartFig: NODES and EDGES are missing — load /data.js first");
        const id = opts.id || "chart", focus = opts.focus && NODES.some((n) => n.id === opts.focus) ? opts.focus : null, href = opts.href || null;
        const L = layout(NODES), { byId, pos, hubPos, K, NAME_FS, HUB_RAD, nodeBox } = L;

        let defs = Object.entries(FACETS).map(([f, v]) => el("marker", { id: `${id}-arr-${f}`, viewBox: "0 0 10 10", refX: 9, refY: 5, markerWidth: 8, markerHeight: 8, orient: "auto-start-reverse" }, el("path", { d: "M 0 0 L 10 5 L 0 10 z", fill: v.c }))).join("");
        defs += el("radialGradient", { id: `${id}-heart`, cx: "50%", cy: "50%", r: "50%" }, el("stop", { offset: "0%", "stop-color": "#7dffc4", "stop-opacity": 0.14 }) + el("stop", { offset: "100%", "stop-color": "#7dffc4", "stop-opacity": 0 }));

        // the set of nodes a focus keeps bright: itself and whatever shares a wire with it
        const near = new Set(focus ? [focus] : []);
        const edgeInfo = EDGES.map((e) => { const A = pos[e.a], B = pos[e.b]; if (!A || !B || !byId[e.a]) return null; const mx = (A.x + B.x) / 2, my = (A.y + B.y) / 2, pull = 0.14; return { ...e, A, B, cx: mx + (CX - mx) * pull, cy: my + (CY - my) * pull }; }).filter(Boolean);
        if (focus) edgeInfo.forEach((e) => { if (e.a === focus || e.b === focus) { near.add(e.a); near.add(e.b); } });
        const fade = (nid) => (focus && !near.has(nid) ? 0.3 : null);

        // hubs: the heart, the two rings, the seven products, the dashed spokes to their components
        let hubs = el("circle", { cx: CX, cy: CY, r: 330, fill: `url(#${id}-heart)` });
        hubs += el("circle", { cx: CX, cy: CY, r: 212, fill: "#06100a", "fill-opacity": 0.35, stroke: "#7dffc4", "stroke-opacity": 0.3, "stroke-width": 1.4 });
        hubs += text(CX, CY + 146, "products built on soft3", { fs: 20, fill: "rgba(125,255,196,0.8)" });
        hubs += el("circle", { cx: CX, cy: CY, r: HUB_R, fill: "none", stroke: "rgba(255,248,240,0.12)", "stroke-width": 1.2, "stroke-dasharray": "4 8" });
        hubs += el("circle", { cx: CX, cy: CY, r: RING_R, fill: "none", stroke: "rgba(255,248,240,0.07)", "stroke-width": 1.2 });
        NODES.filter((n) => n.hub && hubPos[n.id]).forEach((n) => {
            const { x, y } = hubPos[n.id], c = colourOf(n), f = facetOf(n);
            let g = el("circle", { cx: r1(x), cy: r1(y), r: HUB_RAD, fill: "#06100a", "fill-opacity": 0.82 });
            g += el("circle", { cx: r1(x), cy: r1(y), r: HUB_RAD, fill: c, "fill-opacity": 0.2, stroke: c, "stroke-opacity": 0.7, "stroke-width": 2.2 });
            g += text(x, y + 4 * K, n.id, { fs: NAME_FS, w: 700, fill: c });
            g += text(x, y + 36 * K, n.role || "", { fs: 20 * K, fill: "rgba(255,248,240,0.85)" });
            const link = href ? href(n) : null; if (link) g = el("a", { href: link }, g);
            hubs += el("g", { class: "hub" + (n.id === focus ? " focus" : ""), "data-id": n.id, "data-f": f, "data-h": FACETS[f].h, opacity: fade(n.id), style: link ? "cursor:pointer" : null }, g);
        });
        NODES.forEach((n) => { if (n.hub || n.f === "apps") return; const h = hubPos[LADDER[n.layer - 1]], p = pos[n.id]; if (!h || !p) return; hubs += el("line", { x1: r1(h.x + (p.x - h.x) * 0.45), y1: r1(h.y + (p.y - h.y) * 0.45), x2: r1(p.x), y2: r1(p.y), stroke: colourOf(n), "stroke-opacity": 0.16, "stroke-width": 1.2 }); });

        // wires: a quadratic pulled toward the centre, the producer's colour and arrow head; dashed when specified, not yet code
        let edges = "";
        edgeInfo.forEach((e) => {
            const f = facetOf(byId[e.a]), hot = focus && (e.a === focus || e.b === focus);
            const op = focus ? (hot ? 0.95 : 0.03) : e.s ? 0.18 : 0.26, sw = focus ? (hot ? 3 : 1.2) : e.s ? 1.4 : 1.8;
            edges += el("path", { class: "wire", d: `M ${r1(e.A.x)} ${r1(e.A.y)} Q ${r1(e.cx)} ${r1(e.cy)} ${r1(e.B.x)} ${r1(e.B.y)}`, fill: "none", stroke: FACETS[f].c, "stroke-opacity": op, "stroke-width": sw, "stroke-dasharray": e.s ? "6 8" : null, "marker-end": `url(#${id}-arr-${f})`, "data-a": e.a, "data-b": e.b },
                el("title", {}, esc(`${e.a} → ${e.b} · ${e.v}${e.k && e.k.length ? " (" + e.k.join(", ") + ")" : " · specified"}`)));
        });

        // components: pills in the product's colour; the three apps in the centre, cream on ink
        let nodes = "";
        NODES.filter((n) => !n.hub && nodeBox[n.id]).forEach((n) => {
            const { x, y } = pos[n.id], b = nodeBox[n.id], apps = n.f === "apps", f = facetOf(n);
            let g = el("rect", { x: r1(b.x), y: r1(b.y), width: r1(b.w), height: r1(b.h), rx: r1(b.h / 2), fill: apps ? "rgba(6,16,10,0.85)" : colourOf(n), stroke: apps ? "#fff8f0" : "rgba(255,255,255,0.3)", "stroke-width": apps ? 1.8 : 1 });
            g += text(x, y + 11 * K, n.id, { fs: NAME_FS, w: 700, fill: apps ? "#fff8f0" : FACETS[f].t });
            const link = href ? href(n) : null; if (link) g = el("a", { href: link }, g);
            nodes += el("g", { class: "node" + (n.id === focus ? " focus" : ""), "data-id": n.id, "data-f": f, "data-h": FACETS[f].h, opacity: fade(n.id), style: link ? "cursor:pointer" : null }, g);
        });

        // labels, only with a focus: incoming wires carry their producers' verbs — one label each;
        // the outgoing all carry this component's one verb — said once, under the node
        let labels = "";
        if (focus) {
            const n = byId[focus], ins = edgeInfo.filter((e) => e.b === focus), outs = edgeInfo.filter((e) => e.a === focus);
            labels += placeLabels(ins, L);
            if (outs.length) { const p0 = pos[focus], bb = nodeBox[focus]; const y = (bb ? bb.y + bb.h : p0.y + HUB_RAD) + 26; const str = `${n.verb || "feeds"} → ${outs.length}`; const w = str.length * 12.4 + 16; labels += el("rect", { x: r1(p0.x - w / 2), y: r1(y - 14), width: r1(w), height: 28, rx: 6, fill: "#06100a", "fill-opacity": 0.85 }) + text(p0.x, y + 7, str, { fs: 20, fill: colourOf(n) }); }
        }

        return el("svg", { xmlns: "http://www.w3.org/2000/svg", class: opts.class || "chartfig", viewBox: "0 0 1700 1700", preserveAspectRatio: "xMidYMid meet", role: "img", "aria-label": focus ? `the chart, ${focus} in focus` : "the chart", "data-focus": focus },
            el("defs", {}, defs) + el("g", { class: "edges" }, edges) + el("g", { class: "hubs" }, hubs) + el("g", { class: "nodes" }, nodes) + el("g", { class: "labels" }, labels));
    }

    // the side-panel data of the chart page: what arrives and what it feeds, each row with the product it belongs to
    chartFig.neighbours = (id, NODES = G("NODES"), EDGES = G("EDGES")) => {
        const byId = Object.fromEntries(NODES.map((n) => [n.id, n])); const n = byId[id]; if (!n) return null;
        const row = (e, other) => { const o = byId[other]; return { id: other, verb: e.v, s: e.s, k: e.k || [], product: o ? (o.f === "apps" ? "built on soft3" : productOf(o)) : null, colour: o ? colourOf(o) : null }; };
        return { node: n, facet: facetOf(n), colour: colourOf(n), product: productOf(n), question: n.hub ? FACETS[facetOf(n)].q : "", ins: EDGES.filter((e) => e.b === id).map((e) => row(e, e.a)), outs: EDGES.filter((e) => e.a === id).map((e) => row(e, e.b)) };
    };
    chartFig.FACETS = FACETS; chartFig.LADDER = LADDER; chartFig.facetOf = facetOf; chartFig.colourOf = colourOf;
    chartFig.SIZE = { w: 1700, h: 1700 };
    window.chartFig = chartFig;
})();
