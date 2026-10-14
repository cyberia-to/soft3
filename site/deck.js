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
  const P = [['0 · visor', 'now', 'the shop window matches the stack', MINT], ['A · person', 'key · write · hold', 'vault recovery · one neuron on two devices · priced writes · storage market · node health · a human entrance', VC.compute], ['B · network', 'upgrade · fork · law', 'parameter change with a height · fork as a right · information is legal, written down · anti-capture · paid audit · guardianship · offline a day', VC.build], ['C · world', 'joule · iron · bridge · court', 'joule as a unit · RISC-V and plain Linux · bridges into HTTP, mail, BTC, paper · compaction · a second full node · a deed a court can read', VC.focus]];
  P.forEach(([n, when, what, col], i) => { const x = 20 + i * 185; o += `<rect x="${x}" y="16" width="170" height="196" rx="12" fill="${col}" fill-opacity=".08" stroke="${col}" stroke-opacity=".7"/>` + T(x + 12, 42, n, { fs: 14, w: 700 }) + T(x + 12, 60, when, { fs: 9, c: col }); const words = what.split(' '); let line = '', y = 84; for (const w of words) { if ((line + ' ' + w).trim().length > 26) { o += T(x + 12, y, line, { fs: 9, op: .8 }); y += 13; line = w; } else line = (line + ' ' + w).trim(); } o += T(x + 12, y, line, { fs: 9, op: .8 }); if (i < 3) o += `<path d="M${x + 190},114 l-8,-5 v10 z" fill="currentColor" fill-opacity=".5"/>`; });
  return svgo(W, H, 'three constitutions', o);
};

/* ------------------------------------------------------------------ chapters */
const CH = [
  { id: 'why', name: 'why', c: MINT, h: null }, { id: 'only', name: 'only here', c: MINT, h: null }, { id: 'shape', name: 'the shape', c: '#ffffff', h: null },
  ...VERB_ORDER.map((v) => ({ id: v, name: v, c: VC[v], h: VERB[v].h })),
  { id: 'institutions', name: 'institutions', c: VC.learn, h: null }, { id: 'entrance', name: 'the entrance', c: MINT, h: null },
];
const chIdx = (id) => CH.findIndex((c) => c.id === id);
const byVerb = (v) => FEATURES.filter((f) => f.verb === v);
const partsOf = (hub) => [...new Set(EDGES.filter((e) => e.b === hub).map((e) => e.a))];
const stateOf = (name, reg) => reg.find((r) => r.name === name);

const feat = (f) => `
  <div><p class="kick">${f.verb} · ${esc(f.name)}</p><h2>${esc(f.claim)}</h2>
  <p class="lead">${esc(f.line)}</p><p class="mute">${esc(f.why)}</p>
  <div class="parts">${f.parts.map((p) => `<a class="chip" href="https://cyber.page/${p}/" target="_blank" rel="noopener">${p}</a>`).join('')}</div>
  <p class="fine"><b class="st">${esc(f.status)}</b> · <a href="${f.source[1]}" target="_blank" rel="noopener">${esc(f.source[0])}</a> · <a href="#v-${f.verb}">the verb</a></p></div>
  <div class="detail deckdetail" style="--c:${VC[f.verb]}"><div class="fig" data-feat="${f.id}"></div></div>`;

const S = [];
const push = (ch, html, cls = '', extra = {}) => S.push({ ch: chIdx(ch), cls, html, ...extra });
const link = (href, t) => `<a href="${href}" target="_blank" rel="noopener">${t}</a>`;

/* act 1 · why — the product case, from the rooted-civilization proposal and the litepaper */
push('why', `
  <div><p class="kick">soft3 · collective, private, provable computing</p>
  <p class="hero-word">soft3</p>
  <h2>compute once. prove it. <em>everyone remembers.</em></h2>
  <p class="lead">a computer the whole world shares, that keeps every proof it has seen and pays whoever moves its focus. one field, one hash, one proof, one focus — through every layer. a frozen stack a sovereign mind runs on.</p>
  <div class="tiles"><div class="tile"><b>one mind</b>one form · one proof · one focus</div><div class="tile"><b>many languages</b>write · compute · mean</div><div class="tile"><b>open world</b>it holds · composes · admits</div><div class="tile"><b>cargo install soft3</b>a real node, a public chaosnet</div></div>
  <p class="kbd">→ or space to move · <span id="nslides"></span> slides · click the rail to jump</p></div>`, 'one', { id: 'home' });
