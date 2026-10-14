// soft3 — ladder-fig: the ten rungs with their chips as one HTML string. a port of the ladder in /layers/index.html.
// data: NODES from /data.js — a rung is a hub (layers 1–7) or an app (8–10); its chips are the non-hub nodes of the
// same layer, each painted in its own facet (n.f); the rung itself wears the k-th colour of the flag.
// what is not data and lives here: the glyph per rung, the three groups, the app sites, the chip reading order and
// the hover sentences on the chip rows — the presentation the layers page wrote by hand.
//
//   window.ladderFig(opts) → string  (a <div class="ladder"> … , with its <style> in front unless opts.css === false)
//     opts.id        id on the root element (default none)
//     opts.class     extra classes on the root (e.g. "glass")
//     opts.question  true shows the rung's question under the thesis (default false — the page keeps it in title= only)
//     opts.selected  hub id rendered with aria-selected="true"
//     opts.focus     hub id: the other rungs fade to .45 (for a chapter slide)
//     opts.href      (node) => url | null — wraps the name in <a>; default none (the page navigates on click by JS)
//     opts.chipHref  (node) => url | null — wraps every chip in <a class="chip-link"> (default none)
//     opts.css       false to omit the <style>; the stylesheet is also at window.ladderFig.css
//     opts.nodes     override the data (default window.NODES)
//   every rung carries data-p (product id), data-f (facet), data-h (facet hue), data-q (the question), data-site for apps —
//   a host wires mouseenter → worldHue(h - 150) and click → /chart/?sel=<data-p> exactly as the layers page does.
(() => {
    // /data.js declares its tables with top-level const: global lexical bindings, not window properties — resolve them by name
    const G = (name) => { try { return (0, eval)(name); } catch { return window[name]; } };
    const esc = (s) => String(s).replace(/[&<>"]/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" }[c]));
    const FACET = { form: ["#00ff00", 120], mass: ["#0000ff", 240], space: ["#4000ff", 255], life: ["#8000ff", 270], word: ["#ff0000", 0], work: ["#ff8000", 30], play: ["#ffff00", 60], apps: ["#7dffc4", 150] };
    const PF = { 1: "form", 2: "mass", 3: "space", 4: "life", 5: "word", 6: "work", 7: "play" };
    const LADDER = ["nox", "trident", "joy", "cybergraph", "neural", "cy", "soft3", "cyb", "cyber", "cyberia"];
    const GROUPS = [["kernel", [1, 2, 3]], ["core", [4, 5, 6, 7]], ["apps", [8, 9, 10]]];
    const GLYPH = { nox: '<span class="glyph">⚛&#xFE0E;</span>', trident: "🔱", joy: "💋", cybergraph: "🕸️", neural: "🧠", cy: "🧬", soft3: "👙", cyb: "🤖", cyber: "🟢", cyberia: "🌏" };
    const SITE = { cyb: "https://cyb.ai", cyber: "https://cyber.page", cyberia: "https://cyberia.to" };
    // the reading order of the chips, as the layers page wrote them; anything new in the data follows in data order
    const ORDER = {
        nox: ["strata", "hemera", "honeycrisp"], trident: ["tine", "tir", "toll", "forge", "lsp"], joy: ["lens", "zheng", "bbg"],
        cybergraph: ["neuron", "tok", "signal", "file", "tru", "name", "mudra", "tade"], neural: ["eidos", "rune", "inf", "fs", "spark", "atlas", "rs", "nu"],
        cy: ["chroma", "soma", "glia", "seer", "soul", "ward", "vault", "state", "body"], soft3: ["radio", "sync", "foculus", "solong"],
    };
    const ROW_TITLE = {
        trident: "tine · tir · toll · forge · lsp are trident's parts",
        cybergraph: "neuron acts · tok pays · signal commits · file is the thing · tru converges · name resolves · mudra hides · tade is the bytes of a file and the payload of a link on any boundary (one exchange format: soft3/proposals/tade-one-exchange)",
        neural: "eidos proves trident (self-verification) · rune runs neurons · inf queries the book · fs mutates files — the CRUD the graph lacks · spark says what a file renders to, in cells or pixels · atlas is the package registry over the book — packages are cards, publishing is minting; trident's package manager, not written yet · rs is the reference the stack is verified against · nu is the shell: ~20 domain verb groups (filesystem, strings, dates, math, network, formats, hash, …) composed into sentences over typed tables — the dialects here are its verb groups, neu generalises it",
        cy: "chroma — one component: now · voice · name · sense · spacetime · sigma · memory · com · time; behind it the eight unseen",
        soft3: "the network in planes (soft3/proposals/network-planes): radio reaches and carries · sync reconciles — ordering, availability, local merge, history trust · foculus agrees — fork-choice over φ*, finality, epochs, beacon · solong rewards — tickets, fold-mining, Shapley settlement; named after Shapley's own coalition game, So Long Sucker",
    };

    const CSS = `
.ladder { --mint: #7dffc4; --cream: #fff8f0; position: relative; z-index: 1; display: flex; flex-direction: column; gap: 0.4rem; padding: 0.8rem 0.9rem 0.7rem; min-height: 0; overflow: auto; font-family: Play, system-ui, sans-serif; color: var(--cream); }
/* the page gives the ladder a fixed height (the shell is 100vh) and lets groups and rungs split it with flex-basis 0;
   a figure cannot assume a bounded host, so the basis is auto: content height when free, an even fill when bounded */
.ladder .group { flex: 1 1 auto; min-height: 0; display: grid; grid-template-columns: 2rem 1fr; gap: 0.5rem; }
.ladder .group[data-g="core"] { flex-grow: 1.34; }
.ladder .rail { position: relative; border-right: 1px solid color-mix(in srgb, var(--mint) 45%, transparent); }
.ladder .rail span { position: absolute; top: 50%; left: 50%; transform: translate(-50%, -50%) rotate(180deg); writing-mode: vertical-rl; font-family: "JetBrains Mono", ui-monospace, monospace; font-size: 0.64rem; letter-spacing: 0.3em; text-transform: uppercase; color: var(--mint); text-shadow: 0 0 14px rgba(125,255,196,0.45); white-space: nowrap; }
.ladder .rungs { display: flex; flex-direction: column; gap: 0.3rem; min-height: 0; }
.ladder .rung { flex: 1 1 auto; min-height: 3rem; display: grid; grid-template-columns: 1.9rem 2.2rem minmax(0, 1fr) fit-content(52%); gap: 0.6rem; align-items: center; padding: 0.35rem 0.8rem 0.35rem 0.6rem; border-radius: 0.7rem; border: 1px solid color-mix(in srgb, var(--fc) 35%, transparent); background: color-mix(in srgb, var(--mint) calc(12% - var(--k) * 0.8%), rgba(6,16,10,0.94)); cursor: pointer; transition: box-shadow 0.15s ease, border-color 0.15s ease, opacity 0.3s ease; }
.ladder .rung:hover, .ladder .rung[aria-selected="true"] { border-color: color-mix(in srgb, var(--fc) 80%, transparent); box-shadow: 0 0 26px color-mix(in srgb, var(--fc) 30%, transparent); background: color-mix(in srgb, var(--fc) 10%, rgba(6,16,10,0.94)); }
.ladder .rung.dim { opacity: 0.45; }
.ladder .num { font-family: "JetBrains Mono", ui-monospace, monospace; font-size: 1.5rem; line-height: 1; color: var(--mint); text-shadow: 0 0 18px rgba(125,255,196,0.45); text-align: center; }
.ladder .ico { font-size: 1.5rem; line-height: 1; text-align: center; filter: drop-shadow(0 0 10px rgba(125,255,196,0.35)); }
.ladder .ico .glyph { font-family: "Apple Symbols", "Segoe UI Symbol", "Noto Sans Symbols", sans-serif; color: var(--mint); font-size: 1.9rem; }
.ladder .who { min-width: 0; }
.ladder .who h2 { margin: 0; font-family: Play, system-ui, sans-serif; font-size: 1.05rem; font-weight: 700; line-height: 1.1; letter-spacing: 0; text-transform: none; color: var(--cream); animation: none; }
.ladder .who h2 a { color: inherit; text-decoration: none; }
.ladder .who h2 small { font-size: 0.6rem; font-weight: 400; color: rgba(255,248,240,0.6); letter-spacing: 0.06em; text-transform: uppercase; }
.ladder .who .does { margin: 0.15rem 0 0; font-size: 0.72rem; line-height: 1.25; color: rgba(255,248,240,0.8); }
.ladder .who .q { margin: 0.1rem 0 0; font-size: 0.68rem; font-style: italic; color: color-mix(in srgb, var(--fc) 55%, white); }
.ladder .who .fac { display: inline-flex; align-items: center; gap: 0.3rem; margin-left: 0.5rem; font-family: "JetBrains Mono", ui-monospace, monospace; font-size: 0.62rem; letter-spacing: 0.14em; text-transform: uppercase; color: color-mix(in srgb, var(--fc) 62%, white); text-shadow: 0 0 10px color-mix(in srgb, var(--fc) 70%, transparent); vertical-align: middle; }
.ladder .who .fac i { width: 0.62rem; height: 0.62rem; border-radius: 999px; background: var(--fc); box-shadow: 0 0 10px var(--fc); display: inline-block; }
.ladder .deps { display: flex; flex-direction: column; gap: 0.22rem; align-items: flex-end; }
.ladder .deps .row { display: flex; flex-wrap: wrap; gap: 0.2rem; justify-content: flex-end; }
.ladder .deps a.on { font-family: "JetBrains Mono", ui-monospace, monospace; font-size: 0.62rem; padding: 0.14rem 0.45rem; border-radius: 999px; text-decoration: none; background: transparent; color: color-mix(in srgb, var(--fc) 85%, white); border: 1px dashed color-mix(in srgb, var(--fc) 55%, transparent); white-space: nowrap; }
.ladder .deps a.on:hover { background: color-mix(in srgb, var(--fc) 18%, transparent); }
.ladder .deps i { font-style: normal; font-family: "JetBrains Mono", ui-monospace, monospace; font-size: 0.56rem; padding: 0.12rem 0.36rem; border-radius: 999px; background: var(--fc); color: var(--ft); white-space: nowrap; }
.ladder .deps a.chip-link { text-decoration: none; }
.ladder [data-f="form"]  { --fc: #00ff00; --ft: #0a120e; }
.ladder [data-f="mass"]  { --fc: #0000ff; --ft: #fff8f0; }
.ladder [data-f="space"] { --fc: #4000ff; --ft: #fff8f0; }
.ladder [data-f="life"]  { --fc: #8000ff; --ft: #fff8f0; }
.ladder [data-f="word"]  { --fc: #ff0000; --ft: #fff8f0; }
.ladder [data-f="work"]  { --fc: #ff8000; --ft: #0a120e; }
.ladder [data-f="play"]  { --fc: #ffff00; --ft: #0a120e; }
.ladder [data-f="apps"]  { --fc: #fff8f0; --ft: #0a120e; }
@media (max-width: 900px), (max-height: 560px) {
  /* the phone reads the ladder as a list: nothing flexes to fit a height, everything flows */
  .ladder { display: block; overflow: visible; padding: 0.8rem 0.8rem 0.9rem; }
  .ladder .group { display: block; flex: none; min-height: auto; margin-top: 0.5rem; }
  .ladder .rail { border-right: 0; border-bottom: 1px solid color-mix(in srgb, var(--mint) 45%, transparent); height: auto; margin: 0 0 0.45rem; padding-bottom: 0.2rem; }
  .ladder .rail span { position: static; transform: none; writing-mode: horizontal-tb; display: block; }
  .ladder .rungs { display: flex; flex-direction: column; gap: 0.4rem; min-height: auto; }
  .ladder .rung { flex: none; min-height: auto; grid-template-columns: 1.8rem 2rem minmax(0, 1fr); grid-template-areas: "num ico who" "deps deps deps"; row-gap: 0.4rem; padding: 0.55rem 0.7rem; }
  .ladder .num { grid-area: num; } .ladder .ico { grid-area: ico; } .ladder .who { grid-area: who; } .ladder .deps { grid-area: deps; align-items: flex-start; }
  .ladder .deps .row { justify-content: flex-start; flex-wrap: wrap; }
  .ladder .who .does { white-space: normal; }
}
.ladder.list { display: block; overflow: visible; }
.ladder.list .group { display: block; flex: none; min-height: auto; margin-top: 0.5rem; }
.ladder.list .rail { border-right: 0; border-bottom: 1px solid color-mix(in srgb, var(--mint) 45%, transparent); height: auto; margin: 0 0 0.45rem; padding-bottom: 0.2rem; }
.ladder.list .rail span { position: static; transform: none; writing-mode: horizontal-tb; display: block; }
.ladder.list .rungs { min-height: auto; }
.ladder.list .rung { flex: none; min-height: auto; }
`;

    function ladderFig(opts = {}) {
        const NODES = opts.nodes || G("NODES");
        if (!NODES) throw new Error("ladderFig: NODES is missing — load /data.js first");
        const byId = Object.fromEntries(NODES.map((n) => [n.id, n]));
        const chipsOf = (k) => { const own = NODES.filter((n) => n.layer === k && !n.hub && n.f !== "apps"); const pref = ORDER[LADDER[k - 1]] || []; return [...pref.map((id) => own.find((n) => n.id === id)).filter(Boolean), ...own.filter((n) => !pref.includes(n.id))]; };
        const rung = (k) => {
            const pid = LADDER[k - 1], n = byId[pid]; if (!n) return "";
            const apps = k > 7, f = apps ? "apps" : PF[k], [, hue] = FACET[f];
            const q = apps ? "how do we coordinate?" : n.q || "";
            const name = opts.href && opts.href(n) ? `<a href="${esc(opts.href(n))}">${esc(pid)}</a>` : esc(pid);
            const who = `<h2>${name} <small>· ${esc(n.role || "")}</small>${apps ? "" : ` <span class="fac"><i></i>${((G("VERBS") || [])[k - 1] || {}).id || f}</span>`}</h2><p class="does">${esc(n.thesis || "")}</p>${opts.question && !apps && q ? `<p class="q">${esc(q)}</p>` : ""}`;
            let deps;
            if (apps) { const site = SITE[pid]; deps = site ? `<a class="on" href="${site}" target="_blank" rel="noopener">${esc(site.replace(/^https?:\/\//, ""))} ↗</a>` : ""; }
            else deps = chipsOf(k).map((c) => { const chip = `<i data-f="${esc(c.f)}" title="${esc(c.verb || "")}">${esc(c.id)}</i>`; const u = opts.chipHref ? opts.chipHref(c) : null; return u ? `<a class="chip-link" href="${esc(u)}">${chip}</a>` : chip; }).join("");
            const title = ROW_TITLE[pid] ? ` title="${esc(ROW_TITLE[pid])}"` : "";
            const cls = "rung" + (opts.focus && opts.focus !== pid ? " dim" : "");
            return `<section class="${cls}" style="--k:${k}" data-f="${f}" data-h="${hue}" data-q="${esc(q)}" title="${esc(q)}" data-p="${pid}"${apps && SITE[pid] ? ` data-site="${SITE[pid]}"` : ""}${opts.selected === pid ? ' aria-selected="true"' : ""}>` +
                `<div class="num">${k}</div><div class="ico" aria-hidden="true">${GLYPH[pid] || ""}</div><div class="who">${who}</div><div class="deps"><span class="row"${title}>${deps}</span></div></section>`;
        };
        const groups = GROUPS.map(([g, ks]) => `<section class="group" data-g="${g}"><div class="rail"><span>${g}</span></div><div class="rungs">${ks.map(rung).join("")}</div></section>`).join("");
        const root = `<div class="ladder${opts.class ? " " + esc(opts.class) : ""}"${opts.id ? ` id="${esc(opts.id)}"` : ""} aria-label="the stack as a ladder of ten products">${groups}</div>`;
        return (opts.css === false ? "" : `<style data-ladder-css>${CSS}</style>`) + root;
    }
    ladderFig.css = CSS; ladderFig.LADDER = LADDER; ladderFig.FACET = FACET; ladderFig.GLYPH = GLYPH;
    window.ladderFig = ladderFig;
})();
