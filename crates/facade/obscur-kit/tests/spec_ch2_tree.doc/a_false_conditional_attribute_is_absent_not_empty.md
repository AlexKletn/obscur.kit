§2.11: "- simple conditional attributes."

WEAKENED 2026-07-19: this originally asserted that a false conditional
attribute is *absent from the desired tree entirely*
(`desired_prop(button, DISABLED) == None`), and that it is not
`ViewValue::Bool(false)`. That is HTML's boolean-attribute omission rule;
§2.11's entire text here is one bullet, "- simple conditional attributes.",
which does not state it, and `.okui`/`okui!` have no conditional-attribute
form distinct from an ordinary attribute binding. What survives is the part
the spec does support — the attribute must faithfully carry the condition and
must never be stringified — plus the observable consequence, that no
`[disabled]`-style selector matches. Candidate spec amendment: state whether a
false conditional attribute is omitted or carried as `false`, since selector
matching, accessibility and IR size all depend on the answer.
