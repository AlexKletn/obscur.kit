The built-in **user-agent (UA) default stylesheet** — the toolkit's neutral,
overridable base look for the built-in controls (spec §4.14).

This is the toolkit's analogue of a browser's UA stylesheet: a small set of
rules, keyed on each control's stable CSS *type* (`button`, `input`,
`checkbox`, `select`, …), compiled at the **lowest** cascade origin
([`CascadeOrigin::SystemDefault`]). Installed on a
[`UiRuntime`](crate::runtime::UiRuntime) via
[`set_default_stylesheet`](crate::runtime::UiRuntime::set_default_stylesheet)
(or automatically by
[`KitBootstrap::install_default_stylesheet`](crate::bootstrap::KitBootstrap::install_default_stylesheet)),
it is matched *below* every author/theme sheet, so any author rule of the same
or higher specificity overrides it — exactly the browser UA-sheet contract.

It gives the base controls the kit's **Wireframe** look: a grayscale
specification-sheet aesthetic (ink `#2b2a27` on paper `#faf9f6`, hairline
`1.5px` borders, `2px` radii) with the full interaction-state set — a hover
fill shift, a `2px dashed` focus ring with offset, an inverted `:active`, a
dashed muted `:disabled`, and a filled `:checked` — plus `a`, `fieldset`,
`img` and `table` rules. It leans only on properties and interaction
pseudo-classes the engine actually restyles (`:hover`, `:focus`, `:active`,
`:disabled`, `:checked`). Because it sits at `SystemDefault`, any author/theme
sheet overrides it, so themed demos keep their own look untouched.

# Where the sheet lives

The **text** is `obscur_theme_system::stylesheet::ua_css`, whose crate is
chartered for exactly this by `crate-arch §2.6`: "CSS; tokens;
fonts/icons/assets; light/dark/high-contrast presets; component style parts —
это package ресурсов, а не домен". Read that module for the rules themselves,
the Wireframe palette, and the guarantee that every selector in it names
something the toolkit can build.

What stays here is the one step the theme package may not take. Parsing and
compiling CSS is `obscur-css`, a `frontend/` crate, and §2.11 runs that edge
`frontends → components` — a components crate may not depend on a frontend one,
and `xtask check` enforces it. So [`ua_stylesheet`] is here, [`ua_css`] is
re-exported from there, and `obscur_kit::ua_css()` / `obscur_kit::ua_stylesheet()`
are unchanged for a host.
