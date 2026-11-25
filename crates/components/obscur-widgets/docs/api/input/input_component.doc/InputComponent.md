The shipped text-entry control as a mounted component.

It is a unit type, and that is a claim rather than an omission — the same
claim the checkbox makes, and a more surprising one here, because a text
field obviously *does* have private state: a caret, a selection, a scroll
offset, an IME preedit. **None of it lives in the component.** The runtime
owns the editing session and keys it to the retained host node, so the caret
survives a re-render precisely because it was never the component's to lose.
What this component contributes is one frame's worth of output, and that is a
pure function of the props.

It renders the control's **children**, not the control: the runtime
reconciles a component's output *into* its host node, and the host is
already the `<input>` / `<textarea>` element — authored, or built by
[`Input::view`]. Returning the whole widget view would nest a second field
inside the first, and the inner one would carry the local name the editing
session reads.

[`Input::view`]: super::Input::view
