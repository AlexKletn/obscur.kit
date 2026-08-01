Advances every active timeline by `delta` logical seconds, then drops the
finished ones and **returns** them.

Each timeline is advanced via [`Timeline::advance`] in deterministic order
(ascending by [`TimelineId`], from the backing `BTreeMap`), after which any
that finished is pulled out of the active set. A large `delta` — e.g.
spanning several skipped frames — is handled by the timeline's own
multi-iteration overshoot, so scheduling stays smooth regardless of frame
cadence.

The returned timelines are the ones that finished **on this call**, in
ascending id order. A caller that has to keep applying a finished
animation's last value needs the position it settled at, and pruning would
otherwise destroy it — a reversed final iteration settles at progress `0`,
not `1`, so "it finished, assume the end" is wrong.
