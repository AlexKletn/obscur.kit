The seconds since the previous reading, clamped, and moves the mark to `now`.

Answers `0.0` the first time, so a caller may pass it straight to
`advance_motion` without a special case for the first turn.
