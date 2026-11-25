Starts a composition anchored at `anchor` (spec §5.12 "start").

The session begins [`Composing`](CompositionState::Composing) with an
empty preedit. Anchoring at the primary caret's range here also
establishes the one-undo-unit scope the editing engine will use.
