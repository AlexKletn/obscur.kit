Whether `node` lies within a **readonly** control — the node itself or any
of its element ancestors carries the reserved `readonly` structural state
(WHATWG HTML §attr-input-readonly).

**Distinct from [`is_within_disabled`](Self::is_within_disabled).** A
readonly control is still *focusable* and its text still *selectable /
copyable* — a press seats focus, a caret, and drag-selection exactly as on a
normal field. Only its **value is immutable**: the text-mutating edits
(insert / delete / backspace, IME commit, paste, a multiline `Enter`
newline) are dropped. So — unlike the disabled gate, which blocks
focus/caret/editing wholesale — this gates **only** the mutation path
(`UiRuntime::apply_edit`'s mutating commands and
[`UiRuntime::handle_ime`](super::UiRuntime::handle_ime)), never focus /
caret / selection.