push('why', `<div><p class="kick">01 · why · the buyer</p><h2>the buyer is not a zkVM developer.<br><em>the buyer runs an institution.</em></h2>
  <p class="lead">a citizen, a node, a town, an archive, a wallet, a school. civilizational software is what a village, a ship, a city can run when the captured stack is dark.</p>
  <p class="mute">the developer is the second buyer: cargo install soft3, a real node, five operations — particle · cyberlink · query · verify · submit. the first buyer never sees a compiler.</p></div>
  <div class="steps"><div class="step"><b>if you build models</b><span>your architecture is guessed and your data unattributable; here both are derived, and the teachers can be paid</span></div><div class="step"><b>if you build chains</b><span>the hash you grind computes who deserves credit</span></div><div class="step"><b>if you allocate capital</b><span>this is the settlement of meaning, and a currency no central bank and no miner cartel can debase</span></div><div class="step"><b>if you have something true to say</b><span>link it, stake it, and be paid when the picture sharpens</span></div></div>`);
push('why', `<div><p class="kick">01 · why · the captured stack</p><h2>civilization already runs<br><em>on a captured stack.</em></h2>
  <p class="lead">DNS, PKI and KYC for who. Visa and SWIFT for value. Linux, iOS and AWS for compute. Google, GitHub and Wikipedia for knowledge. TCP and SMTP for speech. NTP and GPS for time and place. closed models for mind.</p>
  <p class="mute">money settles without a bank; meaning still rents its rank from whoever owns the index. you cannot open the weights and see why this answer. you cannot pay the teacher. the world has a public ledger for value and none for meaning.</p></div>
  <div class="nums"><div class="n"><big>39<span class="u">M</span></big><small>lines under a typical chain stack with LLVM — nobody has read them, so nobody can freeze them · measured</small></div><div class="n"><big>275<span class="u">k</span></big><small>lines of soft3's own code — small enough to understand, verify, then freeze · measured</small></div><div class="n"><big>1973</big><small>since when an address has confused who you are with where you are</small></div><div class="n"><big>0</big><small>public ledgers for meaning, before this one</small></div></div>`);
push('why', `<div><p class="kick">01 · why · what the market buys</p><h2>the market does not<br><em>buy sumcheck.</em></h2>
  <p class="lead">it buys identity that survives a death, a unit of account, a record a phone can check, a clock everyone shares, an upgrade that does not split the tribe, and boring operations.</p>
  <p class="mute">a rooted stack is grounded in physics — joules, light, silicon — in people — keys, names, inheritance — in a record readable in a hundred years, and in the right to leave. without that soft3 is a great compiler. with it, it is a root.</p></div>
  <div class="steps"><div class="step"><b>a program written today</b><span>runs and proves the same way in fifty years — nox frozen, hemera candidate</span></div><div class="step"><b>every answer carries its proof</b><span>a phone checks it with 32 bytes — building</span></div><div class="step"><b>the one who proved it first</b><span>is paid each time the world remembers — memoization, specified</span></div><div class="step"><b>your own graph, on your own terms</b><span>registered into the one — home books, phase 1</span></div></div>`, '', { bridge: 'four things no other stack does. with their numbers.' });

