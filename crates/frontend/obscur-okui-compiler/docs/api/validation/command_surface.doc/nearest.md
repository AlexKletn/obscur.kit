The declared command name closest to `name`, for a did-you-mean suggestion,
or [`None`] when nothing is close enough.

# The bar

Levenshtein distance, with an explicit threshold — 1 for a name of three
characters or fewer, 2 above that — because a suggestion that is not
obviously the intended name is noise, and short names are too easy to be
two edits from each other by coincidence (`add` and `sub` would suggest each
other at 2). The nearest candidate wins; a tie goes to the first declared,
so the answer is deterministic in declaration order rather than in hash
order.

A spec test recorded this as a candidate amendment: §2.13 asks for the
compile error and says nothing about suggestions, so the bar is an
implementation choice stated here rather than a requirement met.
