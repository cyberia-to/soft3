/* soft3 demo · link — a thought becomes a link, with weight.
   window.demo_link(mount, api) → { destroy() }
   two texts → (p, q, τ, a, v): p, q = sha-256 prefixes (hemera on mainnet), a = stake, v = valence, τ = now. Δφ⋆ is a mock. */
window.demo_link = (mount, api) => {
  const M = api.M, sfx = api.sfx || {};
  const esc = (t) => String(t).replace(/[&<>"]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[c]));
  const sha = async (t) => { const b = await crypto.subtle.digest('SHA-256', new TextEncoder().encode(t)); return [...new Uint8Array(b)].map((x) => x.toString(16).padStart(2, '0')).join(''); };
  const PAIRS = [['proof of work', 'Shapley sampling'], ['a wrong claim', 'costs the staker'], ['light-minutes', 'local finality'], ['my grandmother\'s recipe', 'a particle, forever'], ['the Landauer limit', '3×10⁻²¹ J per bit']];
  mount.innerHTML = `
  <div style="display:grid;grid-template-columns:1fr 1fr;gap:12px">
    <div><label style="margin-top:0">this</label><input type="text" data-a value="proof of work" maxlength="80"></div>
    <div><label style="margin-top:0">belongs with</label><input type="text" data-b value="Shapley sampling" maxlength="80"></div>
  </div>
  <div class="chips" data-chips style="display:flex;margin-top:8px">${PAIRS.map(([a, b], i) => `<span class="chip" data-i="${i}" title="${esc(a)} → ${esc(b)}">${esc(a)}</span>`).join('')}</div>
  <div style="display:grid;grid-template-columns:1fr auto;gap:14px;align-items:end">
    <div><label style="margin-top:8px">stake a · <b data-av>10</b> CYB</label><input type="range" min="1" max="100" value="10" data-stake></div>
    <div><label style="margin-top:8px">valence v</label><div class="dial" data-dial><button data-v="-1">−1</button><button data-v="0" class="on">0</button><button data-v="1">+1</button></div></div>
  </div>
  <div class="row" style="margin-top:8px"><button class="btn" data-go>link</button><span class="fine">a throwaway neuron in your browser ${M('k')}</span></div>
  <div class="out" data-out style="min-height:3.2em;margin-top:8px;font-size:12px;line-height:1.5">a cyberlink is five fields: (p, q, τ, a, v). press link.</div>`;
  const $ = (s) => mount.querySelector(s);
  const ia = $('[data-a]'), ib = $('[data-b]'), stake = $('[data-stake]'), av = $('[data-av]'), dial = $('[data-dial]'), go = $('[data-go]'), out = $('[data-out]'), chips = $('[data-chips]');
  let v = 0, seq = 0, alive = true;
  const onStake = () => { av.textContent = stake.value; sfx.tick && sfx.tick(); };
  const onDial = (e) => { const b = e.target.closest('button'); if (!b) return; v = +b.dataset.v; dial.querySelectorAll('button').forEach((x) => x.classList.toggle('on', x === b)); sfx.tick && sfx.tick(); };
  const onChip = (e) => { const c = e.target.closest('.chip'); if (!c) return; const [a, b] = PAIRS[+c.dataset.i]; ia.value = a; ib.value = b; sfx.tick && sfx.tick(); };
  const onGo = async () => {
    const a = ia.value.trim().slice(0, 80), b = ib.value.trim().slice(0, 80); if (!a || !b) { out.innerHTML = 'both sides, please.'; return; }
    if (a.toLowerCase() === b.toLowerCase()) { out.innerHTML = `a thing linked to itself moves focus by <b>0.0000</b> bits. copying earns nothing. ${M('k')}`; return; }
    const id = ++seq, tau = new Date(), stakeV = +stake.value, vv = v;
    out.innerHTML = `hashing …`;
    const [hp, hq] = await Promise.all([sha(a), sha(b)]); if (!alive || id !== seq) return;
    const dphi = (0.0007 + Math.random() * 0.004) * (vv === 0 ? 1 : 1.3) * Math.sqrt(stakeV / 10);
    out.innerHTML = `<b>${esc(a)}</b> ⟶ <b>${esc(b)}</b>
p  <b>${hp.slice(0, 16)}</b>… sha-256 of “${esc(a)}” · hemera on mainnet
q  <b>${hq.slice(0, 16)}</b>… sha-256 of “${esc(b)}”
τ  ${tau.toISOString().replace('T', ' ').slice(0, 19)}Z · a  <b>${stakeV}</b> CYB at risk · v  <b>${vv > 0 ? '+1' : vv}</b> ${vv === 0 ? '· ranks' : '· bets the truth market'}
Δφ⋆ <b>+${dphi.toFixed(4)} bits</b> · your share: 100% until someone else links it ${M('k')}`;
    sfx.link && sfx.link(); sfx.hash && sfx.hash();
  };
  stake.addEventListener('input', onStake); dial.addEventListener('click', onDial); chips.addEventListener('click', onChip); go.addEventListener('click', onGo);
  return { destroy() { alive = false; stake.removeEventListener('input', onStake); dial.removeEventListener('click', onDial); chips.removeEventListener('click', onChip); go.removeEventListener('click', onGo); mount.innerHTML = ''; } };
};
