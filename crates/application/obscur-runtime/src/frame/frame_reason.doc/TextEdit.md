A pure text edit into the focused editable that cannot change its box
size (a same-size single-line keystroke, or any caret/selection move):
only the field's glyphs + caret change, so the frame re-shapes and
re-composes the field WITHOUT a restyle or relayout — the compose-only
fast path (spec §3.4 phase 5). An edit that can resize the box (a
wrapping/multiline field growing a line, an auto-width field) uses
[`Input`](Self::Input) instead so its box relayouts.
