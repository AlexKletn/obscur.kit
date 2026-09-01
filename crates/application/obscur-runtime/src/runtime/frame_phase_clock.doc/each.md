Print every frame rather than a mean over sixty, set by
`OBSCUR_FRAME_STATS=each`.

The mean is what hid the fault it was meant to find: a window averaging 17 ms
held a frame of 234, and the two read identically. A distribution needs the
frames themselves.
