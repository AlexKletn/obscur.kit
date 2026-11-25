Commits the composition and returns the text to insert: the preedit text
at commit time (spec §5.12 "commit").

Marks the state [`Committed`](CompositionState::Committed) and clears the
preedit. The buffer-side replacement over the [`anchor`](Self::anchor) and
its undo-unit handling live in the editing engine, not here. Committing an
already-finished composition returns an empty string and leaves the state
unchanged.
