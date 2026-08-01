Creates an idle timeline of the given identity and iteration `duration`
(logical seconds).

Defaults: state [`Idle`](TimelineState::Idle), position at
[`start`](TimelinePosition::start), [`Forward`](PlaybackDirection::Forward)
direction, `Finite(1)` (plays once), non-alternating. A `duration` that is
zero, negative or `NaN` is clamped up to
[`MIN_DURATION`](Self::MIN_DURATION).
