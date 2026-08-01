Lowers a parsed selector list into the `obscur-style` selector model the
runtime matcher consumes (§4.13).

This is a pure, id-free structural translation: the parsed AST maps 1:1
onto [`style::SelectorList`], preserving source order. Excluded
dynamic-state pseudo-classes are *not* filtered here (that diagnostic is
raised by [`compile`](Self::compile)); they lower to
[`style::PseudoClassSelector::Other`], which the matcher treats as
matching nothing. Pseudo-elements lower likewise — preserved as
[`style::SimpleSelector::PseudoElement`], which the matcher never
matches because the kit generates no box for one.
