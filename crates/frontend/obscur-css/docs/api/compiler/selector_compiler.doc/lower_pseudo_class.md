Lowers a pseudo-class name (and optional functional argument) into the
`obscur-style` [`style::PseudoClassSelector`].

The name is matched case-insensitively. Functional `:nth-*` forms parse their
argument via [`parse_anb`]; the selector-list forms (`:is()`, `:where()`,
`:not()`, `:has()`, and the `of S` tail of `:nth-child()`) re-parse their raw
argument text via [`parse_nested_list`]; any name the model does not know is
preserved as [`style::PseudoClassSelector::Other`] (which matches nothing).

`:nth-last-child(An+B of S)` is deliberately **not** lowered to a modelled
form: deciding it means testing `S` against *following* siblings, which
`MatchElement` cannot navigate to (see
[`style::PseudoClassSelector::Has`] for the same blocker). It falls through
to `Other`, matching nothing, and is reported as unsupported.
