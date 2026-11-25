Derives the semantic view: a [`COMPONENT_ID`] element carrying the role,
disabled and selected state, a change event bound to `command`, and — as
children — the circular `.radio-dot` indicator followed by the label
[`Text`].

The indicator is a `.radio-dot` element the UA/theme sheet fills; its
child is a `.radio-fill` centre dot present **only when selected**. The
interactive state (`EVENT_CHANGE`, reserved `:checked`/`:disabled` attrs)
stays on the OUTER control element, so a click on the dot bubbles to it
and pseudo-classes drive the whole control.

[`COMPONENT_ID`]: Self::COMPONENT_ID
