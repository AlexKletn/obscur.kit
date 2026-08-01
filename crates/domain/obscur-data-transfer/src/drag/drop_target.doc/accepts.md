Returns whether this target accepts `offer` (spec §6.21).

True when the target accepts any format (its accepted list is empty), or
when any accepted MIME matches one advertised by `offer`.
