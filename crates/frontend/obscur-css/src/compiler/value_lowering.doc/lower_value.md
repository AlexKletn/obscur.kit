Lowers a component-value list into a single [`PropertyValue`].

Only the first component is inspected: the modelled single-valued properties
take one component, and multi-component shorthands are handled separately by
[`super::shorthand_expansion::expand_box_shorthand`]. Returns [`None`] when
the component cannot be lowered.