/* act 2 · only here */
const only = [
  ['collective focus', 'one ranking, computed by everyone — a theorem, not a vote', 'every neuron runs the same contraction on the same graph and lands on the same focus; nobody votes, nobody is elected. measured on bostrom: the kernel contracts at κ = 0.74.', 'κ = 0.74', 'measured on bostrom', 'cfocus'],
  ['compile the graph into a transformer', 'the weights are read off the graph, not trained', 'embedding dimension from spectral entropy, heads from dialects, depth from diameter times convergence rate. on the bootloader corpus tru did this in 62 seconds on one machine, and every weight traces to the neurons whose links produced it.', '62 s', 'measured · d⋆ = 31 · 290 layers', 'models'],
  ['graph, not ledger', 'the state is the graph — there is no ledger', 'bostrom\'s 2,949,732 links were rebuilt from events this way, identical, byte for byte. no floats, no clocks, no randomness: the same answer on every device.', '2,949,732', 'replayed identical · live', 'graph'],
  ['the wire obeys the graph', 'following is a link, an address is a link', 'proven with three nodes: the third reached the first through the second by reading the address out of the graph. a route is graph content; who talks to whom is readable from the graph itself.', '3 nodes', 'live · graph-routed', 'wire'],
];
only.forEach(([name, claim, why, big, st, fid], k) => push('only', `<div><p class="kick">02 · only here · ${k + 1} of 4</p><h2>${esc(claim)}</h2><p class="lead">${esc(why)}</p><p class="fine"><b class="st">${esc(st)}</b> · ${esc(name)} · <a href="#f-${fid}">the feature</a></p></div><div class="nums one-num"><div class="n"><big>${esc(big)}</big><small>${esc(st)}</small></div></div>`, '', k === 3 ? { bridge: 'the shape of the thing: seven verbs, ten rungs, fifty components.' } : {}));

/* act 3 · the shape */
push('shape', `<div><p class="kick">03 · the shape · the loop</p><h2>seven verbs.<br><em>one loop.</em></h2><div class="figwrap" id="fig-loop"></div><p class="mute">a focus becomes a computation, a computation a program, a program a proof, a proof an answer, the book a model, a thought a link, the links one focus — and whoever moved it is paid. the thirty features sit on these seven verbs.</p></div>`, 'one', { id: 'loop' });
push('shape', `<div><p class="kick">03 · the shape · the ladder</p><h2>ten rungs.<br><em>seven questions, three doors.</em></h2><div class="figwrap" id="fig-ladder"></div></div>`, 'one', { id: 'layers' });
push('shape', `<div><p class="kick">03 · the shape · the chart</p><h2>fifty components.<br><em>one dependency chart.</em></h2><div class="figwrap chartwrap" id="fig-chart"></div><p class="mute">every wire is a real Cargo dependency, labelled with its verb. hubs sit on the ring; apps at the centre; products built on soft3 are born on its skin.</p></div>`, 'one', { id: 'chart' });
push('shape', `<div><p class="kick">03 · the shape · the registry</p><h2>every component,<br><em>its honest state.</em></h2><div class="fig" id="registry-fig"></div><p class="mute">live carries real traffic today; published is released and correct but not in the live loop; unwired is built and not connected; spec is paper. the verdict: complete in principle and peer-pair in fact — the wire exists, and the graph routes it.</p></div>`, 'one', { bridge: 'rung by rung.' });
HUBS.forEach((n, i) => { const col = i < 7 ? FACET_C[i] : MINT; const parts = partsOf(n.id); push('shape', `
  <div><p class="kick">03 · the shape · rung ${String(i + 1).padStart(2, '0')} · ${i < 7 ? FACET[i] + ' · ' + esc(n.q) : 'apps'}</p><h2>${n.id}<br><em>${esc(n.role)}.</em></h2>
  <p class="lead">${esc(n.thesis)}.</p>
  <div class="parts">${[n.id, ...parts].map((p) => `<a class="chip" href="https://cyber.page/${p}/" target="_blank" rel="noopener">${p}</a>`).join('')}</div>
  <div class="panel rung-panel" data-rung="${n.id}" style="--c:${col}"></div></div>
  <div class="figwrap chartwrap small" data-chart="${n.id}"></div>`, '', { rung: n.id, parts: [n.id, ...parts], id: 'rung-' + n.id, bridge: i === 9 ? 'now the thirty features, verb by verb.' : undefined }); });

