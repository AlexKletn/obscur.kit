Begins a drag: captures the pointer, remembers which end it grabbed, and jumps the thumb to the press.

A press JUMPS the thumb to where it landed — the behaviour a press on the track
has to have, and a no-op when the press lands on the thumb itself. Capture is
immediate so a drag that leaves the track keeps steering, which is what every
slider anywhere does and what a user relies on when dragging fast.
