A plain string literal: the value of the `content` property on a
`::before` / `::after` generated-content rule (§4.16), stored **as a
string** (its quotes stripped) because the runtime paints it directly and
never crosses it into the interner. It is not a computed value of any real
element's box — the runtime reads it off a matched pseudo-element rule to
build a decoration-only inline box.
