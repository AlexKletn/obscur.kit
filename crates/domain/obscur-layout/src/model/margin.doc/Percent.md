A percentage of the containing block's **inline** size, as a percentage
value (`50.0` for `50%`).

The inline size for *every* edge, including `margin-top` and
`margin-bottom` — CSS 2.1 §8.3 resolves all four against the containing
block's *width*, not each edge's own axis. It reads like a bug and is not:
a block's height often depends on its content, so resolving a block-axis
percentage against it would be circular. One axis is chosen to break the
cycle, and CSS chose the inline one.
