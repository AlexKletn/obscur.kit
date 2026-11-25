The **life** of the runtime's focused editing session (spec §5.11): where a
session comes from, what re-seats it, when it commits, and the readings a host
and a frame take off it.

Nothing here applies an edit. A session is created by focus landing on an
editable or by an app seating a value explicitly, re-seated when a controlled
frame brings a different value back, and torn down when focus leaves — and each
of those is a decision about *which* buffer is live, not about what is in it.
The keystroke path, the IME path and the pointer path each change a live buffer,
and each lives in its own file; all three end in `after_edit`, which lives in a
fourth.

Two things in that life are easy to get wrong, so they are stated once here
rather than at each caller:

- **A blur is a commit.** WHATWG HTML §4.10.5.5 fires `change` when a text field
  loses focus after its value changed, so `sync_editing_to_focus` commits the
  *outgoing* field before it re-seats the session. Re-focusing the same field is
  not a blur, which is why the equality test comes first and returns without
  touching the session: a caret, a selection and a dirty baseline all survive a
  focus event that did not actually move across an editable.
- **A controlled field's value belongs to the app.**
  [`ControlledReseed::decide`](crate::runtime::ControlledReseed::decide) owns the
  three-way discriminator — leave the buffer alone, the session's own edit came
  back, the app rewrote this field — and `reseed_editing_from_retained` is the
  write half that seats its answer and re-derives what a changed buffer
  invalidates.

`editing_view` is the frame's reading rather than a host's, and it is the reason
the measure pass and the compose pass cannot disagree about a focused field: both
consume the same borrow of the same live session, so the box is sized from the
string that is painted into it.
