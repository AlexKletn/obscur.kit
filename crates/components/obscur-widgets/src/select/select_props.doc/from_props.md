Decodes a `<select>` node's properties.

Each of the three is read from **two** sources, in the same order and for the
same reason the checkbox reads its `checked` twice: the widget's own property
first, the authored attribute second. The widget's property is emitted only by
[`Select::view`], so where it is present the node came from the Rust API and
its value is exact; where it is absent the node came from markup.

| Prop | The widget's spelling | Markup's spelling |
|---|---|---|
| `options` | [`Select::PROP_OPTIONS`], labels joined by [`Select::OPTION_SEPARATOR`] — a character no typed label contains | `options="Alpha, Beta"`, comma-separated and trimmed; an empty list names no options |
| `selected` | [`Select::PROP_SELECTED`], an index with `-1` for "nothing picked" | `selected={1}` |
| `open` | [`Select::PROP_OPEN`], a boolean | `open`, where **presence** is the statement — a bare attribute lowers to `ViewValue::Null` |

Nothing here can fail: every prop is defaulted, so a bare `<select/>` decodes
to the empty, closed control rather than reporting. The one value that *could*
be nonsense — a `selected` index past the end of the list — is not rejected
either, because [`Select::contents`] already answers it the way a browser does:
the header falls back to the first option and no row carries the `selected`
class.

[`Select::view`]: super::Select::view
[`Select::PROP_OPTIONS`]: super::Select
[`Select::PROP_SELECTED`]: super::Select
[`Select::PROP_OPEN`]: super::Select
[`Select::OPTION_SEPARATOR`]: super::Select
[`Select::contents`]: super::Select
