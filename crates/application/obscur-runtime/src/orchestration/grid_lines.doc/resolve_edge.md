Resolves one named edge against `map`, returning the concrete
[`GridLine::Line`] it names; a non-named edge, or a name absent from `map`,
keeps `current` (its already-computed value, `GridLine::Auto` for a name).
