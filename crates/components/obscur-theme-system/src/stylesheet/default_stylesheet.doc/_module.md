The built-in **user-agent (UA) default stylesheet** — the toolkit's neutral,
overridable base look for the built-in controls (spec §4.14).

This is the toolkit's analogue of a browser's UA stylesheet: a small set of
rules keyed on each control's stable CSS *type* (`button`, `input`,
`checkbox`, `select`, …) and on the sub-part classes the widgets emit. It is
compiled at the **lowest** cascade origin (`CascadeOrigin::SystemDefault`) by
`obscur_kit::default_stylesheet::ua_stylesheet`, so it is matched *below* every
author/theme sheet and any author rule of the same or higher specificity
overrides it — exactly the browser UA-sheet contract.

It gives the base controls the kit's **Wireframe** look: a grayscale
specification-sheet aesthetic (ink `#2b2a27` on paper `#faf9f6`, hairline
`1.5px` borders, `2px` radii) with the full interaction-state set — a hover
fill shift, a `2px dashed` focus ring with offset, an inverted `:active`, a
dashed muted `:disabled`, and a filled `:checked` — plus `a`, `fieldset`,
`img` and `table` rules. It leans only on properties and interaction
pseudo-classes the engine actually restyles (`:hover`, `:focus`, `:active`,
`:disabled`, `:checked`). Because it sits at `SystemDefault`, any author/theme
sheet overrides it, so themed demos keep their own look untouched.

# Why the text is here and the compilation is not

`crate-arch §2.6` charters this crate with "CSS; tokens; fonts/icons/assets;
light/dark/high-contrast presets; component style parts — это package
ресурсов, а не домен". The sheet is a *resource*: text that ships with the
theme package. Compiling it is a different job, done by `obscur-css`, which is
a `frontend/` crate — and §2.11's edge runs `frontends → components`, so this
crate may not reach it. The facade holds the one function that can
(`ua_stylesheet`), and re-exports `ua_css` from here so a host's
`obscur_kit::ua_css()` is unchanged.

# Every selector names something the toolkit produces

That is a guarded property, not an aspiration:
`crates/facade/obscur-kit/tests/ua_stylesheet_vocabulary.rs` walks every
compound of every selector in this text and fails on a name no widget, no
primitive, no runtime-generated part and no declared base element can put in
the tree. A UA sheet styles the *user agent's* own vocabulary; a rule for
anything else belongs in an author sheet, where it would at least be visible.
