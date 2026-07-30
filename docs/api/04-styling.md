# 04 — Styling

The kit styles with real CSS: an authored string goes through a parser and a
compiler into a matchable `Stylesheet`, and the runtime cascades, computes, lays
out and paints from it. There is no parallel styling API — if it is not CSS, it
is not styling.

```rust
use obscur_kit::css::{compiler::StylesheetCompiler, parser::StylesheetParser, source::CssSource};

let parsed = StylesheetParser::new(&CssSource::anonymous(css)).parse();
let sheet  = StylesheetCompiler::new().compile(&parsed).matchable;
let result = rt.render_now(&view, std::slice::from_ref(&sheet));
```

With `App`, `.stylesheet(css)` does this for you and may be called repeatedly.

---

## `CssPipeline`

`obscur_kit::bootstrap::CssPipeline` is the batteries-included version: it owns
the parse cache, installs the sheets *and* the CSS-authored motion on a runtime,
and keeps the diagnostics.

| Method | |
|---|---|
| `new() -> Self` | |
| `install(&mut self, runtime: &mut UiRuntime, sources: &[CssSource])` | parse, compile, install sheets and motion |
| `sheets() -> &[Stylesheet]` | |
| `motion() -> &CssMotionRules` | the `transition` / `@keyframes` rules |
| `errors() -> &[CssParseError]` | |
| `unsupported() -> &[UnsupportedCssFeature]` | **read this** — it is how the kit tells you a rule was dropped |
| `cached_sources() -> usize`, `invalidate_cache()` | |

Nothing throws on bad CSS. A rule the kit cannot honour is recorded in
`unsupported()` and dropped; a syntax error is recorded in `errors()`. Silence
from both is the only evidence your sheet did what you meant.

## Selectors

| Kind | |
|---|---|
| Simple | type, `*`, `.class`, `#id`, `[attr]`, pseudo-class, pseudo-element |
| Combinators | descendant (` `), child (`>`), next sibling (`+`) |
| Pseudo-classes | `:hover` `:focus` `:focus-visible` `:focus-within` `:active` `:disabled` `:enabled` `:checked` `:first-child` `:last-child` `:only-child` `:root` `:empty` `:nth-child()` `:nth-last-child()` `:nth-of-type()` `:nth-last-of-type()` `:nth-child(… of …)` `:is()` `:where()` `:not()` `:has()` |

No general sibling combinator (`~`).

Pseudo-**elements** parse and are then reported unsupported: `::before` and
`::after` generate nothing. This fails closed by design — a rule that would have
produced content produces none, rather than producing something wrong.

## Cascade origins

Lowest to highest: `SystemDefault` → `Theme` → `GlobalApp` → `ComponentModule`
→ `Inline`.

The UA stylesheet is installed at `SystemDefault`, so **any** author rule of
equal or higher specificity beats it — the browser contract.

## Properties

119 longhands are modelled. Shorthands (`margin`, `padding`, `border`,
`background`, `flex`, `gap`, `inset`, `border-radius`, `outline`, `font`, …)
expand to them.

```
align-content align-items align-self backdrop-filter background-color
background-image background-position background-repeat background-size
border-*-color border-*-style border-*-width border-*-radius
border-block-* border-inline-*  bottom box-shadow box-sizing color content
direction display filter flex-basis flex-direction flex-grow flex-shrink
flex-wrap font-family font-size font-style font-weight gap grid-area
grid-auto-columns grid-auto-flow grid-auto-rows grid-column-* grid-row-*
grid-template-areas grid-template-columns grid-template-rows height
inset-block-* inset-inline-* justify-content justify-items justify-self left
letter-spacing line-height margin-* margin-block-* margin-inline-* max-height
max-width min-height min-width mix-blend-mode object-fit object-position
opacity outline-color outline-offset outline-style outline-width overflow
overflow-wrap padding-* padding-block-* padding-inline-* position right
tab-size text-align text-overflow text-transform top transform transform-origin
visibility white-space width word-break word-spacing writing-mode z-index
```

Logical properties (`*-inline-start`, `*-block-end`, …) resolve onto their
physical counterparts at computed-value time against `direction` and
`writing-mode`.

## Units and math

Absolute (`px`), font-relative (`em`, `rem`, `ch`), viewport-relative (`vw`,
`vh`, `vmin`, `vmax`), and percentages.

`em`, `rem`, `vw`, `vh`, `vmin` and `vmax` fold to pixels **at computed-value
time**, once, so layout and paint cannot disagree about the same box.
Percentages stay symbolic until layout knows the containing block.

