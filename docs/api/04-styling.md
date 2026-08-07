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
| Combinators | descendant (` `), child (`>`), next sibling (`+`), subsequent sibling (`~`) |
| Pseudo-classes | `:hover` `:focus` `:focus-visible` `:focus-within` `:active` `:disabled` `:enabled` `:checked` `:first-child` `:last-child` `:only-child` `:root` `:empty` `:nth-child()` `:nth-last-child()` `:nth-of-type()` `:nth-last-of-type()` `:nth-child(… of …)` `:is()` `:where()` `:not()` `:has()` |

All four combinators are real end to end: the parser reads `~` as
`Combinator::SubsequentSibling`, the compiler lowers it to the style tier's own
`SubsequentSibling`, and the matcher walks preceding siblings for it. This page
used to say the general sibling combinator did not exist; it did, and does.

Pseudo-**elements** parse and are then reported unsupported: `::before` and
`::after` generate nothing. This fails closed by design — a rule that would have
produced content produces none, rather than producing something wrong. `::part()`
is a separate thing and is **not** affected — it is how §4.16's synthetic parts
(`input::part(placeholder)`) are reached.

## Cascade origins

Six of them, lowest to highest: `SystemDefault` → `Theme` → `GlobalApp` →
`ComponentModule` → `Inline` → `RuntimeOverride`.

The UA stylesheet is installed at `SystemDefault`, so **any** author rule of
equal or higher specificity beats it — the browser contract.

