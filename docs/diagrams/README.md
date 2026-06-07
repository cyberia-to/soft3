---
title: soft3 diagram generators
tags: cyber, soft3, tooling
crystal-type: tool
crystal-domain: cyber
---
# diagram generators

the svgbob diagrams in the soft3 docs are produced on an exact character grid
so every box edge and connector lands on the same column — no jogs, no orphan
stubs. hand-counting columns does not reach that standard, so each diagram is
emitted by a small Python layout generator.

| script | produces | rendered in |
|--------|----------|-------------|
| `troika.py` | the who-talks-to-whom troika compass | [[soft3]] README |
| `foundations.py` | the four-substrate bedrock diagram | [[soft3 foundations]] (docs/README) |

## regenerate

```
python3 foundations.py    # prints the svgbob ASCII to stdout
```

paste the output between a ` ```svgbob ` fence. optica's svgbob renderer
([[optica]] `render_svgbob_blocks`) converts the fence to inline SVG with
`currentColor` strokes so it inherits the page's theme.

## editing

each script places boxes at integer `(x, y, w)` coordinates and draws labelled
arrows between anchor points. to move a node, change its coordinate; to relabel
an edge, change its string. avoid `(` `)` in labels — svgbob reads parentheses
as arc primitives.
