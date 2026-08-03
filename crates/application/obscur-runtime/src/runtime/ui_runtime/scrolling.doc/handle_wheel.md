Routes a wheel notch: the refusable `wheel` event first, then the scroll.

Returns the node that took it, or `None` when nothing did — which is what tells
a host its own scroll fallback should run. A node that REFUSED the event is
returned as having taken it, because a refusal is a claim.

The door every wheel comes through.
[`handle_scroll`](Self::handle_scroll) is this with an untrusted, unstamped,
pixel-mode notch built for it — kept because it is the shape every existing
caller uses, and it loses nothing a caller passing two floats ever had.
