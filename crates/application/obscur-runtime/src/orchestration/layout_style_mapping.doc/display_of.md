Resolves the computed `display` keyword to a [`LayoutDisplay`].

Compares the interned symbol against the reserved [`layout_keyword`] handles;
an absent or unrecognized value maps to [`LayoutDisplay::Block`] (which also
covers `DISPLAY_BLOCK`).
