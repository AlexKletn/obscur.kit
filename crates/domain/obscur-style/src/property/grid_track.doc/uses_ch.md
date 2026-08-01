Returns `true` when this track's sizing function contains a `ch` term.

`Fr`, `Auto` and the intrinsic keywords carry no length at all; `MinMax`
recurses into both
bounds, so `minmax(20ch, 1fr)` answers `true`, and `AutoRepeat` recurses
into the pattern it repeats — `repeat(auto-fill, 20ch)` sizes every
repetition from the `0` glyph. See [`Length::is_ch`].
