Which authoring position a tag was written in — the component-vs-element
distinction both front ends already make.

This is an argument rather than something [`component_id_for`] infers,
because the two positions have genuinely different fallbacks and the resolver
used to pick one of them for everything. It is not re-derived from the tag's
spelling here: the front ends own that rule (`.okui` lowering's
`starts_uppercase`, the macro's `is_component`), they record their verdict on
the IR node's [`UiIrNodeKind`](crate::document::UiIrNodeKind), and a second
copy of the rule in this crate could only ever drift from it.
