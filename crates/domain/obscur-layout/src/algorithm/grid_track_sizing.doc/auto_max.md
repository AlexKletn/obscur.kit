Whether this track's *max* sizing function is literally `auto` (as opposed
to `max-content`, which shares the same [`Kind::Max`] content contribution).
§11.8 *Stretch auto Tracks* grows only `auto`-max tracks, so this preserves
the distinction the growth-limit [`Kind`] no longer carries.
