Returns a copy of this stroke dashed with `pattern` (alternating on/off
run lengths in user-space units) starting at phase `offset`.

An empty `pattern` clears any dash, leaving a solid stroke. Backends read
this back verbatim and map it onto their native dash support.
