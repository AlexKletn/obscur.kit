Classifies the literal an attribute carries, for the §2.8 type check.

A quoted value is a string; a braced value is parsed as a Rust expression and
classified only when it is a literal (or a negated numeric literal). Anything
else — a binding `{count}`, an arithmetic expression `{a + b}`, a bare word —
is [`Unknown`](AttributeValueType::Unknown) and left unchecked.
