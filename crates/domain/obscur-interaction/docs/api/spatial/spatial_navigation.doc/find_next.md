Picks the best candidate to focus moving `direction` from `from`.

Only candidates lying in the requested direction are considered: the
candidate's center must be strictly beyond `from`'s center along the
direction's axis, in the direction's sign (e.g. for
[`Right`](NavigationDirection::Right), `candidate.center.x >
from.center.x`).

Each remaining candidate is scored `along + 2.0 * across`, where `along`
is the absolute center distance along the primary (direction) axis and
`across` is the absolute center distance on the cross axis. Weighting the
cross-axis term favours well-aligned neighbours. The candidate with the
smallest score wins; ties are broken by ascending [`NodeId`].

Returns `None` if no candidate lies in `direction`.
