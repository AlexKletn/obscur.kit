The CLDR plural operands of a source number.

A plural rule is **not** written against a number — it is written against
these features of how the number was *written*. That distinction is the whole
game: `1` and `1.0` are the same value and different categories in English
(`one` vs `other`), because `v` differs. A resolver that took an `f64` could
not tell them apart, which is why this type exists and why
[`PluralRules::select`](crate::plural::PluralRules::select) takes it rather
than a number.

The fields are TR35's, transcribed from its *Plural Operand Meanings* table
(see `specs/cldr/README.md` for the pinned source):

| Symbol | Value |
|---|---|
| `n` | the absolute value of the source number |
| `i` | the integer digits |
| `v` | the number of visible fraction digits, **with** trailing zeros |
| `w` | the number of visible fraction digits, **without** trailing zeros |
| `f` | the visible fraction digits with trailing zeros, as an integer |
| `t` | the visible fraction digits without trailing zeros, as an integer |

# `c`/`e` is not modelled

TR35 defines a seventh operand, `c` (compact decimal exponent; `e` is its
deprecated synonym), for numbers written as `1.2c6`. This type does not carry
it, and the consequence is exact and worth stating rather than burying: the
CLDR rule sets whose `many` category is defined in terms of `e` — French,
Spanish, Portuguese, Italian, Catalan — are **not** among the rule sets
[`PluralRules::cardinal`](crate::plural::PluralRules::cardinal) knows, because
half of a rule set is worse than none. Compact decimal formatting is not built
either, so nothing in this workspace can produce a number with a `c` operand
to ask about.
