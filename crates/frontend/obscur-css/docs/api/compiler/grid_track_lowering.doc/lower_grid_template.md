Lowers a `grid-template-columns` / `grid-template-rows` value into a
[`PropertyValue::GridTracks`] track list (spec §4.1).

The value is a space-separated list of tracks; each track is a `<length>`
(`px`/`%`/`em`/… → [`GridTrack::Fixed`]), an `<fr>` (`1fr` → [`GridTrack::Fr`]),
the `auto` keyword ([`GridTrack::Auto`]), or `minmax(min, max)`
([`GridTrack::MinMax`]). A `repeat(N, <tracks>)` is expanded in place into `N`
flat copies of its inner track list. A template can contain one automatic
repeat. Its pattern and surrounding tracks must use fixed sizes, including
valid `minmax()` forms with a fixed bound. Invalid or empty lists return
[`None`] and produce an unsupported-feature diagnostic.
