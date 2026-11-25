The speed (px/s) a lift must exceed for the drag to have *thrown* the list.

Deliberately higher than [`FLING_REST_SPEED`], and not the same question. The
rest speed asks when an existing coast has effectively stopped; this asks
whether there was ever a throw. Conflating them means a slow, deliberate drag
— one that ends at a walking pace because the user was placing the list
exactly — drifts on for another few pixels past where they let go, which
reads as the list disobeying.
