Starts the timeline: sets it [`Running`](TimelineState::Running) and
resets the position to the start of the first iteration.

The reset respects the current direction: a
[`Forward`](PlaybackDirection::Forward) timeline begins at progress `0`
(elapsed `0`), while a [`Reverse`](PlaybackDirection::Reverse) one begins
at progress `1` (elapsed `duration`, its end).

A timeline whose repeat is `Finite(0)` plays no iterations and is set
straight to [`Finished`](TimelineState::Finished).

A [`with_delay`](Self::with_delay) delay is armed here: a positive delay
arms the countdown [`advance`](Self::advance) spends before the first
iteration moves, while a negative one is applied immediately as an
advance of `-delay` seconds, so the timeline starts part-way through its
play cycle (and may even start already finished).
