Derives the semantic view: a [`COMPONENT_ID`] outer element — the box and
the click/`:disabled` target — laid out as a **vertical column** of a
`.select-header` row and, while open, a `.select-list` popup.

The **header** (`.select-header`) is a row of the **current label** (the
selected option, else the first option, else empty — the browser-faithful
closed display) followed by a trailing `.select-arrow` indicator sub-part.
The arrow carries an `open` state class (so a `select-arrow.open { … }` or
`select:disabled .select-arrow { … }` rule can key on it) and shows the
disclosure glyph as its text child — "▼" closed, "▲" open.

When [`is_open`](Self::is_open), a `.select-list` of `.select-option` rows
(one per option, the selected one also carrying a `selected` class) is
appended after the header, so the popup renders as a vertical stack directly
below it. The list is emitted **in normal flow** (not absolutely
positioned): the toolkit's layout/render of a positioned popup subtree is
unreliable here, so an in-flow list is the faithful, deterministic choice —
it grows the control's box downward rather than overlaying following content.
The click event and reserved `:disabled` attr stay on the OUTER element, so a
click anywhere in the control (header or a row) bubbles to the one handler.

[`COMPONENT_ID`]: Self::COMPONENT_ID
