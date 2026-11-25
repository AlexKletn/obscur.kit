Applying an [`EditCommand`](obscur_text::editing::EditCommand) to the focused
session, and everything that is owed once one has been applied (spec §5.11).

`apply_edit` is the **one choke point** every text mutation in the runtime flows
through — a typed insert, a backspace, a paste, an IME commit — and that is what
makes the readonly rule enforceable at all: WHATWG HTML §attr-input-readonly says
a readonly field is focusable and selectable but not editable, so a *mutating*
command is dropped here while `Move` / `SetSelection` / `SelectAll` pass through
unchanged. Written at each entry point instead, the rule would have as many
implementations as there are ways to type.

`after_edit` is the tail every path shares, and its three duties are the reason
the file exists rather than the edits being scattered:

1. **`input` fires from the buffer, not from the keystroke.** The text handed to
   the node's [`EVENT_INPUT`](obscur_view::view::EVENT_INPUT) command is read
   back out of the session *after* the command applied, so an app sees what the
   field now contains rather than what the user pressed.
2. **The caret stays visible.** Both axes are re-derived — see
   `caret_visibility.rs` — because an edit is exactly when the caret can leave
   the content box.
3. **The frame is asked for once.** A dispatched `on-input` has already asked for
   a full frame, since a controlled field needs a reconcile to mirror or reseed;
   only when nothing was dispatched does this ask, and then it asks for the
   *cheapest* frame the edit permits.

That last classification is `edit_is_compose_only`, and it is the difference
between a keystroke that re-lays out the document and one that only re-paints:
a caret or selection move cannot move a box at all, and neither can a mutation
in a field whose box does not depend on its text.

`try_text_edit` sits above them as the keyboard's adapter: it asks
[`KeyEditResolver`](crate::runtime::KeyEditResolver) what a key *means* for this
session and then does it. The resolver returns the meaning and never the effect,
which is why a pre-resolved grapheme cluster arrives here as a selection to set
before the delete rather than as a delete of its own.
