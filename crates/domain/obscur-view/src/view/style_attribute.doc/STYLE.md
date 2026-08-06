The `style` attribute: CSS declarations written on the node itself.

Text, exactly as on the web — `"color:red;width:40px"`. It is TEXT and not a
compiled block because this crate describes what to render and knows nothing
about CSS; the runtime is handed a compiler for it (spec §4.14, origin 5).
