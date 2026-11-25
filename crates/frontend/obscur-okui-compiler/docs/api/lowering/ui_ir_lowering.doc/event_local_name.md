Returns an event attribute's local name — the identifier after the `:` in
`on:click` (`"click"`), falling back to the attribute's own name.

`pub(crate)` because the validators must name an event exactly as the IR does:
a diagnostic about `on:clik` has to quote the string the binding was built
from, and the `ContractValidator` used to keep a byte-identical private copy of
this function to do it.
