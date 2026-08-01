§1.3: "Unused backends and optional integrations must not be linked."

WEAKENED 2026-07-19: this originally required the umbrella crate to declare a
feature literally named `headless` (`features.contains("headless =")`), and
that assertion is what fired. §1.3's only normative sentence is the one
quoted above; the rest of the section is an *indicative* crate list
("Indicative crates:") and names no feature at all — `obscur-kit-headless`
is a crate in that list, not a Cargo feature of the umbrella. What is kept is
the sentence's actual content, tested two ways that both discriminate: the
default feature set drags in no GPU backend or window system, and every
backend dependency is optional so linkage is gated by the feature rather than
by dead-code elimination. Candidate spec amendment: if headless-only is a
supported build configuration (§1.4 lists "headless mode"), name the feature
or profile that selects it, so a consumer has something to ask for.
