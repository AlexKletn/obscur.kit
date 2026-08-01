The sizing function for the `idx`-th *implicit* track on an axis, from the
axis's `grid-auto-columns` / `grid-auto-rows` list.

The list is **cycled** (CSS §7.5): the n-th implicit track uses index
`n % len`. An **empty** list means the default content-sized [`TrackSize::Auto`]
— byte-identical to the historical behaviour where every implicit track was
`Auto`. An `fr` in an implicit track resolves the same way as an explicit `fr`
(via [`resolve_tracks`]); because an implicit track has no free-space basis of
its own, on an indefinite axis its leftover is zero and it collapses to its
floor (its `min`, else `0`) — documented, not special-cased.
