The effects `list` demands that `renderer` reports it will **not** reproduce
([`SupportLevel::Unavailable`]), in first-use paint order (spec §5.2).

Only `Unavailable` is collected: it is the level at which the effect does not
happen at all, and it is knowable from the table alone — the answer is a property
of the backend, so a scene demanding the effect is already the whole story. A
[`SupportLevel::Limited`] or [`SupportLevel::Emulated`] effect *does* reach the
frame, and whether its documented restriction actually bit depends on the frame,
which no table can answer; a backend reports those from its render path with
[`EffectFallbackReporter::report_fallbacks`](super::EffectFallbackReporter::report_fallbacks).
