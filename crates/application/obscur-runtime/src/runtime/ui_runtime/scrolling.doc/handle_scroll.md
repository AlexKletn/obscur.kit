Routes a wheel/scroll gesture of `delta_y` logical px at host `position` to
the **innermost** scroll container under the pointer, scrolling it (clamped)
and requesting an [`Input`](FrameReason::Input) frame. Returns the scrolled
container's [`NodeId`], or [`None`] when no scroll container lies under the
pointer (CSS Overflow 3 §scroll-container).

`position` is in the host's physical (device) space; it is mapped to content
space through the same `physical / device_scale + content_offset` inverse
[`handle_pointer`](Self::handle_pointer) uses, then through each crossed
container's own `scroll_top`, so the hit-tested container is the one the
pointer visually sits over while scrolled.

# The notch is an event first

[`EVENT_WHEEL`](obscur_view::view::EVENT_WHEEL) — `on:wheel="…"` — is dispatched
to the node **under the point** through §2.3's capture / target / bubble before
any container moves, carrying an
[`EventPayload::Wheel`](obscur_interaction::event::EventPayload::Wheel) a handler
reads `delta_y` off. A listener that calls `prevent_default` suppresses the
scroll entirely: nothing moves, and no frame is requested for it.

That refusal is a **claim**, and it is why the return value is not simply "which
container scrolled". A refused wheel returns the node whose binding refused it,
so a host reading `None` as "nobody wanted this" — which is exactly what
`App::on_scroll` is for — does not scroll its own list out from under a listener
that just said no. The three answers are: `Some(container)` scrolled it,
`Some(node)` refused it, `None` nobody took it.

A wheel is dispatched only for a wheel. A fling in flight and a finger driving a
pan both move containers through their recognizers and neither goes through
here, so neither is refusable this way.
