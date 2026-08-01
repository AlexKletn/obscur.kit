Commits the active editing session's `change` event when its value is
dirty (differs from the last committed baseline / the value at focus) — the
shared entry point for the two HTML commit triggers on an editable text
field: **blur** (focus leaving the field, from
[`sync_editing_to_focus`](Self::sync_editing_to_focus)) and a single-line
**`Enter`** (from [`try_text_edit`](Self::try_text_edit)).

Fires the editable node's [`EVENT_CHANGE`](obscur_view::view::EVENT_CHANGE)
command through the [`change_handlers`](Self::change_handlers) registry with
the committed text (WHATWG HTML §4.10.5.5: `change` fires on commit, once,
and only when the value actually changed). When `rebaseline` is set (an
`Enter` commit that keeps the field focused) the session's baseline is
advanced to the committed value first, so a later blur without further edits
does not re-fire. A no session, a clean value, or no `EVENT_CHANGE` binding
makes this a harmless no-op. Returns whether a change handler ran.
