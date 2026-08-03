What unit a wheel delta is in.

Platforms report a wheel two different ways and the difference is real: a
notched mouse wheel reports whole LINES, and a trackpad reports PIXELS it has
already smoothed. Collapsing the two at the adapter — which is what this
toolkit used to do, multiplying lines by a constant inside the winit runner —
loses the one fact a handler might want to treat differently, and buries a
scrolling policy in a platform adapter where nobody would look for it.
