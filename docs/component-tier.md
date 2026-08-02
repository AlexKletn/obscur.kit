# The component tier gets citizens

`docs/api/12-reality.md` recorded the fault three times, in three rows, and it
was one fault seen from three sides:

- **Base-element control tags** — `<input>`, `<textarea>`, `<checkbox>`,
  `<radio>`, `<select>`, `<table>` bound to the shipped widget's `ComponentId`,
  but *"nothing registers a factory for the built-in widget ids"*, so the
  widget's own `view()` — its indicator sub-part, its `:checked` class — never
  ran. The id was a handle with nothing mounted under it.
- **Slots** — *"No shipped component declares a slot: no shipped widget is a
  `Component`."*
- **Components** — `register_component` mounts a factory, and nothing shipped
  one.

The tier compiled. It was spec-tested through §2.11–§2.13. It had **no
citizen**: every `impl Component for` in the workspace sat either inside a
`#[cfg(test)]` module (twelve) or inside `obscur-testkit`, the harness crate
only a test target depends on (three). A registry with no registrations is a
type, not a mechanism.

**Status: closed, five of six.** `Checkbox`, `Radio`, `Select`, `Input`
(with `<textarea>`, which shares its id) and `Button` ship a full component
trio and are registered by default. `<table>` is a **reasoned refusal**, and
so are the six PascalCase-only widgets. The three rows are rewritten with
their old claims kept visible beside the corrections, the way the
`obscur-testkit` row on that page already does it.

## The numbers

Measured on the tree, not copied from a stage report.

| | |
|---|---|
| Suite | **6,360 passed / 0 failed** (6,261 when the campaign opened — **+99**, none lost) |
| `clippy --workspace --all-targets` | **360 warnings**, unchanged across all five stages, and below the 391 the decomposition campaign handed over |
| `cargo fmt --all --check` | clean |
| `xtask check` (§3.16 architecture) | passes |
| `xtask consumer` (out-of-workspace facade consumer) | passes |
| Production `impl Component for` | **5** — `ButtonComponent`, `CheckboxComponent`, `InputComponent`, `RadioComponent`, `SelectComponent`, each with a `ComponentFactory` beside it |
| `BuiltinComponents::registrations()` | **5**, installed by default |
| New source files | 16 — three per widget (`…_props.rs`, `…_component.rs`, `…_factory.rs`) plus `bootstrap/builtin_components.rs` |
| Largest new file | `select_props.rs`, **98 lines of non-test code**. Every new file is an order of magnitude under the 1,000-line bar |
| Tests in the new files | **88** (29 of them the facade integration target `tests/builtin_components.rs`) |

## What was built

### The trio, five times

A builder becomes a component by growing three things beside it, one entity per
file, in the folder §3.3–§3.15 already put the builder in:

- **`…Props`** — the §2.13 contract and `FromProps`, hand-written (see the
  refusal below), pinned field-by-field against `#[derive(Contract)]`'s own
  output.
- **`…Component`** — `impl Component`, whose `render` is the existing `view()`
  body reading those props.
- **`…Factory`** — `impl ComponentFactory`, the thing a registration holds.

Each widget's `view()` was split so its children come from one `contents()` body
the component's `render` calls too. That is the whole of what makes mounting
idempotent on the Rust API — a builder's node carries the same `ComponentId` as
the authored tag, so registering a factory mounts a component on *the builder's
own output*. Without a shared body, the day the built-ins were registered every
existing program's checkboxes would have lost their labels.

### `KitBootstrap` registers them, by default

`install_builtin_components` sits beside `install_default_stylesheet`, and
`App::run` and the layer-shell runner both call it. A host opts out with
`without_builtin_components()` and can read the state back with
`registers_builtin_components()`. Registering by default **changes what
existing markup renders** — that is the point of the campaign, and it is pinned
from both sides: every mount test has a `runtime(false)` half showing the same
markup rendering nothing.

### §2.12 gained the default slot, because `<button>` needed it

`<button>Save</button>` puts "Save" in the node's *children*; `Button::view()`
renders a `label`. A component renders its host's children, so mounting Button
naïvely renders an empty button. Two domain mechanisms had to exist first:

