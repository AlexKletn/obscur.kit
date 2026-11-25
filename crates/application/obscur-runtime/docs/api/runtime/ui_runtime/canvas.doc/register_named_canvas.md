Registers a painter under a NAME, which is how `-obscur-painter` reaches it.

The CSS side of [`register_canvas`](Self::register_canvas): a stylesheet names a
painter with text, because an interned keyword's symbol is minted per
stylesheet and would not survive the trip.
