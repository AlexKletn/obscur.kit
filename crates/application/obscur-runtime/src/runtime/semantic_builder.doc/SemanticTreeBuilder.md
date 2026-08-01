Projects a [`RetainedTree`] into a semantic accessibility tree (spec §7.1).

§7.1 requires accessibility to use "a semantic tree linked to the retained UI
tree". This builder is that link: it walks the live retained tree from its
root and emits one [`SemanticNode`] per retained node — keyed by the very same
[`NodeId`] — deriving each node's [`role`](SemanticRole),
accessible name, and [`state`](SemanticState) from what the retained node
actually carries. It invents no data the tree does not hold.

The mapping is deterministic (`same tree → same semantic tree`) and pure: it
reads the retained tree and produces an owned [`SemanticTree`] / immutable
[`SemanticSnapshot`], leaving the retained tree untouched. The keeps
`obscur-accessibility` free of any dependency on `obscur-tree`.

# Mapping

- [`NodeKind::Root`] → [`SemanticRole::Group`] (the structural grouping root);
- [`NodeKind::Text`] → [`SemanticRole::Text`], with the literal content as the
  accessible name and [`SemanticValue::Text`] value;
- [`NodeKind::Synthetic`] → [`SemanticRole::Group`], flagged
  [`decorative`](SemanticNode::decorative) (system parts are not exposed as
  content);
- [`NodeKind::Element`] → the role named by its CSS *type* / local name (e.g.
  `button` → [`SemanticRole::Button`], `text` → [`SemanticRole::Text`]),
  defaulting to [`SemanticRole::Group`] when the element carries no
  recognised local name. An element's accessible name is drawn from its direct
  text children.

# State

A retained node's [`NodeFlags::HIDDEN`] maps to
[`SemanticState::hidden`], and the `focused` node passed to
[`build`](Self::build) is marked [`SemanticState::focused`]; every interactive
role is marked [`SemanticState::focusable`].

[`disabled`](SemanticState::disabled) and [`readonly`](SemanticState::readonly)
are read from the **same** [`StyleAttributes`](obscur_view::view::StyleAttributes)
the CSS matcher consumes for `:disabled` (see
[`TreeElement`](crate::orchestration::TreeElement)) and that the runtime's
`is_within_disabled` / `is_within_readonly` gates consume for interaction —
*including their inheritance*: a control nested inside a disabled panel is
inert to the runtime, and is published disabled here. So a disabled control
is never [`is_actionable`](SemanticState::is_actionable), and the CSS,
interaction and accessibility paths cannot disagree about it.
[`checked`](SemanticState::checked) carries the `:checked` attribute, but only
for a role that is genuinely checkable ([`Checkbox`](SemanticRole::Checkbox) /
[`RadioButton`](SemanticRole::RadioButton)); every other role stays `None`
(not checkable) rather than claiming a two-state it does not have.

# Actions

[`actions`](SemanticNode::actions) are read from a node's **own event
bindings**, never inferred from its role: a bound
[`EVENT_CLICK`] — or a bound [`EVENT_CHANGE`] on a non-editable control, which
is how the runtime resolves a checkbox/radio activation — yields
[`SemanticAction::Activate`], and an actionable node yields
[`SemanticAction::Focus`]. A disabled node offers no actions, because the
runtime fires none for it. This mirrors `UiRuntime::activation_command_for`
for the node itself; unlike that method it does not bubble, since an action
list answers what invoking *this* node does.

# Not derived

The retained tree holds no source for these, so they are deliberately left at
their defaults rather than invented:

- [`bounds`](SemanticNode::bounds): layout output, not tree state — this
  builder takes only a [`RetainedTree`] and never sees a layout result;
- [`description`](SemanticNode::description) and
  [`relations`](SemanticNode::relations): no `aria-describedby` / `aria-labelledby`
  equivalent exists in the reserved attribute set;
- [`selected`](SemanticState::selected), [`busy`](SemanticState::busy),
  [`required`](SemanticState::required), [`invalid`](SemanticState::invalid),
  [`expanded`](SemanticState::expanded): no reserved attribute carries them.
  A node's raw props *do* survive reconciliation — `set_props` stamps the
  full list and `RetainedNode::props` hands it back — but only the seven
  reserved names (`class` / `id` / `type` / `src` / `disabled` / `checked` /
  `readonly`) are extracted into `StyleAttributes`, and no convention
  establishes what a prop named `selected` would mean. Deriving these would
  be inventing data, not reading it.

A checkbox's *indeterminate* state is likewise not representable: the widget
emits the reserved `checked` attribute only when **definitely** checked, so a
mixed checkbox reads as `Some(false)` here.
