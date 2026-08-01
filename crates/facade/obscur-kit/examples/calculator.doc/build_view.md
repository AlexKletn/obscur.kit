The whole screen: tape, display, keypad — every band sized by `flex`, so the
window's size is the only thing that decides how big anything is.

The outer `.safe` wrapper carries nothing but the `env()` padding. Keeping it
separate from `.app`'s own 12px means the two do not have to be added
together in CSS, which `padding` has no way to express.
