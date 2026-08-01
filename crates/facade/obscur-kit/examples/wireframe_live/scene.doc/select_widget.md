The live select's bare view — now rendered entirely by the real [`Select`]
widget: the header (current label + ▼/▲ arrow) plus, while open, a real
`.select-list` dropdown popup of `.select-option` rows below it (the selected
one inverted). No proxy is needed any more — the widget renders its own list —
so this simply defers to [`Select::view`]. A click fires [`CMD_SEL`]: open when
closed, or pick-next + close when open, via the single shared handler.
