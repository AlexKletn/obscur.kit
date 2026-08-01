The same field, EMPTY, showing its placeholder inside a marked `.placeholder`
overlay child — mirroring what the `Input` widget emits.

The placeholder is a non-editable overlay (CSS `::placeholder` /
`:placeholder-shown`), **not** the field value, so focusing this view must seed
an *empty* editing buffer. A runtime that seeded from the text it finds
underneath would put "Type here…" in the user's field.
