Arms a speculative pan for `pointer` over `container`.

Arming is cheap and speculative: the recognizer starts `Possible` and stays
there unless the finger travels past the slop, so a tap arms one and
nothing else happens.

Catching a coasting list stops it — and the contact that caught it is spent
doing so, so its press is discarded. Without that, grabbing a list that is
still moving would also open whatever row happened to be sliding under the
finger at that instant, which is the least forgiving way a file manager can
be wrong.
