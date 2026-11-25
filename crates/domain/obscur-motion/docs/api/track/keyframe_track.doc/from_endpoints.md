Creates a two-stop linear track running `start` (offset `0`) to `end`
(offset `1`) — the minimal track between two endpoint values (spec §4.17).

Used to build interruption redirects, e.g. a
[`Reverse`](crate::transition::TransitionInterruption::Reverse) that plays
the element's current value back toward the interrupted transition's
origin.
