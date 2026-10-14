/* soft3 demo · focus — the links become one focus.
   window.demo_focus(mount, api) → { destroy() }
   ~40 nodes on a canvas; "iterate" runs a damped random walk (teleport .15) and shows ‖Δφ‖ shrinking by κ̂ each step;
   "another start" runs from a random vector and shows the distance to the first run collapsing (uniqueness). ◌ mock. */
window.demo_focus = (mount, api) => {
  const M = api.M, sfx = api.sfx || {};
  const N = 40, TELE = .15, STEPS = 14, SEED = [1337];
  const rnd = () => { let t = SEED[0] += 0x6D2B79F5; t = Math.imul(t ^ t >>> 15, t | 1); t ^= t + Math.imul(t ^ t >>> 7, t | 61); return ((t ^ t >>> 14) >>> 0) / 4294967296; };
  // preferential attachment, m = 2
  const adj = Array.from({ length: N }, () => []); const edges = []; const link = (a, b) => { if (a === b || adj[a].includes(b)) return; adj[a].push(b); adj[b].push(a); edges.push([a, b]); };
  link(0, 1); link(1, 2); link(2, 0);
  for (let i = 3; i < N; i++) { for (let m = 0; m < 2; m++) { const e = edges[Math.floor(rnd() * edges.length)]; link(i, e[rnd() < .5 ? 0 : 1]); } }
  // layout: a few hundred force steps, once
  const pos = Array.from({ length: N }, () => [rnd() - .5, rnd() - .5]);
  for (let it = 0; it < 260; it++) { const f = pos.map(() => [0, 0]); for (let i = 0; i < N; i++) for (let j = i + 1; j < N; j++) { let dx = pos[i][0] - pos[j][0], dy = pos[i][1] - pos[j][1]; const d2 = dx * dx + dy * dy + 1e-4, k = .0018 / d2; f[i][0] += dx * k; f[i][1] += dy * k; f[j][0] -= dx * k; f[j][1] -= dy * k; } for (const [a, b] of edges) { const dx = pos[b][0] - pos[a][0], dy = pos[b][1] - pos[a][1]; f[a][0] += dx * .06; f[a][1] += dy * .06; f[b][0] -= dx * .06; f[b][1] -= dy * .06; } const c = 1 - it / 260; for (let i = 0; i < N; i++) { pos[i][0] = Math.max(-.5, Math.min(.5, pos[i][0] + f[i][0] * c * .5 - pos[i][0] * .002)); pos[i][1] = Math.max(-.5, Math.min(.5, pos[i][1] + f[i][1] * c * .5 - pos[i][1] * .002)); } }
  const step = (phi) => { const nx = new Float64Array(N).fill(TELE / N); for (let j = 0; j < N; j++) { const s = (1 - TELE) * phi[j] / adj[j].length; for (const i of adj[j]) nx[i] += s; } return nx; };
  const l1 = (a, b) => { let d = 0; for (let i = 0; i < N; i++) d += Math.abs(a[i] - (b ? b[i] : 0)); return d; };

  mount.innerHTML = `
  <canvas data-cv style="width:100%;height:210px;display:block;border:1px solid rgba(255,255,255,.18);border-radius:12px;background:rgba(0,0,0,.5)"></canvas>
  <div class="row" style="margin-top:10px"><button class="btn" data-it>iterate</button><button class="btn ghost" data-re>another start</button><span class="out" data-line style="margin:0;min-height:0;flex:1;font-size:12px">φ₀ = uniform · 40 particles · ${edges.length} links · teleport .15</span></div>
  <div style="display:grid;grid-template-columns:1fr 1fr;gap:12px">
    <div><label style="margin:8px 0 2px">‖Δφ‖₁ per step · κ̂ = <b data-k>—</b></label><div class="cbars" data-c1 style="height:44px;margin:4px 0 0"></div></div>
    <div><label style="margin:8px 0 2px">‖φ_B − φ_A‖₁ · another start</label><div class="cbars" data-c2 style="height:44px;margin:4px 0 0"></div></div>
  </div>
  <div class="fine" style="margin-top:8px">measured on bostrom: κ = 0.74, λ₂ = 0.13 ${M('m')} · this canvas: a 40-node toy ${M('k')} · the fixed point is unique: any start lands on the same φ⋆</div>`;
  const $ = (s) => mount.querySelector(s);
  const cv = $('[data-cv]'), ctx = cv.getContext('2d'), c1 = $('[data-c1]'), c2 = $('[data-c2]'), kEl = $('[data-k]'), line = $('[data-line]'), bIt = $('[data-it]'), bRe = $('[data-re]');
  const col = getComputedStyle(mount).getPropertyValue('--c').trim() || '#00ff41';
  let phi = new Float64Array(N).fill(1 / N), phiA = null, timer = null, run = 0;
  const draw = (p) => {
    const dpr = devicePixelRatio || 1, W = cv.clientWidth, H = cv.clientHeight; if (cv.width !== W * dpr || cv.height !== H * dpr) { cv.width = W * dpr; cv.height = H * dpr; }
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0); ctx.clearRect(0, 0, W, H);
    const X = (i) => W / 2 + pos[i][0] * (W - 40), Y = (i) => H / 2 + pos[i][1] * (H - 36);
    const mx = Math.max(...p) || 1;
    ctx.strokeStyle = 'rgba(255,255,255,.16)'; ctx.lineWidth = 1; ctx.beginPath(); for (const [a, b] of edges) { ctx.moveTo(X(a), Y(a)); ctx.lineTo(X(b), Y(b)); } ctx.stroke();
    for (let i = 0; i < N; i++) { const t = p[i] / mx, r = 2.5 + 9 * Math.sqrt(t); ctx.globalAlpha = .25 + .75 * t; ctx.fillStyle = col; ctx.shadowColor = col; ctx.shadowBlur = 14 * t; ctx.beginPath(); ctx.arc(X(i), Y(i), r, 0, Math.PI * 2); ctx.fill(); }
    ctx.shadowBlur = 0; ctx.globalAlpha = 1;
  };
  const bar = (chart, v, color) => { const b = document.createElement('i'); b.dataset.v = v; if (color) b.style.background = color; chart.appendChild(b); while (chart.children.length > 28) chart.firstChild.remove(); const vs = [...chart.children].map((x) => +x.dataset.v), m0 = vs[0] || 1; [...chart.children].forEach((x) => { x.style.height = Math.max(2, 100 * (+x.dataset.v) / m0) + '%'; }); };
  const animate = (frames, done) => { clearTimeout(timer); const id = ++run; let i = 0; const go = () => { if (id !== run) return; if (i >= frames.length) { done && done(); return; } frames[i++](); timer = setTimeout(go, 85); }; go(); };
  const onIt = () => {
    c1.innerHTML = ''; phi = new Float64Array(N).fill(1 / N); let prev = null; draw(phi);
    const frames = []; for (let s = 1; s <= STEPS; s++) frames.push(() => { const nx = step(phi); const d = l1(nx, phi); phi = nx; draw(phi); bar(c1, d); if (prev) kEl.textContent = (d / prev).toFixed(3); prev = d; line.innerHTML = `step ${s} · ‖Δφ‖₁ = ${d.toExponential(2)}${prev && s > 1 ? ` · bars shrink by κ̂ each step ${M('k')}` : ''}`; sfx.converge && sfx.converge(d); });
    animate(frames, () => { phiA = phi; line.innerHTML += ' · <b>φ⋆ reached</b>'; sfx.chord && sfx.chord(2); });
  };
  const onRe = () => {
    if (!phiA) { // make sure run A exists, silently
      let p = new Float64Array(N).fill(1 / N); for (let s = 0; s < 60; s++) p = step(p); phiA = p;
    }
    c2.innerHTML = ''; let p = new Float64Array(N); let sum = 0; for (let i = 0; i < N; i++) { p[i] = Math.random(); sum += p[i]; } for (let i = 0; i < N; i++) p[i] /= sum; phi = p; draw(phi);
    const frames = [() => { bar(c2, l1(phi, phiA), '#fff'); line.innerHTML = `φ₀ = random · ‖φ_B − φ_A‖₁ = ${l1(phi, phiA).toExponential(2)}`; }];
    for (let s = 1; s <= STEPS; s++) frames.push(() => { phi = step(phi); draw(phi); const dist = l1(phi, phiA); bar(c2, dist, s % 2 ? undefined : '#fff'); line.innerHTML = `step ${s} · ‖φ_B − φ_A‖₁ = <b>${dist.toExponential(2)}</b> ${M('k')}`; sfx.converge && sfx.converge(dist); });
    animate(frames, () => { line.innerHTML += ' · <b>one focus</b>'; sfx.chord && sfx.chord(3); });
  };
  bIt.addEventListener('click', onIt); bRe.addEventListener('click', onRe);
  const onResize = () => draw(phi); window.addEventListener('resize', onResize);
  draw(phi);
  return { destroy() { run++; clearTimeout(timer); bIt.removeEventListener('click', onIt); bRe.removeEventListener('click', onRe); window.removeEventListener('resize', onResize); mount.innerHTML = ''; } };
};
