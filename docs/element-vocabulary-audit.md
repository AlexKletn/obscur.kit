# The element vocabulary: one symbol, two spellings, forty-eight rules

The UA default stylesheet is 48 rules of ordinary CSS, compiled through the
ordinary parser at `CascadeOrigin::SystemDefault` — the lowest normal origin
(spec §4.14), so any author rule of any specificity beats it — and installed once
by `KitBootstrap`, not per frame. None of that changed and none of it may.

What changed is what the sheet's selectors could *reach*. An audit found three
faults, and they were one fault seen from three distances: **the toolkit had two
element vocabularies and thought it had one.** A tag written in markup and a
widget built in Rust produced different trees under the same CSS type, and a
stylesheet cannot tell you that — a rule that matches nothing has no error, no
warning and no missing symbol. It just quietly does not happen.

Three stages closed it. This is the ledger.

**Numbers, measured on the tree and not copied from a stage report.** Suite
**6,231 → 6,261 passed / 0 failed** (+30: +19 in files that already existed,
+11 in the two new test files). `clippy` **360** workspace warnings, unchanged.
`cargo fmt --all --check`, `xtask check` and `xtask consumer` all clean. Every
file the campaign touched is far under the 1,000-line non-test bar — the largest
is `view_codegen.rs` at 458 non-test lines — and each still exports one main
entity, so §3.16 never came into it: this was never a decomposition.

---

## Fault 1 — one symbol carried two meanings

`obscur-view`'s reserved `TYPE` symbol (`0x0000_0003`) carried **both** the
element's local name — what a CSS type selector matches — and the authored
`type="…"` attribute. `name_binding::local_name_for` documented the collision
honestly and resolved it *the author wins*: given an authored `type`, the tag
contributed no local name at all.

The price was silent and landed on the author. `<button type="submit">` — the
most ordinary markup there is — was an element whose CSS type was `submit`. It
matched no `button { … }` rule and lost **the whole** of its UA styling:
background, ink, border colour, font size, every state rule. And `[type=submit]`
could not be written to mean the attribute, because `TreeElement::attribute("type")`
returned the local name. Worse, since every element answered `[type=<its own tag>]`,
`[type=div]` matched every box in the document.

**Fixed by giving the element's name its own symbol.** `LOCAL_NAME`
(`0x0000_000A`, a free slot in the dense reserved space) carries the element's
own name; `TYPE` returns to meaning only what an author wrote. `StyleAttributes`
grew a `control_type` field beside `local_name`, both participating in `is_empty`
and `PartialEq` so a `type` change re-applies attributes and re-matches.
`local_name_for` lost its author-wins bail-out and, with it, its now-unread
`authored_attributes` parameter — the signature is `local_name_for(tag, position)`.
Roughly seventy `TYPE` call sites across twenty-nine files were triaged one at a
time; the ones left on `TYPE` are the ones that mean the attribute.

Two consumers were being silently damaged and are fixed as a side effect, because
the local name was never only a styling concern:
`SemanticBuilder::role_for_local_name` derives the §7.1 accessibility role from
it, and `EditableField::is_multiline` decides multi-line editing by
`local_name() == "textarea"`. An authored `type="…"` used to knock an element out
of its accessibility role as well as its styling.

## Fault 2 — the vocabulary was half-built

`builtin_component_id` gave lowercase element spellings to `button`, `img`,
`canvas`, `scroll`, `portal` and `layer`, and PascalCase-only spellings to
`Input`, `Checkbox`, `Radio`, `Select`, `Table`, `Menu`, `Dialog`, `Tabs`,
`List`, `Tree` and `Docking`. So `<checkbox>` in markup was a styled `div` with
no behaviour while `Checkbox::new()` was the real control — **the same CSS type
over two different trees.** The file's own doc stated the principle it had
stopped short of: "Same 'one handle, two spellings' rule the `Image`/`img` pairs
below already follow."

**Fixed for exactly the controls the sheet styles**: `input`, `textarea`,
`checkbox`, `radio`, `select` and `table` now bind to the shipped widget's
`ComponentId`. `textarea` maps to `Input::COMPONENT_ID` — HTML's two text-entry
tags are one widget here — and the distinction they carry rides on the local
name, which is already what `is_multiline` reads, so nothing new had to be
invented to carry it.

