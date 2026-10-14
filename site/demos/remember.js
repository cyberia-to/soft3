/* soft3 demo · remember — compute once. everyone remembers.
   window.demo_remember(mount, api) → { destroy() }
   the first ask computes (≈900 ms, proof attached, paid to you); every later ask from any device is remembered (◌ mock). */
window.demo_remember = (mount, api) => {
  const M = api.M, sfx = api.sfx || {};
  mount.innerHTML = `
  <div style="display:grid;grid-template-columns:auto 1fr;gap:16px;align-items:center">
    <div><div class="count" data-count style="font-size:clamp(36px,5vmin,64px)">0</div><div class="fine">computed on the planet</div></div>
    <div><div class="count" data-rem style="font-size:clamp(36px,5vmin,64px);opacity:.7">0</div><div class="fine">times remembered</div></div>
  </div>
  <div class="row"><button class="btn" data-ask>ask: what links this file?</button><button class="btn ghost" data-other>ask from another device</button></div>
  <div class="out" data-out style="min-height:2.4em">nobody has asked yet.</div>
  <label>ledger</label>
  <div class="out" data-ledger style="margin-top:0;font-size:11px;min-height:0;opacity:.85"></div>
  <div class="fine" style="margin-top:8px">memoization — nothing is computed twice on the planet; the one who proved it first is paid each time the world remembers · status: specified ${M('t')}</div>`;
  const $ = (s) => mount.querySelector(s);
  const count = $('[data-count]'), rem = $('[data-rem]'), out = $('[data-out]'), ledger = $('[data-ledger]'), ask = $('[data-ask]'), other = $('[data-other]');
  let computed = 0, remembered = 0, busy = false, spin = null, timer = null; const rows = [];
  const DEV = ['your laptop', 'a phone in Lisbon', 'a node in Bali', 'a browser in Oslo', 'a miner in Yakutsk'];
  const push = (row) => { rows.unshift(row); if (rows.length > 5) rows.pop(); ledger.innerHTML = rows.map((r, i) => `${String(rows.length - i).padStart(2, ' ')} · ${r}`).join('\n'); };
  const bump = (el) => { el.style.transform = 'scale(1.25)'; el.style.transition = 'transform .1s'; setTimeout(() => { el.style.transform = ''; el.style.transition = 'transform .4s'; }, 110); };
  const answer = (device) => {
    if (busy) return;
    if (computed === 0) {
      busy = true; let f = 0; const frames = '◐◓◑◒';
      out.innerHTML = `computing on ${device} … ${frames[0]}`;
      spin = setInterval(() => { f++; out.innerHTML = `computing on ${device} … ${frames[f % 4]} <span style="opacity:.6">${(f * 100)} ms</span>`; sfx.tick && sfx.tick(); }, 100);
      timer = setTimeout(() => {
        clearInterval(spin); spin = null; busy = false; computed = 1; count.textContent = '1'; bump(count);
        out.innerHTML = `<b>computed</b> · proof attached · paid to: <b>you</b> ${M('k')}`;
        push(`${device} · computed · 0.9 s · paid to you`); sfx.hash && sfx.hash();
      }, 900);
    } else {
      remembered++; rem.textContent = remembered; bump(rem);
      out.innerHTML = `<b>remembered</b> · proof verified in 0.4 ms ${M('k')} · paid to: <b>the first prover</b>`;
      push(`${device} · remembered · 0.4 ms · paid to the first prover`); sfx.chord && sfx.chord(1);
    }
  };
  const onAsk = () => answer('your laptop');
  const onOther = () => answer(DEV[1 + (remembered + computed) % (DEV.length - 1)]);
  ask.addEventListener('click', onAsk); other.addEventListener('click', onOther);
  return { destroy() { clearInterval(spin); clearTimeout(timer); ask.removeEventListener('click', onAsk); other.removeEventListener('click', onOther); mount.innerHTML = ''; } };
};
