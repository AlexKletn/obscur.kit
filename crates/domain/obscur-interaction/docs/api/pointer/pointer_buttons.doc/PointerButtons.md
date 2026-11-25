The set of pointer buttons currently held down (spec §4.7).

Backed by a `u8` bitset; each [`PointerButton`] contributes one bit. An empty
set means no buttons are pressed (a hover or contact-less move).
