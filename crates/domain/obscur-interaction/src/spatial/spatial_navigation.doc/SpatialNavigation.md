Stateless helper computing directional (spatial) focus navigation
(spec §4.7).

Given a source rectangle and a set of candidate rectangles, [`find_next`]
picks the best candidate to focus in a [`NavigationDirection`]. All geometry
is compared by rectangle *centers*.

[`find_next`]: SpatialNavigation::find_next
