Lowers a compiled [`UiIrDocument`] (or a bare [`UiIrNode`] subtree) into the
runtime desired [`View`], connecting the frontend IR to the reconciler
(spec §2.15).

# Mapping

- **Element / Component** → a [`ViewNode::Element`] whose
  [`ComponentId`](obscur_identity::ComponentId) is resolved through the shared
  [`name_binding`] registry: a built-in widget/primitive tag binds to its real
  `COMPONENT_ID`; otherwise the node's **kind** decides — an
  [`Element`](UiIrNodeKind::Element) binds to the generic `Container`, a
  [`Component`](UiIrNodeKind::Component) to its own
  [`user_component_id`](name_binding::user_component_id), which is the id
  `#[derive(Contract)]` gives the same name and therefore the id a host
  registers the component under. IR
  [`properties`](UiIrNode::properties) become [`ViewProperty`]s (the reserved
  `class`/`id`/`type`/`src`/`disabled`/`checked` attributes map to the
  `obscur-view` symbols the reconciler/matcher read; every other name is
  deterministically hashed). IR [`events`](UiIrNode::events) become
  [`ViewEventBinding`]s (`click`/`input`/`change` bind to the reserved
  `EVENT_*` symbols the runtime dispatcher resolves) and the node's optional
  [`key`](UiIrNode::key) becomes a [`ViewKey`]. An **element** tag additionally
  becomes the node's *local name* — the reserved `LOCAL_NAME` property a §4.13
  type selector matches, so `button { … }` styles an authored `<button>` — under
  [`name_binding::local_name_for`], which also decides that a component tag
  contributes no name. An authored `type="…"` is a separate symbol and travels
  beside it, not instead of it.
- **Text** → a [`ViewNode::Text`] carrying the literal content (and key, if
  any).
- **Fragment** → its children spliced into the surrounding fragment (a
  fragment is transparent).
- **Conditional / Loop** → *static materialization*: the desired tree is
  concrete data with no conditional or loop node, so a
  [`Conditional`](UiIrNodeKind::Conditional) lowers to its guarded body and a
  [`Loop`](UiIrNodeKind::Loop) lowers to its template body, both spliced into
  the surrounding fragment. Evaluating a condition's truth or materializing a
  `<For>` per collection item needs runtime data (the bound values), which
  the IR does not carry; that dynamic expansion is the runtime's job. The
  loop's mandatory `key` and `each` binding are preserved on the IR node for
  that later expansion.

A property or event value that is a literal lowers to the matching
[`ViewValue`]; a dynamic [`Binding`](UiIrExpression::Binding) lowers to its
path as placeholder text (its real value needs runtime resolution) and a
[`Call`](UiIrExpression::Call) lowers to [`ViewValue::Null`].
