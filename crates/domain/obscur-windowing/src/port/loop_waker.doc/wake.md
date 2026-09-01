Ask the loop to turn once.

Means "reconsider the interface" and nothing finer — there is no payload,
because a caller that needs to say *what* changed should put that in the state
the view reads, where the view will actually see it.

Waking a loop that is already awake is harmless, and waking one that has
already exited does nothing.