- **A default slot.** `assigned_to` returned `false` for every `ViewNode::Text`,
  and a text run carries no properties, so the author's most ordinary button
  content could never name a slot however it was written. An outlet naming no
  slot now takes the content naming none — text runs included — under the
  reserved `DEFAULT_SLOT` symbol.
- **Outlet fallback content.** An outlet nothing fills renders its own children,
  which is how `<button label="Save"/>` and `<button>Save</button>` are one
  component rendering one position.
- **A second reconciler retention gate.** `apply_slot_content` kept a host's
  children only when one of them carried `slot="…"`, which default-slot content
  never does. `may_project_content(component)` is the second gate: true for the
  widget range (`0x00D0_xxxx`) and the top-bit user-component range. Every `div`
  in a document is a primitive below both and still pays nothing. The honest
  cost: an id that mounts nothing — `<table>` — now pays one child-list clone
  that nothing reads.

`ButtonComponent::render` is therefore not content at all. It returns exactly
one node: the default-slot outlet, carrying `Button::contents(&label)` as its
fallback.

### A defect that only became reachable because the campaign reached it

`<input value="abc" placeholder="Search"/>` painted **both** — measured, not
theorised: the §4.16 synthetic part present, painted runs `["Search", "abc"]`.
The `:placeholder-shown` gate asked whether the reserved `VALUE` attribute was
empty, and authored markup deliberately never writes that attribute — it
declares a *controlled* value the runtime snaps a drifting buffer back to, and
one symbol for both spellings would make every authored field uneditable. With
no such attribute the gate read a filled field as empty.

`resolve_placeholder` now reads the declared value when the application states
one, and otherwise the text the field renders — the mounted control's editable
run, which is the only place an authored field states its value.

Worth recording separately, because it corrects the campaign's own story: the
authored placeholder reaches the synthetic part now, and **mounting is not what
paid for it**. `resolve_placeholder` keys off attributes on the *host* node, and
a component's render output can never produce a host attribute. What had been
missing was `name_binding::attr_symbol("placeholder")` mapping to the reserved
symbol instead of hashing the name. That row is a front-end attribute-mapping
row wearing a component-tier costume.

## What was deliberately refused

A refusal is a result. Each of these is written down so the next reader does not
re-litigate it.

**`<table>` mounts nothing.** A table's rows and cells are `tr`/`td`/`th` the
author writes or `Table::view()` generates from data. A component there could
only render the outlet that hands them straight back — a mount that changes
nothing. Stated as an assertion (`the_table_id_is_left_unclaimed`) so that
adding a `TableFactory` is a deliberate act with an answer to *"whose content
are the rows?"* behind it.

**`Menu`, `Dialog`, `Tabs`, `List`, `Tree`, `Docking` were not touched.** None
has a lowercase tag and none is styled by type in the UA sheet, so a component
for them would be surface with nothing asking for it.

**A shipped widget cannot use `#[derive(Contract)]`.** The derive expands to
`::obscur_kit::…` paths and crate-arch §3.16 forbids an internal crate from
depending on the facade. Each `…Props::contract()` is written out by hand in
`obscur-widgets` and pinned against the derive's own output in
`obscur-kit/tests/builtin_components.rs` — the only target that can name both
sides. The derive could not have expressed the decoding anyway: a reserved
boolean attribute is set by *presence* (`<checkbox checked/>` lowers to `Null`,
which `:checked` reads as set and `decode_optional::<bool>` reads as absent), a
checkbox has a third mixed state no boolean carries, and a `<select>`'s options
are a *list*, for which `ViewValue` has no variant.

**Prop-mapping for `<button>` was rejected with the code in front of us.**
"A lone text child becomes the `label` prop" cannot express
`<button><span>press</span></button>` — which is the `xtask consumer` fixture's
own markup and is blessed by `okui_child_contracts`. That spelling would have
had to start either vanishing at runtime or reporting at compile time.
Projection handles text and elements by one rule.

**`<select>`'s options are a prop, not projected children.** `with_options`
takes labels (`Into<Box<str>>`), not views, and the control synthesises each
`.select-option` row around one with a per-index `ComponentId` and a computed
`selected` class. Children would have handed the author a slot they could put an
`<img>` in and then required a lossy view→string reduction. Because `ViewValue`
has no list variant, options travel as a delimited string in two spellings —
readable comma-separated `options="…"` when authored, U+001F-joined when
`Select::view` emits it — which is what keeps `with_options(["a,b"])` rendering
one row rather than two.

