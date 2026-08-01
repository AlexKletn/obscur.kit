The [`checked`](SemanticState::checked) tri-state for `node`: the same
`:checked` attribute the CSS matcher reads, but reported only for a role
that is genuinely checkable — a `None` here means "not checkable", which
would be a lie for a button, so a non-checkable role never claims a
two-state at all.
