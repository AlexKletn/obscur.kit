Turns [`LayerSettings`] into what `wlr-layer-shell` wants to hear.

Every piece of protocol vocabulary the kit used to carry in its own types lives
here: the signed exclusive-zone encoding where `-1` is not a quantity, the
anchor bitset, the keyboard-interactivity enum, and the convention that a zero
size means "compositor, you decide".

That is the point of the file. A domain type that knew `-1` meant *ignore* was
a domain type shaped by one protocol, and the next platform's backend would
have had to un-learn it before it could translate. The settings now say what
the application meant; this says what Wayland calls it.
