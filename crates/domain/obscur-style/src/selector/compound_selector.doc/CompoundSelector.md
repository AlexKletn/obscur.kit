A compound selector — a run of simple selectors with no combinator between
them, all of which must match the same element (`div.a#b:hover`).

The order of the simples is preserved from the source but is irrelevant to
matching: an element matches iff it satisfies every simple.