`Inline` is the `style="…"` attribute (see
[09 — Authoring formats](09-authoring-formats.md#the-style-attribute-is-a-declaration-block-not-a-prop)).
It needs a compiler installed on the runtime —
`UiRuntime::set_inline_style_compiler`, which `CssPipeline::install` does for
you. Without one, every `style` attribute is inert.

`RuntimeOverride` is the maximum, and deliberately so: it sits above authored
`!important` as well as above every origin below it, which is what makes it
usable by an inspector and by a driven animation. Losing to markup would make
both of those useless. `CascadeOrigin::is_runtime_override` is the predicate the
cascade asks, and `cascade_origin.rs`'s own tests pin the whole ladder.

## Properties

120 longhands are modelled, one of which is this kit's own (see
[Vendor properties](#vendor-properties)). Shorthands (`margin`, `padding`, `border`,
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
overflow-wrap padding-* padding-block-* padding-inline-* pointer-events position
right tab-size text-align text-overflow text-transform top transform
transform-origin visibility white-space width word-break word-spacing
writing-mode z-index
```

Logical properties (`*-inline-start`, `*-block-end`, …) resolve onto their
physical counterparts at computed-value time against `direction` and
`writing-mode`.

### `pointer-events`

`auto` (initial) or `none`, and it **inherits** — SVG 1.1 §16.6 and css-ui both
say so, and here it is what makes the subtree half of the rule free: a descendant
of a `none` element has already computed `none`, so hit-testing reads one node's
own computed value and needs no walk. A descendant that declares `auto` computes
`auto` and stays hittable, which is the overlay-with-a-live-button pattern.

A `none` element is transparent to the pointer: it never appears in a hit chain,
and a pointer over its box reaches whatever is painted behind it. §4.7's hit
policy is what reads it (`HitTesting::is_pointer_transparent`).

## Custom properties and `var()`

```css
:root { --accent: #c0392b }
.badge { color: var(--accent) }
```

Custom properties are declared, cascaded and **inherited** like any other
property, and `var()` substitutes against what the node inherited. A `style="…"`
attribute may declare one too, and — being origin 5 — it outranks a stylesheet's
declaration of the same name.

This is newly *reached* rather than newly built. Both ends existed for some time:
the parser read custom properties, the compiler collected them onto the compiled
sheet, and `substitute` was written to consume them — but nothing carried the
declarations from the sheet into the cascade, so `substitute` always read an
empty map and **every `var(--x)` in every stylesheet resolved to `initial`**,
silently and with no diagnostic. If you wrote a token layer against this page
before now, re-check what it actually painted.

One consequence of how they are carried, worth knowing if you write a front end:
a custom property's name travels as **text**, not as an interned keyword.
`CssPipeline` builds a compiler per source, so each sheet's interner numbers its
symbols from zero — two sheets declaring different names would otherwise mint the
same symbol for both. `-obscur-painter` carries its value as text for the
identical reason.

## Vendor properties

The convention every browser has followed since the beginning: a property named
`-vendor-something` belongs to one implementation, and every other
implementation ignores it. It is what lets one stylesheet serve renderers that
do not know each other.

**`-obscur-` is this kit's namespace.** A name in it that the compiler does not
know is a *fault* — a typo in your own house is worth reporting, and silence
there would let a misspelled rule sit in a sheet doing nothing.

| Property | Value | Meaning |
| --- | --- | --- |
| `-obscur-painter` | `<ident>` \| `<string>` \| `none` | Draw this element's box with the named painter |

```css
.sparkline { -obscur-painter: sparkline; width: 120px; height: 32px }
```

```rust
App::new("charts")
    .stylesheet(".sparkline { -obscur-painter: sparkline; width: 120px; height: 32px }")
    .painter("sparkline", |region, _paint| {
        // region.painter() is a display list; region.size() is the box
        // *layout* gave the element, not a size any constructor hinted at.
        false // "not animating"; `true` asks for the next frame
    })
    .run()
```

Without `App`, the runtime call underneath is
`UiRuntime::register_named_canvas("sparkline", Box::new(painter))`.

The two keys are **separate namespaces.** `App::painter` / `register_named_canvas`
register under the CSS-visible *name*; `App::painter_id` / `register_canvas`
register under the *symbol* a node names in its own `canvas_painter` property,
which is what the shipped widgets do. A painter registered under one is not found
by the other.

A painter a node binds *itself* wins over one a stylesheet names: a rule matching
a `<progress>` must not be able to replace the routine that draws it. CSS
supplies a painter for elements that declare none.

Registration is by key and the **last one wins**, and `App` applies an
application's registrations *after* the bootstrap's — so
`.painter_id(Progress::PAINTER, …)` replaces the built-in progress painter for
the whole application.

Any element can carry a painter this way — the property is not restricted to
`<canvas>`, and the runtime's painter map is keyed by `PainterKey`, not by node
kind.

**Every other vendor's prefixed _property_ is passed over in silence.**
`-webkit-*`, `-moz-*`, `-ms-*` and any `-whatever-*` **declaration** compiles to
nothing and is recorded at `Info`, not `Warning` — a stylesheet shared with a
browser build carries them by design, and a console full of warnings about them
trains you to ignore the warnings that matter.

A foreign prefix is **not** an alias. `-webkit-filter` does not become `filter`;
spec §4.13 excludes vendor hacks, and this is the exclusion that matters.

> **One at-rule breaks that rule, and it is worth knowing before you rely on
> either half.** `@-webkit-keyframes` is parsed as `@keyframes` — the parser
> matches `"keyframes" | "-webkit-keyframes"` on the at-rule name — so a
> `@-webkit-keyframes spin { … }` block really does install a timeline named
> `spin`. That is an alias, and it is the exact thing the paragraph above says
> the kit does not do. The two are not in conflict about *mechanism* (one is a
> declaration name, the other an at-rule name) but they are in conflict about
> *policy*. The declaration policy is the one this page argues for and the one
> the vendor-namespace commit was written to establish; the at-rule case is a
> single hard-coded pair with no test and no stated reason, and it should
> probably go. Until somebody decides, this note is the truth: **a foreign
> prefix is not an alias, except `@-webkit-keyframes`, which is.**

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
| `@media` | **wired, for the features below and no others** — see the fail-closed rule |
| `@keyframes` | **wired** — drives `UiRuntime::animate` through `set_css_motion` |
| `@container` | **unwired** — it lowers to a real condition, but nothing supplies a container size, so `FeatureQuery::holds` answers `false` on a container axis. The rule never matches |
| `@import` | recognised and reported unsupported |
| `@supports` | not recognised |

> Container query *units* (`cqw`, `cqh`, …) resolve against the viewport rather
> than a query container.

### What `@media` can actually ask, and what happens to the rest

Representable: `width` / `height` and their `min-`/`max-` and range forms, plus
the logical spellings `inline-size` / `block-size`. The reference value must be
a **`px` length** (or a unitless `0`). `@media print` is representable too, as
knowledge rather than ignorance — it is `Never` in this (screen) environment, so
`not print` correctly negates to `Always`.

Everything else — `prefers-color-scheme`, `orientation`,
`prefers-reduced-motion`, boolean features, unknown names, and any `min-`/`max-`
width written in `em` or `rem` or `vw` — is **unrepresentable**, and a query
containing one now lowers to `Never` **and is pushed onto
`CssPipeline::unsupported()`** as `@media (feature)`, which the facade also turns
into a `css.unsupported_at_rule` warning — reaching a host that installed a
diagnostic sink, and readable from `unsupported()` by one that did not.

That is a behaviour change, and it is the direction that matters. Until
`53f00463` an unrepresentable feature lowered to `Always`: a
`@media (prefers-color-scheme: dark)` block applied on **every** screen, always
and silently. A dark theme painting over a light application; a phone layout on
a 4K desktop. **If you have a stylesheet with such a block, its rules were
live and are now dead** — which is what you asked for, but it will look like a
regression until you read `unsupported()`.

Two details the fix had to get right, and which you can rely on:

- **`not` is not applied over an unrepresentable feature.** `Not(Never)` is
  `Always`, which would have reopened the same hole in the opposite polarity.
- **A query _list_ keeps whichever of its queries can be evaluated.** A list is a
  disjunction, so `@media (min-width: 600px), (orientation: landscape)` still
  honours the first half rather than being dropped whole.

## The UA stylesheet

`obscur_kit::ua_css()` returns the source; `ua_stylesheet()` returns the
compiled sheet; `KitBootstrap::install_default_stylesheet(rt)` installs it, and
`App` installs it for you.

It gives the base controls a grayscale "wireframe" look — ink `#2b2a27` on paper
`#faf9f6`, `1.5px` borders, `2px` radii — with the full interaction-state set. It
has rules for `button`, `input`, `textarea`, `checkbox`, `radio`, `select`, `a`,
`fieldset`, `table`, `th`, `td`, `tr` and `img`, for the sub-part classes the
widgets emit (`.checkbox-box`, `.radio-dot`, `.select-header`, …), and for the
`::part(placeholder)` hint.

The **source** lives in `obscur_kit::theme_system::stylesheet::ua_css` — the
crate whose charter is "CSS; tokens; fonts/icons/assets; presets; component
style parts" (`crate-arch §2.6`). Compiling it stays in the facade, because the
parser is a `frontend/` crate and a `components/` crate may not depend on one.
`obscur_kit::ua_css()` re-exports the text, so nothing a host writes changes.

**Every selector in it names something the toolkit can build**, and that is a
test rather than a promise: `tests/ua_stylesheet_vocabulary.rs` builds every
widget in every state, renders each, and rejects any rule for a type, class or
part that did not come out. A stylesheet is the easiest place in a toolkit to
hide a rule that matches nothing — nothing fails, nothing warns, and the only
symptom is a control that does not look the way the sheet says it does.

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
