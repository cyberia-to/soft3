/* soft3 — collective, private, provable computing · the deck
   acts: what · the ladder · seven verbs (thirty features) · the frontier · doors
   data: features.js (FEATURES, VERB, figures), data.js (VERBS, ARCS, NODES, EDGES), registry.json (status.md) */
(() => {
const $ = (s, r = document) => r.querySelector(s);
const el = (h) => { const t = document.createElement('template'); t.innerHTML = h.trim(); return t.content.firstChild; };
const fmt = (n) => n.toLocaleString('en-US');
const esc = (t) => String(t).replace(/[&<>"]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[c]));
const MONO = 'JetBrains Mono,Menlo,monospace', SANS = 'Play,Inter,system-ui,sans-serif';
const MINT = '#7dffc4';
const VC = { compute: '#00ff00', build: '#4d7cff', prove: '#8a6bff', remember: '#b07cff', learn: '#ff3b2f', link: '#ff8c1a', focus: '#ffd400' };
const VERB_ORDER = ['compute', 'build', 'prove', 'remember', 'learn', 'link', 'focus'];
const FACET = ['form', 'mass', 'space', 'life', 'word', 'work', 'play'];
const FACET_C = ['#00ff00', '#0000ff', '#4000ff', '#8000ff', '#ff0000', '#ff8000', '#ffff00'];
const HUB_IDS = ['nox', 'trident', 'joy', 'cybergraph', 'neural', 'cy', 'soft3', 'cyb', 'cyber', 'cyberia'];
const HUBS = HUB_IDS.map((id) => NODES.find((n) => n.id === id)).filter(Boolean);
const COLS = [VC.learn, VC.link, VC.focus, VC.compute, VC.build, VC.prove, VC.remember];

/* ------------------------------------------------------------------ sound · synthesized, no files */
const sfx = (() => {
  let ac = null, master = null, on = localStorage.getItem('cyber.sound') !== 'off', amb = null;
  const ensure = () => { if (ac) return true; try { ac = new (window.AudioContext || window.webkitAudioContext)(); master = ac.createGain(); master.gain.value = .22; master.connect(ac.destination); } catch { return false; } return true; };
  const resume = () => { if (ac && ac.state === 'suspended') ac.resume(); };
  const tone = (f0, f1, dur, type = 'sine', g = .5, at = 0) => { if (!on || !ensure()) return; resume(); const t = ac.currentTime + at; const o = ac.createOscillator(), v = ac.createGain(); o.type = type; o.frequency.setValueAtTime(f0, t); if (f1) o.frequency.exponentialRampToValueAtTime(f1, t + dur); v.gain.setValueAtTime(0, t); v.gain.linearRampToValueAtTime(g, t + .006); v.gain.exponentialRampToValueAtTime(.0005, t + dur); o.connect(v); v.connect(master); o.start(t); o.stop(t + dur + .02); };
  const noise = (dur, g = .3, hp = 2000) => { if (!on || !ensure()) return; resume(); const t = ac.currentTime; const n = ac.sampleRate * dur | 0, b = ac.createBuffer(1, n, ac.sampleRate), d = b.getChannelData(0); for (let i = 0; i < n; i++) d[i] = (Math.random() * 2 - 1) * (1 - i / n); const src = ac.createBufferSource(); src.buffer = b; const f = ac.createBiquadFilter(); f.type = 'highpass'; f.frequency.value = hp; const v = ac.createGain(); v.gain.value = g; src.connect(f); f.connect(v); v.connect(master); src.start(t); };
  const ambient = (start) => { if (!ensure()) return; if (start && !amb && on) { const v = ac.createGain(); v.gain.value = 0; v.connect(master); const f = ac.createBiquadFilter(); f.type = 'lowpass'; f.frequency.value = 220; f.connect(v); const os = [55, 55.4, 82.5].map((fr) => { const o = ac.createOscillator(); o.type = 'triangle'; o.frequency.value = fr; o.connect(f); o.start(); return o; }); v.gain.linearRampToValueAtTime(.06, ac.currentTime + 4); amb = { v, os }; } if (!start && amb) { amb.v.gain.linearRampToValueAtTime(0, ac.currentTime + 1); const a = amb; amb = null; setTimeout(() => a.os.forEach((o) => o.stop()), 1200); } };
  const NOTES = [220, 246.9, 277.2, 329.6, 370, 440, 493.9];
  return {
    get on() { return on; },
    toggle() { on = !on; localStorage.setItem('cyber.sound', on ? 'on' : 'off'); if (on) { ensure(); resume(); ambient(true); tone(660, 990, .12, 'sine', .4); } else ambient(false); return on; },
    wake() { if (on && ensure()) { resume(); ambient(true); } },
    tick() { noise(.02, .25, 3000); tone(1800, 1200, .03, 'square', .08); },
    move(dir, ch) { noise(.03, .2, 1500); tone(dir > 0 ? 440 : 520, dir > 0 ? 560 : 400, .08, 'sine', .25); if (ch != null) { const r = NOTES[ch]; tone(r, null, .5, 'sine', .18, .04); tone(r * 1.5, null, .45, 'sine', .1, .08); } },
    link() { tone(880, 1320, .16, 'sine', .35); tone(1320, 1760, .22, 'triangle', .18, .1); },
    hash() { noise(.012, .12, 4000); },
    ticket() { tone(1760, 1750, .35, 'sine', .3); tone(2640, 2630, .25, 'sine', .12, .02); },
    converge(d) { const f = 220 + 1200 * Math.min(1, Math.max(0, (-Math.log10(Math.max(d, 1e-9)) - 1) / 7)); tone(f, f * 1.02, .07, 'triangle', .22); },
    stake(v) { tone(v ? 160 : 110, 60, .25, 'sine', .45); noise(.05, .15, 800); },
    mentor() { [0, 2, 4, 6].forEach((k, i) => tone(NOTES[k] * 2, null, .3, 'sine', .22, i * .09)); },
    chord(ch) { const r = NOTES[ch]; [1, 1.25, 1.5, 2].forEach((m, i) => tone(r * m, null, .9, 'sine', .14, i * .05)); },
  };
})();


/* ------------------------------------------------------------------ svg helpers */
const svgo = (w, h, label, body) => `<svg class="fig deckfig" viewBox="0 0 ${w} ${h}" role="img" aria-label="${label}">${body}</svg>`;
const T = (x, y, t, o = {}) => `<text x="${x}" y="${y}" font-size="${o.fs || 11}" font-weight="${o.w || 400}" text-anchor="${o.a || 'start'}" fill="${o.c || 'currentColor'}" fill-opacity="${o.op || 1}" font-family="${o.ff || (o.w === 700 ? SANS : MONO)}">${esc(t)}</text>`;

/* one mind · many languages · open world */
const thesisFig = () => {
  const W = 760, H = 280; let o = '';
  const cols = [['one mind', MINT, [['one form', 'one multilinear form over one field'], ['one proof', 'the prover proves the prover'], ['one focus', 'the graph settles into one φ⋆']]], ['many languages', VC.learn, [['write', 'cybermark'], ['compute', 'the trident family'], ['mean', 'neural']]], ['open world', VC.focus, [['it holds', 'one fabric, planet scale'], ['it composes', 'add a cyberlink, not an API'], ['it admits', 'no schema, no gatekeeper']]]];
  cols.forEach(([h, col, rows], i) => { const x = 20 + i * 248; o += `<rect x="${x}" y="16" width="232" height="246" rx="14" fill="${col}" fill-opacity=".08" stroke="${col}" stroke-opacity=".7"/>` + T(x + 116, 48, h, { fs: 17, w: 700, a: 'middle' }); rows.forEach(([a, b], k) => { const y = 92 + k * 54; o += T(x + 18, y, a, { fs: 12.5, w: 700, c: col }) + T(x + 18, y + 17, b, { fs: 9.5, op: .75 }); }); });
  o += T(380, 276, '31 components · each the same substrate specialized to one job', { fs: 9.5, a: 'middle', op: .6 });
  return svgo(W, H, 'one mind, many languages, open world', o);
};

/* the loop: seven verbs on a ring */
const loopFig = () => {
  const W = 760, H = 330, cx = 380, cy = 165, R = 118; let o = '';
  VERBS.forEach((v, i) => { const a = -Math.PI / 2 + i * 2 * Math.PI / 7, x = cx + Math.cos(a) * R, y = cy + Math.sin(a) * R; const col = VC[v.id] || v.c; const na = a + 2 * Math.PI / 7, nx = cx + Math.cos(na) * R, ny = cy + Math.sin(na) * R; o += `<path d="M${(cx + Math.cos(a + .22) * R).toFixed(1)},${(cy + Math.sin(a + .22) * R).toFixed(1)} A${R},${R} 0 0 1 ${(cx + Math.cos(na - .22) * R).toFixed(1)},${(cy + Math.sin(na - .22) * R).toFixed(1)}" fill="none" stroke="currentColor" stroke-opacity=".35" stroke-width="1.5"/>`; o += `<circle cx="${x.toFixed(1)}" cy="${y.toFixed(1)}" r="24" fill="${col}" fill-opacity=".18" stroke="${col}" stroke-width="2"/>` + T(x, y + 4, v.id, { fs: 11, w: 700, a: 'middle' }); const lx = cx + Math.cos(a) * (R + 92), ly = cy + Math.sin(a) * (R + 92); const anchor = Math.cos(a) > .3 ? 'start' : Math.cos(a) < -.3 ? 'end' : 'middle'; o += T(lx, ly - 4, v.who, { fs: 10.5, w: 700, a: anchor, c: col }) + T(lx, ly + 9, v.does.split(' — ')[0], { fs: 8.5, a: anchor, op: .8 }); });
  o += T(cx, cy - 6, 'one loop', { fs: 13, w: 700, a: 'middle' }) + T(cx, cy + 10, 'seven verbs · one focus', { fs: 9, a: 'middle', op: .7 });
  return svgo(W, H, 'the loop', o);
};

/* the ladder: ten rungs */
const ladderFig = () => {
  const W = 760, H = 300; let o = '';
  HUBS.forEach((n, i) => { const y = 14 + i * 28, col = i < 7 ? FACET_C[i] : MINT; o += `<rect x="60" y="${y}" width="640" height="22" rx="5" fill="${col}" fill-opacity="${i < 7 ? .12 : .06}" stroke="${col}" stroke-opacity=".6"/>` + T(40, y + 15, String(i + 1).padStart(2, '0'), { fs: 10, a: 'end', op: .6 }) + T(72, y + 15, n.id, { fs: 12, w: 700 }) + T(190, y + 15, n.role, { fs: 9.5, c: col }) + T(300, y + 15, n.thesis, { fs: 9, op: .8 }) + T(690, y + 15, i < 7 ? n.q : 'app', { fs: 8.5, a: 'end', op: .6 }); });
  return svgo(W, H, 'the ladder', o);
};

/* registry: states */
const registryFig = (reg) => {
  const W = 760, H = 230; let o = ''; const states = ['live', 'published', 'unwired', 'local', 'spec', 'blocked', 'dormant']; const cnt = Object.fromEntries(states.map((s) => [s, reg.filter((r) => r.state === s).length])); const mx = Math.max(...Object.values(cnt));
  states.forEach((s, i) => { const x = 40 + i * 100, h = 120 * cnt[s] / mx; o += `<rect x="${x}" y="${150 - h}" width="72" height="${h}" rx="4" fill="${s === 'live' ? MINT : 'currentColor'}" fill-opacity="${s === 'live' ? .8 : .25}"/>` + T(x + 36, 142 - h, String(cnt[s]), { fs: 18, w: 700, a: 'middle' }) + T(x + 36, 170, s, { fs: 10, a: 'middle', op: .8 }); });
  o += T(380, 205, `${reg.length} components in the registry · ${cnt.live} carry real traffic today · a release is a conformance snapshot`, { fs: 9.5, a: 'middle', op: .65 });
  return svgo(W, H, 'the registry', o);
};

/* the buyer's twelve questions */
const twelveFig = () => {
  const Q = ['it works when the other party is hostile', 'it outlives its authors', 'one person can run it', 'it speaks to the old world', 'a unit of account, a reason to hold the bytes', 'operations are boring', 'a name a human can say', 'a key can be rotated, given, buried', 'a partition leaves local life intact', 'a fork is a constitution, not a catastrophe', 'outsiders can audit it', 'a child can learn it'];
  const W = 760, H = 250; let o = '';
  Q.forEach((q, i) => { const col = i % 2, row = Math.floor(i / 2), x = 30 + col * 370, y = 22 + row * 38; const who = i < 3 ? 'bitcoin' : i === 3 || i === 10 ? 'linux' : i === 6 ? 'unicode' : ''; o += T(x, y, String(i + 1).padStart(2, '0'), { fs: 10, c: MINT }) + T(x + 28, y, q, { fs: 11.5 }) + (who ? T(x + 350, y, who, { fs: 8.5, a: 'end', op: .5 }) : ''); });
  o += T(380, 244, 'bitcoin closed 1–3 · linux 2–4 and 11 · unicode 7 · soft3 aims at all twelve, plus a mind and a state', { fs: 9.5, a: 'middle', op: .65 });
  return svgo(W, H, 'the twelve questions', o);
};

/* the three constitutions */
const phasesFig = () => {
  const W = 760, H = 230; let o = '';
  const P = [['0 · visor', 'now', 'the shop window matches the stack', MINT], ['A · person', 'key · write · hold', 'vault recovery · one neuron on two devices · priced writes · storage market · node health · a human entrance', VC.compute], ['B · network', 'upgrade · fork · law', 'parameter change with a height · fork as a right · deletion policy with an owner · anti-capture · paid audit · guardianship · offline a day', VC.build], ['C · world', 'joule · iron · bridge · court', 'joule as a unit · RISC-V and plain Linux · bridges into HTTP, mail, BTC, paper · compaction · a second full node · a deed a court can read', VC.focus]];
  P.forEach(([n, when, what, col], i) => { const x = 20 + i * 185; o += `<rect x="${x}" y="16" width="170" height="196" rx="12" fill="${col}" fill-opacity=".08" stroke="${col}" stroke-opacity=".7"/>` + T(x + 12, 42, n, { fs: 14, w: 700 }) + T(x + 12, 60, when, { fs: 9, c: col }); const words = what.split(' '); let line = '', y = 84; for (const w of words) { if ((line + ' ' + w).trim().length > 26) { o += T(x + 12, y, line, { fs: 9, op: .8 }); y += 13; line = w; } else line = (line + ' ' + w).trim(); } o += T(x + 12, y, line, { fs: 9, op: .8 }); if (i < 3) o += `<path d="M${x + 190},114 l-8,-5 v10 z" fill="currentColor" fill-opacity=".5"/>`; });
  return svgo(W, H, 'three constitutions', o);
};

/* ------------------------------------------------------------------ chapters */
const CH = [
  { id: 'what', name: 'what', c: MINT }, { id: 'ladder', name: 'the ladder', c: '#ffffff' },
  ...VERB_ORDER.map((v) => ({ id: v, name: v, c: VC[v] })),
  { id: 'frontier', name: 'the frontier', c: VC.learn }, { id: 'doors', name: 'doors', c: MINT },
];
const chIdx = (id) => CH.findIndex((c) => c.id === id);
const byVerb = (v) => FEATURES.filter((f) => f.verb === v);
const partsOf = (hub) => [...new Set(EDGES.filter((e) => e.b === hub).map((e) => e.a))];
const stateOf = (name, reg) => reg.find((r) => r.name === name);

const feat = (f) => `
  <div><p class="kick">${f.verb} · ${esc(f.name)}</p><h2>${esc(f.claim)}</h2>
  <p class="lead">${esc(f.line)}</p><p class="mute">${esc(f.why)}</p>
  <div class="parts">${f.parts.map((p) => `<a class="chip" href="https://cyber.page/${p}/" target="_blank" rel="noopener">${p}</a>`).join('')}</div>
  <p class="fine"><b class="st">${esc(f.status)}</b> · <a href="${f.source[1]}" target="_blank" rel="noopener">${esc(f.source[0])}</a> · <a href="/features/?f=${f.id}">the card</a></p></div>
  <div class="detail deckdetail" style="--c:${VC[f.verb]}"><div class="fig" data-feat="${f.id}"></div></div>`;

const S = [];
const push = (ch, html, cls = '', extra = {}) => S.push({ ch: chIdx(ch), cls, html, ...extra });

/* act 1 · what */
push('what', `
  <div><p class="kick">soft3 · collective, private, provable computing</p>
  <p class="hero-word">soft3</p>
  <h2>one mind. <em>many languages. open world.</em></h2>
  <p class="lead">a frozen stack a sovereign mind runs on: one field, one hash, one proof, one focus — through every layer. thirty-one components, thirty features, seven verbs, ten rungs. every number here says measured or target.</p>
  <div class="tiles"><div class="tile"><b>one form</b>one multilinear form over one field</div><div class="tile"><b>one proof</b>the prover proves the prover</div><div class="tile"><b>one focus</b>the graph settles into one φ⋆</div><div class="tile"><b>275k lines</b>against 39M with LLVM · measured</div></div>
  <p class="kbd">→ or space to move · <span id="nslides"></span> slides · click the rail to jump</p></div>`, 'one');
push('what', `<div><p class="kick">01 · what · the thesis</p><h2>thirty-one components,<br><em>three ideas.</em></h2>${thesisFig()}<p class="mute">the components are not the foundation. each one is the same substrate specialized to one job. learn the three ideas and thirty-one repos stop being thirty-one things to memorize.</p></div>`, 'one wash');
push('what', `<div><p class="kick">01 · what · one replaces many</p><h2>one primitive<br><em>where others build many.</em></h2>
  <p class="lead">every foundational method in soft3 has the same shape: it takes something other systems build as many separate mechanisms and collapses it into one universal primitive.</p></div>
  <div class="steps"><div class="step"><b>one hash is the identity</b><span>other systems assign ids from a registry; here every object's hemera hash is its address</span></div><div class="step"><b>one commitment is the state</b><span>a hash tree here, a commitment scheme there, a proof system elsewhere — here one multilinear polynomial over one field; a read is one opening, ~200 bytes</span></div><div class="step"><b>one proof is the run</b><span>the nox execution trace is the constraint system; running and proving are one act; proofs verify proofs</span></div><div class="step"><b>composition is free</b><span>bbg, zheng, radio and fs reach for the same primitive, so there is nothing to serialize and nothing to translate</span></div></div>`);
push('what', `<div><p class="kick">01 · what · the buyer</p><h2>the buyer is an operator<br><em>of an institution.</em></h2>${twelveFig()}<p class="mute">not a zkVM developer. a citizen, a node, a town, an archive, a wallet, a school. the twelve questions such a buyer asks, in order — the checklist this stack is scored against.</p></div>`, 'one wash');
push('what', `<div><p class="kick">01 · what · the loop</p><h2>seven verbs.<br><em>one loop.</em></h2>${loopFig()}<p class="mute">a focus becomes a computation, a computation a program, a program a proof, a proof an answer, the book a model, a thought a link, the links one focus — and whoever moved it is paid. the thirty features sit on these seven verbs.</p></div>`, 'one wash', { bridge: 'before the features, the shape: ten rungs, one ladder.' });

/* act 2 · the ladder */
push('ladder', `<div><p class="kick">02 · the ladder</p><h2>ten rungs.<br><em>seven questions, three doors.</em></h2>${ladderFig()}<p class="mute">rung k answers the k-th question of cybics — what are the rules, what is it made of, where does it happen, what lives, what does it mean, who does the work, how do we coordinate — and the last three are the bodies that run on the stack.</p></div>`, 'one wash');
HUBS.forEach((n, i) => { const col = i < 7 ? FACET_C[i] : MINT; const parts = partsOf(n.id); push('ladder', `
  <div><p class="kick">02 · the ladder · rung ${String(i + 1).padStart(2, '0')} · ${i < 7 ? FACET[i] + ' · ' + esc(n.q) : 'apps'}</p><h2>${n.id}<br><em>${esc(n.role)}.</em></h2>
  <p class="lead">${esc(n.thesis)}.</p>
  <p class="mute">verb: <b>${esc(n.verb || '')}</b>${parts.length ? ' · fed by ' + parts.length + ' component' + (parts.length > 1 ? 's' : '') + ' in the dependency chart' : ''}</p>
  <div class="parts">${[n.id, ...parts].map((p) => `<a class="chip" href="https://cyber.page/${p}/" target="_blank" rel="noopener">${p}</a>`).join('')}</div></div>
  <div class="panel rung-panel" data-rung="${n.id}" style="--c:${col}"></div>`, '', { rung: n.id, parts: [n.id, ...parts] }); });
push('ladder', `<div><p class="kick">02 · the ladder · the registry</p><h2>every component,<br><em>its honest state.</em></h2><div class="fig" id="registry-fig"></div><p class="mute">live carries real traffic today; published is released and correct but not in the live loop; unwired is built and not connected; spec is paper. the verdict: the stack is complete in principle and peer-pair in fact — the wire exists, and the graph routes it.</p></div>`, 'one wash', { bridge: 'now the thirty features, verb by verb.' });

/* act 3 · seven verbs × thirty features */
VERB_ORDER.forEach((v, vi) => {
  const vv = VERBS.find((x) => x.id === v); const fs = byVerb(v);
  push(v, `<div><p class="kick">${String(vi + 3).padStart(2, '0')} · ${v} · ${esc(vv.who)}</p><h2>${v}.<br><em>${esc(vv.does.split(' — ')[0])}.</em></h2><p class="lead">${esc(vv.does.includes(' — ') ? vv.does.split(' — ')[1] : '')}</p><p class="mute">${fs.length} features · hub ${vv.sel} · parts ${vv.parts.join(' · ')}</p></div>
    <div class="nums">${fs.map((f) => `<div class="n"><big style="font-size:clamp(18px,1.8vw,28px)">${esc(f.name)}</big><small>${esc(f.status)}</small></div>`).join('')}</div>`, 'ink', { bridge: vi < 6 ? `${v} done. next: ${VERB_ORDER[vi + 1]}.` : 'the thirty are the physics of a shared mind. the frontier is the institutions.' });
  fs.forEach((f) => push(v, feat(f), '', { feat: f.id }));
});

/* act 4 · the frontier */
push('frontier', `<div><p class="kick">10 · the frontier · rooted civilization</p><h2>the wall is physics.<br><em>civilization is institutions.</em></h2>
  <p class="lead">the thirty features are the physics of a shared mind. the market the stack enters is rooted civilizational software: what a village, a ship, a city can run when the cloud, the state and the vendor are gone.</p>
  <p class="mute">a rooted stack is grounded in physics, in people, in a record readable in a hundred years, and in the right to leave. the proposal that owns this: rooted civilization, 2026-10-13.</p></div>
  <div class="steps"><div class="step"><b>already held, not on the window</b><span>measurement (lytics) · the bootloader memory · the money · publishing · warriors · vault · conformance · the wire · silicon · land and deed</span></div><div class="step"><b>what civilization already holds</b><span>joules and light · identity that outlives a body · a record a phone can check · names a human says · shared time · the right to leave · a bridge into the old world · boredom · several implementations</span></div></div>`);
push('frontier', `<div><p class="kick">10 · the frontier · the forgotten seven</p><h2>critical, <em>and now owned.</em></h2><p class="lead">seven questions no component roadmap owned. each one is ignorable today and existential at the first real scale. the rooted-civilization proposal takes them as its phases A and B.</p></div>
  <div class="steps"><div class="step"><b>the economics of writing</b><span>who may write, at what cost — an open graph with free writes drowns on day one</span></div><div class="step"><b>key lifecycle</b><span>nothing recovers, rotates or revokes; one person on two devices is two neurons</span></div><div class="step"><b>protocol upgrade</b><span>changing foculus parameters on a live network is undefined</span></div><div class="step"><b>deletion and liability</b><span>illegal bytes will arrive; the hardest political question, currently unasked</span></div><div class="step"><b>storage economics</b><span>nobody has a reason to hold the bytes</span></div><div class="step"><b>adversarial audit</b><span>someone paid to break it must try</span></div><div class="step"><b>operations</b><span>networks die of boring things</span></div></div>`);
push('frontier', `<div><p class="kick">10 · the frontier · the upgrade</p><h2>three<br><em>constitutions.</em></h2>${phasesFig()}<p class="mute">four phases, each a last-merge: the tests of every repository it names are green, and the shop window says measured only of what is measured. A does not wait for 64 KB proofs; B waits for a clock; C waits for a frozen proof format.</p></div>`, 'one wash');
push('frontier', `<div><p class="kick">10 · the frontier · the proof</p><h2>≤ 64 KB.<br><em>post-quantum. ≤ 1 ms.</em></h2><p class="lead">the proof goal, fixed 2026-10-09: at most 64 KB, small statements at most 20 KB, hash-only, verified in a millisecond, constant size by accumulation. today's hash relation proof is 15.6 KB; the public profile is sound and not yet succinct.</p></div>
  <div class="nums"><div class="n"><big>15.6<span class="u">KB</span></big><small>hash relation proof today · measured</small></div><div class="n"><big>≤ 64<span class="u">KB</span></big><small>the bound · target</small></div><div class="n"><big>≤ 20<span class="u">KB</span></big><small>small statements · target</small></div><div class="n"><big>0</big><small>trusted setups, curves, pairings</small></div></div>`);
push('frontier', `<div><p class="kick">10 · the frontier · the train</p><h2>a release is<br><em>a frozen snapshot.</em></h2><p class="lead">every friday 12:00 UTC a candidate is cut from origin default branches only, gated by each repo's gates, published as a draft with receipts, and promoted by the owner alone. the pins for phase 1 live in one manifest; a release is a conformance snapshot, fingerprinted with hemera.</p></div>
  <div class="steps"><div class="step"><b>chaosnet · spacepussy-test</b><span>the real soft3 network: cybergraph + bbg, public rpc at cyb.ai, one signal one block</span></div><div class="step"><b>2026-11-05 · the bootloader reborn</b><span>bostrom and space pussy relaunch on soft3 with settlement mining, fold, consensus by convergence, privacy</span></div><div class="step"><b>cargo install soft3 · true-cyber</b><span>operator and product face, on crates.io</span></div></div>`, '', { bridge: 'the stack is headless. three bodies run on it.' });

/* act 5 · doors */
push('doors', `<div><p class="kick">11 · doors</p><h2>the stack is headless.<br><em>three bodies run on it.</em></h2>
  <div class="tiles"><a class="tile" href="https://cyb.ai" target="_blank" rel="noopener"><b>cyb</b>the robot · cyb.ai</a><a class="tile" href="https://cyber.page"><b>cyber</b>the superintelligence · cyber.page</a><a class="tile" href="https://cyberia.to" target="_blank" rel="noopener"><b>cyberia</b>the state · cyberia.to</a></div>
  <p class="lead">one field, one proof, one focus, through every layer — and three doors into it.</p>
  <div class="ctas"><a class="btn" href="/features/">the thirty features</a><a class="btn ghost" href="/layers/">the layers</a><a class="btn ghost" href="/loop/">the loop</a><a class="btn ghost" href="/chart/">the chart</a><a class="btn ghost" href="https://github.com/cyberia-to/soft3" target="_blank" rel="noopener">github</a><a class="btn ghost" href="#0" id="restart">start again</a></div>
  <p class="fine" style="margin-top:18px">measured · target · proposed — every status word on every feature is the one its card carries. cyberia.meme · cyber.page · soft3.org</p></div>`, 'one ink');

/* ------------------------------------------------------------------ build */
const deck = $('#deck'), segs = $('#segs');
S.forEach((s, i) => deck.appendChild(el(`<section class="slide ch${s.ch} ${s.cls || ''}" data-i="${i}" data-ch="${String(s.ch + 1).padStart(2, '0')}" style="--c:${CH[s.ch].c}"><div class="body">${s.html}</div>${s.bridge ? `<p class="bridge">→ ${esc(s.bridge)}</p>` : ''}</section>`)));
CH.forEach((c, i) => { const n = S.filter((s) => s.ch === i).length; const d = el(`<div class="seg" style="--sc:${c.c};flex:${Math.max(1, Math.sqrt(n))}" data-ch="${i}"><div class="fill"></div><span class="lab">${c.name}</span></div>`); d.addEventListener('click', () => go(S.findIndex((s) => s.ch === i))); segs.appendChild(d); });
/* figures from features.js render into their slide once; the registry panels fill from registry.json */
S.forEach((s, i) => { if (s.feat) { const f = FEATURES.find((x) => x.id === s.feat); const host = deck.querySelectorAll('.slide')[i].querySelector('.fig'); try { host.innerHTML = f.fig(); } catch (e) { host.innerHTML = `<p class="note">figure unavailable: ${esc(e.message)}</p>`; } } });
fetch('/registry.json').then((r) => r.json()).then((reg) => {
  $('#registry-fig').innerHTML = registryFig(reg);
  document.querySelectorAll('.rung-panel').forEach((p) => { const names = S.find((s) => s.rung === p.dataset.rung).parts; const rows = names.map((n) => stateOf(n, reg)).filter(Boolean); p.innerHTML = `<label>in the registry</label><table class="regt">${rows.map((r) => `<tr><td><b>${r.name}</b></td><td>${esc(r.verb)}</td><td class="st-${r.state}">${r.state}</td><td>${r.release || '—'}</td><td class="dim">${esc(r.what)}</td></tr>`).join('')}</table>${rows.length ? '' : '<p class="fine">not in the registry table</p>'}`; });
}).catch(() => {});

$('#nslides').textContent = S.length;
let cur = -1;
const fit = () => {
  const sl = document.querySelectorAll('.slide')[cur]; if (!sl) return;
  const body = sl.querySelector('.body');
  Object.assign(body.style, { transform: '', width: '', height: '', maxHeight: '', margin: '' }); sl.classList.remove('tight', 'tighter', 'scaled');
  const availW = body.clientWidth, availH = body.clientHeight;
  const over = () => body.scrollHeight > availH + 1 || body.scrollWidth > availW + 1;
  if (!over()) return;
  sl.classList.add('tight'); if (!over()) return;
  sl.classList.add('tighter'); if (!over()) return;
  let z = Math.min(availH / body.scrollHeight, availW / body.scrollWidth);
  sl.classList.add('scaled');
  for (let k = 0; k < 5; k++) {
    z = Math.max(.4, Math.floor(z * 100) / 100);
    Object.assign(body.style, { width: availW / z + 'px', height: availH / z + 'px', maxHeight: 'none', margin: '0', transform: `scale(${z})` });
    if (body.scrollHeight <= availH / z + 1 && body.scrollWidth <= availW / z + 1) break;
    z *= .96;
  }
};

let fitT = 0; const refit = () => { clearTimeout(fitT); fitT = setTimeout(fit, 40); };
addEventListener('resize', refit); addEventListener('orientationchange', refit); document.fonts?.ready.then(refit);
const go = (i) => {
  i = Math.max(0, Math.min(S.length - 1, i)); if (i === cur) return;
  const dir = i > cur ? 1 : -1, prevCh = cur >= 0 ? S[cur].ch : -1; cur = i; deck.dataset.dir = dir;
  document.querySelectorAll('.slide').forEach((s, k) => { s.classList.toggle('on', k === i); s.classList.toggle('left', k < i); });
  document.body.classList.toggle('ink', (S[i].cls || '').includes('ink')); document.body.className = document.body.className.replace(/\bch\d+\b/g, '').trim() + ' ch' + S[i].ch;
  const ch = S[i].ch, idx = S.filter((s, k) => s.ch === ch && k <= i).length, n = S.filter((s) => s.ch === ch).length;
  document.querySelectorAll('.seg').forEach((sg, k) => { sg.classList.toggle('cur', k === ch); sg.classList.toggle('done', k < ch); $('.fill', sg).style.width = k < ch ? '100%' : k === ch ? (idx / n * 100) + '%' : '0'; });
  sfx.move(dir, ch !== prevCh ? ch % 7 : null); if (navigator.vibrate) navigator.vibrate(8);
  world.chapter(CH[ch].c); world.pulse();
  $('#pos').textContent = `${String(i + 1).padStart(2, '0')} / ${S.length}`;
  history.replaceState(null, '', '#' + i);
  requestAnimationFrame(() => requestAnimationFrame(fit));
};
window.deckGo = go; window.deckCount = () => S.length; window.deckFit = fit;
$('#prev').onclick = () => go(cur - 1); $('#next').onclick = () => go(cur + 1);
document.addEventListener('keydown', (e) => { if (e.target.tagName === 'INPUT' || (e.target.tagName === 'BUTTON' && [' ', 'Enter'].includes(e.key))) return; if (['ArrowRight', ' ', 'PageDown', 'ArrowDown'].includes(e.key)) { e.preventDefault(); go(cur + 1); } if (['ArrowLeft', 'PageUp', 'ArrowUp'].includes(e.key)) { e.preventDefault(); go(cur - 1); } if (e.key === 'Home') go(0); if (e.key === 'End') go(S.length - 1); });
let wheelLock = 0; document.addEventListener('wheel', (e) => { const t = Date.now(); if (t - wheelLock < 700 || Math.abs(e.deltaY) < 24) return; wheelLock = t; go(cur + (e.deltaY > 0 ? 1 : -1)); }, { passive: true });
let tx = null, ty = 0; document.addEventListener('touchstart', (e) => { tx = e.touches[0].clientX; ty = e.touches[0].clientY; }, { passive: true });
document.addEventListener('touchend', (e) => { if (tx == null) return; const dx = e.changedTouches[0].clientX - tx, dy = e.changedTouches[0].clientY - ty; if (Math.abs(dx) > 60 && Math.abs(dx) > Math.abs(dy)) go(cur + (dx < 0 ? 1 : -1)); tx = null; });
$('#restart')?.addEventListener('click', (e) => { e.preventDefault(); go(0); });
document.addEventListener('pointerdown', (e) => { sfx.wake(); if (e.target.closest('button, a, .seg, .chip')) sfx.tick(); }, { passive: true });
document.addEventListener('keydown', () => sfx.wake(), { once: true });
const sndBtn = $('#snd'); const sndPaint = () => { sndBtn.textContent = sfx.on ? '◉ sound' : '○ sound'; sndBtn.classList.toggle('off', !sfx.on); }; sndBtn.onclick = () => { sfx.toggle(); sndPaint(); }; sndPaint();
if (matchMedia('(pointer:fine)').matches) { const dot = el('<div class="cur-dot"></div>'), ring = el('<div class="cur-ring"></div>'); document.body.append(dot, ring); let rx = 0, ry = 0, px = 0, py = 0; addEventListener('mousemove', (e) => { px = e.clientX; py = e.clientY; dot.style.transform = `translate(${px}px,${py}px)`; ring.classList.toggle('hot', !!e.target.closest('button, a, .seg, .chip, .fig')); }, { passive: true }); const lag = () => { rx += (px - rx) * .18; ry += (py - ry) * .18; ring.style.transform = `translate(${rx}px,${ry}px)`; requestAnimationFrame(lag); }; lag(); document.body.classList.add('has-cursor'); }

/* ------------------------------------------------------------------ background: the dependency chart as a living graph */
const world = (() => {
  const cv = $('#graph'), ctx = cv.getContext('2d'); let W = 0, H = 0, hue = MINT, pulse = 0;
  const resize = () => { W = cv.width = innerWidth * devicePixelRatio; H = cv.height = innerHeight * devicePixelRatio; }; addEventListener('resize', resize); resize();
  const ids = [...new Set(EDGES.flatMap((e) => [e.a, e.b]))]; const P = Object.fromEntries(ids.map((id, i) => { const a = i / ids.length * Math.PI * 2, r = .28 + .18 * Math.random(); return [id, { x: .5 + Math.cos(a) * r, y: .5 + Math.sin(a) * r * .8, vx: 0, vy: 0, hub: !!NODES.find((n) => n.id === id && n.hub) }]; }));
  const tick = () => { ctx.clearRect(0, 0, W, H); const t = performance.now() / 1000; for (const id of ids) { const p = P[id]; p.x += Math.sin(t * .2 + p.y * 9) * .0003; p.y += Math.cos(t * .17 + p.x * 7) * .0003; }
    ctx.lineWidth = .6 * devicePixelRatio; ctx.strokeStyle = hue; ctx.globalAlpha = .13 + pulse * .25; ctx.beginPath(); for (const e of EDGES) { const a = P[e.a], b = P[e.b]; ctx.moveTo(a.x * W, a.y * H); ctx.lineTo(b.x * W, b.y * H); } ctx.stroke();
    for (const id of ids) { const p = P[id]; const r = (p.hub ? 4 : 1.8) * devicePixelRatio * (1 + pulse * .6); ctx.globalAlpha = p.hub ? .7 : .35; ctx.fillStyle = p.hub ? '#fff' : hue; ctx.beginPath(); ctx.arc(p.x * W, p.y * H, r, 0, Math.PI * 2); ctx.fill(); if (p.hub) { ctx.globalAlpha = .45; ctx.font = `${10 * devicePixelRatio}px ${MONO}`; ctx.fillText(id, p.x * W + 8 * devicePixelRatio, p.y * H + 3 * devicePixelRatio); } }
    ctx.globalAlpha = 1; pulse *= .95; requestAnimationFrame(tick); };
  tick(); return { chapter(c) { hue = c; }, pulse() { pulse = Math.max(pulse, .8); } };
})();

addEventListener('hashchange', () => go(parseInt(location.hash.slice(1)) || 0));
go(parseInt(location.hash.slice(1)) || 0);
})();