`Menu`, `Dialog`, `Tabs`, `List`, `Tree` and `Docking` deliberately keep one
spelling. The UA sheet styles none of them by type, a lowercase tag no rule
reaches would be vocabulary with nothing behind it, and most have no HTML element
that means what they mean. That boundary used to be implicit; it is now stated in
code and pinned by `name_binding.rs::the_widgets_the_ua_sheet_does_not_style_keep_one_spelling`,
so it cannot drift without a test failing.

## Fault 3 — seven of the forty-eight rules reached nothing the Rust API builds

| Rule | Why it was dead | Now |
|---|---|---|
| `table`, `th`, `td`, `tr:hover` | `Table`'s rows and cells are real element nodes, but they carried a **private** symbol `PROP_ROLE` (`0x00D1_0A00`) holding the text `"row"`/`"cell"`. Not a class, not a local name — CSS could not see it at all | the widget's parts name themselves `table` / `tr` / `th` / `td` under `LOCAL_NAME`. `PROP_ROLE` is **deleted**: it had zero readers workspace-wide, and `th` vs `td` is a distinction it could not carry and the sheet styles differently |
| `img` | Fault 1's shape one tier down. `builtin_component_id("img")` *is* `Image::COMPONENT_ID`, so `<img>` and `Image::new(…)` are one built-in — but the tag contributed a local name and the primitive contributed none. One id, two trees, one rule reaching one of them | `Image` emits `LOCAL_NAME` `img`. The used value does not move (`LayoutDisplay` already defaults to `Block`), but the primitive also gains `SemanticRole::Image`, which it had been falling through to an unnamed `Group` for |
| `.placeholder` | **The one dead rule whose deadness was visible.** §4.16 makes the placeholder a synthetic, paint-only part — `Input::view` says outright "the placeholder is NOT a child" — resolved through `part_specificity_for`, which reads a `::part()` subject and nothing else. A *class* could never reach it. Measured before the fix: the hint painted in `#2b2a27`, the full ink, indistinguishable from a typed value | `input::part(placeholder), textarea::part(placeholder)`. Measured after: `#9b998f`, for both hosts. **One half of this was still dead and is now closed**: the rule was reached only by `Input::placeholder(…)`, because the front ends hashed an authored `placeholder="…"` into an ordinary prop and `StyleAttributes::placeholder` reads the reserved symbol alone. `attr_symbol` now maps the name to it, so `<input placeholder="…">` generates the part too. **And making it reachable made a second defect reachable with it, now closed too**: `:placeholder-shown` asked whether the *reserved* `VALUE` attribute was empty, which authored markup never writes (that attribute declares a **controlled** value the runtime snaps a drifting buffer back to, while markup's `value="…"` is HTML's *default* value the user then owns — one symbol for both would make every authored field uneditable). So `<input value="abc" placeholder="Search"/>` painted the hint on top of its own text. `resolve_placeholder` now reads the declared value when there is one and otherwise the text the field renders — the mounted control's editable run — which is the same value seen from the only place an authored field states it. Pinned by `tests/builtin_components.rs::an_authored_value_hides_the_placeholder_the_way_a_declared_one_does` |
| `.select-option.disabled` | Aspirational rather than misspelled: it assumed a per-option disabled state. `Select::option(label)` takes a plain string, `disabled` is a property of the whole control, and an option's class is only ever `select-option` or `select-option selected` | `select:disabled .select-option` — the way `select:disabled .select-arrow` beside it already spells the same idea. `(0,2,1)` beats `.select-option`'s `(0,1,0)`, so it wins on specificity, not source order |

## Where the sheet now lives

The sheet's **text** moved to
`crates/components/obscur-theme-system/src/stylesheet/default_stylesheet.rs`.
crate-arch §2.6 gives that crate "CSS; tokens; fonts/icons/assets;
light/dark/high-contrast presets; component style parts — это package ресурсов,
а не домен", and a stylesheet is exactly that.

The **compilation** stayed in the facade, and the reason is a layering law rather
than a preference: `obscur-css` is a `frontend/` crate, §2.11 runs that edge
`frontends → components`, and `xtask check`'s `Layer::Components` admits only
foundation/domain/components — so a components crate physically may not reach a
parser. `obscur_kit::default_stylesheet` re-exports `ua_css` and keeps
`ua_stylesheet`; **every existing `use` path is unchanged**, and that is asserted
by *pointer identity* (`std::ptr::eq`), because two equal strings would be the
exact drift the move exists to rule out.

One consequence is load-bearing: the crate that owns the text cannot check that
the text parses, so the one crate that can is the one that must —
`ua_css_compiles_without_syntax_errors` lives in the facade and is now the only
thing standing between a typo and a silently dropped rule.

