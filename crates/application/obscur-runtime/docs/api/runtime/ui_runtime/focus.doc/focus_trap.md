The innermost open scope that traps focus, if any.

The last trapping scope on the stack: scopes are pushed as they open, so the last
is the innermost, and a dialog opened from a dialog owns the ring the outer one
had.
