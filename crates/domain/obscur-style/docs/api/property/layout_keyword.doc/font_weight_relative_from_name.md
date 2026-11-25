Maps a relative `font-weight` keyword string (`bolder` / `lighter`) to its
reserved [`InternedSymbol`].

The input is ASCII-lowercased before matching. Returns [`None`] for any other
keyword — including the absolute `normal` / `bold` spellings, which lower to
numeric weights rather than reserved handles.