Math functions — `calc()`, `min()`, `max()`, `clamp()` — are accepted wherever a
`<length>` is, including as a component of a 1–4 value box shorthand, and one
that no longer depends on anything unknown collapses to a plain length at the
computed-value step. That collapse is what makes them work on paint-side
properties like `border-radius`: a consumer that reads a length rather than an
expression would otherwise read nothing and contribute zero.

```css
.key { border-radius: min(3vmin, 14px); font-size: min(6vmin, 34px); }
```

> **partial — `ch`.** It cannot fold at computed-value time (it needs the `0`
> glyph measured, which needs a shaper), so it stays symbolic and is resolved by
> layout. Paint-side properties read an absolute pixel length, so a `ch` that
> reaches one unresolved contributes `0` — `border-width: 2ch` reserves space
> and paints no edge.

## `env()`

`env(safe-area-inset-top | -right | -bottom | -left)` resolves against whatever
the host reported through `UiRuntime::set_safe_area_insets` /
`set_keyboard_insets` / `set_environment`. A change re-resolves and reaches
layout; it is not memoized away.

```css
.safe { padding-top: env(safe-area-inset-top); }
```

## At-rules

| | |
|---|---|
| `@media` | **wired** |
| `@keyframes` | **wired** — drives `UiRuntime::animate` through `set_css_motion` |
| `@container` | **unwired** — parses, then lowers to `RuleCondition::Never`. It fails closed: the rule never matches |
| `@import` | recognised and reported unsupported |
| `@supports` | not recognised |

> Container query *units* (`cqw`, `cqh`, …) resolve against the viewport rather
> than a query container.

## The UA stylesheet

`obscur_kit::ua_css()` returns the source; `ua_stylesheet()` returns the
compiled sheet; `KitBootstrap::install_default_stylesheet(rt)` installs it, and
`App` installs it for you.

It gives the base controls a grayscale "wireframe" look — ink `#2b2a27` on paper
`#faf9f6`, `1.5px` borders, `2px` radii — with the full interaction-state set. It
has rules for `button`, `input`, `textarea`, `checkbox`, `radio`, `select`, `a`,
`fieldset`, `table`, `th`, `td` and `img`.

**It carries real geometry, not just colour.** `button { padding: 11px 20px;
border: 1.5px solid … }` applies to every `Button` you place, and your class
cascades *over* it rather than replacing it. See the trap in
[05 — Components](05-components.md#the-trap-worth-knowing-before-you-style-one).

## Themes

`obscur_kit::theme_system` holds the token tier: a theme is a named set of
values a sheet reads, installed at the `Theme` cascade origin — above the UA
sheet, below the application's own rules.

## Motion

`transition` and `@keyframes` compile into `CssMotionRules`, installed with
`UiRuntime::set_css_motion`. The runtime drives them, and **a host must call
`advance_motion(delta)`** for anything to move. `App` does; a host that forgets
gets animations that work in tests and are frozen on screen.

## Known layout traps

Three worth knowing before they cost you an afternoon:

1. **A percentage width inside a padded percentage-width parent** resolved
   against the parent's *border* box, so each nesting level grew by its parent's
   padding. Now carried explicitly as a percentage basis through the constraint —
   but if a box comes out wider than you expect, this is the first thing to check.
   The block axis had the sibling version of the same confusion: the offer and the
   percentage basis were one value, so the *second* child of a block resolved its
   `height: %` against what its predecessors had left over (two `height: 100%`
   siblings measured 240 and **0**, the second vanishing with no diagnostic). Both
   axes now resolve against the containing block, and may overflow it, as
   `css-sizing-3` says they should.
2. **A flex container clamps its own border box** rather than overflowing, so a
   `width: 100%` plus padding gives a different answer under flex than under
   block. Unfixed.
3. **A missing glyph draws nothing at all**, not a box. If a label is blank, the
   font chain is the suspect before the layout is. See
   [07 — Text and fonts](07-text-and-fonts.md).

## Debugging a sheet

| | |
|---|---|
| `CssPipeline::errors()` / `unsupported()` | what was dropped and why |
| `UiRuntime::computed_style(node)` | what actually won for a node |
| `OBSCUR_FULL_RESTYLE=1` | forces the pre-incremental restyle path — if a bug disappears under it, incremental invalidation is the cause |
| `UiRuntime::inspect_snapshot()` | the whole tree with boxes, for assertions |
