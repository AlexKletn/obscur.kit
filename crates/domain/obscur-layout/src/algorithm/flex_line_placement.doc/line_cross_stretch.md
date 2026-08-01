Multi-line only (`stretch_target == None`): when `Some(target)`, the line's
cross box is grown to `max(natural, target)` and `align-items: stretch`
items fill the grown cross. This is how `align-content: stretch` grows each
flex line by its share of the container's leftover cross space
(css-flexbox-1 §8.4). `None` keeps the natural (max item cross) sizing.
