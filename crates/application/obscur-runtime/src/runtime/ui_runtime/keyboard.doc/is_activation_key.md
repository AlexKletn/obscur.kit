Returns whether `key` is an **activation** key — Enter or Space — the two
keystrokes that activate a focused control (spec §3.1, §6.7).

Space is accepted both as the [`NamedKey::Space`] key and as a produced `' '`
[`LogicalKey::Character`], since a layout may surface it either way. (A space
press over a scroll container would conventionally scroll rather than
activate; disambiguating activation from scrolling is a later follow-up.)