/* act 4 · seven verbs × thirty features */
VERB_ORDER.forEach((v, vi) => {
  const vv = VERBS.find((x) => x.id === v); const fs = byVerb(v);
  push(v, `<div><p class="kick">${String(vi + 4).padStart(2, '0')} · ${v} · ${esc(vv.who)}</p><h2>${v}.<br><em>${esc(vv.does.split(' — ')[0])}.</em></h2><p class="lead">${esc(vv.does.includes(' — ') ? vv.does.split(' — ')[1] : '')}</p><div class="nums">${fs.map((f) => `<div class="n"><big style="font-size:clamp(16px,1.6vw,24px)"><a href="#f-${f.id}" style="text-decoration:none">${esc(f.name)}</a></big><small>${esc(f.status)}</small></div>`).join('')}</div></div>
    <div class="figwrap loopwrap" data-loop="${v}"></div>`, 'ink', { id: 'v-' + v, bridge: vi < 6 ? `${v} done. next: ${VERB_ORDER[vi + 1]}.` : 'the thirty are the physics of a shared mind. the institutions come next.' });
  fs.forEach((f) => push(v, feat(f), '', { feat: f.id, id: 'f-' + f.id }));
});

/* act 5 · institutions */
push('institutions', `<div><p class="kick">11 · institutions · the twelve questions</p><h2>the wall is physics.<br><em>civilization is institutions.</em></h2>${twelveFig()}<p class="mute">the buyer's twelve questions, in order. bitcoin closed 1–3 and part of 6; linux 2–4 and 11; unicode 7 forever. soft3 aims at all twelve, plus a mind and a state. that is the checklist this stack is scored against.</p></div>`, 'one');
push('institutions', `<div><p class="kick">11 · institutions · already held, not on the window</p><h2>ten things the market buys<br><em>that the wall never named.</em></h2></div>
  <div class="steps two"><div class="step"><b>measurement</b><span>lytics, live — the strongest single-node evidence under adversarial traffic</span></div><div class="step"><b>the bootloader memory</b><span>2.9M links, 3.1M particles, 47k names — rooted means already rooted in a past</span></div><div class="step"><b>the money</b><span>without a unit of account civilizational software is a library</span></div><div class="step"><b>publishing</b><span>a book that cannot be read as a site does not enter a culture — optica, live</span></div><div class="step"><b>warriors</b><span>energy turned into useful work — erga, live</span></div><div class="step"><b>recovery after device loss</b><span>for rooted identity this outranks a 64 KB proof — vault, spec</span></div><div class="step"><b>conformance</b><span>a release is a frozen snapshot, fingerprinted with hemera</span></div><div class="step"><b>the wire</b><span>closer to a civilizational TCP than trinity is — live, three nodes</span></div><div class="step"><b>silicon</b><span>a stack that does not touch the chip is a cloud — honeycrisp, macOS + Android</span></div><div class="step"><b>land and deed</b><span>the state layer exists beside the machine — cyberia</span></div></div>`);
push('institutions', `<div><p class="kick">11 · institutions · still owed</p><h2>six open questions,<br><em>one settled stance.</em></h2><p class="lead">no component roadmap owned them. each is ignorable today and existential at the first real scale. the rooted-civilization proposal takes them as its phases A and B.</p></div>
  <div class="steps"><div class="step"><b>the economics of writing</b><span>an open graph with free writes drowns in spam on day one</span></div><div class="step"><b>key lifecycle</b><span>a lost seed is a lost neuron forever; one person on two devices is two neurons — a constitutional crisis without a constitution</span></div><div class="step"><b>protocol upgrade</b><span>a hard fork of a φ⋆-converged network is undefined behaviour</span></div><div class="step"><b>information is legal</b><span>there are no illegal bytes: the record keeps everything, a node chooses what it holds, liability belongs to whoever acts — settled</span></div><div class="step"><b>storage economics</b><span>a filecoin-shaped hole between bbg and tok</span></div><div class="step"><b>adversarial audit</b><span>someone paid to break it must try</span></div><div class="step"><b>operations</b><span>networks die of boring things</span></div></div>`);
push('institutions', `<div><p class="kick">11 · institutions · the upgrade</p><h2>three<br><em>constitutions.</em></h2>${phasesFig()}<p class="mute">four phases, each a last-merge: the tests of every repository it names are green, and the shop window says measured only of what is measured. the person before the network, the network before the world.</p></div>`, 'one');
push('institutions', `<div><p class="kick">11 · institutions · the proof</p><h2>≤ 64 KB.<br><em>post-quantum. ≤ 1 ms.</em></h2><p class="lead">the proof goal: at most 64 KB, small statements at most 20 KB, hash-only, verified in a millisecond, constant size by accumulation. today's hash relation proof is 15.6 KB; the public profile is sound and not yet succinct.</p></div>
  <div class="nums"><div class="n"><big>15.6<span class="u">KB</span></big><small>hash relation proof today · measured</small></div><div class="n"><big>≤ 64<span class="u">KB</span></big><small>the bound · target</small></div><div class="n"><big>≤ 20<span class="u">KB</span></big><small>small statements · target</small></div><div class="n"><big>0</big><small>trusted setups, curves, pairings</small></div></div>`, '', { bridge: 'the entrance.' });

