The host-reported environment values every `env()` was resolved against
(spec §6.15). Here for exactly the reason `viewport` is: a key that did
not name it would call the retained styles reusable after a keyboard
appeared, and the padding that was supposed to move out of its way would
silently stay where it was.
