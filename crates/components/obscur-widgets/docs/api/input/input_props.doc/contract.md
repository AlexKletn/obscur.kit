The §2.13 component contract for the built-in text-entry control: the id
[`Input::COMPONENT_ID`], a defaulted `value`, and no children.

The id is not restated here — it is read from the widget, which is the same
value `name_binding::INPUT` gives an authored `<input>` **and** an authored
`<textarea>`, and the same value `#[derive(Contract)]` computes for the name
`"Input"`. That identity is what lets one factory mount on a node either
front end built, under either tag.

`value` is defaulted rather than required: unlike a checkbox's label, an
empty field is the ordinary starting state, so `<input/>` must decode.

`ChildContract::None` is `ComponentContract::new`'s default and is left
implicit, as the derive leaves it implicit. It is also a claim about
`<textarea>`: HTML makes a textarea's child text its *default value*, and
this toolkit spells that `value="…"` instead — see the ledger row in
`docs/api/12-reality.md`.

[`Input::COMPONENT_ID`]: super::Input::COMPONENT_ID
