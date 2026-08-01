The two key presses every text test spells: a printable character, and a named
editing key.

The counterpart of [`mouse`](super::mouse) for the keyboard. Both live here for
the same reason `mouse` does — their signatures name only `obscur-interaction`
types, so one definition serves the unit tests inside `obscur-runtime` **and**
the integration tests beside them, which the same crate compiled twice would
otherwise force to keep two copies of.
