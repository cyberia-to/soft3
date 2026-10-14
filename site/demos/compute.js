/* soft3 demo · compute — the same answer on every device.
   window.demo_compute(mount, api) → { destroy() }
   a tiny program over the Goldilocks field p = 2^64 − 2^32 + 1, run as BigInt on two "devices";
   the phone can "add a float" to show what IEEE doubles do to the same program (◌ mock). */
window.demo_compute = (mount, api) => {
  const M = api.M, sfx = api.sfx || {};
  const P = 18446744069414584321n, MASK = (1n << 64n) - 1n, ROUNDS = 8;
  const esc = (t) => String(t).replace(/[&<>"]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[c]));
  const hex = (v) => v.toString(16).padStart(16, '0');
  const parse = (s) => { s = String(s).trim().toLowerCase(); try { if (/^0x[0-9a-f]+$/.test(s)) return BigInt(s) % P; if (/^\d+$/.test(s)) return BigInt(s) % P; } catch (e) {} let h = 0n; for (const ch of s) h = (h * 131n + BigInt(ch.codePointAt(0))) % P; return h; };
  const pow7 = (y) => { const y2 = y * y % P, y4 = y2 * y2 % P; return y4 * y2 % P * y % P; };
  const exact = (a, b) => { const out = []; let s = a; for (let r = 0; r < ROUNDS; r++) { const x = s * b % P, y = (x + a) % P, z = pow7(y); out.push({ x, y, z }); s = z; } return out; };
  const toyHash = (rounds) => { let h = 0x9E3779B97F4A7C15n; for (const { x, y, z } of rounds) { h ^= z ^ (x << 1n & MASK) ^ (y >> 1n); h = h * 0x100000001B3n & MASK; h ^= h >> 29n; h = h * 0xBF58476D1CE4E5B9n & MASK; h ^= h >> 32n; } return hex(h); };
  const floaty = (a, b) => { const p = 2 ** 64 - 2 ** 32 + 1; const A = Number(a), B = Number(b); const out = []; let s = A; for (let r = 0; r < ROUNDS; r++) { const x = (s * B) % p, y = (x + A) % p, z = (y ** 7) % p; out.push({ x, y, z }); s = z; } return out; };
  const fhex = (v) => Number.isFinite(v) ? hex(BigInt(Math.trunc(Math.abs(v))) & MASK) : String(v);

  mount.innerHTML = `
  <div class="row" style="margin-top:0;align-items:flex-end">
    <div style="flex:1;min-width:0"><label style="margin-top:0">a</label><input type="text" data-a value="3141592653" inputmode="numeric" spellcheck="false"></div>
    <div style="flex:1;min-width:0"><label style="margin-top:0">b</label><input type="text" data-b value="2718281828" inputmode="numeric" spellcheck="false"></div>
  </div>
  <div class="fine" style="margin-top:8px">x = a·b · y = x + a · z = y⁷ (the s-box) · 8 rounds · over 𝔽ₚ, p = 2⁶⁴ − 2³² + 1 · toy hash in the browser, hemera on mainnet</div>
  <div style="display:grid;grid-template-columns:1fr 1fr;gap:10px;margin-top:10px">
    <div class="panel" style="padding:12px 14px"><label style="margin-top:0">💻 the same program · run 1</label><div class="out" data-t1 style="margin-top:4px;font-size:11px;min-height:0"></div><div class="hash" data-h1 style="margin:6px 0 0"></div></div>
    <div class="panel" style="padding:12px 14px"><label style="margin-top:0">📱 the same program · run 2</label><div class="out" data-t2 style="margin-top:4px;font-size:11px;min-height:0"></div><div class="hash" data-h2 style="margin:6px 0 0"></div></div>
  </div>
  <div class="row" style="margin-top:10px"><button class="btn ghost" data-float>add a float</button><span class="out" data-verdict style="margin:0;min-height:0;flex:1"></span></div>
  <div class="fine" style="margin-top:8px">bostrom's 2,949,732 links replayed from events, identical ${M('m')} · field arithmetic, never floats</div>`;

  const $ = (s) => mount.querySelector(s);
  const ia = $('[data-a]'), ib = $('[data-b]'), t1 = $('[data-t1]'), t2 = $('[data-t2]'), h1 = $('[data-h1]'), h2 = $('[data-h2]'), fb = $('[data-float]'), verdict = $('[data-verdict]');
  let useFloat = false;

  const render = () => {
    const a = parse(ia.value), b = parse(ib.value);
    const ex = exact(a, b), hh = toyHash(ex);
    t1.innerHTML = ex.map((r, i) => `r${i + 1} z <b>${hex(r.z)}</b>`).join('\n');
    h1.innerHTML = `hash ${hh}`;
    if (!useFloat) {
      t2.innerHTML = t1.innerHTML; h2.innerHTML = h1.innerHTML;
      verdict.innerHTML = `<b>identical</b> on both runs · same bytes, same hash · this browser plays both ${M('k')}`;
    } else {
      const fl = floaty(a, b); let first = -1;
      t2.innerHTML = fl.map((r, i) => { const same = fhex(r.z) === hex(ex[i].z); if (!same && first < 0) first = i; return same ? `r${i + 1} z <b>${fhex(r.z)}</b>` : `r${i + 1} z <span style="color:var(--c1)">${esc(fhex(r.z))}</span>`; }).join('\n');
      const last = fl[fl.length - 1].z;
      h2.innerHTML = `<span style="color:var(--c1)">hash ${Number.isFinite(last) ? 'drifted' : 'NaN'}</span> ${M('k')}`;
      verdict.innerHTML = first < 0 ? `the doubles got lucky this time; try bigger a, b ${M('k')}` : `<span style="color:var(--c1)">diverged at round ${first + 1}</span> · IEEE-754 rounds, the field does not ${M('k')}`;
    }
  };
  const onInput = () => { render(); sfx.tick && sfx.tick(); };
  const onFloat = () => { useFloat = !useFloat; fb.textContent = useFloat ? 'back to the field' : 'add a float'; fb.classList.toggle('ghost', !useFloat); render(); (useFloat ? sfx.tick : sfx.hash) && (useFloat ? sfx.tick() : sfx.hash()); };
  ia.addEventListener('input', onInput); ib.addEventListener('input', onInput); fb.addEventListener('click', onFloat);
  render(); sfx.hash && sfx.hash();
  return { destroy() { ia.removeEventListener('input', onInput); ib.removeEventListener('input', onInput); fb.removeEventListener('click', onFloat); mount.innerHTML = ''; } };
};
