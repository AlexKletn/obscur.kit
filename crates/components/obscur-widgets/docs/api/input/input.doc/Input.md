A headless text-input control (spec §2.6).

`Input` owns a [`TextBuffer`] — the pure editing engine of spec §5.11 — as
its text state, plus a `placeholder`, the `command` fired on change, and a
disabled flag. Text edits flow in as [`EditCommand`]s through
[`apply`](Self::apply), which delegates to the buffer; this keeps the input
layout-free and lets a platform keymap / IME drive it with the same command
vocabulary the engine already understands.

It bakes in **no** visual theme: [`view`](Self::view) derives a
[`SemanticRole::TextInput`] element carrying the current value and
placeholder as text properties and a change event bound to `command`,
rendered by the style layer keyed on [`COMPONENT_ID`](Self::COMPONENT_ID).

Because [`TextBuffer`] holds a `String` and undo history it is neither
`Copy` nor `Eq`, so `Input` derives only `Debug`, `Clone`, and `PartialEq`.
