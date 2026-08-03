The user asked for a context menu on the hit node — `on:contextmenu="…"`.

Emitted from a **secondary-button press**, which is the only gesture that asks
for one here: there is no context-menu key on the keyboard path and no long-press
route to it, and inventing either would be a name emitted from somewhere nothing
produces. Delivered to the node the press landed on, through §2.3's capture /
target / bubble, after `pointerdown` and before the press does anything.

`prevent_default` on it suppresses **the focus move that a secondary press
performs**, and that is the whole of it — because moving focus is the whole of
what a secondary press does. It seats no caret, anchors no click, arms no
gesture and activates nothing on release; those belong to the primary button.
An application drawing its own menu refuses this when it does not want the
click-through to also move focus out of the field the menu is about.

Refusing `pointerdown` suppresses it too, for the same reason and one event
earlier — but the two names are not alternatives: `contextmenu` is still
dispatched after a refused `pointerdown`, because a listener declining the focus
move has not declined the *request for a menu*.
