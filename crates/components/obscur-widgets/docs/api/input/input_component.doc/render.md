Renders the field's body — [`Input::contents`], the same single `Text` node
[`Input::view`] installs under its own root, so a mounted field and a built
one are the same tree.

The empty case is not an empty render: an empty field still emits one empty
`Text` child, because that node is what the runtime's editing session seeds
from, what the caret and selection index into, and what the IME preedit is
spliced into. Rendering nothing would leave a focused field with no run to
put a caret in.

The context is unused: nothing here reads a signal or asks for a re-render. A
field re-renders when its host node's props change, which the component pass
already tracks — and *typing* does not change them, because the live text of
a focused field belongs to the editing session, not to this output.

[`Input::contents`]: super::Input
[`Input::view`]: super::Input::view
