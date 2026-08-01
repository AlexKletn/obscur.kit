§2.13: "- defaults;"

WEAKENED 2026-07-19: this originally asserted that an omitted prop arrives
carrying an AUTHOR-SPECIFIED default value — `"variant=primary count=5"`.
§2.13's entire text is the bullet "- defaults;", and no such surface exists:
`#[contract(default)]` takes no value and `PropContract::with_default()`
carries none, so "primary"/5 appeared nowhere but this file's doc comments
and the case was unsatisfiable by any implementation. What the spec's bullet
does support, and what is kept, is that a defaulted prop is declared
*optional* rather than required and that omitting it is not an error.
Candidate spec amendment: state whether "- defaults;" means author-specified
default VALUES in the contract, or only optionality with the type's `Default`
— and if the former, name the declaration syntax.
