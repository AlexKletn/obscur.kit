Matches `sheets` against every element node in `tree` in `environment`,
gating each `@media`/`@container` rule by whether its condition holds.

The stateless (no-interaction) counterpart of
[`build_inputs_with_pseudo_env`](Self::build_inputs_with_pseudo_env): only
the media/container environment participates, so `:hover`/`:focus` still
never match.
