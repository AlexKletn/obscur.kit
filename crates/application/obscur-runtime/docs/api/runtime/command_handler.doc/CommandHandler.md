A registered command handler, after normalization: one boxed closure that
takes the propagating [`UiEvent`] (spec §2.3).

# Why there is exactly one stored shape

An application may register a handler in two shapes — `FnMut()` when it needs
nothing from the event, `FnMut(&mut UiEvent)` when it does — and
[`IntoCommandHandler`](super::IntoCommandHandler) converts both into this one.
The runtime's handler tables therefore stay homogeneous
(`HashMap<_, CommandHandler>`), and the dispatch walk has one call shape to
honour rather than a match over arities at every one of §2.6's five links. The
zero-argument form is wrapped in a closure that ignores the event, so it costs
nothing at dispatch time and cannot observe the difference.

# Why `&mut UiEvent` and not `&UiEvent`

Because §2.3's API is *mutating*: `stop_propagation`,
`stop_immediate_propagation` and `prevent_default` all change the event's
state, and they are the entire point of handing the event to a handler. A
`&UiEvent` handler could read the payload but could not steer anything, so the
three flags §2.3 names would be exported and still unreachable — the exact
shape of the gap this type closes. The alternative (interior mutability behind
`&UiEvent`, as the DOM does) would buy nothing here: a handler runs alone,
under `&mut self` on the runtime, so there is no aliasing to work around.
