The derive's answer for the name `"Input"`, to compare with the hand-written
[`InputProps::contract`].

One defaulted `value` and nothing else — which is the whole claim about this
control's authored surface. The `placeholder` is not here because it is a
§4.16 synthetic part the runtime paints from a reserved attribute rather than
a prop the component renders; `readonly` and `disabled` are not here because
they are reserved structural attributes; and `multiline` is not here because
it is the element's *local name* (`<input>` versus `<textarea>`), not a prop.

[`InputProps::contract`]: obscur_kit::widgets::input::InputProps::contract
