Advances the retained-value watermark to `retained` without touching the
buffer — used when the session's OWN edit reached the tree (a passthrough
controlled input), so no reseed is needed but the watermark must track the
value the tree now carries.
