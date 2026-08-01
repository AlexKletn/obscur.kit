Sets the start delay in logical seconds, returning the updated timeline.

A positive `delay` holds the timeline at the start of its first iteration
for that long before it advances; a negative one makes
[`start`](Self::start) seek `-delay` seconds into the playback, so the
timeline appears to have begun part-way through (css-transitions-1 §2.4,
css-animations-1 §3.8). A `NaN` delay is treated as `0`.