**`spec_ch2_contracts`'s weakened assertions were not restored — at the time.**
`a_default_slot_and_a_named_slot_partition_the_children` had its default-side
assertions deliberately removed on 2026-07-19, and its `.doc` asked for exactly
the spec amendment this campaign's mechanism answers. Restoring it while §2.12
still listed named slots only would have been a spec case asserting an
invention, so the campaign closed with a dated note instead.

> **CLOSED 2026-08-02.** §2.12 now states the default slot, `#[derive(Contract)]`
> gained `slot(default)`, `PanelProps` declares what it renders, and the case is
> restored to its full form. `ButtonProps`'s hand-written contract is compared to
> the derive's output **whole** as a result — the exception this campaign carved
> for it is gone.

**The `.okui` front end still refuses `<slot/>` and children on an outlet.**
The runtime has the mechanism; there is no authored spelling for the default
slot or for fallback content, and `DEFAULT_SLOT` is a reserved name no author
can write. Both diagnostics were rewritten to say *that* instead of saying the
feature does not exist. No test was weakened.

**The `:placeholder-shown` gate does not consult the live editing session.**
Typing into an authored field with a no-op handler leaves the retained tree's
text at `""` — the character lives in the editing session and is not painted
either. That is the pre-existing app-driven value model: the app re-renders from
its own model on `on:input`, exactly as `Input::view` does. Gating on rendered
text keeps hint and field agreeing in every case. A session-aware gate would
need `resolve_generated_content` to take the session, which it does not and
should not.

**`Select::contents` takes `open` and not `disabled`.** Writing
`if open && !disabled { list }` in the component would have left the UA rule
`select:disabled .select-option` with nothing to match, silently, because
`Select::open` refuses on a disabled control — a rule about a *transition*
leaking into the render of a *state*. Written into `select.doc/contents.md` so
the next reader does not "fix" it, and pinned twice.

## Behaviour changes, each with the test that pins it

