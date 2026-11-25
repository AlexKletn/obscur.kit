Maps a fontconfig `%{slant}` to a [`FontSlant`].

`fontconfig.h`: `FC_SLANT_ROMAN` 0, `FC_SLANT_ITALIC` 100, `FC_SLANT_OBLIQUE`
110. Anything at or past oblique reads as oblique; anything past roman but
short of italic reads as italic, since a slanted face is nearer italic than
upright.
