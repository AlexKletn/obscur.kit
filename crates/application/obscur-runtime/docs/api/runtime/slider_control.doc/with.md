The ends with `grab` set to `value`.

A single slider is a range of one point and both its ends move together —
applying the no-crossing rule to one would clamp its value against ITSELF, and
the thumb could never move.

On a range the ends never cross: a thumb dragged past its neighbour stops there
rather than swapping, because swapping mid-drag would leave the pointer holding
the other thumb.
