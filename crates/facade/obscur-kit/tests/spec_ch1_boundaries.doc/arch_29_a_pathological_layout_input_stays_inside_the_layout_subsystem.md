§1.7: "Layout, renderer and resource failures use subsystem-specific
recovery boundaries."

WEAKENED 2026-07-19, and read the reason — the original failure message was
actively misleading. It ended by demanding "exactly one layout diagnostic"
with node context and a recovery action, and reported `0`, which reads as a
bug in the layout subsystem. It is not: `calc(1px / 0)` never reaches layout.
`LengthExpr::resolve` returns `None` for a zero divisor
(obscur-style/src/property/length_expr.rs:112-118), so the declaration fails
to fold in the STYLE subsystem and the used value falls back to `auto`. No
non-finite value is ever produced, and the case's premise — "a pathological
layout input" — is never constructed by this fixture. The missing emission it
would otherwise report is the style subsystem's, and that is already tracked
as a real gap by `arch_16_a_malformed_declaration_kills_neither_its_rule_nor_the_sheet`;
duplicating it here under a layout label would misattribute it.

§1.7's sentence says failures use subsystem-specific recovery boundaries. It
does not say each recovery emits a diagnostic — §1.6 specifies the *shape* of
diagnostics that are emitted, not an obligation to emit one per recovery. The
containment assertions, which do encode the quoted sentence and do
discriminate against an implementation that escalates a value failure to the
component boundary, are kept.

Candidate spec amendment: two. (1) State whether every subsystem-local
recovery must emit a diagnostic, or only those that change observable
geometry. (2) Provide a way to construct a genuinely pathological *layout*
input through the public surface, since at present style-level folding
catches every route to one and this boundary cannot be tested at all.
