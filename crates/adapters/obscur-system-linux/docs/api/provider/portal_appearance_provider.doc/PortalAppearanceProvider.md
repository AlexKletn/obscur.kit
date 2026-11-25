Appearance as the portal states it — the cross-desktop answer.

`org.freedesktop.appearance` is what GTK and Qt applications read themselves, so
it agrees with the rest of the desktop by construction, and it is the only
source here that answers inside a sandbox where the config files are not
reachable.

Narrow in exchange: three keys — colour scheme, accent, contrast. No fonts, no
palette, no icon theme. Which is why [`Auto`](super::AppearanceSource::Auto)
asks it first and then keeps going.
