The one image that has been acquired and not yet taken.

A slot rather than a channel, and the difference is the whole point: an image
in a channel can only be received by the loop that is waiting for it, while a
slot can be **reclaimed by the acquire thread** — which is what has to happen
before the surface is reconfigured under it.
