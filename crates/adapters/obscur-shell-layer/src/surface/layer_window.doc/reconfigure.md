Re-applies `config` to the live surface — a panel changing its height, its
anchor, or how much space it reserves.

The compositor answers with a fresh configure; the host should treat the
resulting [`LayerEvent::Configured`](crate::event::LayerEvent::Configured)
as the new size rather than assuming the request was honoured verbatim.