## The guard

Fixing four rules is worth less than making the class of fault impossible, so
`crates/facade/obscur-kit/tests/ua_stylesheet_vocabulary.rs` enforces the
property instead of the instances. It builds every widget and primitive **in
every state that emits a distinct name, part or class**, renders each through a
real `UiRuntime`, harvests the element names, `::part()` names and classes off
the retained tree, then walks every compound of every selector in the sheet —
head *and* each tail step, not just the subject — and rejects anything outside
that harvest.

Three design notes, because each was a trap:

- **The vocabulary must be harvested, not declared.** Since `local_name_for`,
  *every* authored tag becomes its own local name, so "markup could produce it"
  is true of any string whatsoever and a guard built on reachability-from-markup
  would be vacuous.
- **The harvest must enumerate states, not widgets.** `.checked` appears only on
  a checked `Checkbox`, `.checkbox-bar` only on an indeterminate one,
  `.select-list` / `.select-option` only while open. Two API facts are written
  into the test because they bit: `Select::select(i)` **closes** the popup (so
  the selection must precede `open()`), and `Select::open()` **refuses** when
  disabled.
- **Every compound, not `subject()`.** Reading subjects alone would check
  `.checkbox-box` in `checkbox:hover .checkbox-box` and never `checkbox`, leaving
  the whole descendant-spelled half of the sheet unguarded.

Pseudo-classes are checked through `PseudoClassSelector::required_state().is_some()`
— "a state the runtime can put a node into" — rather than a hard-coded list, so
`:hover` / `:focus` / `:active` / `:disabled` / `:checked` pass and
`button:first-child` is rejected. Any id selector is rejected outright: a UA
sheet must not target one.

The guard's teeth are kept rather than demonstrated once and thrown away —
`the_guard_rejects_a_rule_for_a_name_nothing_builds` feeds it a bogus type, class,
part, id and unreachable pseudo-class and requires all five back. **Verified by
experiment at close:** injecting `dialog { padding: 4px }`,
`.tooltip-arrow { width: 6px }` and `select::part(chevron) { color: #000 }` into
the *real* sheet made the real guard fail, naming exactly those three and nothing
else; the injection was then reverted and the sheet is byte-identical.

Harvested vocabulary, for the record — 13 element names, 12 classes, 1 part:

```
elements: a button checkbox fieldset img input radio select table td textarea th tr
classes:  checkbox-bar checkbox-box checked indeterminate open radio-dot radio-fill
          select-arrow select-header select-list select-option selected
parts:    placeholder
```

## Deliberately left open

- **`a`, `a:hover`, `a:disabled`, `fieldset`** — four rules with no widget and no
  primitive behind them. They are alive through hand-written markup only; nothing
  a program builds is ever an `<a>`. They are the guard's **entire** escape hatch,
  `MARKUP_ONLY_ELEMENTS`, deliberately two names long, and each entry is *proved*
  to reach the tree rather than trusted.
- **The base-element tags mount no factory.** An authored `<checkbox>` is a
  checkbox to the cascade, to hit-testing and to the accessibility tree, but the
  widget's own `view()` — its indicator sub-part, its `:checked` class — does not
  run, because nothing registers a component factory for the built-in widget ids
  by default. The id is the handle a host *could* register under. Recorded in
  `docs/api/12-reality.md`.

  > **Since closed**, by the campaign that made the shipped widgets components.
  > `KitBootstrap` registers a factory for `<checkbox>`, `<radio>`, `<select>`,
  > `<input>` and `<textarea>` by default; `<button>` and `<table>` still mount
  > nothing. The live ledger is `docs/api/12-reality.md`, not this line.
- **Contract validation (§2.13) is keyed by the authored tag string**, so the new
  lowercase tags stay unvalidated exactly as `<button>` already did: `<Table/>` is
  `no_children()` and `<table>` is not, `<Checkbox/>` requires `label` and
  `<checkbox>` does not. Consistent with the existing rule, but it means the
  base-element spellings are permissive by design.

  > **Narrowed since**, in exactly one place and for exactly one reason: the five
  > lowercase tags that now mount a factory (`input`, `textarea`, `checkbox`,
  > `radio`, `select`) carry a `no_children()` *child* policy, because a mounted
  > component's output replaces its host's children and an authored child under
  > one of them could otherwise vanish without a word. Their prop surfaces stay
  > open — declaring one prop would close the whole surface. See
  > `mounted_element_contracts`.
