Resolves a tag name, written in `position`, to the [`ComponentId`] it binds
to.

A built-in's **id** always wins: `<Button/>` resolves to the shipped widget's
`COMPONENT_ID` rather than to a hash of the string, and `img`/`Image`,
`button`/`Button` are one handle each. That is a claim about the handle, not
about what renders — a host that registers its own factory under
`Button::COMPONENT_ID` (which is what `#[contract(name = "Button")]` derives)
takes the tag over, and this function neither knows nor prevents it.

Otherwise the fallback follows the position — an element tag is the generic
[`CONTAINER`] box, a component tag is its own [`user_component_id`].
