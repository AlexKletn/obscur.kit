Decodes a `<button>` node's properties.

There is one, it is optional, and its absence is the ordinary case: a button
authored as `<button>Save</button>` or built by
[`Button::new`](super::Button::new) states its text as content, and only the
property spelling `<button label="Save"/>` — the one the `.okui` compiler
requires of `<Button/>` — puts it here. An absent label is therefore an empty
one and never an error; the component renders the author's content in its
place.
