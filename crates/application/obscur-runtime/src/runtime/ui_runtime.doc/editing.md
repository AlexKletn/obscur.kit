The runtime-owned editing session for the focused editable, if any (spec
§5.11, §6.7). Seeded from the field's current text when focus lands on an
editable (a node whose ancestor chain binds
[`EVENT_INPUT`](obscur_view::view::EVENT_INPUT)) and cleared when focus
leaves one. The runtime is the source of truth for the focused field's
text/caret/selection; keys resolve to
[`EditCommand`](obscur_text::editing::EditCommand)s applied to its buffer.
