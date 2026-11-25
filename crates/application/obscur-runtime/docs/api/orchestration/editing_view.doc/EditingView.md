A borrowed snapshot of the runtime's active text-editing session, threaded
into [`SceneCoordinator::compose`](super::SceneCoordinator::compose) so the
focused editable paints from the **live** session state (spec §5.11, §6.7).

Step 1 gave the runtime an editing session (owning the focused field's text,
caret and selection); this is the read-only view the compose phase consumes to
(a) render the editing field's text from the session `text` — so a typed
character is visible immediately, even before the retained
[`NodeKind::Text`](obscur_tree::node::NodeKind::Text) content catches up — and
(b) paint the caret line and selection highlight from `selection`.

`node` is the **editable field** the session is bound to (the ancestor that
binds [`EVENT_INPUT`](obscur_view::view::EVENT_INPUT)), not necessarily the
`Text` leaf that carries the glyphs: the compose walk treats the field's `Text`
descendant as the editing text. `text` borrows the session buffer's current
string and `selection` is its caret/selection (a collapsed selection is a
plain caret). The runtime builds this from `self.editing` before each compose
and passes `None` when no session is active (the byte-identical, no-caret
path).
