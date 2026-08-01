Records an access at logical `tick`: updates `last_access` and increments
the hit count (spec §5.19).

`tick` is a logical, caller-supplied counter — never a wall-clock reading
— so recency stays deterministic. The hit count saturates.
