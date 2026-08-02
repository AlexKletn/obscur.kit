The §2.13 component contract for the built-in `button`: the id
[`Button::COMPONENT_ID`], a defaulted `label`, one slot — the default one —
and children of any type.

The id is not restated here; it is read from the widget, which is the same
value `name_binding::BUTTON` gives an authored `<button>` and the same value
`#[derive(Contract)]` computes for the name `"Button"`. That three-way
identity is what lets a factory registered under this contract mount on a
node either front end built.

`ChildContract::Any` and the [`DEFAULT_SLOT`] declaration are the departure
from every other shipped control, and they are the whole design: a checkbox,
a radio, a select and a field all render content of their own from their
props, so an authored child would have nowhere to go and their contracts say
`None`. A button renders **the author's** content — a text run, an icon and a
label, whatever was written between the tags — and says so here, in the field
§2.13 reserves for saying it.

[`Button::COMPONENT_ID`]: super::Button::COMPONENT_ID
[`DEFAULT_SLOT`]: obscur_view::view::DEFAULT_SLOT
