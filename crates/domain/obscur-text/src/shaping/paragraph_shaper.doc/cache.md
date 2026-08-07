Memoized paragraph runs, keyed by every input that affects the output: a
two-generation, two-level table rather than a single bounded map, so that
crossing the bound evicts only the least recently used generation and a
lookup allocates nothing.
