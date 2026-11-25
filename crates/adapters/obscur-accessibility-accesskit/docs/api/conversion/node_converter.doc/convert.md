Builds an AccessKit [`Node`] for `node`, attaching `children` (already
mapped to domain ids) as its child list.

State maps as follows: `disabled` → [`Node::set_disabled`], an ignored
node (decorative or hidden) → [`Node::set_hidden`], `selected` →
[`Node::set_selected`], `required` → [`Node::set_required`], `readonly`
→ [`Node::set_read_only`], `busy` → [`Node::set_busy`], `invalid` →
[`Node::set_invalid`], a `checked` tri-state → [`Node::set_toggled`],
and an `expanded` tri-state → [`Node::set_expanded`]. A
[`SemanticValue::Text`] becomes the AccessKit `value` string; a
[`SemanticValue::Number`] becomes the numeric value plus its bounds.

A [`SemanticValue::Range`] — a two-thumb control — becomes the bounds plus
a value STRING naming both ends (`"20 to 80"`), and deliberately leaves
`numeric_value` unset: AccessKit carries one number per node, so filling
it would announce one end as the whole control. That string is currently
English regardless of the application's locale, which is a known
limitation of stating two numbers through a slot built for one.
