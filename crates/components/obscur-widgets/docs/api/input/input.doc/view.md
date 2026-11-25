Derives the semantic view: a [`COMPONENT_ID`] element carrying the role,
disabled state, current value, and placeholder as properties, plus two
event bindings to `command` — the HTML `input`/`change` event model
(WHATWG HTML §4.10.5.5 *Common input element APIs*): a per-keystroke
[`EVENT_INPUT`] (each typed character / `Backspace` drives the runtime's
editing session, which delivers the resulting field text to the app) and a
commit-only [`EVENT_CHANGE`]. The widget only *declares* both bindings; the
runtime does the routing — for this editable it fires `change` on **commit**
(blur, or `Enter` for a single-line field) carrying the committed value, and
deliberately *not* on a focus click (unlike a checkbox/radio/select, whose
toggling click **is** the change).

The current text is carried *both* as the [`PROP_VALUE`] property (the
semantic / accessibility surface) *and*, when non-empty, as a single
[`Text`] child node, so the field renders visibly and the runtime's
editing-session auto-seeding — which reads the focused editable's
descendant [`ViewNode::Text`] content — picks up the value.

**Placeholder as a §4.16 synthetic part.** The `placeholder` hint is carried
as the reserved [`PLACEHOLDER`](obscur_view::view::PLACEHOLDER) structural
attribute — *not* materialised into the tree as a child element. The runtime
paints it as a synthetic part (paint-only, DevTools-visible, hit-transparent,
and **not** a component-tree child) while the field is empty — the toolkit's
analogue of the CSS **`::placeholder`** pseudo-element, shown only while
**`:placeholder-shown`** holds (an empty value): CSS Pseudo-Elements Level 4
§4.5 (`::placeholder`) and CSS Selectors Level 4 §11.2 (`:placeholder-shown`).
Per that model the placeholder is a separate, non-editable rendering layer —
never seeded into the editing buffer and never a hit target. An author can
restyle it through the exported-part selector `Input::part(placeholder)`.

[`COMPONENT_ID`]: Self::COMPONENT_ID
[`PROP_VALUE`]: Self::PROP_VALUE
[`Text`]: obscur_view::view::ViewNode::text
