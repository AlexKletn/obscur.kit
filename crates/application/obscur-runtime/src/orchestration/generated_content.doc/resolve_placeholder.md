Resolves the synthetic **placeholder** run for a text control (§4.16), or
[`None`] when the node carries no non-empty placeholder or its value is not
empty (the CSS `:placeholder-shown` rule — the hint shows only while the field
is empty).

Unlike an ordinary component-internal part (which crosses the scope boundary
only from inside a mounted component host), the placeholder's host is the field
element **itself**: the exported-part selector `Component::part(placeholder)`
is matched with the field as the host, so `Input::part(placeholder) { color }`
recolours the hint without the field having to be a component host. Its text is
the reserved `PLACEHOLDER` attribute the widget emits.