- **`SemanticRole` has no `ColumnHeader`**, so `th` maps to `Cell` — the nearest
  true role, rather than falling through to `Group`. Adding the variant is a
  domain change beyond this work, and the near-miss is stated at the mapping.
- **The other widgets' `PROP_ROLE` symbols are equally write-only.** Across
  button / checkbox / dialog / docking / input / list / menu / select / tabs, the
  only read anywhere is `Menu::PROP_ROLE` inside Menu's own unit test. They feed
  nothing — the accessibility role comes from `LOCAL_NAME`. Left alone as out of
  scope; a future stage could retire the lot.

## Tests: what was rewritten, and why

The campaign's hard rule is that no test is ever weakened to make a change pass.
Seven test *names* disappeared. Every one is accounted for, and the accounting is
the point — six are the same assertion re-aimed, one is a test whose subject
ceased to exist.

| Was | Now | Why |
|---|---|---|
| `okui_base_elements.rs::an_authored_type_attribute_wins_the_overloaded_slot` | `::an_authored_type_attribute_and_the_tag_are_two_channels` | It asserted the node's local name was `submit` and that `button { … }` **missed** it. Same markup, inverted expectation, plus the `[type=submit]` half and a negative case |
| `tree_element.rs::attribute_resolves_id_and_type_only` | `::attribute_resolves_id_and_the_authored_type_only` | It asserted `attribute("type") == Some("button")` on a node that had **no** `type` at all — the local name answering for an attribute nobody wrote. It now writes a real `type="submit"` and asserts the tag and the attribute separately |
| `ui_ir_view_bridge.rs::an_authored_type_wins_over_the_implicit_local_name` | `::an_authored_type_travels_beside_the_implicit_local_name` | The `TYPE` assertion is kept verbatim; the `LOCAL_NAME` half is added |
| `view_codegen.rs::an_authored_type_replaces_the_implicit_local_name` | `::an_authored_type_is_emitted_beside_the_implicit_local_name` | It carried `assert!(!out.contains("button"))`, which *is* the bug. It now asserts one property under each symbol |
| `view_codegen.rs::an_element_tag_emits_its_local_name_under_the_type_symbol` | `::…_under_the_local_name_symbol` | Both assertions ported unchanged; only the symbol moved `0x3 → 0xa`, which is the whole point |
| `default_stylesheet.rs::placeholder_class_is_muted_with_the_wireframe_faint_ink` | `::the_placeholder_part_is_muted_with_the_wireframe_faint_ink` | It encoded the defect: it asserted the sheet contains a `.placeholder` rule with `#9b998f` and mistook that for the outcome, while its own docstring claimed the hint "reads as a muted hint" — which was false. The colour claim is unchanged; the subject is the one the runtime resolves |
| `name_binding.rs::an_authored_type_attribute_wins_the_local_name_slot` | *deleted* | Its entire subject was the author-wins bail-out, which no longer exists. Two new tests cover the ground that survives it |

Two unit tests in `obscur-primitives/src/image.rs` kept their names and changed
how they read: `emits_source_only_without_alt` and `emits_alt_when_set` asserted
props by **position and count** (`props.len() == 1`, `props[0]`) — exactly the
shape that made adding a property indistinguishable from changing one, and
exactly why `img { … }` could not reach the node. The claims about `src` and
`alt` are preserved in meaning; only the lookup changed, to be by symbol.

**No test disagreed with the vocabulary change in stage 2** — the suite went
6,234 → 6,250 with zero failures at any point. That absence is itself a finding:
`<checkbox>`-as-a-`div` and the four dead table rules survived precisely because
nothing asserted anything either way.

## What pins what

