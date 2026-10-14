# mounting world.js outside the old shell

`world.js` is the living canvas behind every soft3 page: a seeded forest — sky, five planets, a sun, trees, hills, a lake with
fish, ferns, mushrooms, flowers, butterflies, birds, spores — repainted every frame on a full-viewport 2D canvas. One seed
(`SEED = 0x50f73`) and one clock (`sessionStorage["soft3.world.epoch"]`, set on the first visit of the session) so the scene is
the same on every page and keeps moving across page swaps. These notes are what it took to run it in a page that is not the
`.shell` layout, verified with `figtest.html` (loop, chart and ladder figures over the world, desktop 1440×2400 and phone 430 wide).

## 1. the DOM it needs

```html
<body>
  <canvas id="world" aria-hidden="true"></canvas>   <!-- required: the id is hard-wired, read once at load -->
  <div id="tint" aria-hidden="true"></div>           <!-- optional: the colour wash the pages use, see §4 -->
  …your content, stacked above with z-index ≥ 1…
  <script src="/world.js?v=14"></script>             <!-- after the canvas, classic script, once -->
</body>
```

- `document.getElementById("world")` runs at the top of the script and `canvas.getContext("2d", …)` right after it: the
  canvas must be in the DOM before the script executes. Put the tag at the end of body (as the pages do), or `defer` it.
- It is a plain classic script, not a module, and it is not an IIFE: its top-level `const`/`let`/`function` names become
  global lexical bindings — `SEED rnd EPOCH organHue canvas SMALL screen off octx ctx W H dpr t0 mx my mxt myt trees mushrooms
  flowers ferns birds fish butterflies spores planets hsl resize seed drawSky drawPlanets drawSun drawTree drawHills drawWater
  drawFish drawBird drawFern drawMushroom drawFlower drawButterfly drawSpores GLITCH_EVERY GLITCH_LEN glitchPhase glitchBlit
  skip frame`. Any other classic script that declares one of these at top level throws `SyntaxError: Identifier … has already
  been declared` and the whole script is dropped. `deck.js` is wrapped in an IIFE, `data.js` declares only
  `VERBS ARCS NODES EDGES`, `features.js` declares `VERB GH n d t table FEATURES bars stackfig …` — no collision today, but
  any new top-level script needs checking. Note `const screen` shadows `window.screen` for every later classic script
  (bare `screen.width` is the canvas context, not the display).
- Load it once. A second load re-registers `resize`/`pointermove` listeners and starts a second frame loop on the same canvas.

## 2. the CSS it needs

From `base.css` (copy these if the page does not load base.css):

```css
html, body { background: #06100a; }
#world { position: fixed; inset: 0; width: 100%; height: 100%; display: block; z-index: 0; }
#tint  { position: fixed; inset: 0; z-index: 0; pointer-events: none; background: var(--facet, #7dffc4);
         mix-blend-mode: color; opacity: var(--tint, 0); transition: background .5s ease, opacity .5s ease; }
#tint::after { content: ""; position: absolute; inset: 0; background: var(--facet, #7dffc4); opacity: .25; }
@media (pointer: coarse) { #tint { mix-blend-mode: normal; opacity: calc(var(--tint, 0) * .3); } }
```

