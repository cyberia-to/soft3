# the soft3 deck — notes for whoever edits it next

Live at https://new.soft3.org/ (static files on cyberproxy, `/var/www/html/new.soft3.org/`,
deploy = `rsync -az --delete --exclude .git site/ cyberproxy:/var/www/html/new.soft3.org/`).
soft3.org itself stays the shop window (`origin/main`); this branch is not meant to be
merged into it until the owner says so.

## what the second cut answers

The first deck was rejected for: a disconnected narrative, components shown but not as
products, charts pasted without interaction, no structure, logic, mystery, sequence or
beauty. The second cut is built against that list:

| complaint | what the deck does now |
|---|---|
| no structure, no sequence | cold open → one question → seven verbs (each: question → answer → demo → properties) → ten products → the road → the door; the rail shows the chapter, every chapter break names the next verb's question |
| disconnected narrative | one loop figure is the spine: it is lit on every verb slide, tapping a verb jumps there, the twelve buyer questions each name the verb that answers them |
| components, not products | the ladder is interactive: a rung opens a detail panel with "for whom", verb, registry state, release; the dependency chart focuses on a hub and lists what feeds it and what it feeds |
| charts without interaction | every figure is live (`figs-live.js`): hover dims the rest, click focuses; the registry chart lists every component under its state |
| no interactivity | seven demonstrations, one per verb (`demos/*.js`): run twice and diverge with a float, count steps before running, proof-size bars by statement, ask/remember, compile-shape slider, a signed cyberlink, a contraction to one focus |
| no beauty | the mushroom world is the page; text sits on it with shadow, glass only where a demo needs a surface; chapter hue drives `worldHue` |

## files

- `index.html` — shell: world canvas, HUD, rail, script order (world → data → features → figs → figs-live → demos → deck).
- `deck.js` — the slides (`S` array), the per-verb copy (`Q`, `TRY`), mounting of figures and demos, navigation, sound, fit.
- `deck.css` — the house stylesheet (same family as cyber.page, cyb.ai, cyberia.meme). `v2.css` — this deck's overrides.
- `figs-live.js` — interactive wrappers over `loop-fig.js`, `chart-fig.js`, `ladder-fig.js` (API in its header).
- `demos/*.js` — `window.demo_<verb>(mount, api) → {destroy()}`; `demos/test.html` mounts all seven.
- `registry.json` — parsed from `status.md`; `features.js` — the 30 properties with claim, parts, status, source.

## rules every number obeys

Marks: ● measured · ○ target · ◌ mock. Proof size today 15.6 KB, goal ≤ 64 KB (small statements ≤ 20 KB),
verify 2.5 ms today, 1 ms is a goal. Never "2 KB", never "STARK". There are no illegal bytes: information is legal.

## verification

Headless Chrome over raw CDP (`/tmp/jsd/cdp-deck.js`, `cdp-fit.js`, `cdp-await.js`): every slide
screenshotted at 1440×900 and 390×844, zero overflow across twelve viewports, every demo and figure
driven and asserted. If localhost dies (SYN_SENT, ping 127.0.0.1 lost) it is the VPN kill switch's
pf ruleset without `set skip on lo0`; see cyberia/dev.md § platform.
