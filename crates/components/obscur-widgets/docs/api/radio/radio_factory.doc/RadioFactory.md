Mounts the built-in `radio` under [`Radio::COMPONENT_ID`].

It is the second instance of the shape [`CheckboxFactory`] introduced: four
methods, no state, every answer delegated to the widget's own id and its props
struct's `contract()`. Two of them agree line for line, which is the evidence
that the shape is right — and also the evidence that there is nothing yet to
generalise: what varies between them is three type names, and a generic
adapter parameterised on three type names would be longer than the thing it
replaced.

[`Radio::COMPONENT_ID`]: super::Radio::COMPONENT_ID
[`CheckboxFactory`]: crate::checkbox::CheckboxFactory
