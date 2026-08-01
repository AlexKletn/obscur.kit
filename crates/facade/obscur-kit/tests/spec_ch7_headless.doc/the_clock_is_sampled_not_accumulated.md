§7.4: "Testing facilities:" / "- deterministic clock;" with §3.5 "animation
progress is sampled from time/value, never slowed to match missed frames."

WEAKENED 2026-07-19: the five comparisons were `assert_eq!` on `f32` and
demanded the three runs agree BIT FOR BIT. Fifteen 10ms steps measured
149.99998 against one 150ms step's 150.0 — an f32 rounding residue of
1.3e-7 relative, not a behavioural difference. §3.5's sentence forbids
frame-rate-dependent SLOWING: a runtime that advanced one fixed quantum per
frame would report ~10 here, not 149.99998. §7.4's "- deterministic clock;"
requires the same script to give the same answer, which it does (see
`real_time_does_not_leak_into_the_deterministic_clock`), not that different
step patterns be bit-identical. The comparisons now carry a tolerance well
below anything a slowed clock could hide in. Candidate spec amendment: state
whether the deterministic clock accumulates a delta per step or samples an
absolute time — only the latter makes bit-identical step patterns a
requirement rather than luck.
