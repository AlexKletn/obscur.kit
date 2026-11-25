The [`ChildMeasurer`] the coordinator hands to an algorithm.

It measures a child purely — by recursing into [`measure`] — and never
touches the results map, which is what keeps [`arrange`](super::arrange_pass::arrange) borrow-clean (the
`boxes` map is not borrowed for the duration of the algorithm call).
