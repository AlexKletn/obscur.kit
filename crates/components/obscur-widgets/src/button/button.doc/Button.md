A headless push-button control (spec §2.6).

`Button` owns its interactive state — whether it is currently
[`pressed`](Self::is_pressed) and whether it is
[`disabled`](Self::is_disabled) — plus the `command` it fires when
activated. It bakes in **no** visual theme: [`view`](Self::view) derives a
semantic [`ViewNode`] tree (a [`SemanticRole::Button`] element carrying its
state as properties, a click event bound to `command`, and a single
[`Text`] label child) that the style layer renders keyed on
[`COMPONENT_ID`](Self::COMPONENT_ID).

The state machine is a plain press/release cycle: [`press`](Self::press)
arms the button and [`release`](Self::release) reports whether that cycle
*activated* it (pressed while enabled). A disabled button neither arms nor
activates.
