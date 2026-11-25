Re-parses the raw argument text of a selector-list pseudo-class into the
lowered `obscur-style` [`style::SelectorList`].

The selector parser captures a functional pseudo-class's argument as raw
source text (it is grammar-dependent), so the nested list is parsed here, on
demand. Argument text that does not parse — and the empty argument — yields
an **empty** list, which matches nothing and contributes zero specificity:
that is what selectors-4 §4.2 prescribes for an empty `:is()` list ("the
pseudo-class is valid but matches nothing"), and it is the fail-closed answer
for the malformed case. Note this is *not* the full `<forgiving-selector-list>`
behaviour: one invalid selector discards the whole list rather than only
itself.
