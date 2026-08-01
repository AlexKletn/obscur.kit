The cardinal plural rules of one locale (spec §7.2).

`PluralRules` answers exactly one question — [`select`](Self::select): which
[`PluralCategory`] do these [`PluralOperands`] fall into for this locale? It
is the piece §7.2 names Russian to guard: `if n == 1 { singular } else {
plural }` is not a simplification of this, it is a different and wrong
function that happens to agree in English.

# Honest capability

The rule sets are a **documented subset** of CLDR, not all of it — see
[`cardinal`](Self::cardinal). A locale outside the subset gets the root rules
(everything is `other`) and says so through
[`availability`](Self::availability), which answers
[`SupportLevel::Unavailable`]. It does not guess, and it does not fall back to
English.
