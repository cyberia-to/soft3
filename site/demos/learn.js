/* soft3 demo · learn — the graph compiled into a transformer.
   window.demo_learn(mount, api) → { destroy() }
   a log slider over |E|; d⋆, h⋆, L⋆ and params follow the deck's model, anchored at the measured bostrom point.
   only the bostrom point is measured (●); every other readout is a model (◌). */
window.demo_learn = (mount, api) => {
  const M = api.M, sfx = api.sfx || {};
  const E0 = 2.95e6, L0 = Math.log10(E0), KAPPA = .74, EPS = 1e-3, DIAM0 = 10;
  const BOS = { d: 31, h: 12, L: 290, params: 4e5, secs: 62 };
  const sup = (n) => String(n).replace(/-/g, '⁻').replace(/\d/g, (c) => '⁰¹²³⁴⁵⁶⁷⁸⁹'[c]);
  const sci = (E) => { const e = Math.floor(Math.log10(E)), m = E / 10 ** e; return (Math.abs(m - 1) < .02 ? '' : m.toFixed(2) + ' × ') + '10' + sup(e); };
  const human = (n) => n >= 1e9 ? (n / 1e9).toFixed(1) + 'B' : n >= 1e6 ? (n / 1e6).toFixed(1) + 'M' : n >= 1e3 ? (n / 1e3).toFixed(0) + 'k' : n.toFixed(0);
  const dur = (s) => s < 90 ? s.toFixed(0) + ' s' : s < 5400 ? (s / 60).toFixed(0) + ' min' : s < 2e5 ? (s / 3600).toFixed(1) + ' h' : s < 3e7 ? (s / 86400).toFixed(0) + ' d' : (s / 3.156e7).toFixed(1) + ' y';
  const model = (E) => {
    const r = E / E0;
    const d = Math.min(1e4, 31 * Math.pow(r, .25));
    const h = Math.max(1, Math.round(12 + 4 * Math.log10(r)));
    const diam = DIAM0 * Math.log(E) / Math.log(E0);
    const depth = Math.ceil(Math.log(1 / EPS) / Math.log(1 / KAPPA)); // 23 with κ .74, ε 1e-3
    const L = BOS.L * (diam / DIAM0); // anchored at the measured 290 (= 10 · 23 · 1.26 with these constants)
    const params = BOS.params * (d / 31) ** 2 * (L / BOS.L);
    const secs = BOS.secs * r * (d / 31) ** 2;
    return { d, h, L, params, secs, depth };
  };
  mount.innerHTML = `
  <label style="margin-top:0">graph size · |E| cyberlinks · <b data-e></b> <span data-mark></span></label>
  <input type="range" min="4" max="12" step="0.01" value="${L0.toFixed(2)}" data-range>
  <div class="chips" data-chips style="display:flex"><span class="chip" data-x="4">10⁴ · a seed</span><span class="chip" data-x="${L0}">bostrom · 2.95 × 10⁶</span><span class="chip" data-x="9">10⁹ · cognition</span><span class="chip" data-x="12">10¹² · thermodynamic</span></div>
  <div class="nums" style="grid-template-columns:repeat(4,minmax(0,1fr));gap:10px;margin-top:12px">
    <div class="n"><big data-d style="font-size:26px"></big><small>d⋆ · effective rank</small></div>
    <div class="n"><big data-h style="font-size:26px"></big><small>h⋆ ≥ dialects</small></div>
    <div class="n"><big data-l style="font-size:26px"></big><small>L⋆ · depth</small></div>
    <div class="n"><big data-p style="font-size:26px"></big><small>params ≈ 12·d²·L</small></div>
  </div>
  <svg viewBox="0 0 520 120" style="width:100%;height:auto;display:block;margin-top:8px" data-svg></svg>
  <div class="out" data-line style="margin-top:6px;min-height:0;font-size:12px"></div>
  <div class="fine" style="margin-top:6px">L⋆ = diam·⌈log(1/ε)/log(1/κ)⌉ · κ 0.74 · ε 10⁻³ · diam ∝ log|E| · d⋆ ∝ |E|^¼ (clipped 10⁴) · h⋆ ∝ log|E| · curves anchored at the bostrom point ${M('k')}</div>`;
  const $ = (s) => mount.querySelector(s);
  const range = $('[data-range]'), svg = $('[data-svg]'), line = $('[data-line]'), chips = $('[data-chips]');
  const SANS = 'Play,Inter,system-ui,sans-serif', MONO = 'JetBrains Mono,Menlo,monospace';
  const draw = (x, m, atBos) => {
    // left: a "mind" — a ring of nodes whose count and link density grow with log|E|; right: three log bars
    const n = Math.round(6 + (x - 4) * 3.5), cx = 70, cy = 60, R = 46; let o = '';
    const pts = []; for (let i = 0; i < n; i++) { const a = i / n * Math.PI * 2 - Math.PI / 2; pts.push([cx + R * Math.cos(a), cy + R * Math.sin(a)]); }
    const k = Math.max(1, Math.round((x - 3) / 1.5));
    o += `<g stroke="currentColor" stroke-opacity=".22" stroke-width=".8">`; for (let i = 0; i < n; i++) for (let j = 1; j <= k; j++) { const q = (i * 7 + j * 3) % n; if (q !== i) o += `<line x1="${pts[i][0].toFixed(1)}" y1="${pts[i][1].toFixed(1)}" x2="${pts[q][0].toFixed(1)}" y2="${pts[q][1].toFixed(1)}"/>`; } o += '</g>';
    for (const [px, py] of pts) o += `<circle cx="${px.toFixed(1)}" cy="${py.toFixed(1)}" r="${(1.6 + (x - 4) * .22).toFixed(1)}" fill="var(--c)"/>`;
    o += `<text x="${cx}" y="${cy + 4}" text-anchor="middle" font-size="11" font-family="${MONO}" fill="currentColor" fill-opacity=".8">${n} · ${k}</text>`;
    const bars = [['d⋆', m.d, 1e4], ['h⋆', m.h, 40], ['L⋆', m.L, 600]];
    bars.forEach(([lab, v, mx], i) => {
      const bx = 180 + i * 115, bw = 90, H = 78, hh = Math.max(3, H * Math.log10(1 + v) / Math.log10(1 + mx)), by = 96 - hh;
      o += `<rect x="${bx}" y="18" width="${bw}" height="78" fill="currentColor" fill-opacity=".05" rx="4"/>`;
      o += `<rect x="${bx}" y="${by.toFixed(1)}" width="${bw}" height="${hh.toFixed(1)}" fill="var(--c)" fill-opacity="${atBos ? .9 : .55}" rx="4"/>`;
      o += `<text x="${bx + bw / 2}" y="110" text-anchor="middle" font-size="11" font-family="${SANS}" font-weight="700" fill="currentColor">${lab}</text>`;
      o += `<text x="${bx + bw / 2}" y="${(by - 4).toFixed(1)}" text-anchor="middle" font-size="10" font-family="${MONO}" fill="currentColor" fill-opacity=".8">${lab === 'd⋆' ? Math.round(v) : lab === 'L⋆' ? Math.round(v / 10) * 10 : v}</text>`;
    });
    if (atBos) o += `<text x="330" y="12" text-anchor="middle" font-size="9" font-family="${MONO}" fill="var(--c)" letter-spacing="2">MEASURED · BOSTROM</text>`;
    svg.innerHTML = o;
  };
  const show = () => {
    const x = +range.value, atBos = Math.abs(x - L0) < .06, E = atBos ? E0 : 10 ** x, m = model(E);
    $('[data-e]').textContent = sci(E); $('[data-mark]').innerHTML = atBos ? M('m') : M('k');
    if (atBos) { $('[data-d]').textContent = BOS.d; $('[data-h]').textContent = '≥ ' + BOS.h; $('[data-l]').textContent = BOS.L; $('[data-p]').textContent = '~0.4M'; }
    else { $('[data-d]').textContent = Math.round(m.d).toLocaleString('en-US'); $('[data-h]').textContent = '≥ ' + m.h; $('[data-l]').textContent = Math.round(m.L / 10) * 10; $('[data-p]').textContent = human(m.params); }
    draw(x, atBos ? { d: BOS.d, h: BOS.h, L: BOS.L } : m, atBos);
    line.innerHTML = atBos ? `measured: d⋆ 31 · h⋆ ≥ 12 · L⋆ 290 · ~0.4M params · 62 s one machine ${M('m')}` : `model: one machine ≈ <b>${dur(m.secs)}</b> · ${(m.params / BOS.params).toFixed(m.params < BOS.params ? 3 : 0)}× bostrom's weights ${M('k')}`;
    chips.querySelectorAll('.chip').forEach((c) => { const on = Math.abs(+c.dataset.x - x) < .06; c.style.borderColor = on ? 'var(--c)' : ''; c.style.opacity = on ? '1' : ''; });
  };
  let last = 0;
  const onRange = () => { show(); const now = performance.now(); if (now - last > 70) { last = now; sfx.tick && sfx.tick(); } };
  const onChip = (e) => { const c = e.target.closest('.chip'); if (!c) return; range.value = (+c.dataset.x).toFixed(2); show(); (Math.abs(+c.dataset.x - L0) < .06 ? sfx.chord && sfx.chord(2) : sfx.tick && sfx.tick()); };
  range.addEventListener('input', onRange); chips.addEventListener('click', onChip);
  show();
  return { destroy() { range.removeEventListener('input', onRange); chips.removeEventListener('click', onChip); mount.innerHTML = ''; } };
};