| Claim | Test |
|---|---|
| `<button type="submit">` keeps the **whole** UA button look | `tests/ua_stylesheet_vocabulary.rs::an_authored_type_attribute_does_not_cost_the_element_its_ua_styling` |
| the tag and the attribute are two channels | `tests/okui_base_elements.rs::an_authored_type_attribute_and_the_tag_are_two_channels` |
| `[type=…]` reads the authored attribute; a plain element matches no `[type=…]` | `tree_element.rs::attribute_resolves_id_and_the_authored_type_only`, `::an_element_without_an_authored_type_has_no_type_attribute` |
| `LOCAL_NAME` and `TYPE` are independent, and equality/emptiness see both | `style_attribute.rs::the_local_name_and_the_authored_type_are_two_independent_channels`, `::equality_includes_the_authored_type` |
| all twelve reserved style-attribute symbols are pairwise distinct | `style_attribute.rs::every_reserved_style_attribute_symbol_is_distinct` |
| both lowering paths emit the two symbols separately | `ui_ir_view_bridge.rs::an_authored_type_travels_beside_the_implicit_local_name`, `view_codegen.rs::an_authored_type_is_emitted_beside_the_implicit_local_name` |
| every control the sheet styles has both spellings, resolving to the shipped widget | `name_binding.rs::every_control_the_ua_sheet_styles_has_both_spellings`, `tests/okui_name_binding_drift.rs::every_control_the_ua_sheet_styles_resolves_to_the_shipped_widget`, `tests/okui_base_elements.rs::every_control_the_ua_sheet_styles_is_the_shipped_widgets_id_in_markup` |
| widget and tag agree on one CSS type, for the whole vocabulary | `tests/okui_name_binding_drift.rs::every_control_and_its_tag_agree_on_one_css_type` |
| `<textarea>` is the `Input` widget and still the multi-line one | `name_binding.rs::textarea_is_the_input_widget_and_says_so_in_its_local_name`; behaviourally `tests/okui_base_elements.rs::an_authored_textareas_enter_inserts_a_newline_and_an_inputs_does_not` |
| the six unstyled widgets keep one spelling | `name_binding.rs::the_widgets_the_ua_sheet_does_not_style_keep_one_spelling` |
| `Table`'s parts name themselves in HTML's table vocabulary | `table.rs::every_part_names_itself_in_the_html_table_vocabulary`, `tests/table_widget_is_css_visible.rs::the_widgets_parts_name_themselves_in_htmls_table_vocabulary` |
| `th`/`td`/`tr:hover` reach the widget's own nodes, and a widget and hand-written markup compute alike | `tests/table_widget_is_css_visible.rs::the_ua_sheets_th_and_td_rules_reach_the_widgets_own_cells`, `::the_ua_sheets_tr_hover_rule_reaches_a_row`, `::the_widget_and_hand_written_markup_are_styled_by_the_same_rules` |
| the placeholder paints in the faint ink, both hosts | `tests/ua_stylesheet_vocabulary.rs::the_placeholder_hint_is_painted_in_the_faint_ink` |
| `Image::new` and `<img>` reach one name | `image.rs::the_primitive_names_itself_img`, `tests/ua_stylesheet_vocabulary.rs::the_image_primitive_answers_to_the_ua_img_rule` |
| a disabled select's options read muted, an enabled one's do not | `tests/ua_stylesheet_vocabulary.rs::a_disabled_selects_options_read_muted` |
| `th` is announced as a cell; a `textarea` as a text input | `semantic_builder.rs::the_whole_table_vocabulary_is_announced_as_a_table`, `::a_textarea_is_announced_as_a_text_input` |
| every selector names something the toolkit builds — and the guard bites | `tests/ua_stylesheet_vocabulary.rs::every_selector_in_the_ua_sheet_names_something_the_toolkit_builds`, `::the_guard_rejects_a_rule_for_a_name_nothing_builds`, `::the_markup_only_vocabulary_really_reaches_the_tree` |
| the facade compiles the theme package's own text, at `SystemDefault`, once | `default_stylesheet.rs::the_text_the_facade_compiles_is_the_theme_packages_own`, `::ua_stylesheet_is_at_the_system_default_origin` |
| the sheet is a self-contained resource, in the palette the docs name | `stylesheet/default_stylesheet.rs::the_sheet_is_a_self_contained_resource`, `::the_palette_the_docs_name_is_the_palette_the_text_uses` |

## The one consumer that had to migrate

The `.okui` showcase examples styled `<div type="card">` with a bare
`card { … }` type selector — which worked only because of the very collision
this campaign removed. Both were rewritten to `[type=card] { … }`, the rule was
proved load-bearing (rendering with a deliberately non-matching selector changes
the PNG digest), and the interpreted and codegen paths still render
byte-identical scenes. No `.okui` file and none of the workspace's `okui!`
invocations used any of the six newly-reserved tags — re-checked at close against
the pre-campaign tree, where every hit for `<input`, `<textarea`, `<checkbox`,
`<radio`, `<select` or `<table` is a prose comment or a wireframe caption string,
never markup — so no existing markup changed meaning behind anyone's back.

Anyone outside the workspace who styled `type="…"` with a bare type selector has
the same one-line migration, and it is called out in
`docs/api/09-authoring-formats.md`.
