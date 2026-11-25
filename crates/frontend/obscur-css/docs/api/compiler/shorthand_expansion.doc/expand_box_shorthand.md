Expands a `margin` / `padding` shorthand into its four longhand declarations.

The value list is lowered component-by-component into lengths (a unitless `0`
is accepted as `0px`); the CSS 1/2/3/4-value edge-replication rules then fan
the components out to the four sides. If any component is not a length, or the
list is empty or longer than four components, the whole shorthand degrades to
a single [`CompiledDeclaration::Unsupported`] rather than being dropped.
