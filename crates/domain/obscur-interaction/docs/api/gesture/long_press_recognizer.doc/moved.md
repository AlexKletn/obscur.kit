Reports that the contact has moved `distance` from where it went down,
failing the long press if that is more than `slop`.

A finger that wandered is doing something else — scrolling, most likely —
and a long press that survived the wander would fire a context action in
the middle of someone's scroll.
