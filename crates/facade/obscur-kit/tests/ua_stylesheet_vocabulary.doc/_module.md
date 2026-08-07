**The UA default stylesheet says only true things.**

Every selector in `obscur_theme_system::stylesheet::ua_css` is walked here and
checked against a vocabulary *harvested from real frames*: build every widget
and primitive in every state that emits a name or a class, render each through
a `UiRuntime` with the sheet installed, read the retained tree and the DevTools
part projection back, and require that every type, class and `::part()` the
sheet names is in what came out.

# Why the sheet needed a guard

A rule that matches nothing is the "built but not wired" defect
`docs/api/12-reality.md` tracks, and a stylesheet is where it hides best:
nothing fails, nothing warns, the rule parses and compiles and sits in the
cascade forever, and the only symptom is a control that does not look the way
the sheet says it does. The audit that opened this stage found three, in a
48-rule sheet:

1. `table`, `th`, `td`, `tr:hover` — the `Table` widget's rows and cells are
   real element nodes, but they carried a **private** property (`role`, with
   the text `row`/`cell`) instead of a name. Not a class, not a local name,
   invisible to matching. Fixed in the stage before this one, and pinned by
   `table_widget_is_css_visible.rs`.
2. `img` — `<img>` in markup and `Image::new(…)` in Rust are one built-in
   (`builtin_component_id("img")` *is* `Image::COMPONENT_ID`), but the tag
   contributed a local name and the primitive contributed none, so one rule
   reached one of the two trees.
3. `.placeholder` — and this is the one that showed. §4.16 makes the
   placeholder a synthetic, paint-only part, resolved through `::part()` and
   nothing else; the sheet spelled it as a CLASS, no node carries that class,
   and the hint was therefore painted in the full `#2b2a27` ink,
   indistinguishable from a typed value.

A fourth was aspirational rather than dead: `.select-option.disabled` named a
per-option disabled state that `Select` does not have.

# What the guard will and will not catch

It catches a rule for a name nothing builds — a widget renamed, a class
renamed, a control that never shipped, a `::part()` that is not generated. It
does **not** check that a rule's *properties* do anything, and it does not
check that a rule wins the cascade; those are the individual proofs below it
and in `component_output_is_styled.rs`, `ua_default_proof` and the widget
tests.

Its one escape hatch is `MARKUP_ONLY_ELEMENTS`, and that is deliberate: in this
engine every authored tag becomes its own local name, so "reachable from
markup" is true of any string and would make the guard vacuous. The three names
that need it are declared, argued and separately proved to render — and one of
them, `canvas`, is there because defect 2 above is still open for the canvas
primitive.

# The drawn controls

The tests at the foot of this file are about a different way a UA sheet can
fail to say anything: not a rule for a name nothing builds, but no rule at all.
`progress`, `slider` and `canvas` are the elements whose interior is *painted*
rather than built out of boxes — each is a box with a `<canvas>` child naming a
painter, and the runtime hands that painter exactly the child's layout box to
draw into. The sheet named none of them, so with no author CSS the child had an
`auto` height, the region had no area, `paint_progress` and `paint_slider`
returned before their first stroke, and the control was invisible with its
painter registered and running. Each of the three now has a documented
intrinsic size, an author override that beats it, and — for the slider — a test
that the box the pointer measures and the box the painter draws into are one
rectangle.
