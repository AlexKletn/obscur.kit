The compositor does not implement `wlr-layer-shell`.

Expected on GNOME and on any non-wlroots compositor that has not adopted
the protocol. A host that also has an ordinary window path should fall
back to it rather than treat this as an error.
