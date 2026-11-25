Returns `true` if `element` satisfies every simple selector in
`compound`.

Type selectors compare local names ASCII case-insensitively; ids and
classes compare exactly. Attribute selectors require the attribute to be
present and, when an operator is given, to satisfy it. Pseudo-elements
never match: the kit generates no box for them, so a compound containing
one is inert (see [`SimpleSelector::PseudoElement`]). Structural
pseudo-classes consult the element's position; state pseudo-classes
consult its [`PseudoState`]; unmodeled pseudo-classes never match.
