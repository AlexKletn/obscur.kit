Sets this element's `style` attribute — a declaration block written *on* the
element, which the cascade admits at origin `Inline` (spec §4.14). A no-op on a
text node, like [`with_prop`](Self::with_prop).

The text is exactly what would go between the braces of a rule:
`.style("width: 50%; opacity: 0.5")`. It outranks every stylesheet by ORIGIN,
so no selector an author can write reaches it — which is what makes it the
wrong tool for anything a class could express, and the right one for a value
computed per element.

It is compiled by whatever inline-style compiler the host installed on the
runtime. A runtime driven with no CSS front end wired to it has none, and the
attribute is then carried but never read — the declarations are never invented
from a compiler that is not there.
