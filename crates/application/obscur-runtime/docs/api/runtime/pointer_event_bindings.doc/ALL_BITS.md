All seven bits — the point at which a scan can stop early.

Also the budget: seven of `u8`'s eight are spent, so **one** gated name is still
affordable and the next one after that forces `bound` to a `u16`. That has to be
a decision someone takes rather than one someone discovers, because a name added
without a bit fails closed and in silence — `bit_for` answers `0`, `binds` reads
`0` as "not bound", and the event is simply never dispatched.
`the_gate_has_one_bit_left_and_the_eighth_name_must_widen_the_word` is what makes
the day loud; it fails on the eighth name, one before the word is out of room.
