Derives the semantic view: a [`COMPONENT_ID`] element carrying the role
and disabled state, the tri-state checked value as an integer property
(`-1` = mixed/`None`, `0` = unchecked, `1` = checked), a change event
bound to `command`, and — as children — the boxed indicator sub-part
followed by the label [`Text`].

The indicator is a `.checkbox-box` element the UA/theme sheet fills; it
carries a state class (`checked` when definitely on, `indeterminate` when
mixed) so a `.checkbox-box.checked { … }` rule (or a
`checkbox:hover .checkbox-box { … }` descendant rule) can key on it. Its
child is the tick [`Text`] "✓" when checked, or a thin `.checkbox-bar`
element when mixed; nothing when unchecked. The interactive state
(`EVENT_CHANGE`, reserved `:checked`/`:disabled` attrs) stays on the OUTER
control element, so a click on the box bubbles to it and pseudo-classes
still drive the whole control.

[`COMPONENT_ID`]: Self::COMPONENT_ID
