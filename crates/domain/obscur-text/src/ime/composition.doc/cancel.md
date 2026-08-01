Cancels the composition (spec §5.12 "cancel").

Marks the state [`Cancelled`](CompositionState::Cancelled) and clears the
preedit; the anchored range is left untouched by the caller. Ignored when
the composition is no longer composing.