/* act 6 · the entrance */
push('entrance', `<div><p class="kick">12 · the entrance · today</p><h2>a real node.<br><em>a public chaosnet.</em></h2><p class="lead">cargo install soft3 for the operator, cargo install true-cyber for the product face. the node processes cyberlinks into authenticated state, advances height on finalize, and serves product RPC. spacepussy-test: cybergraph + bbg, public rpc at cyb.ai.</p></div>
  <div class="panel"><label>install</label><pre class="code">cargo install soft3        # the stack CLI · a real node
cargo install true-cyber   # the product face (binary: cyber)
soft3 node --home ~/.spacepussy-test --bind 127.0.0.1:7780</pre><label>chaosnet</label><pre class="code">name       spacepussy-test
engine     cybergraph + bbg
public rpc https://cyb.ai/spacepussy-test</pre><p class="fine">every friday 12:00 UTC a candidate is cut from origin default branches, gated, published as a draft with receipts, promoted by the owner alone. a release is a conformance snapshot.</p></div>`);
push('entrance', `<div><p class="kick">12 · the entrance · 2026-11-05</p><h2>the bootloader<br><em>reborn.</em></h2><p class="lead">five years to the day after the bostrom genesis, bostrom and space pussy relaunch as the first two networks on soft3 — same keys, same balances, the burial snapshots as genesis, and the cores the old chains never had. what does not fit the date is cut from it, never squeezed in.</p></div>
  <div class="steps"><div class="step"><b>phase 1 · 2026-11-05</b><span>settlement mining, fold, consensus by convergence, truth markets, two-axis staking, privacy — and content visible in cyb: a graph nobody can read is a graph nobody links</span></div><div class="step"><b>phase 2 · 90 days</b><span>canary: real use, weekly red team, every economic invariant holds</span></div><div class="step"><b>phase 3 · $CYB mainnet</b><span>several implementations agree on 10⁶ blocks; the formal spine green</span></div></div>`);
push('entrance', `<div><p class="kick">12 · the entrance · three doors</p><h2>the stack is headless.<br><em>three bodies run on it.</em></h2>
  <div class="tiles"><a class="tile" href="https://cyb.ai" target="_blank" rel="noopener"><b>cyb</b>the robot — links, proves, mines and pays from your pocket · cyb.ai</a><a class="tile" href="https://cyber.page"><b>cyber</b>the superintelligence — one binary runs the chain, and this graph is its law · cyber.page</a><a class="tile" href="https://cyberia.to" target="_blank" rel="noopener"><b>cyberia</b>the state — citizenship, land, the door to a world that runs on this money · cyberia.to</a></div>
  <p class="lead">civilizational software installs as an app, a key, a name, a book — not as a compiler. the human entrance is the cyb shell speaking cy with mudra keys: the named next milestone after the wire.</p>
  <div class="ctas"><a class="btn" href="#home">start again</a><a class="btn ghost" href="https://cyber.page/soft3/">docs</a><a class="btn ghost" href="https://github.com/cyberia-to/soft3" target="_blank" rel="noopener">github</a><a class="btn ghost" href="https://cyber.page/soft3/status/">status</a><a class="btn ghost" href="/scheme/">the scheme</a></div>
  <p class="fine">measured · target · proposed — every status word is the one its source carries. don't trust. don't fear. don't beg.</p></div>`, 'one', { id: 'doors' });

