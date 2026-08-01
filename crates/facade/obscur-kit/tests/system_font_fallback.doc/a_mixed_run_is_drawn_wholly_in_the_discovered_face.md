Fallback is chosen per **run**, not per character — so a run mixing covered
and uncovered text is drawn entirely in the discovered face.

Worth pinning as the behaviour rather than leaving it to be discovered: the
text is itemized by script, a face is chosen for each item, and one character
the chain cannot draw therefore pulls its whole item into the host's face.
`ab⌫cd` comes out in DejaVu rather than mixing Noto with DejaVu. That is a
visible difference from a browser, and it is the price of choosing a face per
run; splitting an item at coverage boundaries would be the fix, and is not
built.
