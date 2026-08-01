Validates every dynamic-attribute expression in `doc`, recursively
(spec §2.10).

Each element's attributes are inspected; every attribute that carries a
dynamic `{expr}` value has its expression text validated at the
expression's byte offset, then the element's children are walked the same
way. Literal string attributes have no expression and are skipped.
