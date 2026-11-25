Every timeline the runtime is running, the bindings that sample them onto
style, and the authored CSS rules that decide when a new one starts.

# Why these nine belong together

Three of them are one change-detection baseline in three parts, and they are
only meaningful as a set: `transition_targets` says what value each watched
property last settled at, `transition_runs` says what reversal bookkeeping a
running transition carries, and `css_animations_started` says which
`@keyframes` have already fired once. Installing a new stylesheet has to clear
**all three** — a sheet re-observing from scratch against a stale baseline
either misses the first change or replays an animation that already ran. That
is one operation, [`set_css_rules`](Self::set_css_rules), rather than three
clears a caller must remember to pair.

The other six are the running state a sample needs: the scheduler holding live
timelines, the bindings that map a timeline to a node and property, the
progress a finished timeline settled at (the scheduler prunes it, but the
animation still *holds* its final value), the next id to mint, and the
presentation overlay rebuilt from the live timelines each frame.

# What it deliberately does not do

It never asks for a frame and never touches the tree. `animate_with` and
`drive_css` report that motion started; the runtime decides that means a
frame. Selector matching needs the tree and the recorded pseudo-state, so
`drive_css` takes both as arguments rather than holding them.
