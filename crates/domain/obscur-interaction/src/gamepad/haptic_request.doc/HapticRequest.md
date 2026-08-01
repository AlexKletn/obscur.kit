A request for haptic feedback (spec §6.16 haptics).

A request pairs a [`pattern`](Self::pattern) with an
[`intensity`](Self::intensity) in `0.0..=1.0` and a
[`duration`](Self::duration_millis) in milliseconds. It is a pure
description: this crate emits the request but never actuates hardware or
reads a clock. A request is [`silent`](Self::is_silent) when its intensity
or duration is zero.
