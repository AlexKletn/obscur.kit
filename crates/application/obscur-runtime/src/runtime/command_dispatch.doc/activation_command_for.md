Resolves the **activation** command for `target`: a control is activated
by a click or an Enter/Space keypress, and activation fires its
[`EVENT_CLICK`] command if one is bound (bubbling up its ancestors), else —
for a **non-editable** control (`editable` is `false`) — its
[`EVENT_CHANGE`] command (spec §2.3, §3.1).

This is the single mapping both pointer and keyboard activation share: a
button (bound to `on-click`) fires its click command; a
checkbox/radio/select (bound to `on-change`, and *not* editable) fires its
change command *on the click* — for those controls the click **is** the
change (WHATWG HTML §4.10.5.5: a checkbox/radio `change` fires on the
toggling click; a `select`'s on the pick). Click binds are checked first,
so a node carrying both prefers its click command.

A **text field is different.** For an *editable* control — one whose
ancestor chain binds [`EVENT_INPUT`] — `EVENT_CHANGE` is **not** a
pointer-click activation: per the HTML model a text field's `change` fires
once on **commit** (blur, or `Enter` for a single-line `input`) carrying the
committed value, and must *not* fire on a focus/click. So the change
fallback is deliberately skipped for an editable target here — a click into
a text field resolves no activation command — and the commit-change is fired
separately (driven by the editing session's blur and single-line `Enter`
paths). Per-keystroke [`EVENT_INPUT`] entry is likewise not activation.
Returns `None` when neither event applies.

[`EVENT_INPUT`]: obscur_view::view::EVENT_INPUT

Returns the pair `(current_target, command)`: the node whose binding
resolved — §2.3's `current_target` — and the command it bound. See
[`binding_for`](Self::binding_for).
