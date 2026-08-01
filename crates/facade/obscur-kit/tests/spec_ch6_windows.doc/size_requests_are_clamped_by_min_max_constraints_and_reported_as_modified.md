§6.2: "- min/max constraints;" listed as a dynamic window property alongside
"- size;", plus "Operations return typed results."

A request outside the constraints is satisfied in a modified form: the *used*
size is clamped, and the operation still answers with a typed result.

OBSERVATION MOVED + WEAKENED 2026-07-19. The original failure message —
"constraints stored but never applied" — was false, and the case was looking
in the wrong place. `WindowProperties::size` is the *desired* size, which is
exactly what the caller asked for and is meant to be echoed back verbatim;
the used size is `Window::constrained_size()`
(obscur-windowing/src/window/window.rs:95-99), which applies
`WindowConstraints::constrain` and is correct. The clamping assertion is kept
in full, read off the used size.

Separately DROPPED: `support_level() != Some(Native)` for a clamped request,
i.e. that a modified outcome must be reported as non-native. §6.2 lists
"- size;" and "- min/max constraints;" among *potential* window properties
and says "Operations return typed results." with the four capability
categories, but nowhere ties the category to whether the used value matched
the requested one. Candidate spec amendment: say what an operation satisfied
in a modified form reports — `emulated`, or `native` with a differing used
value — and whether the caller can discover the difference from the result
alone rather than by re-reading the window.
