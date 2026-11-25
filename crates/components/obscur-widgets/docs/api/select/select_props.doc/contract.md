The §2.13 component contract for the built-in `select`: the id
[`Select::COMPONENT_ID`], three defaulted props — `options`, `selected`,
`open` — and no children.

Nothing is required, and that is the control's own statement rather than
laxness: `Select::new(command)` with no options is a legal, empty select, so a
`<select/>` must decode rather than report.

The id is not restated here — it is read from the widget, which is the same
value `name_binding::SELECT` gives an authored `<select>` and the same value
`#[derive(Contract)]` computes for the name `"Select"`. That three-way identity
is what lets a factory registered under this contract mount on a node either
front end built.

`ChildContract::None` is `ComponentContract::new`'s default and is left
implicit for exactly the reason the derive leaves it implicit: a select builds
its own header, arrow, list and rows, so authored children would have nowhere
to go.

[`Select::COMPONENT_ID`]: super::Select::COMPONENT_ID
