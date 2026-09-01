How long the loop is willing to sleep while something is animating.

Idle, the loop sleeps until the compositor or a waker has something to say. With
a frame pending it bounds the wait instead, so animation continues without a
thread to spare.
