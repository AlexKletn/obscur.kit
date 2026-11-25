Created and committed, waiting for the compositor's first configure.

**Nothing may be drawn yet.** Attaching a buffer before the first
configure is a protocol error, not a race.
