The result of testing one selector against one node.

It records whether the selector matched and, when it did, the
[`SelectorSpecificity`] the matched selector contributes to the cascade.

The actual matching — walking combinators against the tree, evaluating
`:nth-*`, attribute tests, and the required [`PseudoState`]s — is performed
by `obscur-css` / the matcher. This type is only the typed outcome the
cascade consumes.

[`PseudoState`]: super::pseudo_state::PseudoState
