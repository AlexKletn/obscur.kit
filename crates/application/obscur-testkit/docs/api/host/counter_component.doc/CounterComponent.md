A component whose whole observable behaviour is one number.

It decodes a `step` prop, adds it to its own [`LocalState`], and emits the
running total as its single text child. That one design choice is what makes it
useful as a fixture: the rendered text answers three different questions at
once — whether the props decoded, whether `render` ran *this* frame, and whether
the instance (and therefore its local state) survived reconciliation. A test
that expects "2" and reads "4" has learned that the component re-rendered when
it should not have.

Two knobs on [`registration`](Self::registration) cover the reactive cases. A
non-zero `self_updates` budget makes the component call
`ComponentContext::request_update` from inside its own render exactly that many
times and then settle — the loop that has to schedule a *following* frame rather
than spin inside this one. A `tracks` source makes it call
`ComponentContext::track`, so a host's `notify_signal` can invalidate it; passing
`None` models a component that reads no app state at all and must therefore be
untouched however much state changes.
