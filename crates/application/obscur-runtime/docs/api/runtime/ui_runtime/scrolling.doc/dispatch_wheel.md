Dispatches [`EVENT_WHEEL`](obscur_view::view::EVENT_WHEEL) to the node under
`content`, and returns the node that **refused** the scroll — or [`None`] when
nothing refused it, including when nothing was listening at all.

`Some` is therefore "this notch is claimed and no container should move", which
is what lets [`handle_scroll`](UiRuntime::handle_scroll) return a taker for a
refused wheel: a listener that declines the scroll has taken the event, and a
host fallback that ran on `None` would otherwise scroll its own list instead.

Gated on the tree binding the name at all — see
[`PointerEventBindings`](crate::runtime::PointerEventBindings) — so the hit test
and the ancestor walk below it cost nothing for the application that binds
nothing, which is nearly all of them.
