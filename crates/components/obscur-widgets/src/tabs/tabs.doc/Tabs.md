A headless tab strip selecting one of several panels (spec §2.6).

`Tabs` owns its list of tab `labels` and the index of the currently
[`active`](Self::active_index) tab. It bakes in **no** visual theme:
[`view`](Self::view) derives a `tablist`-role element carrying the active
index as a property, with one `tab`-role child element per label (each
carrying a selected flag and a [`Text`] label child), rendered by the style
layer keyed on [`COMPONENT_ID`](Self::COMPONENT_ID).

The [`semantic_role`](Self::semantic_role) reported for the strip is
[`SemanticRole::Tab`] (the accessible model has no dedicated `tablist` role
in the common cross-platform set). Conceptually each label child is a
[`SemanticRole::Tab`] and the content it reveals would be a
[`SemanticRole::TabPanel`]; `Tabs` models only the strip, leaving the panel
to the host.

The state machine is index selection ([`activate`](Self::activate), which
validates the index) plus wrapping movement ([`next`](Self::next),
[`previous`](Self::previous)).
