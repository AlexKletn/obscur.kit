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

Use `::before` and `::after` to add decorative boxes. A quoted `content`
value creates a box, including an empty string for a decoration made entirely
with box styling. `content: none` removes it. The boxes participate in the
owner's layout and paint order. They stay outside the component tree,
accessibility tree, and event targeting. For text-only blocks with simple
decorations the runs share a row. Full inline formatting is outside this
layout model.

```css
.badge::before { content: "!"; color: orange; }
.badge::after { content: ""; display: block; height: 2px; background: orange; }
```

Use `::part()` for exported synthetic parts such as
`input::part(placeholder)`.

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
direction display filter flex-basis flex-direction flex-flow flex-grow flex-shrink
flex-wrap font-family font-size font-style font-weight gap row-gap column-gap grid-area
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

`flex-flow` accepts `flex-direction` and `flex-wrap` in either order. An
omitted component resets to its initial value (`row` or `nowrap`).
A `var()` can supply the whole shorthand or a component, with an optional
fallback that can itself reference another variable. The substituted value
is validated as one shorthand.

Longhands substitute `var()` before parsing the result as the target
property. References can appear inside functions such as `calc()` and
`min()`, including inside a fallback. Layout resolves percentage math in
sizing properties against the corresponding containing axis, including
`calc()`, `min()`, `max()`, and `clamp()`. Percentage margin expressions
use the containing inline extent and can produce negative offsets.
`gap` sets both axes. With two values it sets `row-gap` first and
`column-gap` second. A variable can supply one or both values, and the
substituted shorthand is validated before each axis is applied.
Percentage gap expressions use the container's content extent on each
definite axis in Flex and Grid. On a content-sized Grid axis, the percentage
component contributes zero to intrinsic sizing and resolves against the
resulting content extent for placement.
A resolved
value with the wrong grammar uses the property's invalid-at-computed-value
behavior.

### Grid placement limits

An authored grid placement can extend the explicit grid by at most 1,024
implicit tracks on either side of each axis. A placement wholly outside
the limit uses the nearest edge track. When arrangement clamps a placement,
the runtime reports `layout.grid_track_limit` with the first affected node.

### `pointer-events`

`auto` (initial) or `none`, and it **inherits** — SVG 1.1 §16.6 and css-ui both
say so, and here it is what makes the subtree half of the rule free: a descendant
of a `none` element has already computed `none`, so hit-testing reads one node's
own computed value and needs no walk. A descendant that declares `auto` computes
`auto` and stays hittable, which is the overlay-with-a-live-button pattern.

A `none` element is transparent to the pointer: it cannot be the geometric target,
and a pointer over its box reaches whatever is painted behind it. §4.7's hit
policy is what reads it (`HitTesting::is_pointer_transparent`).

## Percentage translations

`translate(x, y)`, `translateX(x)`, and `translateY(y)` accept percentages,
including in `calc()`. The horizontal percentage uses the arranged border box
width and the vertical percentage uses its height. Rendering and hit testing
use the same resolved transform.

CSS keyframes support percentages and linear `calc()` expressions that mix
pixels and percentages in these translations. The presentation sample uses
the box size at the frame being drawn, including when the size is animated.

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
Percentages in dimensions, positions, margins, padding and gaps stay
symbolic until layout knows their basis. Math expressions that include a
percentage use the same basis as the corresponding bare percentage.

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
| `@container` | **wired in `UiRuntime`** for named and unnamed size queries |
| `@import` | recognised and reported unsupported |
| `@supports` | not recognised |

### Container size queries

Declare the query container with `container-type: inline-size` or
`container-type: size`. Give it a name with `container-name` to target it
explicitly. The runtime selects the closest eligible logical ancestor and
evaluates its content box in logical pixels. Use `width`, `height`,
`inline-size`, or `block-size` with a pixel threshold.

```css
.sidebar { container: sidebar / inline-size; }
@container sidebar (min-inline-size: 280px) {
    .details { display: flex; }
}
```

