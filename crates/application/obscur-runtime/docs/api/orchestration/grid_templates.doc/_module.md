Bridges an authored CSS grid template into an `obscur-layout` [`GridTemplate`].

`grid-template-columns` / `grid-template-rows` are computed on a node's
[`ComputedStyle`] as a style-local [`PropertyValue::GridTracks`] list (each a
[`GridTrack`]). Layout consumes its own vocabulary, so — exactly as
[`layout_style_from`](super::layout_coordinator) maps computed lengths into
[`Dimension`](obscur_layout::model::Dimension)s — this module converts each
[`GridTrack`] into an `obscur_layout` [`TrackSize`] and assembles the
[`GridTemplate`] a grid container lays out with (spec §4.1).

A [`GridTrack::Fixed`] length is resolved to logical pixels through the same
[`LengthContext`] the box-model lengths use. A bare percentage inside a track
has no known basis at this point (the grid container's content size is not yet
known here), so it is carried symbolically as a [`TrackSize::Percent`]
fraction and resolved against the container content extent *inside* the layout
algorithm. A `repeat(auto-fill | auto-fit, …)` track — whose repetition count
also depends on the (not-yet-known) container size — is likewise carried
symbolically: it becomes a layout [`AutoRepeat`] inside the axis
[`TrackList`], expanded to concrete tracks by the grid algorithm. Line
placement / spanning remain out of scope for this step.
