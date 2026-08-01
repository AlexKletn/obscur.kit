How visible a cached artifact is on screen, driving eviction priority (spec
§5.19 *visibility priority*).

The variants are declared in **ascending importance** order so the derived
`Ord` makes the most-evictable the *smallest*: `Hidden < Offscreen <
NearViewport < Visible`. An eviction policy that prefers to drop the least
important artifact can therefore simply pick the minimum visibility (see
[`crate::cache::EvictionPolicy`]). `Hidden` is thus the first eviction
candidate and `Visible` is retained longest.