The next scheduled frame applies changes after the initial layout or a resize.
With `container-type: size`, descendants do not contribute to the layout size
on either axis. With `inline-size`, only the logical inline axis is contained.
Explicit dimensions, min/max bounds, and sizes assigned by the parent still
apply. Descendants are laid out inside the resulting box and can overflow it.
Use `contain-intrinsic-size: 120px 80px` to provide the intrinsic content
width and height for size-contained axes. One length sets both axes and
`none` clears the override. The physical width/height and logical
inline-size/block-size longhands participate in the same cascade.
Remembered sizing with `auto` remains pending.

Both `container-name` and `container` accept a space-separated list of
case-sensitive names or the keyword `none`. In the shorthand, an optional
`/ normal`, `/ inline-size`, or `/ size` follows the list. Omitting the type
resets it to `normal`. A standalone `container: var(--preset)` substitutes
the whole shorthand. Variables can also supply individual parts, as in
`container: var(--names) / var(--kind, inline-size)`. An optional
fallback is used when the referenced custom property cannot be resolved.
That fallback can contain further variable references. The result is
validated as one shorthand before applying each longhand.

Container query units (`cqw`, `cqh`, `cqi`, `cqb`, `cqmin`, `cqmax`) use the
nearest eligible ancestor's content box for each axis. An axis with no
eligible ancestor uses the logical viewport. The same follow-up frame
updates these units when the query container changes size.

### Container style queries

`style(--name)` checks for a resolved custom property. `style(--name: value)`
compares its token value. A standalone `var(--other)` in the comparison
resolves on the selected container. A style-only query can select any
ancestor element, including one with `container-type: normal`. With a name,
it selects the nearest matching ancestor. A query combining style and size
features selects one ancestor eligible for all queried axes.

```css
.toolbar { container-name: toolbar; --density: compact; }
@container toolbar style(--density: compact) {
    .tool { padding: 4px; }
}
```

Changes to the selected ancestor's custom properties are applied on the
next scheduled frame. Use parentheses to combine `and`, `or`, and `not`
across size and style conditions or inside `style()`. For example,
`@container toolbar style((--density: compact) or (--density: dense))`
matches either value on the selected container. An unsupported condition
stays unknown under negation.

Standard-property queries support `display`, `position`, `box-sizing`,
`direction`, `writing-mode`, `visibility`, `pointer-events`, `isolation`,
`mix-blend-mode`, `color`, `background-color`, `opacity`, `order`,
`flex-grow`, `flex-shrink`, `flex-direction`, `flex-wrap`, `z-index`,
`align-content`, `justify-content`, `align-items`, `justify-items`, `align-self`,
`justify-self`, `overflow-x`, `overflow-y`, `overscroll-behavior-x`,
`overscroll-behavior-y`, `text-align`, `text-overflow`, `text-transform`,
`white-space`, `overflow-wrap`, `word-break`, `width`, `height`, `min-width`,
`min-height`, `max-width`, `max-height`, `flex-basis`, the physical and logical
`margin-*` and `padding-*` longhands, physical and logical insets, and
`font-size`. The logical margin, padding, and inset properties use the
selected container's writing mode and direction. Overflow queries use
the computed pair of axes, so `overflow-y: scroll` can compute
`overflow-x` to `auto`. For example,
`style(width: 120px)` compares the computed CSS width. An omitted value
tests whether the computed value differs from the initial value, and
`style(opacity: initial)` tests for the initial value. For the maximum size
properties, `none` matches an unbounded maximum. Sizing properties also
compare intrinsic keywords such as `min-content`, `max-content`, and
`fit-content`. Expected lengths
can use absolute or relative units, percentages, and math expressions where
the property grammar accepts them. For example, `style(width: 4em)` uses
the selected container's font size. Expected `font-size` values use the same bases as
normal declarations, including the parent font for `em`, `ch`, and
percentages. A standalone `var(--target, 4em)` in an expected value reads the
selected container's custom property, with an optional fallback that can
itself be a standalone `var()`. A resolved variable is validated using
the same rules as a literal comparison value. An invalid resolved value
makes the condition unknown, including under negation. An expression with an unresolved percentage compares to
the computed expression after normalizing its length components. Comparisons
of other standard properties remain pending.

