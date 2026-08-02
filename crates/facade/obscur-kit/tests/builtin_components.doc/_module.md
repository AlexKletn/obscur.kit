The shipped widgets that are real components, end to end.

Three claims per control, and they are the three rows
`docs/api/12-reality.md` records against the component tier:

1. **The ids line up.** Each props struct's `contract()` is hand-written in
   `obscur-widgets`, because a components-tier crate may not name the facade
   and so cannot use `#[derive(Contract)]`. This file is the one place that
   can hold both, so it derives the same contract from the same name and
   compares the two values whole.
2. **A factory is registered.** `<checkbox label="…" checked/>`,
   `<radio label="…" checked/>` and `<select options="…" open/>` each mount
   the real control: the sub-part the UA sheet fills appears in the painted
   tree, and `:checked` / `:disabled` still reach the cascade from the
   authored node.
3. **Mounting is idempotent on the Rust API.** `Checkbox::new(…).view()` and
   its siblings build nodes carrying the same ids, so they mount too — and
   must render the same tree they did before anything was registered.

Four controls are here, and they are the four that **own their content**: a
checkbox's indicator, a radio's dot, a select's header, arrow, list and rows,
and a field's editable run are the control's, not the author's. None of them
needs §2.12 slot machinery, which is why they could come first.

`Select` is the one that had a decision to make rather than a shape to
follow — its options could have been a prop or projected children — and
`an_authored_select_and_a_built_one_render_the_same_tree` is where the choice
is answerable: whichever way the options are stated, the two front ends must
produce one tree.

`Input` is the one whose *host* does most of the work. Two tags share its id,
the editing session and caret are the runtime's rather than the component's,
and its placeholder is not in its output at all — so the tests that matter
for it are the ones that check what mounting did **not** disturb.

