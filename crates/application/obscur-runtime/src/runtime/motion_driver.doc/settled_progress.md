Where each finished timeline settled, so the overlay keeps applying the
value the animation ended on — which for a reversed final iteration is
progress `0`, not `1` (css-animations-1 §3.9 `forwards`).

Reachable only through a live binding's timeline, so
[`retain_live`](Self::retain_live) keeps it to exactly the bindings that
survive.
