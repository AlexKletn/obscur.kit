The contact's velocity in px per second at the moment it lifted, given the
lift's logical timestamp `micros` — what a fling is thrown with.

**Zero if the contact had come to rest**: a finger that stops and *then*
lifts has thrown nothing, however fast it was moving a moment earlier.
This needs saying explicitly because no event arrives while a finger rests,
so the last sample still holds the speed it had before the pause; a
recognizer that reported it would fling a list the user had deliberately
stopped — the classic touch-scrolling bug. `rest_micros` is how long
without a sample counts as stopped.