The script sets the canvas's own `width/height` (backing store, `innerWidth × dpr`) and `style.width/height` in px on every
resize, so the element size is always the viewport regardless of CSS; `position: fixed; inset: 0; z-index: 0` is only there
to pin it under the content. The context is created with `alpha: false`: the canvas is opaque and covers everything behind it
— there is no way to see the body background through it. In the deck, `#graph` sits at `opacity: .85`; the world should run at
opacity 1 (the pages' glass panels `rgba(6,16,10,.62–.78)` + `backdrop-filter` are what make text readable).

Content layering: the pages use `.shell { position: relative; z-index: 2 }`; the deck has `#deck { z-index: 1 }`, `.hud`
5, `.rail` above — all fine, the canvas is at 0 and `#tint` is at 0 after it in DOM order (so it paints over the canvas).

## 3. init

There is no init call. Loading the script is the init: it calls `resize()` (which also `seed()`s the scene) and
`requestAnimationFrame(frame)` at the bottom. It also hooks `addEventListener("resize", resize)` and
`addEventListener("pointermove", …)` on window for the parallax (`mx/my` ease toward the pointer: planets, trees, mushrooms,
hills, the sun and the water all shift a few px).

## 4. setting the colour from outside — the only API

```js
window.worldHue(h)   // h is a hue OFFSET in degrees added to every hue in the scene; 0 = the natural scene
```

That is the whole surface. `organHue` is added to the sky gradient, the aurora, the planets, the sun's corona, foliage, hills,
water, mushrooms, flowers, butterflies, spores and the soft-light grade — so the whole world slides toward the facet. The pages
all call it the same way, with the facet/verb hue from `VERBS[i].h` / `chartFig.FACETS[f].h` / `ladderFig.FACET[f][1]`:

```js
function tint(h, c) {                        // h = facet hue (120 form … 60 play), c = its colour; null = neutral
  const full = h == null;
  document.documentElement.style.setProperty("--facet", full ? "#fff8f0" : c);
  document.documentElement.style.setProperty("--accent-h", full ? 150 : h);   // base.css var, unused by world.js
  document.documentElement.style.setProperty("--tint", full ? 0 : 0.72);
  if (window.worldHue) worldHue(full ? 0 : h - 150);   // the scene is green (~150) by nature; shift it onto the facet
}
```

Two things the deck must know:

- `worldHue` is instantaneous — the next frame is already in the new hue. The `#tint` overlay transitions over .5 s, the
  canvas does not. For a chapter change that should glide, tween it from the host:
  `let cur = 0; const toHue = (target) => { const step = () => { cur += (target - cur) * .08; worldHue(cur); if (Math.abs(target - cur) > .5) requestAnimationFrame(step); }; step(); };`
- the hue is an offset, so hues that are 360° apart are the same; `h - 150` for `learn` (h = 0) is −150, which is fine (the
  `hsl()` helper wraps modulo 360).

Chapter → hue map for the deck, from `data.js`: what/doors (mint) → 0 · ladder (white) → 0 · compute 120−150 · build 240−150 ·
prove 255−150 · remember 270−150 · learn 0−150 · link 30−150 · focus 60−150 · frontier (learn red) → −150.

## 5. the burst

What the owner sees as "the burst" is the glitch: every 30 s (`GLITCH_EVERY`), for 0.12 s (`GLITCH_LEN`), the frame is drawn
to an offscreen canvas and blitted back with two horizontal slices displaced sideways. It is **phone-only**
(`glitchPhase` returns −1 unless `SMALL`), on a fixed timer from the session epoch, and **not exposed**: there is no function
to fire it, and `world.js` must not be edited for this task. Ways to give the deck a reaction on chapter change without touching
the file:

1. **tint flash** — raise `--tint` above its resting value for one transition and let it ease back:
   `root.style.setProperty("--tint", .95); setTimeout(() => root.style.setProperty("--tint", .72), 120);` (uses the
   existing .5 s transition on `#tint`; on phones the overlay is `normal` blend at 0.3×, so the flash is quieter there).
2. **a CSS filter pulse on the canvas** — `#world { transition: filter .4s }` and briefly set
   `filter: brightness(1.6) saturate(1.4)` then clear it. Cheap on desktop; on a phone a filter on a full-screen canvas is a
   full-screen composite per frame while it transitions — keep it short.
3. **a two-slice slip done by the host** — a sibling `<canvas>` over `#world` that, for ~120 ms, `drawImage`s two strips of
   `#world` displaced by ±18 px (exactly what `glitchBlit` does), then clears. This reads the world canvas back, which the
   script avoids on purpose for its own frame; for a 120 ms burst it is fine on desktop and tolerable on a phone.
4. if the owner later allows a one-line change in `world.js`: `let burstUntil = 0; window.worldBurst = (len = GLITCH_LEN) =>
   { burstUntil = (Date.now() - EPOCH) / 1000 + len; };` and in `glitchPhase`, before the `SMALL` test, `if (t < burstUntil)
   return 1 - (burstUntil - t) / GLITCH_LEN;` — the real glitch, on demand, on every device.

## 6. phones (`SMALL`)

`SMALL = matchMedia("(max-width: 900px), (pointer: coarse)").matches`, evaluated once at load (a desktop window resized
below 900 px later stays "desktop"; a tablet with touch is "small"). When small:

- `desynchronized: false` on the context (the desynchronised buffer tears on phones); `dpr` capped at 1 (desktop: 2.5);
- half the mushrooms and flowers (`× 0.5`), the other populations unchanged;
- every other vsync is skipped: the world runs at ~30 fps so the page stays responsive;
- the glitch burst is on (see §5); on desktop it is off because "the glass hides the world and it read as noise";
- base.css lightens the glass (`blur(10px)` instead of 22) and turns `#tint` into a plain overlay at 0.3× opacity.

## 7. gotchas hit while mounting

- **`window.NODES` is undefined.** `data.js` writes `const NODES = …` at top level, which is a global lexical binding, not a
  window property. Bare `NODES` resolves, `window.NODES` does not. The three figure modules resolve the tables by name
  (`(0, eval)("NODES")`, falling back to `window[name]`) so they work whether data.js is a classic script or someone later
  assigns the tables onto window. Any deck code that reads `window.VERBS` will get `undefined` the same way.
- **headless screenshots:** `--virtual-time-budget` never completes against the world's endless `requestAnimationFrame` loop
  (the shell waits for the page to go idle, it never does). A plain `--screenshot` with no virtual-time flag returns in ~2 s.
- **resize reseeds the scene.** `resize()` calls `seed()`, and `rnd` is not reset, so every window resize or orientation
  change rolls a different forest (the planets are fixed; trees, mushrooms, flowers move). Not a bug for the pages; worth
  knowing if the deck re-fits on resize.
- **the canvas is the viewport, always.** It uses `innerWidth/innerHeight`, not the element box — it cannot be mounted inside
  a panel or a slide at a smaller size.
- **the `#tint` overlay is `mix-blend-mode: color` over the whole viewport** — with `--tint: .72` it also colours the glass
  panels and text slightly (see the orange figtest shot at `?hue=30`). The pages accept this; the deck's `.slide.ink` chapters
  (solid verb colour) would be double-painted — set `--tint` to 0 on ink slides and let `worldHue` alone carry the colour.
- **cost.** One full-viewport 2D repaint per frame with hundreds of gradients; on an M-series desktop it is light, but stacking
  it under the deck's `backdrop-filter` glass and a `.grain` overlay is three full-screen passes per frame. If the deck feels
  heavy on phones, the world already halves itself; the next lever is dropping `.grain`.
- **fig.css repaints SVG text.** `.fig svg text { fill: var(--cream) }` is a stylesheet rule and beats presentation
  attributes; the figure modules put `fill` and `font-family` in `style=""` on every `<text>` so a figure inside a `.fig` keeps
  its verb colours. `.fig svg { max-width: 560px }` will still cap a figure's width unless the host overrides it.
- **the ladder needs no fixed height, the page's does.** The layers page gives the ladder the shell's 100vh row and lets
  groups and rungs split it with `flex: 1 1 0; min-height: 0` — in an auto-height host that computes to zero height. The
  module's stylesheet uses `flex: 1 1 auto` (content height when free, an even-ish fill when bounded) and adds a `.ladder.list`
  modifier that forces the phone list layout at any width.
- **marker ids.** Each SVG defines `<marker>`/`<radialGradient>` ids; two figures on one page must use different `opts.id`
  prefixes or the second one's arrows pick up the first one's defs.

## 8. the deck, in short (not applied — index.html and deck.js are untouched)

1. in `index.html`: replace `<canvas id="graph">` with `<canvas id="world" aria-hidden="true"></canvas><div id="tint" aria-hidden="true"></div>`,
   add the `#world` / `#tint` rules from §2 (or load `/base.css` — but base.css locks the body to one screen on desktop and
   styles `.shell/.top/.glass`, so copying the four rules is cleaner), and load `/world.js?v=14` after the canvas and before
   `/deck.js`;
2. in `deck.js`: drop the `world` IIFE at the bottom and provide the same two methods over the real world:
   `chapter(c)` → `tint(hueOf(c), c)` with the tween from §4, `pulse()` → the tint flash from §5;
3. figures: `loopFig({ id: "deck-loop", focus: verb })` on the loop slide and on each verb chapter opener,
   `chartFig({ id: "deck-chart", focus: hub })` on each rung slide (`chartFig.neighbours(hub)` gives the in/out rows for the
   text column), `ladderFig({ focus: hub, css: false })` with `ladderFig.css` injected once into a `<style>`. All three are
   strings: `host.innerHTML = …`, no iframes.
