What a canvas painter is told about the node it is drawing.

A struct rather than a growing parameter list: adding a fact a painter needs
should not change every painter's signature. It lives here, beside the property
that names a painter, because that is the whole canvas protocol and because
both ends of it — the widget that draws and the runtime that calls — can see
this crate while neither can see the other.
