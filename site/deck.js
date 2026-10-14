/* soft3 — collective, private, provable computing · the deck, second cut
   a cold open, one question, seven verbs each with a working demonstration, ten products, the chart alive, the road, the door.
   data: data.js (VERBS ARCS NODES EDGES) · features.js (FEATURES) · registry.json · demos/*.js · figs-live.js · world.js */
(() => {
const $ = (s, r = document) => r.querySelector(s);
const el = (h) => { const t = document.createElement('template'); t.innerHTML = h.trim(); return t.content.firstChild; };
const esc = (t) => String(t).replace(/[&<>"]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[c]));
const M = (k) => `<i class="mk ${k}"></i>`;
const MINT = '#7dffc4';
const VC = { compute: '#00ff00', build: '#5b8def', prove: '#8a6bff', remember: '#b07cff', learn: '#ff3b2f', link: '#ff8c1a', focus: '#ffd400' };
const VH = { compute: 120, build: 240, prove: 255, remember: 270, learn: 0, link: 30, focus: 60 };
const VERB_ORDER = ['compute', 'build', 'prove', 'remember', 'learn', 'link', 'focus'];

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


/* ------------------------------------------------------------------ chapters */
const CH = [
  { id: 'open', name: 'soft3', c: MINT, h: null },
  { id: 'question', name: 'the question', c: MINT, h: null },
  ...VERB_ORDER.map((v) => ({ id: v, name: v, c: VC[v], h: VH[v] })),
  { id: 'products', name: 'ten products', c: '#ffffff', h: null },
  { id: 'road', name: 'the road', c: VC.learn, h: 0 },
  { id: 'door', name: 'the door', c: MINT, h: null },
];
const chIdx = (id) => CH.findIndex((c) => c.id === id);
const S = []; const push = (ch, html, cls = '', extra = {}) => S.push({ ch: chIdx(ch), cls, html, ...extra });
const byVerb = (v) => FEATURES.filter((f) => f.verb === v);
const vv = (v) => VERBS.find((x) => x.id === v);

/* the seven verbs: the question each chapter asks, the answer, the measured line */
const Q = {
  compute: { q: 'can two machines ever agree?', a: 'only if there is nothing to disagree about. one field, one hash, no floats, no clocks, no randomness: the same bytes in, the same bytes out, on a laptop and on a phone. that is the floor — and nox, the machine at the bottom, is frozen: eighteen instructions that will run and prove the same way in fifty years.', n: '2,949,732', nl: 'bostrom links replayed from events — byte-identical', nk: 'm' },
  build: { q: 'what does it cost before it runs?', a: 'every loop has a bound, there is no recursion, there is no heap. a trident program\'s steps are counted at compile time, so there is no halting problem on this machine and the price of a computation is printed before the computation.', n: '275k', nl: 'lines in the whole stack, against 39 million with LLVM — small enough that every step is counted', nk: 'm' },
  prove: { q: 'why should anyone believe you?', a: 'they do not have to. the run emits its own proof as a by-product: hash-only, post-quantum, no trusted setup, no curve a quantum computer breaks. the goal is a proof that fits in a pocket — at most 64 KB, a millisecond to verify — and the compiler that proves it compiled itself.', n: '15.6 KB', nl: 'hash relation proof today · goal ≤ 64 KB, small statements ≤ 20 KB', nk: 'm' },
  remember: { q: 'why compute anything twice?', a: 'a proven answer is a file with a hash. the book keeps it; anyone will check it with 32 bytes; nobody recomputes it; the one who proved it first is paid each time the world remembers. the state is the graph — there is no ledger — and your own graph plugs into the shared one.', n: '32 B', nl: 'what a phone needs to check any answer · provable reads, building', nk: 't' },
  learn: { q: 'where do the weights come from?', a: 'from the book. a transformer\'s embedding dimension is the effective rank of focus, its heads are the dialects, its depth is the diameter times the convergence rate — read off the weighted graph, not learned from text. every weight names its sources and pays them. alignment is a distance between two foci, on a dashboard.', n: '62 s', nl: 'bostrom\'s 2.95M links compiled into a model on one machine · d⋆ 31 · 290 layers', nk: 'm' },
  link: { q: 'how does a thought enter the machine?', a: 'as a cyberlink: two files, a stake, a valence, a signature. you address by name and the name is yours; the wire obeys the graph — following is a link, an address is a link; an agent is a neuron like you. bring a mind and own a share of its economy.', n: '3 nodes', nl: 'C found A\'s address in the graph and reached it through B — the wire obeys the graph', nk: 'm' },
  focus: { q: 'who decides what matters?', a: 'nobody. every neuron runs the same contraction on the same graph and lands on the same focus — a theorem, not a vote. honesty is the only strategy that pays, miners sell order and agreement, a fact is final when enough attention has gathered: in about a second here, and across light-minutes between planets.', n: 'κ = 0.74', nl: 'the kernel contracts on bostrom · one fixed point, by Banach', nk: 'm' },
};
const TRY = {"compute": "type any two numbers; both devices run eight rounds. then add a float and watch them disagree.", "build": "move the bound: the cost is printed before you press run. then run, and count.", "prove": "pick a statement; the goal bar moves. every other bar is as its project published it.", "remember": "ask once: it is computed and proven. ask again, from any device: it is remembered, and the first prover is paid.", "learn": "slide the graph size. the shape of the model follows the book, not a text corpus.", "link": "name two things, stake, choose a valence, link. a throwaway neuron signs it in your browser.", "focus": "iterate from a uniform start, then from a random one. both land on the same focus."};
const forWhom = { nox: 'anyone who must trust a result', trident: 'the programmer who must know the price first', joy: 'the prover and the miner', cybergraph: 'anyone with something true to say', neural: 'the builder of models', cy: 'the operator, and the agent', soft3: 'the node, the town, the archive', cyb: 'a person', cyber: 'the network', cyberia: 'the citizen' };

/* ------------------------------------------------------------------ act 0 · cold open */
push('open', `<div class="cold"><p class="hero-line">compute <em>once.</em></p></div>`, 'one bare');
push('open', `<div class="cold"><p class="hero-line ghost">compute once.</p><p class="hero-line">prove <em>it.</em></p></div>`, 'one bare');
push('open', `<div class="cold"><p class="hero-line ghost">compute once. prove it.</p><p class="hero-line">everyone <em>remembers.</em></p></div>`, 'one bare');
push('open', `<div class="cold"><p class="kick">soft3 · collective, private, provable computing</p><p class="hero-word">soft3</p><p class="lead big">a computer the whole world shares, that keeps every proof it has seen and pays whoever moves its focus.</p><p class="kbd">→ or space · <span id="nslides"></span> slides · the rail jumps</p></div>`, 'one bare', { id: 'home' });

/* ------------------------------------------------------------------ act 1 · the question */
const RENT = [['who', 'DNS · PKI · KYC', 'a name you own, a key you hold', 'names · mudra'], ['value', 'Visa · SWIFT', 'a token minted when the network learns', 'tok · $CYB'], ['compute', 'Linux · iOS · AWS', 'a frozen machine on your own silicon', 'nox · honeycrisp'], ['knowledge', 'Google · GitHub · Wikipedia', 'one graph nobody can unpublish', 'cybergraph · bbg'], ['speech', 'TCP · SMTP', 'a wire that obeys the graph', 'radio · the wire'], ['time', 'NTP · GPS', 'a clock made of work, facts final when enough has gathered', 'foculus'], ['mind', 'closed models', 'a model compiled from the book, every weight with an author', 'neural · tru']];
push('question', `<div class="col"><p class="kick">the question</p><h2>every computer on earth recomputes<br><em>what another already proved.</em></h2><p class="lead">and every answer you get is rented. civilization already runs on a captured stack — tap a tile to see what it rents, and what replaces it.</p></div>
  <div class="rent" id="rent">${RENT.map(([k, from, to, parts], i) => `<button class="tile rent-tile" data-i="${i}" style="--c:${['#ff3b2f', '#ff8c1a', '#ffd400', '#00ff00', '#5b8def', '#8a6bff', '#b07cff'][i]}"><b>${k}</b><span class="from">${from}</span><span class="to">${to}</span><span class="parts">${parts}</span></button>`).join('')}</div>`, '', { bridge: 'nobody buys a proof system. they buy their stack back.' });
push('question', `<div class="col"><p class="kick">the question · the buyer</p><h2>the buyer runs<br><em>an institution.</em></h2><p class="lead">a citizen, a node, a town, an archive, a wallet, a school. the market buys identity that survives a death, a unit of account, a record a phone can check, a clock everyone shares, an upgrade that does not split the tribe, and boring operations.</p><p class="mute">twelve questions such a buyer asks, in order. bitcoin closed 1–3 and part of 6; linux 2–4 and 11; unicode 7 forever. soft3 aims at all twelve, plus a mind and a state — each one has a verb, and the verbs come next.</p></div>
  <ol class="twelve">${[['it works when the other party is hostile', 'bitcoin', 'prove'], ['it outlives its authors', 'bitcoin', 'compute'], ['one person can run it', 'bitcoin', 'the door'], ['it speaks to the old world', 'linux', 'link'], ['a unit of account, a reason to hold the bytes', '', 'focus'], ['operations are boring', '', 'the road'], ['a name a human can say', 'unicode', 'link'], ['a key can be rotated, given, buried', '', 'the road'], ['a partition leaves local life intact', '', 'focus'], ['a fork is a constitution, not a catastrophe', '', 'the road'], ['outsiders can audit it', 'linux', 'prove'], ['a child can learn it', '', 'build']].map(([q, who, verb], i) => `<li><i>${String(i + 1).padStart(2, '0')}</i>${q}<em>${who}</em><u style="color:${VC[verb] || MINT}">→ ${verb}</u></li>`).join('')}</ol>`);
push('question', `<div class="col"><p class="kick">the question · the answer has a shape</p><h2>seven verbs.<br><em>one loop.</em></h2><p class="lead">so the question: what must a computer be, that an institution would buy it and nobody could rent it back? seven verbs. a focus becomes a computation, a computation a program, a program a proof, a proof an answer, the book a model, a thought a link, the links one focus — and whoever moved it is paid. every chapter that follows is one verb, one question, one thing you can try.</p><p class="mute">touch a verb to jump to it.</p></div><div class="livefig" data-live="loop" id="loop-main"></div>`, '', { id: 'loop', bridge: 'first verb. can two machines ever agree?' });

/* ------------------------------------------------------------------ act 2 · seven verbs */
VERB_ORDER.forEach((v, vi) => {
  const q = Q[v], w = vv(v), fs = byVerb(v), n = String(vi + 1).padStart(2, '0');
  push(v, `<div class="col"><p class="kick">${n} · ${v} · ${esc(w.who)} · <b>${esc(w.sel)}</b></p><h2>${esc(q.q)}</h2><p class="lead">${esc(q.a)}</p></div><div class="livefig small" data-live="loop" data-current="${v}"></div>`, '', { id: 'v-' + v });
  push(v, `<div class="col"><p class="kick">${n} · ${v} · try it</p><h2>${esc(w.does.split(' — ')[0])}<em>${w.does.includes(' — ') ? ' — ' + esc(w.does.split(' — ')[1]) : ''}</em></h2><p class="try">${esc(TRY[v])}</p><div class="nums one-num"><div class="n"><big>${esc(q.n)}</big><small>${esc(q.nl)} ${M(q.nk)}</small></div></div></div><div class="demo" data-demo="${v}"></div>`, '', { demo: v });
  push(v, `<div class="col"><p class="kick">${n} · ${v} · what is born here</p><h2>${fs.length} ${fs.length === 1 ? 'property' : 'properties'}<br><em>born in ${v}.</em></h2><p class="mute">each card is a property of the stack with the status word its source carries — live · measured · building · specified · proposed · research. the chips are the components that make it.</p></div>
    <div class="cards">${fs.map((f) => `<div class="card"><b>${esc(f.name)}</b><span>${esc(f.claim)}</span><div class="parts">${f.parts.map((p) => `<a class="chip" href="https://cyber.page/${p}/" target="_blank" rel="noopener">${p}</a>`).join('')}</div><i class="st">${esc(f.status)}</i></div>`).join('')}</div>`, '', { bridge: vi < 6 ? `<b style="color:${VC[VERB_ORDER[vi + 1]]}">${String(vi + 2).padStart(2, '0')} · ${VERB_ORDER[vi + 1]}.</b> ${esc(Q[VERB_ORDER[vi + 1]].q)}` : 'seven verbs make ten products.' });
});

/* ------------------------------------------------------------------ act 3 · ten products */
push('products', `<div class="col"><p class="kick">ten products</p><h2>ten rungs.<br><em>one ladder.</em></h2><p class="lead">seven are the stack — one per verb, the rung each demonstration ran on — and three are the bodies that run on it. touch a rung.</p><div class="panel rung-detail" id="rung-detail"><label>pick a rung</label></div></div><div class="livefig" data-live="ladder" id="ladder-main"></div>`, '', { id: 'products' });
push('products', `<div class="col"><p class="kick">ten products · the chart</p><h2>every wire is<br><em>a real dependency.</em></h2><p class="lead">fifty components, one dependency chart drawn from the Cargo manifests, every wire labelled with its verb. hover or tap a component to see what feeds it and what it feeds; tap a hub to focus the chart on it.</p><div class="panel" id="chart-detail"><label>hover or tap a component</label></div></div><div class="livefig" data-live="chart" id="chart-main"></div>`, '', { id: 'chart' });
push('products', `<div class="col"><p class="kick">ten products · honest state</p><h2>every component,<br><em>its honest state.</em></h2><p class="lead">live carries real traffic today; published is released but not in the live loop; unwired is built and not connected; spec is paper. the verdict: complete on paper; two real nodes already talk through the graph.</p></div><div class="fig" id="registry-fig"></div>`, '', { bridge: 'what is still owed, and when.' });

/* ------------------------------------------------------------------ act 4 · the road */
push('road', `<div class="col"><p class="kick">the road · already held</p><h2>ten things already built<br><em>that the seven verbs never named.</em></h2></div>
  <div class="cards two">${[['measurement', 'lytics, live — the strongest single-node evidence under adversarial traffic'], ['the bootloader memory', '2.9M links, 3.1M particles, 47k names — rooted means already rooted in a past'], ['the money', 'without a unit of account civilizational software is a library'], ['publishing', 'a book that cannot be read as a site does not enter a culture — optica, live'], ['warriors', 'energy turned into useful work — erga, live'], ['recovery after device loss', 'for rooted identity this outranks a 64 KB proof — vault, spec'], ['conformance', 'a release is a frozen snapshot, fingerprinted with hemera'], ['the wire', 'closer to a civilizational TCP than anything above — live, three nodes'], ['silicon', 'a stack that does not touch the chip is a cloud — honeycrisp, macOS + android'], ['land and deed', 'the state layer exists beside the machine — cyberia']].map(([a, b]) => `<div class="card"><b>${a}</b><span>${b}</span></div>`).join('')}</div>`);
push('road', `<div class="col"><p class="kick">the road · still owed</p><h2>six open questions,<br><em>one settled stance.</em></h2><p class="lead">no component roadmap owned them. each is ignorable today and existential at the first real scale.</p></div>
  <div class="cards two">${[['the economics of writing', 'an open graph with free writes drowns in spam on day one'], ['key lifecycle', 'a lost seed is a lost neuron; one person on two devices is two neurons'], ['protocol upgrade', 'a hard fork of a network that has agreed on one focus is undefined behaviour'], ['storage economics', 'who is paid to keep the bytes: nothing yet sits between storage (bbg) and money (tok)'], ['adversarial audit', 'someone paid to break it must try'], ['operations', 'networks die of boring things'], ['information is legal', 'there are no illegal bytes: the record keeps everything, a node chooses what it holds, liability belongs to whoever acts', 'settled']].map(([a, b, st]) => `<div class="card"${st ? ` style="--c:${MINT}"` : ''}><b>${a}</b><span>${b}</span>${st ? `<i class="st">${st}</i>` : ''}</div>`).join('')}</div>`);
push('road', `<div class="col"><p class="kick">the road · three constitutions</p><h2>the person,<br><em>the network, the world.</em></h2><p class="lead">phase 0 is this deck: it claims nothing it has not measured. then three phases; each ends when every repository it names is green. A does not wait for 64 KB proofs; B waits for a clock; C waits for a frozen proof format.</p></div>
  <div class="cards">${[['A · person · key, write, hold', 'vault recovery · one neuron on two devices · priced writes · storage market · node health · a human entrance', VC.compute], ['B · network · upgrade, fork, law', 'parameter change with a height · fork as a right · the information-is-legal page with an owner · anti-capture · paid audit · guardianship · offline a day', VC.build], ['C · world · joule, iron, bridge, court', 'joule as a unit · RISC-V and plain linux · bridges into HTTP, mail, BTC, paper · compaction · a second full node · a deed a court can read', VC.focus]].map(([a, b, c]) => `<div class="card" style="--c:${c}"><b>${a}</b><span>${b}</span></div>`).join('')}</div>`, '', { bridge: 'the door.' });

/* ------------------------------------------------------------------ act 5 · the door */
push('door', `<div class="col"><p class="kick">the door · today</p><h2>a real node.<br><em>a public chaosnet.</em></h2><p class="lead">a node you can run tonight: it takes cyberlinks, agrees on a height, answers queries. a new candidate every friday; only the owner promotes one to a release.</p></div>
  <div class="panel"><label>install</label><pre class="code">cargo install soft3        # the stack CLI · a real node
cargo install true-cyber   # the product face (binary: cyber)
soft3 node --home ~/.spacepussy-test --bind 127.0.0.1:7780</pre><label>chaosnet</label><pre class="code">spacepussy-test · cybergraph + bbg
public rpc  https://cyb.ai/spacepussy-test</pre><p class="fine">2026-11-05 · bostrom and space pussy relaunch on soft3, same keys, same balances — and cyb reads them</p></div>`, '', { id: 'install' });
push('door', `<div class="cold"><p class="kick">the door · three bodies</p><h2>the stack is headless.<br><em>three bodies run on it.</em></h2>
  <div class="tiles doors"><a class="tile" href="https://cyb.ai" target="_blank" rel="noopener"><b>cyb</b>the robot · links, proves, mines and pays from your pocket</a><a class="tile" href="https://cyber.page"><b>cyber</b>the superintelligence · one binary runs the chain, the cyber graph is its law</a><a class="tile" href="https://cyberia.to" target="_blank" rel="noopener"><b>cyberia</b>the state · citizenship, land, a world that runs on this money</a></div>
  <div class="ctas"><a class="btn" href="#home">start again</a><a class="btn ghost" href="https://cyber.page/soft3/">docs</a><a class="btn ghost" href="https://github.com/cyberia-to/soft3" target="_blank" rel="noopener">github</a><a class="btn ghost" href="https://cyber.page/soft3/status/">status</a><a class="btn ghost" href="/scheme/">the scheme</a></div>
  <p class="fine">measured · target · mock — every number wears the word its source carries. don't trust. don't fear. don't beg.</p></div>`, 'one bare', { id: 'doors' });

/* ------------------------------------------------------------------ build */
const deck = $('#deck'), segs = $('#segs');
S.forEach((s, i) => deck.appendChild(el(`<section class="slide ch${s.ch} ${s.cls || ''}" data-i="${i}" ${s.id ? `id="${s.id}"` : ''} style="--c:${CH[s.ch].c}"><div class="body">${s.html}</div>${s.bridge ? `<p class="bridge">→ ${s.bridge}</p>` : ''}</section>`)));
CH.forEach((c, i) => { const n = S.filter((s) => s.ch === i).length; const d = el(`<div class="seg" style="--sc:${c.c};flex:${Math.max(1, Math.sqrt(n))}" data-ch="${i}"><div class="fill"></div><span class="lab">${c.name}</span></div>`); d.addEventListener('click', () => go(S.findIndex((s) => s.ch === i))); segs.appendChild(d); });
$('#nslides').textContent = S.length;

/* the rented stack tiles flip */
document.querySelectorAll('.rent-tile').forEach((t) => t.addEventListener('click', () => { t.classList.toggle('flip'); sfx.tick(); }));

/* registry chart */
fetch('/registry.json').then((r) => r.json()).then((reg) => { const states = ['live', 'published', 'unwired', 'local', 'spec', 'blocked', 'dormant']; const cnt = Object.fromEntries(states.map((s) => [s, reg.filter((r) => r.state === s).length])); const mx = Math.max(...Object.values(cnt)); let o = ''; states.forEach((s, i) => { const x = 40 + i * 100, h = 120 * cnt[s] / mx; o += `<rect x="${x}" y="${150 - h}" width="72" height="${h}" rx="4" fill="${s === 'live' ? MINT : '#fff'}" fill-opacity="${s === 'live' ? .85 : .3}"/><text x="${x + 36}" y="${142 - h}" font-size="18" font-weight="700" text-anchor="middle" fill="#fff" font-family="Play,sans-serif">${cnt[s]}</text><text x="${x + 36}" y="170" font-size="10" text-anchor="middle" fill="#fff" fill-opacity=".8" font-family="JetBrains Mono,monospace">${s}</text>`; }); $('#registry-fig').innerHTML = `<svg class="deckfig" viewBox="0 0 760 190">${o}</svg><div class="regchips">${states.filter((st) => cnt[st]).map((st) => `<div class="regrow"><label>${st}</label>${reg.filter((r) => r.state === st).map((r) => `<span class="chip" data-st="${st}" title="${esc(r.what || '')}">${esc(r.name)}</span>`).join('')}</div>`).join('')}</div><p class="fine">${reg.length} components in the registry · ${cnt.live} carry real traffic today · a release is a conformance snapshot</p>`; window.REG = reg; }).catch(() => {});

/* live figures and demos mount on first entry */
const api = { sfx: null, M };
const mounted = new Set();
const mountFor = (i) => {
  const sl = document.querySelectorAll('.slide')[i];
  sl.querySelectorAll('.livefig').forEach((h) => {
    if (mounted.has(h) || !window.liveFigs) return; mounted.add(h); const kind = h.dataset.live;
    if (kind === 'loop') { const lf = liveFigs.loop(h, { current: h.dataset.current || null, dimOthers: !!h.dataset.current, onHover: (hue) => world.hint(hue), onVerb: (id) => go(S.findIndex((s) => s.id === 'v-' + id)) }); h._fig = lf; }
    if (kind === 'ladder') { liveFigs.ladder(h, { selected: 'nox', onHover: (hue) => world.hint(hue), onRung: (id) => rungDetail(id) }); rungDetail('nox'); }
    if (kind === 'chart') { liveFigs.chart(h, { onHub: (id) => { sfx.chord(2); chartDetail(id); }, onHover: (hue, id) => { world.hint(hue); if (id) chartDetail(id); } }); }
  });
  sl.querySelectorAll('.demo').forEach((h) => { if (mounted.has(h)) return; const fn = window['demo_' + h.dataset.demo]; if (!fn) return; mounted.add(h); try { fn(h, api); } catch (e) { h.innerHTML = `<p class="fine">demo unavailable: ${esc(e.message)}</p>`; } });
};
const rungDetail = (id) => { const n = NODES.find((x) => x.id === id); if (!n) return; const reg = (window.REG || []).find((r) => r.name === id); $('#rung-detail').innerHTML = `<label>${String(n.layer).padStart(2, '0')} · ${esc(n.role || '')}${n.q ? ' · ' + esc(n.q) : ''}</label><b class="big">${n.id}</b><p>${esc(n.thesis)}.</p><p class="mute">for ${esc(forWhom[id] || '')}${reg ? ` · does: ${esc(reg.verb)} · <b class="st">${esc(reg.state)}</b>${reg.release ? ' · ' + reg.release : ''}` : ''}</p>${reg && reg.what ? `<p class="fine">${esc(reg.what)}</p>` : ''}`; sfx.tick(); };
const chartDetail = (id) => { const n = NODES.find((x) => x.id === id); if (!n) return; const d = $('#chart-detail'); const ins = EDGES.filter((e) => e.b === id).map((e) => ({ id: e.a, verb: e.v })), outs = EDGES.filter((e) => e.a === id).map((e) => ({ id: e.b, verb: e.v })); d.innerHTML = `<label>${esc(n.id)}${n.thesis ? ' · ' + esc(n.thesis) : ''}</label><p class="fine">${ins.length ? 'fed by: ' + ins.map((x) => `<b>${esc(x.id)}</b> <i>${esc(x.verb)}</i>`).join(' · ') : 'fed by nothing in the chart'}</p><p class="fine">${outs.length ? 'feeds: ' + outs.map((x) => `<b>${esc(x.id)}</b> <i>${esc(x.verb)}</i>`).join(' · ') : 'feeds nothing in the chart'}</p>`; };

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
  document.body.className = document.body.className.replace(/\bch\d+\b/g, '').trim() + ' ch' + S[i].ch;
  const ch = S[i].ch, idx = S.filter((s, k) => s.ch === ch && k <= i).length, n = S.filter((s) => s.ch === ch).length;
  document.querySelectorAll('.seg').forEach((sg, k) => { sg.classList.toggle('cur', k === ch); sg.classList.toggle('done', k < ch); $('.fill', sg).style.width = k < ch ? '100%' : k === ch ? (idx / n * 100) + '%' : '0'; });
  sfx.move(dir, ch !== prevCh ? ch % 7 : null); if (navigator.vibrate) navigator.vibrate(8);
  world.chapter(CH[ch]); world.pulse();
  $('#pos').textContent = `${String(i + 1).padStart(2, '0')} / ${S.length}`;
  history.replaceState(null, '', '#' + (S[i].id || i));
  mountFor(i); document.querySelectorAll('.livefig[data-live=loop]').forEach((h) => { if (h._fig && !h.dataset.current && S[i].ch >= 2 && S[i].ch <= 8) h._fig.set(CH[S[i].ch].id); });
  requestAnimationFrame(() => requestAnimationFrame(fit));
};
window.deckGo = go; window.deckCount = () => S.length; window.deckFit = fit;
$('#prev').onclick = () => go(cur - 1); $('#next').onclick = () => go(cur + 1);
document.addEventListener('keydown', (e) => { if (['INPUT', 'TEXTAREA'].includes(e.target.tagName) || (e.target.tagName === 'BUTTON' && [' ', 'Enter'].includes(e.key))) return; if (['ArrowRight', ' ', 'PageDown', 'ArrowDown'].includes(e.key)) { e.preventDefault(); go(cur + 1); } if (['ArrowLeft', 'PageUp', 'ArrowUp'].includes(e.key)) { e.preventDefault(); go(cur - 1); } if (e.key === 'Home') go(0); if (e.key === 'End') go(S.length - 1); });
let wheelLock = 0; document.addEventListener('wheel', (e) => { if (e.target.closest('.demo, .livefig, .panel')) return; const t = Date.now(); if (t - wheelLock < 700 || Math.abs(e.deltaY) < 24) return; wheelLock = t; go(cur + (e.deltaY > 0 ? 1 : -1)); }, { passive: true });
let tx = null, ty = 0; document.addEventListener('touchstart', (e) => { tx = e.target.closest('input[type=range], canvas, .demo') ? null : e.touches[0].clientX; ty = e.touches[0].clientY; }, { passive: true });
document.addEventListener('touchend', (e) => { if (tx == null) return; const dx = e.changedTouches[0].clientX - tx, dy = e.changedTouches[0].clientY - ty; if (Math.abs(dx) > 60 && Math.abs(dx) > Math.abs(dy)) go(cur + (dx < 0 ? 1 : -1)); tx = null; });
document.addEventListener('pointerdown', (e) => { sfx.wake(); if (e.target.closest('button, a, .seg, .chip, .rung, .verb, .hub, .node')) sfx.tick(); }, { passive: true });
document.addEventListener('keydown', () => sfx.wake(), { once: true });
const sndBtn = $('#snd'); const sndPaint = () => { sndBtn.textContent = sfx.on ? '◉ sound' : '○ sound'; sndBtn.classList.toggle('off', !sfx.on); }; sndBtn.onclick = () => { sfx.toggle(); sndPaint(); }; sndPaint();
api.sfx = sfx;
if (matchMedia('(pointer:fine)').matches) { const dot = el('<div class="cur-dot"></div>'), ring = el('<div class="cur-ring"></div>'); document.body.append(dot, ring); let rx = 0, ry = 0, px = 0, py = 0; addEventListener('mousemove', (e) => { px = e.clientX; py = e.clientY; dot.style.transform = `translate(${px}px,${py}px)`; ring.classList.toggle('hot', !!e.target.closest('button, a, .seg, .chip, .rung, .verb, .hub, .node, input, canvas')); }, { passive: true }); const lag = () => { rx += (px - rx) * .18; ry += (py - ry) * .18; ring.style.transform = `translate(${rx}px,${ry}px)`; requestAnimationFrame(lag); }; lag(); document.body.classList.add('has-cursor'); }

/* ------------------------------------------------------------------ the world (world.js) — hue per chapter, a hint on hover, a flash per move */
const world = (() => {
  const root = document.documentElement; let curH = 0, target = 0, raf = 0, base = 0;
  const step = () => { curH += (target - curH) * .08; if (window.worldHue) worldHue(curH); if (Math.abs(target - curH) > .5) raf = requestAnimationFrame(step); else raf = 0; };
  const to = (t) => { target = t; if (!raf) raf = requestAnimationFrame(step); };
  return {
    chapter(c) { base = c.h == null ? 0 : c.h - 150; root.style.setProperty('--facet', c.h == null ? '#fff8f0' : c.c); root.style.setProperty('--tint', c.h == null ? 0 : .35); to(base); },
    hint(h) { to(h == null ? base : h - 150); },
    pulse() { const v = getComputedStyle(root).getPropertyValue('--tint').trim() || '0'; root.style.setProperty('--tint', Math.min(.9, +v + .3)); setTimeout(() => root.style.setProperty('--tint', v), 160); },
  };
})();

const fromHash = () => { const h = location.hash.slice(1); if (!h) return 0; const k = S.findIndex((x) => x.id === h); return k >= 0 ? k : (parseInt(h) || 0); };
addEventListener('hashchange', () => go(fromHash()));
go(fromHash());
})();
