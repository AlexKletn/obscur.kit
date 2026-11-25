The state a layer surface is in, as a host can actually observe it.

Deliberately smaller than the full protocol dance. A surface being *created*
is not observable from outside — the call that creates one either returns a
window or an error — and *failed* is carried by that error rather than by a
state a caller could poll. What remains is what a running application can
meaningfully branch on.
