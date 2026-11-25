The §2.13 component contract for the built-in `checkbox`: the id
[`Checkbox::COMPONENT_ID`], a required `label`, a defaulted `checked`, and
no children.

The id is not restated here — it is read from the widget, which is the same
value `name_binding::CHECKBOX` gives an authored `<checkbox>` and the same
value `#[derive(Contract)]` computes for the name `"Checkbox"`. That
three-way identity is what lets a factory registered under this contract
mount on a node either front end built.

`ChildContract::None` is `ComponentContract::new`'s default and is left
implicit for exactly the reason the derive leaves it implicit: a checkbox
renders its own label, so authored children would have nowhere to go.

[`Checkbox::COMPONENT_ID`]: super::Checkbox::COMPONENT_ID
