The eight reserved **click command** symbols, one per switcher tab: tab `i`'s
node binds `on-click → KIT_TAB_CMD[i]` (via [`EVENT_CLICK`]). These are purely
*bindings* — the static PNG examples never register a handler, so a click is a
no-op there; the windowed `uikit_window` example registers a handler per symbol
that live-swaps to `themes()[i]` and rebuilds `view(i)`. The values sit in an
app-owned command namespace (`0x4B10 + i`, "KIT tab i"), distinct from the
showcase scene's `0x50xx` commands, so nothing collides.
