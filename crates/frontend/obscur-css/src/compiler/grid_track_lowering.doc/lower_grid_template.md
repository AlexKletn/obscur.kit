Lowers a `grid-template-columns` / `grid-template-rows` value into a
[`PropertyValue::GridTracks`] track list (spec §4.1).

The value is a space-separated list of tracks; each track is a `<length>`
(`px`/`%`/`em`/… → [`GridTrack::Fixed`]), an `<fr>` (`1fr` → [`GridTrack::Fr`]),
the `auto` keyword ([`GridTrack::Auto`]), or `minmax(min, max)`
([`GridTrack::MinMax`]). A `repeat(N, <tracks>)` is expanded in place into `N`
flat copies of its inner track list. Returns [`None`] (so the declaration
degrades to an unsupported-feature diagnostic) when the list is empty or any
component is not a recognised track.
