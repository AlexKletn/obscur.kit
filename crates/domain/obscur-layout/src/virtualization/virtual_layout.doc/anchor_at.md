Captures a [`ScrollAnchor`] at the current viewport top (spec §4.9).

The anchor pins the item at `scroll_offset`, remembering how far into that
item the top sits, so the scroll position can be restored after items above
it are inserted or re-measured.
