Converts one axis's style [`GridTrack`] list into a layout [`TrackList`],
splitting it around its (first) `repeat(auto-fill | auto-fit, …)`.

Tracks before the auto-repeat become the leading run, the auto-repeat's pattern
becomes the [`AutoRepeat`], and tracks after it become the trailing run — the
shape [`TrackList`] and the grid algorithm expand against the container size.
A list with no auto-repeat yields a leading-only `TrackList` that expands
byte-identically to its explicit tracks. Only the first auto-repeat is honoured
(as in CSS, which allows one per axis); any later `AutoRepeat` degrades through
[`track_size_of`] to a single [`TrackSize::Auto`] track.
