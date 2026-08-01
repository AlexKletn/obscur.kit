The exported contract for one style part of a component (spec §2.12).

A style part is a named, addressable region of a component that stylesheets
may target (for example a "track" or "thumb"). Alongside its name it
exports the set of visual states it can be in (for example "hover" or
"disabled"). Both the part name and every state are interned handles, so
styling references them by identity rather than by string.