Numeric custom-property ranges support numbers, percentages, and CSS
lengths. Relative lengths resolve in the selected container's context,
including its font metrics, root font size, viewport, and eligible ancestor
sizes for `cq*` units. The property can appear on either side of a comparison,
as in `style(--gap >= 12px)` or `style(12px <= --gap)`, or between two
bounds, as in `style(12px <= --gap < 40px)`. Chained comparisons must point
in the same direction. Values compare within the same dimension, with
unitless zero also usable as a length. Thresholds can reference custom
properties on the same container, including a literal fallback, as in
`style(var(--min, 12px) <= --gap < var(--max))`. Numbers, lengths, and
percentages can use `calc()`, `min()`, `max()`, and `clamp()` in values,
bounds, and fallbacks. For example, `style(--level >= calc(2 * 3))` compares
numbers, and `style(--gap >= calc(1em + 4px))` computes a length in the
selected container. Math that mixes lengths and percentages has no
percentage basis in this comparison. A fallback can reference another
property, as in `style(--gap >= var(--limit, var(--default, 12px)))`.
A resolved value that fails the comparison does not advance the fallback
chain.

### Media queries

Media queries support viewport width and height, their min/max and range
forms, and the logical spellings `inline-size` and `block-size`. Numeric
bounds use pixels. `orientation` follows the viewport's proportions, with
`portrait` matching square viewports. `resolution` uses the surface's
device pixel ratio, including fractional scales. It accepts `dppx` (or
`x`), `dpi`, and `dpcm`, with min/max and range forms. Both bounds of
`(1.5dppx <= resolution < 2dppx)` must hold. In the screen environment,
`print` evaluates to false.

`prefers-color-scheme`, `prefers-reduced-motion` and `prefers-contrast`
read the runtime's media preferences. A system appearance snapshot updates
all three. Without a preference from the host, the screen uses light mode
and no motion or contrast preference. The runtime represents increased
contrast as `more`.

`pointer` and `hover` use the primary device. `any-pointer` and `any-hover`
include additional devices. Hosts report capabilities with
`UiRuntime::set_media_input` or `AppControl::set_media_input`. Pass
`Some(MediaInput::default())` for a known absence of pointing devices.
With `None`, capabilities are unknown and these queries do not match.
Wayland windows, panels and panel popups receive a profile from seat capabilities.
A seat pointer is treated as the primary fine hovering device when touch
is also available. An explicit override takes precedence until
`clear_media_input_override()` restores the detected profile. Non-Wayland
window backends rely on the host to provide a profile.

Media queries support `and`, `or`, and `not`. Group mixed operators with
parentheses. Unsupported features are reported through
`CssPipeline::unsupported()` and evaluate to unknown. A known true branch
of `or` still matches, and negating an unknown term preserves it as unknown.
A syntax error disables only the affected comma-separated branch and is
reported as a parse error. An empty media list matches all media.
Transition
rules, animation declarations and keyframe definitions use the same media
and container query environment as style rules. The winning `transition` shorthand
replaces the property list according to importance, selector specificity,
and source order. A winning `transition: none` clears the list and stops
running CSS transitions on the element.

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

## Incremental style updates

For a child-list edit, sibling combinators add the candidate children and
their subtrees to the invalidation scope. Candidate children must have a
required class or ID from the right-hand compound. Unkeyed sibling rules
consider all children. Positional and relational pseudo-classes retain
conservative invalidation. A changed
stylesheet or style environment triggers a full pass.

## Debugging a sheet

| | |
|---|---|
| `CssPipeline::errors()` / `unsupported()` | what was dropped and why |
| `UiRuntime::computed_style(node)` | what actually won for a node |
| `OBSCUR_FULL_CSS_MOTION_MATCH=1` | also matches motion selectors on frames with unchanged style inputs |
| `OBSCUR_RESTYLE_TRACE=1` | logs the nodes that start incremental invalidation and whether broad structural rules are present |
| `OBSCUR_FULL_RESTYLE=1` | forces the pre-incremental restyle path — if a bug disappears under it, incremental invalidation is the cause |
| `UiRuntime::inspect_snapshot()` | the whole tree with boxes, for assertions |
