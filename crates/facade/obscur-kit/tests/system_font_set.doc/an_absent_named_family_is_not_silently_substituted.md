`fc-match` never fails — asked for a family that does not exist it returns the
system default with exit status 0. A named-family lookup built on it alone would
resolve every absent family to a substituted face, and an author's fallback
chain would never be walked.

The adapter guards this twice over (`fc-list`, which returns nothing for an
absent family, plus a name filter), and the guards are redundant: this test only
fails once **both** are gone, so it pins the outcome rather than either
mechanism. That is what it should pin — the substitution is the bug.
