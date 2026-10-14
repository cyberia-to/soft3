/* soft3 demo · build — the cost is known before it runs.
   window.demo_build(mount, api) → { destroy() }
   a six-line trident-style listing with a static loop bound N; cost comes from a formula, then a run confirms it (◌ mock). */
window.demo_build = (mount, api) => {
  const M = api.M, sfx = api.sfx || {};
  const P = 18446744069414584321n;
  const fmt = (n) => Number(n).toLocaleString('en-US');
  const cost = (N) => { const steps = 3 + 5 * N, ops = 3 * N + 24, rows = 1 << Math.ceil(Math.log2(steps)); return { steps, ops, rows }; };
  const MAX = cost(1024);

  mount.innerHTML = `
  <div class="out" style="margin-top:0;font-size:12px;line-height:1.5;min-height:0;opacity:.95"><span style="opacity:.5">1</span>  fn work(a: F) -> F          <span style="opacity:.5">// N is a constant</span>
<span style="opacity:.5">2</span>    let mut s = a
<span style="opacity:.5">3</span>    for i in 0..<b data-n>8</b>              <span style="opacity:.5">// bound known at build</span>
<span style="opacity:.5">4</span>      s = s · s + a            <span style="opacity:.5">// 2 field ops</span>
<span style="opacity:.5">5</span>      s = s + i                <span style="opacity:.5">// 1 field op</span>
<span style="opacity:.5">6</span>    hash(s)                    <span style="opacity:.5">// 24 ops, one permutation</span></div>
  <label>N · loop bound · <b data-nl>8</b></label>
  <input type="range" min="1" max="1024" value="8" data-range>
  <div class="bars" style="margin-top:10px;font-size:12px">
    <div class="bar me" style="grid-template-columns:140px 1fr 90px"><span>steps 3 + 5·N</span><i data-b1></i><span data-v1></span></div>
    <div class="bar" style="grid-template-columns:140px 1fr 90px"><span>field ops 3·N + 24</span><i data-b2></i><span data-v2></span></div>
    <div class="bar" style="grid-template-columns:140px 1fr 90px"><span>rows 2^⌈log₂ steps⌉</span><i data-b3></i><span data-v3></span></div>
  </div>
  <div class="row"><button class="btn" data-run>run it</button><span class="out" data-out style="margin:0;min-height:0;flex:1">predicted before execution ${M('k')} · press run to count the real steps</span></div>
  <div class="fine" style="margin-top:8px">no recursion, no heap, bounded loops: there is no halting problem on this machine.</div>`;

  const $ = (s) => mount.querySelector(s);
  const range = $('[data-range]'), out = $('[data-out]'), run = $('[data-run]');
  const bars = [[$('[data-b1]'), $('[data-v1]'), 'steps', MAX.steps], [$('[data-b2]'), $('[data-v2]'), 'ops', MAX.ops], [$('[data-b3]'), $('[data-v3]'), 'rows', MAX.rows]];
  let N = 8, predicted = cost(N);
  const show = () => {
    predicted = cost(N); $('[data-n]').textContent = N; $('[data-nl]').textContent = N;
    for (const [bar, val, k, mx] of bars) { bar.style.transform = `scaleX(${Math.max(predicted[k] / mx, .01)})`; val.innerHTML = `${fmt(predicted[k])} ${M('k')}`; }
  };
  const onRange = () => { N = +range.value; show(); out.innerHTML = `predicted before execution ${M('k')} · press run to count the real steps`; sfx.tick && sfx.tick(); };
  const onRun = () => {
    let steps = 0, s = 7n; const a = 7n; const t0 = performance.now();
    steps += 2; // fn entry + let
    for (let i = 0; i < N; i++) { s = (s * s % P + a) % P; s = (s + BigInt(i)) % P; steps += 5; } // mul, add, add, i++, bound check
    steps += 1; // hash
    const t = (performance.now() - t0).toFixed(2);
    const ok = steps === predicted.steps;
    out.innerHTML = `ran · counted <b>${fmt(steps)}</b> steps · predicted ${fmt(predicted.steps)} · ${ok ? '<b>equal</b>' : 'MISMATCH'} · ${t} ms in js ${M('k')}`;
    if (ok) { sfx.chord && sfx.chord(2); } else { sfx.tick && sfx.tick(); }
  };
  range.addEventListener('input', onRange); run.addEventListener('click', onRun);
  show();
  return { destroy() { range.removeEventListener('input', onRange); run.removeEventListener('click', onRun); mount.innerHTML = ''; } };
};
