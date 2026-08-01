The revision of a [`CssSource`](crate::source::CssSource)'s text (spec §4.12).

§4.12 requires CSS parsing to be "cached per unique source/version and
invalidated explicitly". A [`SourceId`](crate::source::SourceId) answers *which
stylesheet*; this answers *which revision of it*, so a cache can tell a hit
from a stale entry without comparing the text.

# Why not hash the text

Because that is a different guarantee. Hashing says "the bytes match"; a
version says "the owner declares this is the same revision". §4.12 asks for
*explicit* invalidation, and an explicit version is what makes it explicit: an
author who edits a sheet bumps it, and nothing silently decides on their behalf
by re-reading and re-hashing the very text the cache exists to avoid touching.
