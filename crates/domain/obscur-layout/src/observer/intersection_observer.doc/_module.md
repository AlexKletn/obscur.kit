The [`IntersectionObserver`] — watches how much a node overlaps a root rect
(spec §4.4).

Modelled on the web `IntersectionObserver`: it tracks the fraction of the
target rectangle covered by a root/viewport rectangle and fires when that
ratio crosses its `threshold`, i.e. when the target transitions between
*intersecting* and *not intersecting*. Like all geometry observers it
reports the coalesced net change after layout commit.
