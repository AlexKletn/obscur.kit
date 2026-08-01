§2.3's event model, reached the way an application reaches it.

The model §2.3 specifies — four propagation phases, and an event API carrying
`target`, `current_target`, `stop_propagation`, `stop_immediate_propagation`,
`prevent_default` and a trusted/untrusted origin — was fully implemented in
`obscur-interaction` and **called by nothing**: outside its own crate,
`UiEvent` and `EventDispatcher` appeared in exactly two places, this facade's
prelude re-export and the §2.3 spec test. An application's handler was an
`impl FnMut()` given nothing, so none of it was reachable however correct it
was.

These cases are written from the *consumer's* side — only `obscur_kit`'s public
API, no crate internals — because the gap was a delivery gap, and a delivery
gap is only closed if it is closed at the boundary an application stands on.
They deliberately overlap the runtime's own unit tests: those prove the walk is
correct, these prove it is *reachable*.

Not a `spec_*` file: the spec-derived suites are an oracle written without
reading this repository, and these cases were written against a change to it.
The §2.3 conformance statements live in `spec_ch2_interaction.rs` and are not
touched here.
