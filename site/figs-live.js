// soft3 — figs-live: the three figure modules wired for a presentation. each figure is rendered once into a mount and
// interaction toggles classes on the rendered elements — no re-render on hover. only the chart re-renders, on focus.
// data: VERBS / NODES / EDGES from /data.js by bare name (global lexical bindings, not window properties).
//
//   window.liveFigs.loop(mount, opts)   → { set(verbId), current, svg, caption }
//     opts.current   verb id shown lit (class "lit"); opts.onVerb(id) on click/tap (the clicked verb also becomes lit);
//     opts.onHover(h, id) on hover with the verb hue (h = null, id = null when the pointer leaves).
//     hover: the verb and its two arcs get "hot", every other verb and arc gets "dim". caption under the ring:
//     "<verb> · who · does" for the hovered verb, defaulting to the current one.
//   window.liveFigs.chart(mount, opts)  → { focus(id), focused, svg, caption, host }
//     opts.focus     node id rendered in focus (chartFig's focus); opts.onHub(id) on hub click (re-renders focused on it);
//     opts.onNode(id, isHub) on any node click; opts.onFocus(id|null) on every focus change; opts.onHover(h, id).
//     hover: wires touching the node get "hot" (by data-a / data-b), the rest "dim"; nodes on those wires get "near",
//     the rest "dim". caption: "<id> · <verb> · <product> · fed by: a verb, b verb · feeds: x, y" (° = specified only).
//     click on the background clears the focus.
//   window.liveFigs.ladder(mount, opts) → { select(id), selected, root, caption }
//     opts.selected  rung marked aria-selected; opts.onRung(id) on rung click/tap; opts.onHover(h, id) with the facet hue;
//     opts.onChip(chipId, rungId) if given takes a chip tap instead of the rung; the other ladderFig opts pass through.
//     caption under the ladder: the hovered chip's "<chip> · verb — thesis | note | feeds …", default the selected rung's thesis.
//   window.liveFigs.css — the dim/bright states and the captions; injected once into <head> on the first mount.
//
// touch: tap = click everywhere; browsers emulate mouseover before the click, so a tap also shows the hover state until the
// next tap elsewhere. nothing essential lives in hover: the captions default to the current / focused / selected item.
(() => {
    const G = (name) => { try { return (0, eval)(name); } catch { return window[name]; } };
    const esc = (s) => String(s).replace(/[&<>"]/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" }[c]));
    const call = (fn, ...a) => { if (typeof fn === "function") fn(...a); };
    const on = (el, ev, fn) => el.addEventListener(ev, fn);
    const closest = (t, sel) => (t && t.closest ? t.closest(sel) : null);
    // caption ink: the facet colour lifted toward cream — pure blue at 12px on the dark glass is unreadable
    const ink = (c) => `color-mix(in srgb, ${c || "#fff8f0"} 62%, #fff8f0)`;

    const CSS = `
.live-cap { font-family: "JetBrains Mono", ui-monospace, monospace; font-size: 12px; line-height: 1.5; opacity: 0.7; color: #fff8f0; margin: 0.5rem 0 0; min-height: 1.5em; transition: opacity 0.2s ease; }
.live-cap b { font-weight: 700; }
.live-cap i { font-style: italic; opacity: 0.8; }
/* loop */
.loopfig.live .verb, .loopfig.live .arc { transition: opacity 0.2s ease, filter 0.2s ease; }
.loopfig.live .verb { cursor: pointer; }
.loopfig.live .verb.dim, .loopfig.live .arc.dim { opacity: 0.3; }
.loopfig.live .verb.lit circle { stroke-width: 4; }
.live-cap { background: rgba(6,16,10,.62); padding: 4px 10px; border-radius: 8px; width: max-content; max-width: 100%; margin: .5rem auto 0; opacity: .9; }
.loopfig.live .verb.lit { filter: drop-shadow(0 0 14px var(--c)); }
.loopfig.live .verb.hot { opacity: 1; filter: drop-shadow(0 0 26px var(--c)) brightness(1.15); }
.loopfig.live .arc path { transition: stroke-opacity 0.2s ease, stroke-width 0.2s ease; }
.loopfig.live .arc.hot path { stroke-opacity: 1; stroke-width: 13; }
/* chart */
.chartfig.live .hub, .chartfig.live .node { cursor: pointer; transition: opacity 0.2s ease, filter 0.2s ease; }
.chartfig.live .wire { transition: stroke-opacity 0.2s ease, stroke-width 0.2s ease; }
.chartfig.live .hub.dim, .chartfig.live .node.dim { opacity: 0.22; }
.chartfig.live .hub.near, .chartfig.live .node.near { opacity: 1; }
.chartfig.live .hub.hot, .chartfig.live .node.hot { filter: brightness(1.2) drop-shadow(0 0 16px rgba(255,248,240,0.45)); }
.chartfig.live .wire.hot { stroke-opacity: 0.95; stroke-width: 3; }
.chartfig.live .wire.dim { stroke-opacity: 0.04; }
.chartfig.live .labels { transition: opacity 0.2s ease; }
.chartfig.live.hovering .labels { opacity: 0.25; }
/* ladder */
.ladder.live .rung { cursor: pointer; }
.ladder.live .deps i { cursor: help; transition: box-shadow 0.2s ease, filter 0.2s ease; }
.ladder.live .deps i.hot, .ladder.live .deps i:hover { box-shadow: 0 0 12px var(--fc); filter: brightness(1.15); }
`;
    function injectCss() {
        if (document.querySelector("style[data-figs-live]")) return;
        const s = document.createElement("style"); s.setAttribute("data-figs-live", ""); s.textContent = CSS; document.head.appendChild(s);
    }
    function makeCaption(mount, kind) {
        const c = document.createElement("div"); c.className = "live-cap live-cap-" + kind; c.setAttribute("aria-live", "polite"); mount.appendChild(c); return c;
    }

    // ── loop: the seven-verb ring ─────────────────────────────────────────────────────────────────────────────────
    function loop(mount, opts = {}) {
        injectCss();
        const VERBS = opts.verbs || G("VERBS");
        if (!window.loopFig || !VERBS) throw new Error("liveFigs.loop: loopFig and VERBS are needed — load /data.js and /loop-fig.js first");
        const byId = Object.fromEntries(VERBS.map((v) => [v.id, v]));
        mount.innerHTML = loopFig({ id: opts.id || "live-loop", href: null, verbs: opts.verbs, arcs: opts.arcs, heart: opts.heart, title: opts.title, sub: opts.sub, class: opts.class });
        const svg = mount.querySelector("svg"); svg.classList.add("live");
        const verbs = [...svg.querySelectorAll("g.verb")], arcs = [...svg.querySelectorAll("g.arc")];
        verbs.forEach((g) => g.style.setProperty("--c", g.dataset.c));
        const caption = makeCaption(mount, "loop");
        let current = byId[opts.current] ? opts.current : null, hover = null;
        const line = (v) => `<b style="color:${esc(v.ink || v.c)}">${esc(v.id)}</b> · ${esc(v.who)} · ${esc(v.does)}`;
        function apply() {
            verbs.forEach((g) => { const id = g.dataset.id; g.classList.toggle("lit", id === current); g.classList.toggle("hot", id === hover); g.classList.toggle("dim", hover ? id !== hover : (!!current && !!opts.dimOthers && id !== current)); });
            arcs.forEach((g) => { const key = hover || (opts.dimOthers ? current : null); const hot = !!key && (g.dataset.from === key || g.dataset.to === key); g.classList.toggle("hot", !!hover && hot); g.classList.toggle("dim", !!key && !hot); });
            const v = byId[hover] || byId[current];
            caption.innerHTML = v ? line(v) : esc(opts.hint || "tap a verb");
        }
        on(svg, "mouseover", (e) => {
            const g = closest(e.target, "g.verb"), id = g ? g.dataset.id : null;
            if (id === hover) return;
            hover = id; apply();
            call(opts.onHover, id ? +g.dataset.h : null, id);
        });
        on(svg, "mouseleave", () => { if (hover === null) return; hover = null; apply(); call(opts.onHover, null, null); });
        on(svg, "click", (e) => {
            const g = closest(e.target, "g.verb"); if (!g) return;
            e.preventDefault();
            const id = g.dataset.id; hover = null; current = id; apply();
            call(opts.onVerb, id, byId[id]);
        });
        apply();
        return { set(id) { current = byId[id] ? id : null; apply(); }, get current() { return current; }, svg, caption };
    }

    // ── chart: the dependency chart ──────────────────────────────────────────────────────────────────────────────
    function chart(mount, opts = {}) {
        injectCss();
        const NODES = opts.nodes || G("NODES"), EDGES = opts.edges || G("EDGES");
        if (!window.chartFig || !NODES || !EDGES) throw new Error("liveFigs.chart: chartFig, NODES and EDGES are needed — load /data.js and /chart-fig.js first");
        const byId = Object.fromEntries(NODES.map((n) => [n.id, n]));
        const host = document.createElement("div"); host.className = "live-fig live-chart"; mount.appendChild(host);
        const caption = makeCaption(mount, "chart");
        let focused = null, hover = null, svg = null, wires = [], groups = [];
        const describe = (id) => {
            const nb = chartFig.neighbours(id, NODES, EDGES); if (!nb) return esc(id);
            const name = (r) => `<span style="color:${esc(ink(r.colour))}">${esc(r.id)}</span>${r.s ? "°" : ""}`;
            const fed = nb.ins.map((r) => `${name(r)} <i>${esc(r.verb)}</i>`).join(", "), feeds = nb.outs.map(name).join(", ");
            return `<b style="color:${esc(ink(nb.colour))}">${esc(id)}</b> · ${esc(nb.node.verb || "")}${nb.product && nb.product !== id ? ` · ${esc(nb.product)}` : ""} · fed by: ${fed || "—"} · feeds: ${feeds || "—"}`;
        };
        function apply() {
            const id = hover;
            if (id) {
                const near = new Set([id]);
                wires.forEach((w) => { const hot = w.dataset.a === id || w.dataset.b === id; if (hot) { near.add(w.dataset.a); near.add(w.dataset.b); } w.classList.toggle("hot", hot); w.classList.toggle("dim", !hot); });
                groups.forEach((g) => { const n = near.has(g.dataset.id); g.classList.toggle("hot", g.dataset.id === id); g.classList.toggle("near", n); g.classList.toggle("dim", !n); });
            } else {
                wires.forEach((w) => w.classList.remove("hot", "dim"));
                groups.forEach((g) => g.classList.remove("hot", "near", "dim"));
            }
            if (svg) svg.classList.toggle("hovering", !!id);
            caption.innerHTML = id ? describe(id) : focused ? describe(focused) : esc(opts.hint || "tap a product to focus it · tap the background to clear");
        }
        function render(id) {
            focused = id && byId[id] ? id : null; hover = null;
            host.innerHTML = chartFig({ id: opts.id || "live-chart", focus: focused, nodes: opts.nodes, edges: opts.edges, class: opts.class });
            svg = host.querySelector("svg"); svg.classList.add("live");
            wires = [...svg.querySelectorAll("path.wire")]; groups = [...svg.querySelectorAll("g.hub, g.node")];
            apply();
        }
        const at = (t) => closest(t, "g.hub, g.node");
        on(host, "mouseover", (e) => {
            const g = at(e.target), id = g ? g.dataset.id : null;
            if (id === hover) return;
            hover = id; apply();
            call(opts.onHover, id ? +g.dataset.h : null, id);
        });
        on(host, "mouseleave", () => { if (hover === null) return; hover = null; apply(); call(opts.onHover, null, null); });
        on(host, "click", (e) => {
            const g = at(e.target);
            if (!g) { if (focused) { render(null); call(opts.onFocus, null); } return; }
            e.preventDefault();
            const id = g.dataset.id, hub = g.classList.contains("hub");
            render(id); call(opts.onFocus, id);
            if (hub) call(opts.onHub, id, byId[id]);
            call(opts.onNode, id, hub, byId[id]);
        });
        render(opts.focus);
        return { focus(id) { render(id); call(opts.onFocus, focused); }, get focused() { return focused; }, get svg() { return svg; }, caption, host };
    }

    // ── ladder: the ten rungs ────────────────────────────────────────────────────────────────────────────────────
    function ladder(mount, opts = {}) {
        injectCss();
        const NODES = opts.nodes || G("NODES");
        if (!window.ladderFig || !NODES) throw new Error("liveFigs.ladder: ladderFig and NODES are needed — load /data.js and /ladder-fig.js first");
        if (!document.querySelector("style[data-ladder-css]")) { const s = document.createElement("style"); s.setAttribute("data-ladder-css", ""); s.textContent = ladderFig.css; document.head.appendChild(s); }
        const byId = Object.fromEntries(NODES.map((n) => [n.id, n]));
        const { onRung, onHover, onChip, hint, ...pass } = opts;
        mount.innerHTML = ladderFig({ ...pass, css: false, href: null });
        const root = mount.querySelector(".ladder"); root.classList.add("live");
        const rungs = [...root.querySelectorAll(".rung")], chips = [...root.querySelectorAll(".deps i[data-f]")];
        const caption = makeCaption(mount, "ladder");
        let selected = byId[opts.selected] ? opts.selected : null, hoverChip = null, hoverRung = null;
        const colourOf = (n) => (ladderFig.FACET[n.f] || ladderFig.FACET.apps)[0];
        const chipLine = (id) => {
            const n = byId[id]; if (!n) return esc(id);
            let body = n.thesis || n.note || "";
            if (!body && window.chartFig && chartFig.neighbours) { const nb = chartFig.neighbours(id, NODES); if (nb && nb.outs.length) body = "feeds " + nb.outs.map((r) => r.id).join(", "); }
            return `<b style="color:${esc(ink(colourOf(n)))}">${esc(id)}</b> · ${esc(n.verb || "")}${body ? " — " + esc(body) : ""}`;
        };
        const rungLine = (id) => { const n = byId[id]; return n ? `<b>${esc(id)}</b> · ${esc(n.role || "")} — ${esc(n.thesis || "")}` : ""; };
        function apply() {
            rungs.forEach((r) => { if (r.dataset.p === selected) r.setAttribute("aria-selected", "true"); else r.removeAttribute("aria-selected"); });
            chips.forEach((c) => c.classList.toggle("hot", c.textContent.trim() === hoverChip));
            caption.innerHTML = hoverChip ? chipLine(hoverChip) : selected ? rungLine(selected) : esc(hint || "tap a rung");
        }
        on(root, "mouseover", (e) => {
            const chip = closest(e.target, ".deps i[data-f]"), rung = closest(e.target, ".rung");
            const cid = chip ? chip.textContent.trim() : null, rid = rung ? rung.dataset.p : null;
            if (cid !== hoverChip) { hoverChip = cid; apply(); }
            if (rid !== hoverRung) { hoverRung = rid; call(onHover, rid ? +rung.dataset.h : null, rid); }
        });
        on(root, "mouseleave", () => {
            let changed = false;
            if (hoverChip !== null) { hoverChip = null; changed = true; }
            if (hoverRung !== null) { hoverRung = null; call(onHover, null, null); }
            if (changed) apply();
        });
        on(root, "click", (e) => {
            if (closest(e.target, "a.on")) return; // the app's site link keeps its own navigation
            const rung = closest(e.target, ".rung"); if (!rung) return;
            const chip = closest(e.target, ".deps i[data-f]"), id = rung.dataset.p;
            if (chip) { hoverChip = chip.textContent.trim(); if (onChip) { apply(); call(onChip, hoverChip, id); return; } }
            selected = id; apply();
            call(onRung, id, byId[id]);
        });
        apply();
        return { select(id) { selected = byId[id] ? id : null; apply(); }, get selected() { return selected; }, root, caption };
    }

    window.liveFigs = { loop, chart, ladder, css: CSS };
})();