| Change | Pinned by |
|---|---|
| `<checkbox label="…" checked/>` renders the `.checkbox-box` indicator plus the label; the UA `.checkbox-box.checked` fill reaches it and `:checked` still matches the host | `an_authored_checkbox_renders_the_control_not_an_empty_element`, `the_ua_sheet_fills_the_mounted_indicator_and_checked_still_reaches_the_cascade`, and the negative half `without_the_registration_the_same_markup_is_still_an_empty_element` |
| `<radio label="…" checked/>` renders `.radio-dot` / `.radio-fill` | `an_authored_radio_renders_the_dot_and_its_checked_state_reaches_the_cascade` |
| `<select options="…" selected={n} open/>` renders `.select-header`, `.select-arrow`, `.select-list` and its rows, node-for-node identical to the built control | `an_authored_select_and_a_built_one_render_the_same_tree` |
| A disabled open `<select>`'s rows still reach `select:disabled .select-option` | `a_disabled_selects_option_rows_still_read_muted_once_the_control_is_mounted` |
| `<input value="abc"/>` renders its editable run, where an authored value was invisible before | `an_authored_input_renders_its_value_and_without_the_registration_does_not` |
| `<textarea>` stays multiline and `<input>` single-line after mounting (one registration, two tags) | `mounting_leaves_a_textarea_multiline_and_an_input_single_line` — by *typing* `a`, Enter, `b` through the mounted control |
| **`<button>Save</button>` renders "Save"** — the campaign's acceptance test | `an_authored_buttons_children_still_render` |
| `<button><span>press</span></button>` keeps its span as the button's own child | `an_authored_buttons_element_children_still_render_too` |
| `<button label="Save"/>` renders "Save" — the label the `.okui` compiler already required and nothing read | `the_label_prop_spelling_renders_the_same_button` |
| `<button label="X">Y</button>` renders Y, not X: the fallback belongs to the empty slot only | `authored_content_beats_the_label_prop_rather_than_displacing_it` |
| Every Rust-API entry point renders the same tree with and without the registration | `a_built_{checkbox,radio,select,input,button}_renders_the_same_tree_whether_or_not_the_built_ins_are_registered` — five tests over 4/2/3/7/3 built states |
| `<checkbox/>` with no label reports `PropsError::Missing` and renders nothing under the node, instead of a silently blank control | `a_labelless_authored_checkbox_reports_rather_than_mounting_a_blank_control` |
| `Checkbox::view()` and `Radio::view()` also emit `label` as a prop; `Select::view()` emits `PROP_OPTIONS`, and only when the control has options | the idempotence tests above, plus `the_widgets_own_list_round_trips_a_label_containing_a_comma` |
| A radio's authored selection is the reserved `checked`, never `selected`; `<Radio selected/>` is now an `UnknownAttribute` diagnostic | `radio_spells_its_selection_the_way_the_node_does` |
| `<textarea>Hello</textarea>` and `<select><option/></select>` are compile-time reports naming the `value="…"` / `options="…"` migration | `mounted_element_contracts`'s tests in `builtin_contracts.rs` |
| An outlet nothing fills renders its own children as fallback | `an_outlet_nothing_fills_renders_its_fallback` |
| An outlet naming no slot receives all content naming none, text runs included; named outlets unaffected | `an_unassigned_child_reaches_no_outlet` (assertion untouched, comment rewritten to state the real rule) |
| The reconciler retains authored children for any id a component can mount under | `only_a_mountable_id_retains_its_content` in `slot.rs`, and `the_default_outlet_takes_the_content_that_names_no_slot` beside it |
| `<input value="abc" placeholder="Search"/>` no longer paints the hint on top of its own text; `value=""` still shows it | `an_authored_value_hides_the_placeholder_the_way_a_declared_one_does` — asserts all three of no synthetic part, hint unpainted, value still painted |
| `KitBootstrap` registers the built-ins by default, and a host can see and refuse it | `a_host_can_see_and_refuse_the_default_registration` |
| `<table>`'s id is left unclaimed | `the_table_id_is_left_unclaimed` |
| `Button::view()` with an empty label emits no text child instead of an empty one | `a_built_button_renders_the_same_tree_whether_or_not_the_built_ins_are_registered`, whose third state is `Button::new("", cmd)` |

New public API: `ButtonProps`/`ButtonComponent`/`ButtonFactory` and the same
three for `Checkbox`, `Radio`, `Select`, `Input`; `BuiltinComponents`;
`KitBootstrap::{install_builtin_components, without_builtin_components,
registers_builtin_components}`; `obscur-view`'s `DEFAULT_SLOT` and
`may_project_content`. No existing signature changed.

## Where it still stops

- **`<table>`** — the refusal above. A decision, not a gap.
- ~~**An outlet's name is checked against nothing.**~~ **CLOSED 2026-08-02.**
  `SlotMismatch::find` compares both sides against the contract and
  `ComponentRenderPass` reports `view.undeclared_slot`. Closing it surfaced a
  latent defect it had been hiding: the derive hashed slot names with a bare
  FNV, high bit clear, which is *not* what `InternedSymbol::named` produces —
  invisible for as long as nothing compared a contract's slot symbol to
  anything.
- ~~**§2.12 does not state the default slot.**~~ **CLOSED 2026-08-02.** Stated,
  spelled `slot(default)` on a derived contract, and the spec case restored.
- **No authored spelling for the default slot or for fallback content.** The
  `.okui` front end refuses both, now with diagnostics that say why.
- **An unregistered or misspelled component tag mounts nothing with no
  diagnostic**, and the `okui!` macro cannot ever check it: the registry is a
  runtime value of trait objects, and the macro runs while that crate compiles.

## Verification method

Everything above was re-measured at close, and the mount claims were checked
against the real rendered tree rather than against the tests that assert them —
a throwaway probe rendered all five authored tags and all five Rust-API entry
points twice, with and without the registration, and dumped the retained tree
and the paint commands. The two halves were identical for every Rust-API view
and, for the authored tags, differed exactly by the sub-parts the campaign set
out to make reachable. The probe was deleted; the assertions it confirmed live
in `crates/facade/obscur-kit/tests/builtin_components.rs`.
