/* soft3 demo · prove — a proof that fits in a pocket.
   window.demo_prove(mount, api) → { destroy() }
   published proof sizes vs soft3 today and the soft3 goal; the goal bar follows the statement toggle. */
window.demo_prove = (mount, api) => {
  const M = api.M, sfx = api.sfx || {};
  const ROWS = [['stwo', 617, 'm', '617 KB'], ['neptune', 1000, 'm', '~1 MB'], ['risc zero', 200, 'm', '~200 KB'], ['parano1d', 913, 'm', '913 KB'], ['quantus', 127, 'm', '127 KB'], ['soft3 today · hash', 15.6, 'm', '15.6 KB']];
  const STM = [['hash', 20], ['signature', 20], ['transfer', 20], ['a whole block', 64]];
  const MAX = 1000;
  mount.innerHTML = `
  <label style="margin-top:0">proof size · bytes on the wire</label>
  <div class="bars" data-bars style="font-size:12px;gap:7px">${ROWS.map(([n, v, k, t], i) => `<div class="bar${n.startsWith('soft3') ? ' me' : ''}" style="grid-template-columns:140px 1fr 96px"><span>${n}</span><i style="transform:scaleX(0)" data-i="${i}"></i><span>${t} ${M(k)}</span></div>`).join('')}
    <div class="bar me" style="grid-template-columns:140px 1fr 96px"><span data-gn>soft3 goal · hash</span><i style="transform:scaleX(0)" data-goal></i><span data-gv>≤ 20 KB ${M('t')}</span></div>
  </div>
  <label style="margin-top:8px">statement</label>
  <div class="chips" data-chips style="display:flex">${STM.map(([n], i) => `<span class="chip${i === 0 ? ' on' : ''}" data-s="${i}" style="${i === 0 ? 'border-color:var(--c);opacity:1' : ''}">${n}</span>`).join('')}</div>
  <label style="margin-top:8px">verify</label>
  <div class="chips" style="display:flex"><span class="chip" style="cursor:default">today 2.5 ms ${M('m')}</span><span class="chip" style="cursor:default;border-color:var(--c);opacity:1">goal ≤ 1 ms ${M('t')}</span></div>
  <div class="fine" style="margin-top:8px">sizes as published by each project ${M('m')} · soft3 today = the hash relation ${M('m')} · goal ≤ 20 KB small statements, ≤ 64 KB a block ${M('t')} · zheng proofs: no FRI, no merkle</div>`;
  const $ = (s) => mount.querySelector(s);
  const goal = $('[data-goal]'), gn = $('[data-gn]'), gv = $('[data-gv]'), chips = $('[data-chips]');
  const scale = (v) => Math.max(v / MAX, .006);
  const paint = () => { mount.querySelectorAll('[data-i]').forEach((el) => { el.style.transform = `scaleX(${scale(ROWS[+el.dataset.i][1])})`; }); };
  let cur = 0;
  const pick = (i) => {
    cur = i; const [n, kb] = STM[i];
    gn.textContent = `soft3 goal · ${n}`; gv.innerHTML = `≤ ${kb} KB ${M('t')}`; goal.style.transform = `scaleX(${scale(kb)})`;
    chips.querySelectorAll('.chip').forEach((c) => { const on = +c.dataset.s === i; c.style.borderColor = on ? 'var(--c)' : ''; c.style.opacity = on ? '1' : ''; });
  };
  const onChip = (e) => { const c = e.target.closest('.chip'); if (!c || !chips.contains(c)) return; pick(+c.dataset.s); sfx.tick && sfx.tick(); };
  chips.addEventListener('click', onChip);
  const t = setTimeout(() => { paint(); pick(0); }, 30);
  return { destroy() { clearTimeout(t); chips.removeEventListener('click', onChip); mount.innerHTML = ''; } };
};
