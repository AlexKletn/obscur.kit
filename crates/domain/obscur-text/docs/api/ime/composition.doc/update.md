Replaces the preedit with `preedit`, keeping the state
[`Composing`](CompositionState::Composing) (spec §5.12 "update").

Ignored (a no-op) when the composition is no longer composing, so a stray
update after commit/cancel cannot resurrect a finished session.
