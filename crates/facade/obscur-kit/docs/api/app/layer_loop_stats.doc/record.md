Add one turn, and print when the window is full.

`waited` is time spent in `dispatch` **while a frame was pending**; an idle wait
is the loop asleep with nothing to do and is not counted, or it would swamp
everything and mean nothing.

The worst `present` is carried alongside the average because the defect this was
written for is a *burst*: most turns are fine and a few are catastrophic, and
sixty turns averaged reads as healthy.