/* ------------------------------------------------------------------ build */
const deck = $('#deck'), segs = $('#segs');
S.forEach((s, i) => deck.appendChild(el(`<section class="slide ch${s.ch} ${s.cls || ''}" data-i="${i}" data-ch="${String(s.ch + 1).padStart(2, '0')}" ${s.id ? `id="${s.id}"` : ''} style="--c:${CH[s.ch].c}"><div class="body">${s.html}</div>${s.bridge ? `<p class="bridge">→ ${esc(s.bridge)}</p>` : ''}</section>`)));
/* the figures from the shared modules */
$('#fig-loop').innerHTML = loopFig({ id: 'deck-loop', href: (v) => '#v-' + v.id });
$('#fig-ladder').innerHTML = ladderFig({ id: 'deck-ladder', class: 'glass', question: true, href: (n) => '#rung-' + n.id });
$('#fig-chart').innerHTML = chartFig({ id: 'deck-chart' });
document.querySelectorAll('[data-chart]').forEach((h, k) => { h.innerHTML = chartFig({ id: 'deck-chart-' + k, focus: h.dataset.chart }); });
document.querySelectorAll('[data-loop]').forEach((h, k) => { h.innerHTML = loopFig({ id: 'deck-loop-' + k, focus: h.dataset.loop, heart: false, href: null }); });
CH.forEach((c, i) => { const n = S.filter((s) => s.ch === i).length; const d = el(`<div class="seg" style="--sc:${c.c};flex:${Math.max(1, Math.sqrt(n))}" data-ch="${i}"><div class="fill"></div><span class="lab">${c.name}</span></div>`); d.addEventListener('click', () => go(S.findIndex((s) => s.ch === i))); segs.appendChild(d); });
/* figures from features.js render into their slide once; the registry panels fill from registry.json */
S.forEach((s, i) => { if (s.feat) { const f = FEATURES.find((x) => x.id === s.feat); const host = deck.querySelectorAll('.slide')[i].querySelector('.fig'); try { host.innerHTML = f.fig(); } catch (e) { host.innerHTML = `<p class="note">figure unavailable: ${esc(e.message)}</p>`; } } });
fetch('/registry.json').then((r) => r.json()).then((reg) => {
  $('#registry-fig').innerHTML = registryFig(reg);
  document.querySelectorAll('.rung-panel').forEach((p) => { const names = S.find((s) => s.rung === p.dataset.rung).parts; const rows = names.map((n) => stateOf(n, reg)).filter(Boolean); p.innerHTML = `<label>in the registry</label><table class="regt">${rows.map((r) => `<tr><td><b>${r.name}</b></td><td>${esc(r.verb)}</td><td class="st-${r.state}">${r.state}</td><td>${r.release || '—'}</td><td class="dim" title="${esc(r.what)}">${esc(r.what.length > 64 ? r.what.slice(0, 62) + '…' : r.what)}</td></tr>`).join('')}</table>${rows.length ? '' : '<p class="fine">not in the registry table</p>'}`; });
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
  world.chapter(CH[ch]); world.pulse();
  $('#pos').textContent = `${String(i + 1).padStart(2, '0')} / ${S.length}`;
  history.replaceState(null, '', '#' + (S[i].id || i));
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

/* ------------------------------------------------------------------ the world: soft3's own living canvas (world.js), hue per chapter */
const world = (() => {
  const root = document.documentElement; let curH = 0, target = 0, raf = 0;
  const step = () => { curH += (target - curH) * .08; if (window.worldHue) worldHue(curH); if (Math.abs(target - curH) > .5) raf = requestAnimationFrame(step); else raf = 0; };
  return {
    chapter(c) { const h = c.h; root.style.setProperty('--facet', h == null ? '#fff8f0' : c.c); root.style.setProperty('--tint', h == null ? 0 : .55); target = h == null ? 0 : h - 150; if (!raf) raf = requestAnimationFrame(step); },
    pulse() { const v = getComputedStyle(root).getPropertyValue('--tint').trim() || '0'; root.style.setProperty('--tint', Math.min(.95, +v + .35)); setTimeout(() => root.style.setProperty('--tint', v), 140); },
  };
})();
const fromHash = () => { const h = location.hash.slice(1); if (!h) return 0; const k = S.findIndex((x) => x.id === h); return k >= 0 ? k : (parseInt(h) || 0); };
addEventListener('hashchange', () => go(fromHash()));
go(fromHash());
})();
