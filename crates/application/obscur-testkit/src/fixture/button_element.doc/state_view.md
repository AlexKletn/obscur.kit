The same button, carrying the reserved `:disabled` attribute when `disabled`.

Mirrors what `Button::view` emits, so a `button:disabled` rule is being matched
against the structural state the real widget reports rather than against a
hand-written attribute.
