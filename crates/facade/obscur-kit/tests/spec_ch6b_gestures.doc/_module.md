§6.16 Mobile gestures — the arena's default priority direction.

Specification-derived. The load-bearing sentence is "System gestures have
priority by default": system priority is the default and application priority
is the opt-in, not the other way round.

NOTE ON THE SETUP. §6.16 also names "edge/reserved gesture regions" and
"exclusion regions where allowed". This crate exposes no way to declare a
reserved system-gesture region, no way to mark a contender as a *system*
gesture, and no exclusion-region registration API, so the case's scenario 2
(the app opts out of the reserved strip and then wins) cannot be written.
What is written here is scenario 1 — the default direction — expressed
through the only arena surface that exists, and scenario 3 as its control.
