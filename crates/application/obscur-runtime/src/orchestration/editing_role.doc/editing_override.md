The editing session that overrides `node`'s text this frame, if any: the
session view when `node` is the field's value text, or its `.placeholder`
overlay while the session buffer is non-empty; `None` otherwise. Shared by the
compose (scene) and measure/arrange (layout) passes so the box is sized from —
and the glyphs painted from — the *same* string.
