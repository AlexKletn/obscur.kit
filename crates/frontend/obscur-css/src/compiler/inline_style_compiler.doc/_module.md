Compiles a `style` attribute — the declarations an author wrote on the element
itself (spec §4.14, origin 5).

It goes through the ordinary stylesheet compiler, wrapped in a synthetic rule,
rather than getting a parser of its own. `style="margin:4px"` has to mean what
`margin:4px` means everywhere else, and a second implementation is a second
thing to drift.
