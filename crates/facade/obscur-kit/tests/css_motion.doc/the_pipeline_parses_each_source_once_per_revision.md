A host that installs CSS through the kit gets §4.12's parse cache without
asking for it.

Proven by the one observation only a cache can produce: the *same* source
identity at the *same* revision, but with different text, must lay out with
the FIRST revision's rules — a pipeline that re-parsed would pick up the new
text. Bumping the revision then re-parses, so the cache is a cache and not a
one-shot.
